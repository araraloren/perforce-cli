//! Verification of the `p4 fixes` builder.
//!
//! ```text
//! p4 [g-opts] fixes [-i] [-m max] [-j job] [-c changelist] [FileSpec [revSpec]]
//! ```
//!
//! Every option is optional and file arguments are optional, so both
//! `spawn_with(())` and `spawn_with((specs,))` are available.

use perforce_cli::P4Cli;
use perforce_cli::cmd::Fixes;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_fixes() -> Fixes {
    Fixes::new(echo_bin("fixes"), GlobalOpts::new())
}

// ---------------------------------------------------------------------------
// Bare form
// ---------------------------------------------------------------------------

#[test]
fn bare_form() {
    let p4 = P4Cli::default();

    expect_chain!(["fixes"], p4.cmd_fixes());

    let mut cmd = echo_fixes();
    assert_eq!(spawned_args(&mut cmd, ()), ["fixes"]);
}

// ---------------------------------------------------------------------------
// File arguments
// ---------------------------------------------------------------------------

#[test]
fn file_arguments() {
    let mut cmd = echo_fixes();
    assert_eq!(
        spawned_args(&mut cmd, (["//depot/proj1/...", "file.c#5"],)),
        ["fixes", "//depot/proj1/...", "file.c#5"]
    );

    let mut cmd = echo_fixes();
    assert_eq!(
        spawned_args(&mut cmd, (["file.c#5,5"],)),
        ["fixes", "file.c#5,5"]
    );
}

// ---------------------------------------------------------------------------
// Options
//
// All options combine freely and inject in synopsis order `-i -m -j -c`
// before the file arguments.
// ---------------------------------------------------------------------------

#[test]
fn include_integrations_flag() {
    let p4 = P4Cli::default();

    expect_chain!(["fixes", "-i"], p4.cmd_fixes().include_integrations(true));

    // A false flag leaves the option out entirely.
    expect_chain!(["fixes"], p4.cmd_fixes().include_integrations(false));
}

#[test]
fn limit_option() {
    let p4 = P4Cli::default();

    expect_chain!(["fixes", "-m", "5"], p4.cmd_fixes().limit(5));
}

#[test]
fn job_name_option() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["fixes", "-j", "job000141"],
        p4.cmd_fixes().job_name("job000141")
    );
}

#[test]
fn changelist_option() {
    let p4 = P4Cli::default();

    expect_chain!(["fixes", "-c", "414"], p4.cmd_fixes().changelist("414"));
}

#[test]
fn options_combine_in_synopsis_order() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["fixes", "-i", "-m", "5", "-j", "job000141", "-c", "414"],
        p4.cmd_fixes()
            .changelist("414")
            .job_name("job000141")
            .limit(5)
            .include_integrations(true)
    );

    let mut cmd = echo_fixes()
        .include_integrations(true)
        .limit(5)
        .job_name("job000141")
        .changelist("414");
    assert_eq!(
        spawned_args(&mut cmd, ([".c"],)),
        [
            "fixes",
            "-i",
            "-m",
            "5",
            "-j",
            "job000141",
            "-c",
            "414",
            ".c"
        ]
    );
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

#[test]
fn accessors() {
    let mut cmd = echo_fixes()
        .include_integrations(true)
        .limit(5)
        .job_name("job000141")
        .changelist("414");

    assert!(cmd.get_include_integrations());
    assert_eq!(cmd.get_limit(), Some(5));
    assert_eq!(cmd.get_job_name().map(String::as_str), Some("job000141"));
    assert_eq!(cmd.get_changelist().map(String::as_str), Some("414"));

    cmd.set_include_integrations(false)
        .set_limit(9)
        .set_job_name("job000142")
        .set_changelist("415");

    assert!(!cmd.get_include_integrations());
    assert_eq!(cmd.get_limit(), Some(9));
    assert_eq!(cmd.get_job_name().map(String::as_str), Some("job000142"));
    assert_eq!(cmd.get_changelist().map(String::as_str), Some("415"));
}

// ---------------------------------------------------------------------------
// Global options
//
// Global options inject before the subcommand name.
// ---------------------------------------------------------------------------

#[test]
fn global_options() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let cmd = Fixes::new("p4", global_opts.clone());
    expect_args(&cmd, &["-p", "localhost:1666", "-q", "fixes"]);

    let cmd = Fixes::new("p4", global_opts)
        .include_integrations(true)
        .limit(5);
    expect_args(
        &cmd,
        &["-p", "localhost:1666", "-q", "fixes", "-i", "-m", "5"],
    );
}
