//! Helpers shared by the command-line verification tests.
//!
//! Each integration test target includes this module with `mod common;`.
//! The per-target `echo_<command>()` constructors stay in their own files
//! because they name a specific command type; everything else lives here.

#![allow(dead_code)]

use std::io;
use std::path::PathBuf;
use std::process::Child;

use perforce_cli::cmd::SubCommand;
use perforce_cli::spawn::ParameterizedSpawn;

/// Collects the arguments assembled on a [`std::process::Command`] as
/// strings for easy comparison.
pub fn args_of(command: &std::process::Command) -> Vec<String> {
    command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

/// Asserts the arguments produced by `setup_command` (everything except
/// operands that are added on spawn).
pub fn expect_args(cmd: &impl SubCommand, expected: &[&str]) {
    assert_eq!(args_of(&cmd.setup_command("p4")), expected);
}

/// Asserts that one chained builder expression assembles `expected`.
///
/// ```ignore
/// expect_chain!(["client", "-d", "-f", "-Fs"],
///               p4.cmd_client().delete().force().keep_shelved());
/// ```
macro_rules! expect_chain {
    ([$($e:literal),* $(,)?], $($chain:tt)*) => {{
        let cmd = $($chain)*;
        expect_args(&cmd, &[$($e),*]);
    }};
}

pub(crate) use expect_chain;

/// A tiny batch helper that just prints its arguments.
///
/// `name` identifies the test target so concurrent targets do not fight
/// over the same temporary file. The samples used by the tests contain no
/// spaces, so `%*` captures the exact argv tail.
pub fn echo_bin(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("p4cli_{name}_echo_args.bat"));
    std::fs::write(&path, "@echo %*\r\n").expect("write echo helper");
    path
}

/// Normalizes the echo helper output into logical arguments.
pub fn parse_echo(stdout: &[u8]) -> Vec<String> {
    // cmd.exe quotes args containing `=` (e.g. `--serverid=X`) when
    // launching the batch wrapper; those quotes are command-line
    // escaping, not part of the logical argument, so strip them.
    String::from_utf8_lossy(stdout)
        .split_whitespace()
        .map(|s| s.trim_matches('"').to_string())
        .collect()
}

/// Actually spawns the echo helper and returns the argv tail it received.
pub fn spawned_args<C, I>(cmd: &mut C, input: I) -> Vec<String>
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
