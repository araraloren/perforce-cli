//! Verification of the `p4 fix` builder.
//!
//! ```text
//! p4 [g-opts] fix [-d] [-s status] -c changelist jobName ...
//! ```
//!
//! `-c changelist` is required, so the builder starts in the `Unselected`
//! state and only [`Fix::changelist`] moves it to `Fix<Changelist>`, the only
//! state that implements `ParameterizedSpawn`.

use perforce_cli::P4Cli;
use perforce_cli::cmd::fix::Fix;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_fix() -> Fix {
    Fix::new(echo_bin("fix"), GlobalOpts::new())
}

// ---------------------------------------------------------------------------
// Bare form
// ---------------------------------------------------------------------------

#[test]
fn bare_form() {
    let p4 = P4Cli::default();

    expect_chain!(["fix", "-c", "201"], p4.cmd_fix().changelist("201"));

    let mut cmd = echo_fix().changelist("201");
    assert_eq!(
        spawned_args(&mut cmd, (["job000141", "job002034"],)),
        ["fix", "-c", "201", "job000141", "job002034"]
    );
}

// ---------------------------------------------------------------------------
// Options
//
// `-d` and `-s status` combine freely and inject in synopsis order before the
// required `-c changelist`.
// ---------------------------------------------------------------------------

#[test]
fn delete_flag() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["fix", "-d", "-c", "201"],
        p4.cmd_fix().delete_fix_record(true).changelist("201")
    );

    // A false flag leaves the option out entirely.
    expect_chain!(
        ["fix", "-c", "201"],
        p4.cmd_fix().delete_fix_record(false).changelist("201")
    );
}

#[test]
fn status_option() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["fix", "-s", "suspended", "-c", "201"],
        p4.cmd_fix().status("suspended").changelist("201")
    );
}

#[test]
fn options_combine_in_synopsis_order() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["fix", "-d", "-s", "same", "-c", "201"],
        p4.cmd_fix()
            .status("same")
            .delete_fix_record(true)
            .changelist("201")
    );
    expect_chain!(
        ["fix", "-d", "-s", "same", "-c", "201"],
        p4.cmd_fix()
            .delete_fix_record(true)
            .status("same")
            .changelist("201")
    );

    let mut cmd = echo_fix()
        .delete_fix_record(true)
        .status("same")
        .changelist("201");
    assert_eq!(
        spawned_args(&mut cmd, (["job000141"],)),
        ["fix", "-d", "-s", "same", "-c", "201", "job000141"]
    );
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

#[test]
fn accessors() {
    let mut cmd = echo_fix().delete_fix_record(true).status("same");
    assert!(cmd.get_delete_fix_record());
    assert_eq!(cmd.get_status().map(String::as_str), Some("same"));

    cmd.set_delete_fix_record(false).set_status("suspended");
    assert!(!cmd.get_delete_fix_record());
    assert_eq!(cmd.get_status().map(String::as_str), Some("suspended"));
}

// ---------------------------------------------------------------------------
// Global options
//
// Global options inject before the subcommand name.
// ---------------------------------------------------------------------------

#[test]
fn global_options() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let cmd = Fix::new("p4", global_opts.clone()).changelist("201");
    expect_args(&cmd, &["-p", "localhost:1666", "-q", "fix", "-c", "201"]);

    let cmd = Fix::new("p4", global_opts)
        .delete_fix_record(true)
        .status("same")
        .changelist("201");
    expect_args(
        &cmd,
        &[
            "-p",
            "localhost:1666",
            "-q",
            "fix",
            "-d",
            "-s",
            "same",
            "-c",
            "201",
        ],
    );
}
