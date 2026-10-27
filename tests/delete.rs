//! Compile-time verification of every documented `p4 delete` syntax form.
//!
//! For each documented command line this walks the legal builder chains
//! through the builder API, asserting that they all assemble the same
//! arguments. The reference is the current command reference:
//!
//! ```text
//! p4 [g-opts] delete [-c changelist] [-n -k -v] [--remote=remote] file ...
//! ```
//!
//! On servers older than 2016.1 the `--remote` option does not exist, and the
//! 2024.1 reference renames the operand to `-c PendingChangelist` over
//! `FileSpec`; every spelling assembles identical arguments, so the
//! assertions below hold across all supported versions.

use std::io;
use std::path::PathBuf;
use std::process::Child;

use perforce_cli::P4Cli;
use perforce_cli::cmd::Delete;
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
/// expect_chain!(["delete", "-n"], p4.cmd_delete().preview(true));
/// ```
macro_rules! expect_chain {
    ([$($e:literal),* $(,)?], $($chain:tt)*) => {{
        let cmd = $($chain)*;
        expect_args(&cmd, &[$($e),*]);
    }};
}

/// A tiny batch helper that just prints its arguments. `p4 delete` only
/// appends the subcommand name, flags, changelist numbers, and file names;
/// none of the samples below contain spaces, so `%*` captures the exact
/// argv tail.
fn echo_bin() -> PathBuf {
    let path = std::env::temp_dir().join("p4cli_delete_echo_args.bat");
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
fn echo_delete() -> Delete {
    Delete::new(echo_bin(), GlobalOpts::new())
}

// Bare form -----------------------------------------------------------------
// `p4 delete file ...`
//
// The bare form opens the given files for deletion with no options; it is
// the state returned by `cmd_delete`.

fn bare_form() {
    let p4 = P4Cli::default();

    expect_chain!(["delete"], p4.cmd_delete());

    let mut cmd = echo_delete();
    assert_eq!(spawned_args(&mut cmd, (["file.c"],)), ["delete", "file.c"]);

    let mut cmd = echo_delete();
    assert_eq!(
        spawned_args(&mut cmd, (["a.c", "b.h"],)),
        ["delete", "a.c", "b.h"]
    );
}

// Mode flags ----------------------------------------------------------------
// `p4 delete [-n -k -v] file ...`
//
// `-n`, `-k`, and `-v` may be combined freely and selected in any order;
// they inject in the synopsis order `-n -k -v`.

fn mode_flags() {
    let p4 = P4Cli::default();

    expect_chain!(["delete", "-n"], p4.cmd_delete().preview(true));
    expect_chain!(["delete", "-k"], p4.cmd_delete().keep_localfile(true));
    expect_chain!(["delete", "-v"], p4.cmd_delete().delete_unsynced(true));

    // A false flag leaves the option out entirely.
    expect_chain!(["delete"], p4.cmd_delete().preview(false));
    expect_chain!(["delete"], p4.cmd_delete().keep_localfile(false));
    expect_chain!(["delete"], p4.cmd_delete().delete_unsynced(false));

    // Any selection order produces the same injection order.
    expect_chain!(
        ["delete", "-n", "-k"],
        p4.cmd_delete().preview(true).keep_localfile(true)
    );
    expect_chain!(
        ["delete", "-n", "-k"],
        p4.cmd_delete().keep_localfile(true).preview(true)
    );
    expect_chain!(
        ["delete", "-n", "-k", "-v"],
        p4.cmd_delete()
            .preview(true)
            .keep_localfile(true)
            .delete_unsynced(true)
    );
    expect_chain!(
        ["delete", "-n", "-k", "-v"],
        p4.cmd_delete()
            .delete_unsynced(true)
            .keep_localfile(true)
            .preview(true)
    );

    // The builder methods may be repeated in any order; each toggles its
    // own flag.
    expect_chain!(
        ["delete", "-n", "-k", "-v"],
        p4.cmd_delete()
            .preview(true)
            .keep_localfile(true)
            .delete_unsynced(true)
            .preview(true)
            .keep_localfile(true)
            .delete_unsynced(true)
    );

    let mut cmd = echo_delete().preview(true).keep_localfile(true);
    assert_eq!(
        spawned_args(&mut cmd, (["locked.c", "stale.c"],)),
        ["delete", "-n", "-k", "locked.c", "stale.c"]
    );
}

// Remote option -------------------------------------------------------------
// `p4 delete [--remote=remote] file ...` (2016.1+)
//
// `--remote` combines with the other mode flags and injects last, in
// `--remote=value` form.

#[cfg(not(feature = "lt2016_1"))]
fn remote_option() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["delete", "--remote=myRemote"],
        p4.cmd_delete().remote("myRemote")
    );
    expect_chain!(
        ["delete", "-k", "--remote=myRemote"],
        p4.cmd_delete().keep_localfile(true).remote("myRemote")
    );
    expect_chain!(
        ["delete", "-n", "-v", "--remote=myRemote"],
        p4.cmd_delete()
            .remote("myRemote")
            .preview(true)
            .delete_unsynced(true)
    );

    let mut cmd = p4.cmd_delete().remote("myRemote");
    assert_eq!(cmd.get_remote().map(String::as_str), Some("myRemote"));
    cmd.set_remote("otherRemote");
    expect_args(&cmd, &["delete", "--remote=otherRemote"]);

    let mut cmd = echo_delete().remote("myRemote");
    assert_eq!(
        spawned_args(&mut cmd, (["cloned.c"],)),
        ["delete", "--remote=myRemote", "cloned.c"]
    );
}

// Changelist ----------------------------------------------------------------
// `p4 delete -c changelist [options] file ...`
//
// `-c` combines with every mode flag, and injects before them.

fn changelist() {
    let p4 = P4Cli::default();

    expect_chain!(["delete", "-c", "42"], p4.cmd_delete().changelist("42"));
    expect_chain!(
        ["delete", "-c", "42", "-n"],
        p4.cmd_delete().changelist("42").preview(true)
    );
    expect_chain!(
        ["delete", "-c", "42", "-n"],
        p4.cmd_delete().preview(true).changelist("42")
    );
    expect_chain!(
        ["delete", "-c", "42", "-k", "-v"],
        p4.cmd_delete()
            .keep_localfile(true)
            .delete_unsynced(true)
            .changelist("42")
    );
    #[cfg(not(feature = "lt2016_1"))]
    expect_chain!(
        ["delete", "-c", "42", "--remote=origin"],
        p4.cmd_delete().remote("origin").changelist("42")
    );

    // Accessors.
    let mut cmd = p4.cmd_delete().changelist("42");
    assert_eq!(cmd.get_changelist().map(String::as_str), Some("42"));
    cmd.set_changelist("99");
    expect_args(&cmd, &["delete", "-c", "99"]);

    let mut cmd = echo_delete().changelist("42").preview(true);
    assert_eq!(
        spawned_args(&mut cmd, (["gone.c"],)),
        ["delete", "-c", "42", "-n", "gone.c"]
    );
}

// Global options ------------------------------------------------------------
// Every form accepts the global options, which are injected before the
// subcommand name.

fn global_options() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let bare = Delete::new("p4", global_opts.clone());
    expect_args(&bare, &["-p", "localhost:1666", "-q", "delete"]);

    let mode = Delete::new("p4", global_opts.clone())
        .preview(true)
        .keep_localfile(true);
    expect_args(&mode, &["-p", "localhost:1666", "-q", "delete", "-n", "-k"]);

    let with_changelist = Delete::new("p4", global_opts).changelist("42");
    expect_args(
        &with_changelist,
        &["-p", "localhost:1666", "-q", "delete", "-c", "42"],
    );
}

fn main() -> io::Result<()> {
    bare_form();
    mode_flags();
    #[cfg(not(feature = "lt2016_1"))]
    remote_option();
    changelist();
    global_options();
    println!("all p4 delete syntax combinations verified");
    Ok(())
}
