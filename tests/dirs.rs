//! Verification of every documented `p4 dirs` syntax form.
//!
//! For each documented command line this walks the legal builder chains
//! through the type-state API, asserting that they all assemble the same
//! arguments and spawn the same argv tail. The reference is the command
//! reference, whose synopsis grows over the supported versions:
//!
//! ```text
//! 2014.1 / 2015.2:
//! p4 [g-opts] dirs [-C -D -H] [-S stream] depot_directory[revRange]...
//!
//! 2016.1 / 2017.1:
//! p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directory[revRange]...
//!
//! 2017.2 / 2018.1:
//! p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directory[revRange]...
//! p4 [g-opts] dirs [-S stream] [-i] depot_directory[revRange]...
//!
//! 2018.1 / 2022.1:
//! p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directories*[revSpec]
//!
//! 2022.2 / 2025.2:
//! p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directories*[revSpec]
//! p4 [g-opts] dirs [-D] [--streamviews] depot_directories*[revSpec]
//! ```
//!
//! `-i` is added in 2016.1 (`lt2016_1`), `--streamviews` in 2022.2
//! (`lt2022_2`), and `--ignore-changeview` in 2025.2 (`lt2025_2`).

use perforce_cli::P4Cli;
use perforce_cli::cmd::Dirs;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_dirs() -> Dirs {
    Dirs::new(echo_bin("dirs"), GlobalOpts::new())
}

// Bare form -----------------------------------------------------------------
// `p4 [g-opts] dirs`

#[test]
fn bare_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs"], p4.cmd_dirs());

    let bin = echo_bin("dirs");
    let cmd = Dirs::new(&bin, GlobalOpts::new());
    assert_eq!(cmd.get_bin(), bin.as_path());
    assert!(!cmd.get_include_deleted());

    let mut cmd = echo_dirs();
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs"]);
}

// Client view ---------------------------------------------------------------
// `-C` and `-H` are always available and select the workspace view form.

#[test]
fn client_mapped_only_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs", "-C"], p4.cmd_dirs().client_mapped_only());

    let mut cmd = Dirs::new("p4", GlobalOpts::new()).client_mapped_only();
    assert!(cmd.get_client_mapped_only());
    assert!(!cmd.get_client_have_list_only());
    expect_args(&cmd, &["dirs", "-C"]);

    cmd.set_client_mapped_only(false);
    assert!(!cmd.get_client_mapped_only());
    expect_args(&cmd, &["dirs"]);

    cmd.set_client_mapped_only(true);
    expect_args(&cmd, &["dirs", "-C"]);

    let mut cmd = echo_dirs().client_mapped_only();
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs", "-C"]);
}

#[test]
fn client_have_list_only_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs", "-H"], p4.cmd_dirs().client_have_list_only());

    let mut cmd = Dirs::new("p4", GlobalOpts::new()).client_have_list_only();
    assert!(cmd.get_client_have_list_only());
    expect_args(&cmd, &["dirs", "-H"]);

    cmd.set_client_have_list_only(false);
    expect_args(&cmd, &["dirs"]);

    let mut cmd = echo_dirs().client_have_list_only();
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs", "-H"]);
}

// Stream view ---------------------------------------------------------------
// `p4 [g-opts] dirs [-S stream]`

#[test]
fn stream_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs", "-S", "main"], p4.cmd_dirs().stream("main"));

    let mut cmd = Dirs::new("p4", GlobalOpts::new()).stream("main");
    assert!(!cmd.get_client_mapped_only());
    assert_eq!(cmd.get_stream(), Some("main"));
    expect_args(&cmd, &["dirs", "-S", "main"]);

    cmd.set_stream(String::from("release"));
    assert_eq!(cmd.get_stream(), Some("release"));
    expect_args(&cmd, &["dirs", "-S", "release"]);

    let mut cmd = echo_dirs().stream("main");
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs", "-S", "main"]);
}

// `-D` ----------------------------------------------------------------------
// `[-D]` is shared by every form.

#[test]
fn include_deleted_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs", "-D"], p4.cmd_dirs().include_deleted(true));
    expect_chain!(
        ["dirs", "-D", "-C"],
        p4.cmd_dirs().client_mapped_only().include_deleted(true)
    );

    let mut cmd = Dirs::new("p4", GlobalOpts::new()).include_deleted(true);
    assert!(cmd.get_include_deleted());
    expect_args(&cmd, &["dirs", "-D"]);

    cmd.set_include_deleted(false);
    assert!(!cmd.get_include_deleted());
    expect_args(&cmd, &["dirs"]);

    let mut cmd = echo_dirs().client_mapped_only().include_deleted(true);
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs", "-D", "-C"]);
}

// `-i` ----------------------------------------------------------------------
// Added in 2016.1.

#[cfg(not(feature = "lt2016_1"))]
#[test]
fn case_insensitive_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs", "-i"], p4.cmd_dirs().case_insensitive());
    expect_chain!(
        ["dirs", "-C", "-i"],
        p4.cmd_dirs().client_mapped_only().case_insensitive(true)
    );

    let mut cmd = Dirs::new("p4", GlobalOpts::new()).case_insensitive();
    assert!(cmd.get_case_insensitive());
    expect_args(&cmd, &["dirs", "-i"]);

    cmd.set_case_insensitive(false);
    assert!(!cmd.get_case_insensitive());
    expect_args(&cmd, &["dirs"]);

    let mut cmd = echo_dirs().client_mapped_only().case_insensitive(true);
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs", "-C", "-i"]);
}

// `--streamviews` -----------------------------------------------------------
// Added in 2022.2, as its own form: `dirs [-D] [--streamviews]`.

#[cfg(not(feature = "lt2022_2"))]
#[test]
fn streamviews_form() {
    let p4 = P4Cli::default();

    expect_chain!(["dirs", "--streamviews"], p4.cmd_dirs().stream_view());
    expect_chain!(
        ["dirs", "-D", "--streamviews"],
        p4.cmd_dirs().stream_view().include_deleted(true)
    );

    let cmd = Dirs::new("p4", GlobalOpts::new()).stream_view();
    expect_args(&cmd, &["dirs", "--streamviews"]);

    let mut cmd = echo_dirs().stream_view().include_deleted(true);
    assert_eq!(spawned_args(&mut cmd, ()), ["dirs", "-D", "--streamviews"]);
}

// `--ignore-changeview` -----------------------------------------------------
// Added in 2025.2.

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn ignore_changeview_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["dirs", "--ignore-changeview"],
        p4.cmd_dirs().ignore_changeview()
    );
    expect_chain!(
        ["dirs", "-C", "--ignore-changeview"],
        p4.cmd_dirs().client_mapped_only().ignore_changeview(true)
    );

    let mut cmd = Dirs::new("p4", GlobalOpts::new()).ignore_changeview();
    assert!(cmd.get_ignore_changeview());
    expect_args(&cmd, &["dirs", "--ignore-changeview"]);

    cmd.set_ignore_changeview(false);
    assert!(!cmd.get_ignore_changeview());
    expect_args(&cmd, &["dirs"]);

    let mut cmd = echo_dirs().client_mapped_only().ignore_changeview(true);
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["dirs", "-C", "--ignore-changeview"]
    );
}

// Argument order ------------------------------------------------------------
// `-D` is injected first, then the selected form's own options in the order
// of the documented synopsis: `-C`, `-H`, `-S stream`, `-i`,
// `--ignore-changeview`.

#[test]
fn workspace_view_arguments() {
    // Older versions expose neither `set_case_insensitive` nor
    // `set_ignore_changeview`, so `mut` is unused there.
    #[allow(unused_mut)]
    let mut cmd = Dirs::new("p4", GlobalOpts::new())
        .client_mapped_only()
        .client_have_list_only(true)
        .stream("main")
        .include_deleted(true);

    #[cfg(not(feature = "lt2016_1"))]
    cmd.set_case_insensitive(true);
    #[cfg(not(feature = "lt2025_2"))]
    cmd.set_ignore_changeview(true);

    #[cfg(feature = "lt2016_1")]
    expect_args(&cmd, &["dirs", "-D", "-C", "-H", "-S", "main"]);
    #[cfg(all(not(feature = "lt2016_1"), feature = "lt2025_2"))]
    expect_args(&cmd, &["dirs", "-D", "-C", "-H", "-S", "main", "-i"]);
    #[cfg(not(feature = "lt2025_2"))]
    expect_args(
        &cmd,
        &[
            "dirs",
            "-D",
            "-C",
            "-H",
            "-S",
            "main",
            "-i",
            "--ignore-changeview",
        ],
    );
}

// Directories on spawn ------------------------------------------------------
// `p4 [g-opts] dirs depot_directories*[revSpec]`

#[test]
fn directories_on_spawn() {
    let mut cmd = echo_dirs();
    let args = spawned_args(&mut cmd, (vec!["//depot/main/*"],));
    assert_eq!(args, ["dirs", "//depot/main/*"]);

    let mut cmd = echo_dirs().client_mapped_only();
    let args = spawned_args(&mut cmd, (vec!["//depot/main/*", "//depot/main/mobile/*"],));
    assert_eq!(
        args,
        ["dirs", "-C", "//depot/main/*", "//depot/main/mobile/*"]
    );
}

// Global options ------------------------------------------------------------
// Global options come before the subcommand name and its local options.

#[test]
fn global_opts_precede_local_opts() {
    let cmd = Dirs::new(
        "p4",
        GlobalOpts::new().port("localhost:1666").quiet_mode(true),
    )
    .stream("main")
    .include_deleted(true);

    expect_args(
        &cmd,
        &["-p", "localhost:1666", "-q", "dirs", "-D", "-S", "main"],
    );
}
