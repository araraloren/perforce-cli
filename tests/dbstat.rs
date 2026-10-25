//! Verification of every documented `p4 dbstat` syntax form.
//!
//! For each documented command line this walks the legal builder chains
//! through the type-state API, asserting that they all assemble the same
//! arguments and spawn the same argv tail. The reference is the current
//! command reference:
//!
//! ```text
//! p4 [g-opts] dbstat [-h][-f] | [-p [-w worklevel] [-n numberOfFiles]] {-a | dbtable ...}
//! p4 [g-opts] dbstat -s
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::process::Child;

use perforce_cli::P4Cli;
use perforce_cli::cmd::DBStat;
use perforce_cli::cmd::SubCommand;
use perforce_cli::global::GlobalOpts;
use perforce_cli::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collects the arguments assembled on a [`std::process::Command`] as
/// strings for easy comparison.
fn args_of(command: &std::process::Command) -> Vec<String> {
    command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

/// Asserts the arguments produced by `setup_command`.
fn expect_args(cmd: &impl SubCommand, expected: &[&str]) {
    assert_eq!(args_of(&cmd.setup_command("p4")), expected);
}

/// Asserts that one chained builder expression assembles `expected`.
///
/// ```ignore
/// expect_chain!(["dbstat", "-a"], p4.cmd_dbstat().all());
/// ```
macro_rules! expect_chain {
    ([$($e:literal),* $(,)?], $($chain:tt)*) => {{
        let cmd = $($chain)*;
        expect_args(&cmd, &[$($e),*]);
    }};
}

/// A tiny batch helper that just prints its arguments. `p4 dbstat` only
/// appends the subcommand name, flags, and table names; none of the samples
/// below contain spaces, so `%*` captures the exact argv tail.
fn echo_bin() -> PathBuf {
    let path = std::env::temp_dir().join("p4cli_dbstat_echo_args.bat");
    std::fs::write(&path, "@echo %*\r\n").expect("write echo helper");
    path
}

/// Normalizes the echo helper output into logical arguments.
fn parse_echo(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .split_whitespace()
        .map(|s| s.trim_matches('"').to_string())
        .collect()
}

/// Actually spawns the echo helper and returns the argv tail it received.
fn spawned_args<C, I>(cmd: &mut C, input: I) -> Vec<String>
where
    C: ParameterizedSpawn<I, Output = Child, Error = io::Error>,
{
    let output = cmd
        .spawn_with(input)
        .expect("spawn echo helper")
        .wait_with_output()
        .expect("wait for echo helper");
    assert!(output.status.success(), "echo helper exited unsuccessfully");
    parse_echo(&output.stdout)
}

/// Starts a builder whose executable is the local echo helper.
fn echo_dbstat() -> DBStat {
    DBStat::new(echo_bin(), GlobalOpts::new())
}

// Form 1 -------------------------------------------------------------------
// `p4 dbstat dbtable ...` and `p4 dbstat -a`
//
// The bare `{-a | dbtable ...}` selection tail is carried by the
// `Unselected` and `DisplayAllTables` states: the former spawns with a list
// of table names, the latter supplies `-a` and spawns without arguments.

#[test]
fn plain_form_with_table_list() {
    let p4 = P4Cli::default();
    expect_chain!(["dbstat"], p4.cmd_dbstat());

    let mut cmd = echo_dbstat();
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch", "db.change"],)),
        ["dbstat", "db.arch", "db.change"]
    );
}

#[test]
fn plain_form_all() {
    let p4 = P4Cli::default();
    expect_chain!(["dbstat", "-a"], p4.cmd_dbstat().all());

    let mut cmd = echo_dbstat().all();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-a"]);
}

// Form 2 -------------------------------------------------------------------
// `p4 dbstat [-h][-f] {-a | dbtable ...}`
//
// Both flags live on the `Scan` state; they are entered from the bare
// command with a transition that pre-selects one of them, and can then be
// toggled with the corresponding builder.

#[test]
fn scan_form_histogram() {
    let p4 = P4Cli::default();

    expect_chain!(["dbstat", "-h"], p4.cmd_dbstat().show_histogram());
    expect_chain!(
        ["dbstat", "-h", "-a"],
        p4.cmd_dbstat().show_histogram().all()
    );
    expect_chain!(
        ["dbstat", "-a"],
        p4.cmd_dbstat().show_histogram().show_histogram(false).all()
    );

    let mut cmd = echo_dbstat().show_histogram();
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch"],)),
        ["dbstat", "-h", "db.arch"]
    );

    let mut cmd = echo_dbstat().show_histogram().all();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-h", "-a"]);
}

#[cfg(not(feature = "lt2016_1"))]
#[test]
fn scan_form_free_pages() {
    let p4 = P4Cli::default();

    expect_chain!(["dbstat", "-f"], p4.cmd_dbstat().show_free_pages());
    expect_chain!(
        ["dbstat", "-f", "-a"],
        p4.cmd_dbstat().show_free_pages().all()
    );

    let mut cmd = echo_dbstat().show_free_pages();
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch"],)),
        ["dbstat", "-f", "db.arch"]
    );

    let mut cmd = echo_dbstat().show_free_pages().all();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-f", "-a"]);
}

#[cfg(not(feature = "lt2016_1"))]
#[test]
fn scan_form_both_flags_in_either_order() {
    let p4 = P4Cli::default();

    // `-h` is injected before `-f` regardless of which builder ran first.
    expect_chain!(
        ["dbstat", "-h", "-f"],
        p4.cmd_dbstat().show_histogram().show_free_pages(true)
    );
    expect_chain!(
        ["dbstat", "-h", "-f"],
        p4.cmd_dbstat().show_free_pages().show_histogram(true)
    );
    expect_chain!(
        ["dbstat", "-h", "-f", "-a"],
        p4.cmd_dbstat().show_histogram().show_free_pages(true).all()
    );

    let mut cmd = echo_dbstat().show_histogram().show_free_pages(true);
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch"],)),
        ["dbstat", "-h", "-f", "db.arch"]
    );
}

#[test]
fn scan_accessors_toggle_flags() {
    let mut cmd = DBStat::new("p4", GlobalOpts::new()).show_histogram();
    assert!(cmd.get_show_histogram());

    cmd.set_show_histogram(false);
    assert!(!cmd.get_show_histogram());
    expect_args(&cmd, &["dbstat"]);

    #[cfg(not(feature = "lt2016_1"))]
    {
        let mut cmd = DBStat::new("p4", GlobalOpts::new()).show_free_pages();
        assert!(cmd.get_show_free_pages());

        cmd.set_show_free_pages(false);
        assert!(!cmd.get_show_free_pages());
        expect_args(&cmd, &["dbstat"]);
    }
}

// Transitions from the `-a` state ------------------------------------------
// `DisplayAllTables` can enter the scan form while retaining `-a`.

#[test]
fn all_state_enters_scan_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["dbstat", "-h", "-a"],
        p4.cmd_dbstat().all().show_histogram()
    );
    expect_chain!(
        ["dbstat", "-a"],
        p4.cmd_dbstat().all().show_histogram().show_histogram(false)
    );

    let mut cmd = echo_dbstat().all().show_histogram();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-h", "-a"]);
}

#[cfg(not(feature = "lt2016_1"))]
#[test]
fn all_state_enters_scan_form_with_free_pages() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["dbstat", "-f", "-a"],
        p4.cmd_dbstat().all().show_free_pages()
    );

    let mut cmd = echo_dbstat().all().show_free_pages();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-f", "-a"]);
}

// Form 3 -------------------------------------------------------------------
// `p4 dbstat -p [-w worklevel] [-n numberOfFiles] {-a | dbtable ...}`
//
// The checkpoint state injects `-p` first, then `-w`, then `-n`, in the
// documented order regardless of the order the accessors are called. Both
// `-w` and `-n` stay available after `all()`.

#[cfg(not(feature = "lt2023_1"))]
#[test]
fn checkpoint_form_worklevel_and_number_files() {
    let p4 = P4Cli::default();

    expect_chain!(["dbstat", "-p"], p4.cmd_dbstat().checkpoint());
    expect_chain!(
        ["dbstat", "-p", "-w", "2"],
        p4.cmd_dbstat().checkpoint().worklevel(2)
    );
    expect_chain!(
        ["dbstat", "-p", "-w", "2", "-n", "3"],
        p4.cmd_dbstat().checkpoint().worklevel(2).number_files(3)
    );
    expect_chain!(
        ["dbstat", "-p", "-w", "2", "-n", "3"],
        p4.cmd_dbstat().checkpoint().number_files(3).worklevel(2)
    );

    // `worklevel` / `number_files` also enter the checkpoint form directly.
    expect_chain!(["dbstat", "-p", "-w", "2"], p4.cmd_dbstat().worklevel(2));
    expect_chain!(["dbstat", "-p", "-n", "3"], p4.cmd_dbstat().number_files(3));
    expect_chain!(
        ["dbstat", "-p", "-w", "2", "-a"],
        p4.cmd_dbstat().worklevel(2).all()
    );

    let mut cmd = echo_dbstat().worklevel(2);
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch"],)),
        ["dbstat", "-p", "-w", "2", "db.arch"]
    );

    let mut cmd = echo_dbstat().number_files(3);
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch"],)),
        ["dbstat", "-p", "-n", "3", "db.arch"]
    );

    let mut cmd = echo_dbstat().checkpoint().worklevel(2).number_files(3);
    assert_eq!(
        spawned_args(&mut cmd, (["db.arch"],)),
        ["dbstat", "-p", "-w", "2", "-n", "3", "db.arch"]
    );
}

#[cfg(not(feature = "lt2023_1"))]
#[test]
fn checkpoint_form_all() {
    let p4 = P4Cli::default();

    expect_chain!(["dbstat", "-p", "-a"], p4.cmd_dbstat().checkpoint().all());
    expect_chain!(
        ["dbstat", "-p", "-w", "2", "-a"],
        p4.cmd_dbstat().checkpoint().worklevel(2).all()
    );
    // `-w` and `-n` remain available in the `DisplayAllTables` state.
    expect_chain!(
        ["dbstat", "-p", "-w", "2", "-n", "3", "-a"],
        p4.cmd_dbstat()
            .checkpoint()
            .all()
            .worklevel(2)
            .number_files(3)
    );

    let mut cmd = echo_dbstat().checkpoint().worklevel(2).all();
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["dbstat", "-p", "-w", "2", "-a"]
    );
}

#[cfg(not(feature = "lt2023_1"))]
#[test]
fn all_state_enters_checkpoint_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dbstat", "-p", "-a"], p4.cmd_dbstat().all().checkpoint());
    expect_chain!(
        ["dbstat", "-p", "-w", "2", "-a"],
        p4.cmd_dbstat().all().worklevel(2)
    );
    expect_chain!(
        ["dbstat", "-p", "-n", "3", "-a"],
        p4.cmd_dbstat().all().number_files(3)
    );

    let mut cmd = echo_dbstat().all().checkpoint();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-p", "-a"]);

    let mut cmd = echo_dbstat().all().worklevel(2);
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["dbstat", "-p", "-w", "2", "-a"]
    );
}

#[cfg(not(feature = "lt2023_1"))]
#[test]
fn checkpoint_accessors() {
    let mut cmd = DBStat::new("p4", GlobalOpts::new()).checkpoint();
    assert_eq!(cmd.get_worklevel(), None);
    assert_eq!(cmd.get_number_files(), None);

    cmd.set_worklevel(4).set_number_files(5);
    assert_eq!(cmd.get_worklevel(), Some(4));
    assert_eq!(cmd.get_number_files(), Some(5));
    expect_args(&cmd, &["dbstat", "-p", "-w", "4", "-n", "5"]);
}

// Form 4 -------------------------------------------------------------------
// `p4 dbstat -s`
//
// The size form accepts no other option and spawns without arguments.

#[test]
fn size_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dbstat", "-s"], p4.cmd_dbstat().size());

    let mut cmd = echo_dbstat().size();
    assert_eq!(spawned_args(&mut cmd, ()), ["dbstat", "-s"]);
}

// Global options ------------------------------------------------------------
// Every form accepts the global options, which are injected before the
// subcommand name.

#[test]
fn global_options_precede_subcommand() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let plain = DBStat::new("p4", global_opts.clone());
    expect_args(&plain, &["-p", "localhost:1666", "-q", "dbstat"]);

    let all = DBStat::new("p4", global_opts.clone()).all();
    expect_args(&all, &["-p", "localhost:1666", "-q", "dbstat", "-a"]);

    let scan = DBStat::new("p4", global_opts.clone()).show_histogram();
    expect_args(&scan, &["-p", "localhost:1666", "-q", "dbstat", "-h"]);

    let size = DBStat::new("p4", global_opts.clone()).size();
    expect_args(&size, &["-p", "localhost:1666", "-q", "dbstat", "-s"]);

    #[cfg(not(feature = "lt2023_1"))]
    {
        let checkpoint = DBStat::new("p4", global_opts).checkpoint().number_files(2);
        expect_args(
            &checkpoint,
            &["-p", "localhost:1666", "-q", "dbstat", "-p", "-n", "2"],
        );
    }
}

// Accessors -----------------------------------------------------------------

#[test]
fn bin_accessors() {
    let mut cmd = DBStat::new("p4", GlobalOpts::new());
    assert_eq!(cmd.get_bin(), Path::new("p4"));

    cmd.set_bin("/usr/local/bin/p4");
    assert_eq!(cmd.get_bin(), Path::new("/usr/local/bin/p4"));

    let cmd = cmd.bin("p4.exe");
    assert_eq!(cmd.get_bin(), Path::new("p4.exe"));
}

// Rejected combinations -----------------------------------------------------
//
// The chains below contradict the documented forms and must fail to
// compile; they are kept as executable documentation of the type-state
// rules. Uncomment any line to verify it is rejected.
//
// // The scan form cannot be re-entered or converted to the checkpoint form.
// let _ = P4Cli::default().cmd_dbstat().show_histogram().show_histogram();
// let _ = P4Cli::default().cmd_dbstat().show_histogram().checkpoint();
// let _ = P4Cli::default().cmd_dbstat().show_histogram().worklevel(2);
// let _ = P4Cli::default().cmd_dbstat().checkpoint().show_histogram();
// let _ = P4Cli::default().cmd_dbstat().all().show_histogram().checkpoint();
//
// // `-w`/`-n` are unavailable in the scan and size forms.
// let _ = P4Cli::default().cmd_dbstat().size().worklevel(2);
//
// // The spawn arity follows the table selection.
// let _ = P4Cli::default().cmd_dbstat().spawn_with(());
// let _ = P4Cli::default().cmd_dbstat().all().spawn_with(("db.arch",));
// let _ = P4Cli::default().cmd_dbstat().size().spawn_with(("db.arch",));
//
// // Older servers reject `-f` (`lt2016_1`) and `-p` (`lt2023_1`).
// let _ = P4Cli::default().cmd_dbstat().show_free_pages();
// let _ = P4Cli::default().cmd_dbstat().checkpoint();
