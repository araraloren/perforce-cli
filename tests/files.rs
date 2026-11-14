//! Integration tests for `p4 files`.
//!
//! Regular options inject in synopsis order. The `-U` and `--streamviews`
//! forms use distinct type states, so they cannot be selected together.

use perforce_cli::P4Cli;
use perforce_cli::cmd::Files;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

fn echo_files() -> Files {
    Files::new(echo_bin("files"), GlobalOpts::new())
}

#[test]
fn regular_form_and_file_arguments() {
    let p4 = P4Cli::default();
    expect_chain!(["files"], p4.cmd_files());

    let mut cmd = echo_files();
    assert_eq!(
        spawned_args(&mut cmd, (["//depot/...", "file.c#5"],)),
        ["files", "//depot/...", "file.c#5"]
    );
}

#[test]
fn regular_flags() {
    let p4 = P4Cli::default();

    expect_chain!(["files", "-a"], p4.cmd_files().all(true));
    expect_chain!(["files"], p4.cmd_files().all(false));
    expect_chain!(["files", "-A"], p4.cmd_files().archive_depot_only(true));
    expect_chain!(["files", "-e"], p4.cmd_files().exclude_deleted(true));
    expect_chain!(["files", "-m", "5"], p4.cmd_files().limit(5));
}

#[cfg(not(feature = "lt2016_1"))]
#[test]
fn case_insensitive_flag() {
    let p4 = P4Cli::default();
    expect_chain!(["files", "-i"], p4.cmd_files().case_insensitive(true));
    expect_chain!(["files"], p4.cmd_files().case_insensitive(false));
}

#[cfg(not(feature = "lt2025_2"))]
#[test]
fn ignore_changeview_flag() {
    let p4 = P4Cli::default();
    expect_chain!(
        ["files", "--ignore-changeview"],
        p4.cmd_files().ignore_changeview(true)
    );
    expect_chain!(["files"], p4.cmd_files().ignore_changeview(false));
}

#[test]
fn options_combine_in_synopsis_order() {
    #[allow(unused_mut)]
    let mut cmd = echo_files()
        .limit(5)
        .exclude_deleted(true)
        .archive_depot_only(true)
        .all(true);

    #[cfg(not(feature = "lt2016_1"))]
    cmd.set_case_insensitive(true);
    #[cfg(not(feature = "lt2025_2"))]
    cmd.set_ignore_changeview(true);

    #[cfg(feature = "lt2016_1")]
    expect_args(&cmd, &["files", "-a", "-A", "-e", "-m", "5"]);
    #[cfg(all(not(feature = "lt2016_1"), feature = "lt2025_2"))]
    expect_args(&cmd, &["files", "-a", "-A", "-e", "-i", "-m", "5"]);
    #[cfg(not(feature = "lt2025_2"))]
    expect_args(
        &cmd,
        &[
            "files",
            "-a",
            "-A",
            "-e",
            "-i",
            "-m",
            "5",
            "--ignore-changeview",
        ],
    );
}

#[test]
fn unload_depot_form() {
    let p4 = P4Cli::default();
    expect_chain!(["files", "-U"], p4.cmd_files().unload_depot());

    let mut cmd = echo_files().all(true).unload_depot();
    assert!(cmd.get_all());
    assert_eq!(
        spawned_args(&mut cmd, (["unload.c"],)),
        ["files", "-a", "-U", "unload.c"]
    );
}

#[cfg(not(feature = "lt2022_2"))]
#[test]
fn stream_view_form() {
    let p4 = P4Cli::default();
    expect_chain!(["files", "--streamviews"], p4.cmd_files().stream_view());

    let mut cmd = echo_files().all(true).stream_view();
    assert_eq!(
        spawned_args(&mut cmd, (["//depot/main/..."],)),
        ["files", "-a", "--streamviews", "//depot/main/..."]
    );
}

#[cfg(not(feature = "lt2026_1"))]
#[test]
fn alternate_form_option_order() {
    let p4 = P4Cli::default();

    expect_chain!(
        [
            "files",
            "-a",
            "-A",
            "-e",
            "-i",
            "-m",
            "5",
            "-U",
            "--ignore-changeview"
        ],
        p4.cmd_files()
            .ignore_changeview(true)
            .limit(5)
            .case_insensitive(true)
            .exclude_deleted(true)
            .archive_depot_only(true)
            .all(true)
            .unload_depot()
    );

    expect_chain!(
        [
            "files",
            "-a",
            "-A",
            "-e",
            "-i",
            "-m",
            "5",
            "--streamviews",
            "--ignore-changeview"
        ],
        p4.cmd_files()
            .stream_view()
            .ignore_changeview(true)
            .limit(5)
            .case_insensitive(true)
            .exclude_deleted(true)
            .archive_depot_only(true)
            .all(true)
    );
}

#[test]
fn accessors() {
    #[allow(unused_mut)]
    let mut cmd = echo_files()
        .all(true)
        .archive_depot_only(true)
        .exclude_deleted(true)
        .limit(5);

    assert!(cmd.get_all());
    assert!(cmd.get_archive_depot_only());
    assert!(cmd.get_exclude_deleted());
    assert_eq!(cmd.get_limit(), Some(5));

    cmd.set_all(false)
        .set_archive_depot_only(false)
        .set_exclude_deleted(false)
        .set_limit(9);

    assert!(!cmd.get_all());
    assert!(!cmd.get_archive_depot_only());
    assert!(!cmd.get_exclude_deleted());
    assert_eq!(cmd.get_limit(), Some(9));

    #[cfg(not(feature = "lt2016_1"))]
    {
        cmd.set_case_insensitive(true);
        assert!(cmd.get_case_insensitive());
        cmd.set_case_insensitive(false);
        assert!(!cmd.get_case_insensitive());
    }

    #[cfg(not(feature = "lt2025_2"))]
    {
        cmd.set_ignore_changeview(true);
        assert!(cmd.get_ignore_changeview());
        cmd.set_ignore_changeview(false);
        assert!(!cmd.get_ignore_changeview());
    }
}

#[test]
fn global_options_inject_before_subcommand() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let cmd = Files::new("p4", global_opts.clone());
    expect_args(&cmd, &["-p", "localhost:1666", "-q", "files"]);

    let cmd = Files::new("p4", global_opts).all(true).limit(5);
    expect_args(
        &cmd,
        &["-p", "localhost:1666", "-q", "files", "-a", "-m", "5"],
    );
}
