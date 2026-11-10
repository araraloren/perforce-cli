//! Verification of the `p4 fetch` type-state design.
//!
//! The builder starts in the submitted-files syntax form with no mode (`M`),
//! no submitted-only option (`O`), and no target (`S`) selected. Every
//! selection encodes the chosen alternative in a type parameter, so the
//! `p4 fetch -s shelf` form only stays reachable while `M`, `O`, and `S` are
//! all `Unselected`.
//!
//! ```text
//! p4 [g-opts] fetch [-r remotespec] [-m depth] [-v -k] [-n | -t] [-Ox] [-S stream | filespec]
//! p4 [g-opts] fetch [-r remotespec] [-v] [-n] [-Ox] -s shelf
//! ```
//!
//! `-u` replaces `-t` in 2015.1 (`lt2015_2`), `-s shelf` and `-Ox` appear in
//! 2016.1 (`lt2016_1`), `--remote-user` in 2020.2 (`lt2020_2`), and filespec
//! becomes `FileSpec[revSpec]` in 2022.2 (`lt2022_2`).

#![cfg(not(feature = "lt2015_1"))]

use perforce_cli::P4Cli;
use perforce_cli::cmd::fetch::{Fetch, Submitted};
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_fetch() -> Fetch<Submitted> {
    Fetch::new(echo_bin("fetch"), GlobalOpts::new())
}

// ---------------------------------------------------------------------------
// Bare form
// ---------------------------------------------------------------------------

#[test]
fn bare_form() {
    let p4 = P4Cli::default();

    expect_chain!(["fetch"], p4.cmd_fetch());

    let mut cmd = echo_fetch();
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch"]);
}

// ---------------------------------------------------------------------------
// Submitted-only options
//
// `-m` and `-k` move the builder into
// `Fetch<Submitted<M, SubmittedOnlyOpts, S>>`: from there `shelved` is gone at
// the type level, because `p4 fetch -s shelf` accepts neither option.
// ---------------------------------------------------------------------------

#[test]
fn depth_moves_to_submitted_only_state() {
    let mut cmd = echo_fetch().depth(2);

    assert_eq!(cmd.get_depth(), Some(2));
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-m", "2"]);
}

#[test]
fn suppress_workspace_sync_moves_to_submitted_only_state() {
    let mut cmd = echo_fetch().suppress_workspace_sync(true);

    assert!(cmd.get_suppress_workspace_sync());
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-k"]);

    // `false` still enters the state, but injects nothing.
    let mut cmd = echo_fetch().suppress_workspace_sync(false);

    assert!(!cmd.get_suppress_workspace_sync());
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch"]);
}

#[test]
fn submitted_only_options_accumulate() {
    let mut cmd = echo_fetch().depth(3).suppress_workspace_sync(true);

    assert_eq!(cmd.get_depth(), Some(3));
    assert!(cmd.get_suppress_workspace_sync());
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-m", "3", "-k"]);
}

// ---------------------------------------------------------------------------
// Shared options
//
// `-r` and `--remote-user` live on `Fetch<M>`, so they stay available in
// every state and never affect `M`, `O`, or `S`.
// ---------------------------------------------------------------------------

#[test]
fn remote_spec_is_shared_by_both_forms() {
    let mut cmd = echo_fetch().remote_spec("origin");

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-r", "origin"]);
}

#[test]
#[cfg(not(feature = "lt2020_2"))]
fn remote_user_is_shared_by_both_forms() {
    let mut cmd = echo_fetch().remote_user("bruno");

    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["fetch", "--remote-user", "bruno"]
    );
}

#[test]
fn verbose_is_shared_by_both_forms() {
    let mut cmd = echo_fetch().verbose(true);

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-v"]);
}

#[test]
#[cfg(not(feature = "lt2016_1"))]
fn output_flags() {
    let mut cmd = echo_fetch()
        .output_changelist(true)
        .output_files(true)
        .output_integration(true);

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-Oc", "-Of", "-Oi"]);
}

// ---------------------------------------------------------------------------
// Mode
// ---------------------------------------------------------------------------

#[test]
fn dry_run_mode() {
    let mut cmd = echo_fetch().dry_run();

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-n"]);
}

#[test]
fn tangent_relocate_mode() {
    let mut cmd = echo_fetch().tangent_relocate();

    #[cfg(feature = "lt2015_2")]
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-u"]);
    #[cfg(not(feature = "lt2015_2"))]
    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-t"]);
}

// ---------------------------------------------------------------------------
// Target
// ---------------------------------------------------------------------------

#[test]
fn stream_target() {
    let mut cmd = echo_fetch().stream("//stream/dev");

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-S", "//stream/dev"]);
}

#[test]
fn file_specs_via_spawn_input_come_last() {
    let mut cmd = echo_fetch().depth(1).verbose(true);

    assert_eq!(
        spawned_args(&mut cmd, (vec!["//depot/main/...", "//depot/rel/..."],)),
        [
            "fetch",
            "-m",
            "1",
            "-v",
            "//depot/main/...",
            "//depot/rel/..."
        ]
    );
}

// ---------------------------------------------------------------------------
// Argument order
// ---------------------------------------------------------------------------

#[test]
fn arguments_follow_the_synopsis_order() {
    let mut cmd = echo_fetch()
        .remote_spec("origin")
        .depth(5)
        .verbose(true)
        .suppress_workspace_sync(true)
        .dry_run()
        .stream("//stream/dev");

    assert_eq!(
        spawned_args(&mut cmd, ()),
        [
            "fetch",
            "-r",
            "origin",
            "-m",
            "5",
            "-v",
            "-k",
            "-n",
            "-S",
            "//stream/dev",
        ]
    );
}

#[test]
#[cfg(not(feature = "lt2020_2"))]
fn remote_user_precedes_depth() {
    let mut cmd = echo_fetch()
        .remote_spec("origin")
        .remote_user("bruno")
        .depth(1);

    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["fetch", "-r", "origin", "--remote-user", "bruno", "-m", "1",]
    );
}

// ---------------------------------------------------------------------------
// Shelved form
//
// Only reachable while `M`, `O`, and `S` are all `Unselected`; shared options
// do not change those type parameters.
// ---------------------------------------------------------------------------

#[test]
#[cfg(not(feature = "lt2016_1"))]
fn shelved_form() {
    let mut cmd = echo_fetch().shelved("42");

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-s", "42"]);
}

#[test]
#[cfg(not(feature = "lt2016_1"))]
fn shelved_keeps_shared_options() {
    let mut cmd = echo_fetch()
        .remote_spec("origin")
        .verbose(true)
        .shelved("42");

    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["fetch", "-r", "origin", "-v", "-s", "42"]
    );
}

#[test]
#[cfg(not(feature = "lt2016_1"))]
fn shelved_dry_run() {
    let mut cmd = echo_fetch().shelved("42").dry_run(true);

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-n", "-s", "42"]);
}

#[test]
#[cfg(not(feature = "lt2016_1"))]
fn dry_run_carries_over_to_shelved() {
    let mut cmd = echo_fetch().dry_run();

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-n"]);

    let mut cmd = echo_fetch().dry_run().shelved("42");

    assert_eq!(spawned_args(&mut cmd, ()), ["fetch", "-n", "-s", "42"]);
}

// ---------------------------------------------------------------------------
// Global options
// ---------------------------------------------------------------------------

#[test]
fn global_opts_are_injected_once() {
    let cmd = Fetch::new(
        echo_bin("fetch"),
        GlobalOpts::new().port("localhost:1666").quiet_mode(true),
    );

    expect_args(&cmd, &["-p", "localhost:1666", "-q", "fetch"]);
}
