//! Example of the `p4 print` command using the builder pattern.
//!
//! Builder methods consume and return `Self`, so options can be chained
//! fluently in a single expression. Run with:
//!
//! ```text
//! cargo run --example print
//! ```

use perforce_cli::P4Cli;
use perforce_cli::spawn::{ParameterizedOutput, ParameterizedSpawn};
use std::ffi::OsStr;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let p4 = P4Cli::default();

    // Chain builders: print all revisions of the first two matching files to
    // a local output file, without the depot header line.
    let mut print = p4
        .print()
        .all_revisions(true)
        .quiet_mode(true)
        .limit(2)
        .redirect_output("print-output.txt");

    // Run the command to completion and capture its output.
    let file = Path::new("//depot/project/README.md");
    let output = print.output_with(&[file.as_os_str()])?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    // Or stream the contents directly to the terminal with `spawn_with`.
    let mut child = p4
        .print()
        .spawn_with(&[OsStr::new("//depot/project/README.md")])?;
    child.wait()?;

    Ok(())
}
