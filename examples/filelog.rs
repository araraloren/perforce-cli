//! Example of the `p4 filelog` command using the builder pattern.
//!
//! `p4 filelog` prints detailed information about the revisions of files and
//! requires at least one file or file pattern. Two orthogonal dimensions are
//! tracked at compile time by the command's type parameters:
//!
//! - the description output: [`FileLog::full_description`](perforce_cli::cmd::filelog::FileLog::full_description)
//!   (`-l`) or [`FileLog::truncated_description`](perforce_cli::cmd::filelog::FileLog::truncated_description)
//!   (`-L`)
//! - the content history mode: [`FileLog::content_history`](perforce_cli::cmd::filelog::FileLog::content_history)
//!   (`-h`), which unlocks [`FileLog::skip_promoted_tasks`](perforce_cli::cmd::filelog::FileLog::skip_promoted_tasks)
//!   (`-p`)
//!
//! Run with:
//!
//! ```text
//! cargo run --example filelog
//! ```

use perforce_cli::P4Cli;
use perforce_cli::spawn::{OutputExt1, ParameterizedSpawn};
use std::ffi::OsStr;

fn main() -> std::io::Result<()> {
    let p4 = P4Cli::default();

    // Revision history for every file under //depot/project: follow the file
    // history across branches (`-i`), display the time as well as the date
    // (`-t`), and list only the first 10 changes per file (`-m 10`).
    let mut filelog = p4
        .cmd_filelog()
        .follow_branches(true)
        .include_time(true)
        .limit(10);

    let files = [OsStr::new("//depot/project/...")];
    let output = filelog.output(&files)?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Long output: `full_description()` transitions the command into the
    // `FullDescription` state (`-l`), printing the full text of each
    // changelist description. A revision range limits the listed revisions,
    // e.g. changelists 100 through 120.
    let mut long_log = p4.cmd_filelog().full_description();

    let output = long_log.output(&["//depot/project/main.c@100,@120"])?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Content history: `content_history()` transitions the command into the
    // `DisplayContentHistory` state (`-h`), which unlocks `-p` to skip the
    // content of promoted task streams. Only revisions submitted at
    // changelist 4321 are displayed (`-c 4321`).
    let mut history = p4
        .cmd_filelog()
        .content_history()
        .skip_promoted_tasks(true)
        .changelist("4321");

    let mut child = history.spawn_with((&files,))?;
    child.wait()?;

    Ok(())
}
