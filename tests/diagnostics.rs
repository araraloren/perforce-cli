//! Verification of every documented `p4 diagnostics` syntax form.
//!
//! For each documented command line this walks the legal builder chains
//! through the type-state API, asserting that they all assemble the same
//! arguments and spawn the same argv tail. The reference is the current
//! command reference:
//!
//! ```text
//! 2024.2 / 2025.1:
//! p4 [g-opts] diagnostics [-L limit]
//! p4 [g-opts] diagnostics [-l] [-s] [-a [-L limit]]
//!
//! 2025.2+:
//! p4 [g-opts] diagnostics [-D] [-j] [-J [-L limit]]
//! p4 [g-opts] diagnostics [-D] [-l] [-s [--strace-runtime secs]] [-p] [-t] [-a [-J [-L limit]]]
//! ```
//!
//! The command first appears in 2024.2, so the whole test target is compiled
//! out on older versions. `lt2025_2` marks the 2024.2/2025.1 documentation,
//! which lacks `-D`, `-j`, `-J`, `-p`, `-t`, and `--strace-runtime`.

#![cfg(not(feature = "lt2024_2"))]

use std::path::Path;

use perforce_cli::P4Cli;
use perforce_cli::cmd::Diagnostics;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_diagnostics() -> Diagnostics {
    Diagnostics::new(echo_bin("diagnostics"), GlobalOpts::new())
}

// Basic form ----------------------------------------------------------------
// `p4 [g-opts] diagnostics`
//
// The default `Unselected` state and the explicit `basic()` transition both
// spawn the bare command.

#[test]
fn basic_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics"], p4.cmd_diagnostics());
    expect_chain!(["diagnostics"], p4.cmd_diagnostics().basic());

    let mut cmd = echo_diagnostics();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics"]);

    let mut cmd = echo_diagnostics().basic();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics"]);
}

// Journal -------------------------------------------------------------------
// 2024.2 / 2025.1: `diagnostics [-L limit]` — the journal and log copy are
// always included in the basic form, so only `-L limit` exists and it is the
// entry point into the journal state.
// 2025.2+: `diagnostics [-J [-L limit]]` — `journal()`/`journal_with()` select
// `-J`, which does not exist before 2025.2.

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn journal_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics", "-J"], p4.cmd_diagnostics().journal());
    expect_chain!(
        ["diagnostics", "-J"],
        p4.cmd_diagnostics().basic().journal()
    );

    let mut cmd = echo_diagnostics().journal();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-J"]);
}

#[test]
fn journal_with_limit() {
    let p4 = P4Cli::default();

    #[cfg(not(feature = "lt2025_2"))]
    expect_chain!(
        ["diagnostics", "-J", "-L", "10m"],
        p4.cmd_diagnostics().basic().journal_with("10m")
    );
    #[cfg(feature = "lt2025_2")]
    expect_chain!(
        ["diagnostics", "-L", "10m"],
        p4.cmd_diagnostics().limit("10m")
    );

    #[cfg(not(feature = "lt2025_2"))]
    expect_chain!(
        ["diagnostics", "-J", "-L", "2g"],
        p4.cmd_diagnostics().journal().limit("2g")
    );
    #[cfg(feature = "lt2025_2")]
    expect_chain!(
        ["diagnostics", "-L", "2g"],
        p4.cmd_diagnostics().basic().limit("2g")
    );

    // The limit accepts anything that converts into a `String`.
    #[cfg(not(feature = "lt2025_2"))]
    let mut cmd = echo_diagnostics().basic().journal_with(String::from("10m"));
    #[cfg(feature = "lt2025_2")]
    let mut cmd = echo_diagnostics().limit(String::from("10m"));
    #[cfg(not(feature = "lt2025_2"))]
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["diagnostics", "-J", "-L", "10m"]
    );
    #[cfg(feature = "lt2025_2")]
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-L", "10m"]);
}

#[test]
fn journal_limit_accessors() {
    #[cfg(not(feature = "lt2025_2"))]
    let mut cmd = Diagnostics::new("p4", GlobalOpts::new())
        .basic()
        .journal_with("10m");
    #[cfg(feature = "lt2025_2")]
    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).limit("10m");

    assert_eq!(cmd.get_limit(), Some("10m"));

    cmd.set_limit("1t");
    assert_eq!(cmd.get_limit(), Some("1t"));

    #[cfg(not(feature = "lt2025_2"))]
    expect_args(&cmd, &["diagnostics", "-J", "-L", "1t"]);
    #[cfg(feature = "lt2025_2")]
    expect_args(&cmd, &["diagnostics", "-L", "1t"]);
}

// JSON format ---------------------------------------------------------------
// `-j` exists only in 2025.2+.

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn json_format_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics", "-j"], p4.cmd_diagnostics().json_format());
    expect_chain!(
        ["diagnostics", "-j", "-J"],
        p4.cmd_diagnostics().json_format().journal()
    );
    expect_chain!(
        ["diagnostics", "-j"],
        p4.cmd_diagnostics().basic().json_format(true)
    );
    expect_chain!(
        ["diagnostics"],
        p4.cmd_diagnostics().basic().json_format(false)
    );

    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).json_format();
    assert!(cmd.get_json_format());

    cmd.set_json_format(false);
    assert!(!cmd.get_json_format());
    expect_args(&cmd, &["diagnostics"]);

    let mut cmd = echo_diagnostics().json_format();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-j"]);
}

// Uncompressed --------------------------------------------------------------
// `-D` exists only in 2025.2+, and is injected before every other local
// option.

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn uncompressed_flag() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["diagnostics", "-D"],
        p4.cmd_diagnostics().uncompressed(true)
    );
    expect_chain!(
        ["diagnostics", "-D"],
        p4.cmd_diagnostics().uncompressed(true).basic()
    );
    expect_chain!(
        ["diagnostics", "-D", "-l"],
        p4.cmd_diagnostics().uncompressed(true).run_lsof()
    );
    expect_chain!(["diagnostics"], p4.cmd_diagnostics().uncompressed(false));

    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).uncompressed(true);
    assert!(cmd.get_uncompressed());

    cmd.set_uncompressed(false);
    assert!(!cmd.get_uncompressed());
    expect_args(&cmd, &["diagnostics"]);

    let mut cmd = echo_diagnostics().uncompressed(true);
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-D"]);
}

// Advance form --------------------------------------------------------------
// Every advance option is injectable in the documented order
// `-l -s[--strace-runtime] -p -t -a[-J[-L]]`. There is no empty advance
// transition: entering any advance-only option (or `-a`) assembles the same
// command line as the basic form until that option is selected.

#[test]
fn lsof_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics", "-l"], p4.cmd_diagnostics().run_lsof());
    // Once in the advance state, the boolean setter can toggle `-l` off again.
    expect_chain!(
        ["diagnostics"],
        p4.cmd_diagnostics().run_lsof().run_lsof(false)
    );

    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).run_lsof();
    assert!(cmd.get_run_lsof());

    cmd.set_run_lsof(false);
    assert!(!cmd.get_run_lsof());
    expect_args(&cmd, &["diagnostics"]);

    let mut cmd = echo_diagnostics().run_lsof();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-l"]);
}

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn ps_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics", "-p"], p4.cmd_diagnostics().run_ps());
    // Once in the advance state, the boolean setter can toggle `-p` off again.
    expect_chain!(["diagnostics"], p4.cmd_diagnostics().run_ps().run_ps(false));

    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).run_ps();
    assert!(cmd.get_run_ps());

    cmd.set_run_ps(false);
    assert!(!cmd.get_run_ps());
    expect_args(&cmd, &["diagnostics"]);
}

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn top_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics", "-t"], p4.cmd_diagnostics().run_top());
    // Once in the advance state, the boolean setter can toggle `-t` off again.
    expect_chain!(
        ["diagnostics"],
        p4.cmd_diagnostics().run_top().run_top(false)
    );

    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).run_top();
    assert!(cmd.get_run_top());

    cmd.set_run_top(false);
    assert!(!cmd.get_run_top());
    expect_args(&cmd, &["diagnostics"]);
}

#[test]
fn strace_form() {
    let p4 = P4Cli::default();

    expect_chain!(["diagnostics", "-s"], p4.cmd_diagnostics().run_strace());
    // The `-l` selection survives the transition to `-s`.
    expect_chain!(
        ["diagnostics", "-l", "-s"],
        p4.cmd_diagnostics().run_lsof().run_strace()
    );

    let mut cmd = echo_diagnostics().run_strace();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-s"]);
}

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn strace_runtime_option() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["diagnostics", "-s", "--strace-runtime", "30"],
        p4.cmd_diagnostics().run_strace_with(30)
    );
    expect_chain!(
        ["diagnostics", "-s", "--strace-runtime", "15"],
        p4.cmd_diagnostics().run_strace().strace_runtime(15)
    );
    expect_chain!(
        ["diagnostics", "-l", "-s", "--strace-runtime", "15"],
        p4.cmd_diagnostics().run_lsof().run_strace_with(15)
    );

    let mut cmd = Diagnostics::new("p4", GlobalOpts::new()).run_strace_with(30);
    assert_eq!(cmd.get_strace_runtime(), Some(30));

    cmd.set_strace_runtime(60);
    assert_eq!(cmd.get_strace_runtime(), Some(60));
    expect_args(&cmd, &["diagnostics", "-s", "--strace-runtime", "60"]);

    let cmd = cmd.strace_runtime(45);
    assert_eq!(cmd.get_strace_runtime(), Some(45));

    let mut cmd = echo_diagnostics().run_strace_with(30);
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["diagnostics", "-s", "--strace-runtime", "30"]
    );
}

// Basic diagnostic files of the advance form --------------------------------
// `-a [-J [-L limit]]`

#[test]
fn advance_with_basic_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["diagnostics", "-a"],
        p4.cmd_diagnostics().advance_with_basic()
    );
    expect_chain!(
        ["diagnostics", "-l", "-a"],
        p4.cmd_diagnostics().run_lsof().with_basic()
    );
    expect_chain!(
        ["diagnostics", "-s", "-a"],
        p4.cmd_diagnostics().run_strace().with_basic()
    );

    let mut cmd = echo_diagnostics().advance_with_basic();
    assert_eq!(spawned_args(&mut cmd, ()), ["diagnostics", "-a"]);
}

#[test]
fn advance_with_basic_journal() {
    let p4 = P4Cli::default();

    #[cfg(not(feature = "lt2025_2"))]
    expect_chain!(
        ["diagnostics", "-a", "-J"],
        p4.cmd_diagnostics().advance_with_basic().journal()
    );

    #[cfg(not(feature = "lt2025_2"))]
    expect_chain!(
        ["diagnostics", "-l", "-a", "-J", "-L", "10m"],
        p4.cmd_diagnostics()
            .run_lsof()
            .with_basic()
            .journal_with("10m")
    );
    #[cfg(feature = "lt2025_2")]
    expect_chain!(
        ["diagnostics", "-l", "-a", "-L", "10m"],
        p4.cmd_diagnostics().run_lsof().with_basic().limit("10m")
    );

    #[cfg(not(feature = "lt2025_2"))]
    let mut cmd = Diagnostics::new("p4", GlobalOpts::new())
        .advance_with_basic()
        .journal_with("2g");
    #[cfg(feature = "lt2025_2")]
    let mut cmd = Diagnostics::new("p4", GlobalOpts::new())
        .advance_with_basic()
        .limit("2g");
    assert_eq!(cmd.get_limit(), Some("2g"));

    cmd.set_limit("512k");
    assert_eq!(cmd.get_limit(), Some("512k"));

    #[cfg(not(feature = "lt2025_2"))]
    expect_args(&cmd, &["diagnostics", "-a", "-J", "-L", "512k"]);
    #[cfg(feature = "lt2025_2")]
    expect_args(&cmd, &["diagnostics", "-a", "-L", "512k"]);

    #[cfg(not(feature = "lt2025_2"))]
    let mut cmd = echo_diagnostics()
        .run_strace()
        .with_basic()
        .journal_with("1g");
    #[cfg(feature = "lt2025_2")]
    let mut cmd = echo_diagnostics().run_strace().with_basic().limit("1g");
    #[cfg(not(feature = "lt2025_2"))]
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["diagnostics", "-s", "-a", "-J", "-L", "1g"]
    );
    #[cfg(feature = "lt2025_2")]
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["diagnostics", "-s", "-a", "-L", "1g"]
    );
}

// Complete command line -----------------------------------------------------

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn all_options_in_syntax_order() {
    let p4 = P4Cli::default();

    expect_chain!(
        [
            "diagnostics",
            "-D",
            "-l",
            "-s",
            "--strace-runtime",
            "25",
            "-p",
            "-t",
            "-a",
            "-J",
            "-L",
            "2g"
        ],
        p4.cmd_diagnostics()
            .uncompressed(true)
            .run_lsof()
            .run_strace_with(25)
            .run_ps(true)
            .run_top(true)
            .with_basic()
            .journal_with("2g")
    );

    // The same selections entered in a different order converge on the
    // documented injection order.
    expect_chain!(
        [
            "diagnostics",
            "-D",
            "-l",
            "-s",
            "--strace-runtime",
            "25",
            "-p",
            "-t",
            "-a",
            "-J",
            "-L",
            "2g"
        ],
        p4.cmd_diagnostics()
            .advance_with_basic()
            .journal_with("2g")
            .run_top(true)
            .run_ps(true)
            .run_lsof(true)
            .run_strace_with(25)
            .uncompressed(true)
    );
}

#[cfg(feature = "lt2025_2")]
#[test]
fn all_options_in_syntax_order() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["diagnostics", "-l", "-s", "-a", "-L", "2g"],
        p4.cmd_diagnostics()
            .run_lsof()
            .run_strace()
            .with_basic()
            .limit("2g")
    );
}

// Global options ------------------------------------------------------------
// Every form accepts the global options, which are injected before the
// subcommand name.

#[test]
fn global_options_precede_subcommand() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let plain = Diagnostics::new("p4", global_opts.clone());
    expect_args(&plain, &["-p", "localhost:1666", "-q", "diagnostics"]);

    let basic = Diagnostics::new("p4", global_opts.clone()).basic();
    expect_args(&basic, &["-p", "localhost:1666", "-q", "diagnostics"]);

    let advance = Diagnostics::new("p4", global_opts.clone()).run_lsof();
    expect_args(
        &advance,
        &["-p", "localhost:1666", "-q", "diagnostics", "-l"],
    );

    #[cfg(not(feature = "lt2025_2"))]
    let journal = Diagnostics::new("p4", global_opts)
        .basic()
        .journal_with("10m");
    #[cfg(not(feature = "lt2025_2"))]
    expect_args(
        &journal,
        &[
            "-p",
            "localhost:1666",
            "-q",
            "diagnostics",
            "-J",
            "-L",
            "10m",
        ],
    );

    #[cfg(feature = "lt2025_2")]
    let journal = Diagnostics::new("p4", global_opts).limit("10m");
    #[cfg(feature = "lt2025_2")]
    expect_args(
        &journal,
        &["-p", "localhost:1666", "-q", "diagnostics", "-L", "10m"],
    );
}

// Accessors -----------------------------------------------------------------

#[test]
fn bin_accessors() {
    let mut cmd = Diagnostics::new("p4", GlobalOpts::new());
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
// // The command form is selected once: `basic`/`journal` and `run_*` cannot
// // be mixed.
// let _ = P4Cli::default().cmd_diagnostics().run_lsof().basic();
// let _ = P4Cli::default().cmd_diagnostics().journal().run_lsof();
//
// // `-a` is entered once, and the journal state only exists inside it.
// let _ = P4Cli::default().cmd_diagnostics().advance_with_basic().with_basic();
// let _ = P4Cli::default().cmd_diagnostics().journal().journal();
//
// // `journal()`/`journal_with()` name `-J`, which exists only in 2025.2+.
// let _ = P4Cli::default().cmd_diagnostics().journal();
// let _ = P4Cli::default().cmd_diagnostics().basic().journal();
// let _ = P4Cli::default().cmd_diagnostics().advance_with_basic().journal();
//
// // `limit(...)` is the pre-2025.2 journal entry point and is rejected
// // afterwards.
// let _ = P4Cli::default().cmd_diagnostics().limit("1g");
// let _ = P4Cli::default().cmd_diagnostics().basic().limit("1g");
// let _ = P4Cli::default().cmd_diagnostics().advance_with_basic().limit("1g");
//
// // `--strace-runtime` requires the `-s` state.
// let _ = P4Cli::default().cmd_diagnostics().run_lsof().strace_runtime(30);
//
// // `-p`, `-t`, `-j`, `-D`, and `--strace-runtime` are rejected by
// // 2024.2/2025.1 (`lt2025_2`).
// let _ = P4Cli::default().cmd_diagnostics().json_format();
// let _ = P4Cli::default().cmd_diagnostics().uncompressed(true);
// let _ = P4Cli::default().cmd_diagnostics().run_ps();
// let _ = P4Cli::default().cmd_diagnostics().run_top();
// let _ = P4Cli::default().cmd_diagnostics().run_strace_with(30);
//
// // `p4 diagnostics` itself does not exist before 2024.2 (`lt2024_2`).
// let _ = P4Cli::default().cmd_diagnostics();
