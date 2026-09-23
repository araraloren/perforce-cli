//! Example of the `p4 diff` command using the builder pattern.
//!
//! `p4 diff` has three mutually exclusive modes, tracked at compile time by
//! the command's type parameter:
//!
//! - [`Diff<Unselected>`]: no mode selected yet — enter workspace mode with
//!   [`Diff::force`], [`Diff::differing_only`], or [`Diff::diff_nontext`], or
//!   stream-spec mode with [`Diff::stream_spec_mode`].
//! - [`Diff<WorkspaceMode<M>>`]: diff workspace files against the depot. The
//!   inner `M` parameter isolates `-m max` ([`WorkspaceRegularMode`]) from
//!   `-soptions` ([`WorkspaceDisplayMode`]).
//! - [`Diff<StreamSpecMode>`]: diff stream specs via `-As`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example diff
//! ```

use std::ffi::OsStr;

use perforce_cli::P4Cli;
use perforce_cli::cmd::DiffOptionsBuilder;
use perforce_cli::cmd::diff::DisplayOptions;
use perforce_cli::spawn::{ParameterizedOutput, ParameterizedSpawn};

fn main() -> std::io::Result<()> {
    let p4 = P4Cli::default();

    // Workspace mode: force a diff against head, use the unified format with
    // whitespace-insensitive comparison, and limit output to the first 10
    // files. `force(true)` transitions the command into `WorkspaceMode`, and
    // `limit(10)` further transitions it into `WorkspaceRegularMode` (where
    // `-soptions` is unavailable).
    let mut diff = p4
        .diff()
        .force(true)
        .diff_options(DiffOptionsBuilder::unified(None).ignore_all_whitespace())
        .limit(10);

    let files = [OsStr::new("//depot/project/src/...")];
    let output = diff.output_with((&files,))?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Display mode: instead of full diffs, list only the unopened files that
    // differ from the depot. `display_options` transitions `WorkspaceMode`
    // into `WorkspaceDisplayMode`, where `-m max` is unavailable.
    let mut list_changed = p4
        .diff()
        .force(true)
        .display_options(DisplayOptions::UnopenedChanged);

    let output = list_changed.output_with((&files,))?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Stream-spec mode: diff a privately edited stream spec against the head
    // version of another stream. `stream_spec_mode` transitions the command
    // into `StreamSpecMode`; the stream spec is passed to `spawn_with`.
    let mut stream_diff = p4.diff().stream_spec_mode();
    let mut child = stream_diff.spawn_with(("//streams/main@head",))?;
    child.wait()?;

    Ok(())
}
