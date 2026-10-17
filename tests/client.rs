//! Scratch verification of every documented `p4 client` syntax form.
//!
//! This is a throwaway example: for each documented command line it walks
//! EVERY legal selection order through the type-state builder, asserting
//! that they all converge on the same arguments. The reference is the
//! current command reference:
//!
//! ```text
//! p4 client [-f] [-t template] [-T type] [name]
//! p4 client -d [-f [-Fs|-Fd]] name
//! p4 client -o [-t template] [-T type] [name]
//! p4 client -S stream [[-c change] -o] [name]
//! p4 client -s [-f] -S stream [name]
//! p4 client -s [-f] -t template [name]
//! p4 client -i [-f]
//! p4 client -d -f --serverid=X [-Fs] name
//! ```
//!
//! Run with:
//!
//! ```text
//! cargo run --example test
//! ```

use std::io;
use std::path::PathBuf;
use std::process::{Child, Stdio};

use perforce_cli::P4Cli;
use perforce_cli::cmd::SubCommand;
use perforce_cli::cmd::client::{Client, ShelvedHandling, WorkSpaceType};
use perforce_cli::global::GlobalOpts;
use perforce_cli::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collects the arguments assembled on a [`std::process::Command`] as
/// strings for easy comparison.
fn args_of(command: &std::process::Command) -> Vec<String> {
    command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

/// Asserts the arguments produced by `setup_command` (everything except
/// the trailing optional/mandatory `name` operand, which is added on
/// spawn).
fn expect_args(cmd: &impl SubCommand, expected: &[&str]) {
    assert_eq!(args_of(&cmd.setup_command("p4")), expected);
}

/// Asserts that one chained builder expression assembles `expected`.
///
/// ```ignore
/// expect_chain!(["client", "-d", "-f", "-Fs"],
///               p4.client().delete().force().keep_shelved());
/// ```
macro_rules! expect_chain {
    ([$($e:literal),* $(,)?], $($chain:tt)*) => {{
        let cmd = $($chain)*;
        expect_args(&cmd, &[$($e),*]);
    }};
}

/// A tiny batch helper that just prints its arguments. `p4 client` only
/// appends the subcommand name, flags, and the optional `name`; none of
/// those contain spaces, so `%*` captures the exact argv tail.
fn echo_bin() -> PathBuf {
    let path = std::env::temp_dir().join("p4cli_client_echo_args.bat");
    std::fs::write(&path, "@echo %*\r\n").expect("write echo helper");
    path
}

/// Normalizes the echo helper output into logical arguments.
fn parse_echo(stdout: &[u8]) -> Vec<String> {
    // cmd.exe quotes args containing `=` (e.g. `--serverid=X`) when
    // launching the batch wrapper; those quotes are command-line
    // escaping, not part of the logical argument, so strip them.
    String::from_utf8_lossy(stdout)
        .split_whitespace()
        .map(|s| s.trim_matches('"').to_string())
        .collect()
}

/// Actually spawns the echo helper with one operand (the optional or
/// mandatory `name`) and returns the argv tail the helper received.
fn spawned_args<A, C>(cmd: &mut C, input: A) -> Vec<String>
where
    C: ParameterizedSpawn<A, Output = Child, Error = io::Error>,
{
    let output = cmd
        .spawn_with(input)
        .expect("spawn echo helper")
        .wait_with_output()
        .expect("wait for echo helper");
    assert!(output.status.success(), "echo helper exited unsuccessfully");
    parse_echo(&output.stdout)
}

/// Actually spawns the echo helper with no operand (`name` defaults to
/// `P4CLIENT`) and returns the argv tail the helper received.
fn spawned_args_unit<C>(cmd: &mut C) -> Vec<String>
where
    C: ParameterizedSpawn<(), Output = Child, Error = io::Error>,
{
    let output = cmd
        .spawn_with(())
        .expect("spawn echo helper")
        .wait_with_output()
        .expect("wait for echo helper");
    assert!(output.status.success(), "echo helper exited unsuccessfully");
    parse_echo(&output.stdout)
}

/// Starts a builder whose executable is the local echo helper.
fn echo_client() -> Client {
    Client::new(echo_bin(), GlobalOpts::new())
}

// Form 1 -------------------------------------------------------------------
// `p4 client [-f] [-t template] [-T type] [name]`
//
// `-f`, `-t`, and `-T` are orthogonal: every subset is legal and every
// selection order converges on syntax order.

fn form_1_general() {
    let p4 = P4Cli::default();
    let g = WorkSpaceType::Graph;

    // Every subset of {`-f`, `-t`, `-T`}.
    expect_chain!(["client"], p4.client());
    expect_chain!(["client", "-f"], p4.client().force());
    expect_chain!(["client", "-t", "tpl"], p4.client().template("tpl"));
    expect_chain!(["client", "-T", "graph"], p4.client().workspace_type(g));
    expect_chain!(
        ["client", "-f", "-t", "tpl"],
        p4.client().force().template("tpl")
    );
    expect_chain!(
        ["client", "-f", "-T", "graph"],
        p4.client().force().workspace_type(g)
    );
    expect_chain!(
        ["client", "-t", "tpl", "-T", "graph"],
        p4.client().template("tpl").workspace_type(g)
    );
    expect_chain!(
        ["client", "-t", "tpl", "-T", "graph"],
        p4.client().workspace_type(g).template("tpl")
    );

    // Every permutation (3!) of the full option set converges identically.
    expect_chain!(
        ["client", "-f", "-t", "tpl", "-T", "graph"],
        p4.client().force().template("tpl").workspace_type(g)
    );
    expect_chain!(
        ["client", "-f", "-t", "tpl", "-T", "graph"],
        p4.client().force().workspace_type(g).template("tpl")
    );
    expect_chain!(
        ["client", "-f", "-t", "tpl", "-T", "graph"],
        p4.client().template("tpl").force().workspace_type(g)
    );
    expect_chain!(
        ["client", "-f", "-t", "tpl", "-T", "graph"],
        p4.client().template("tpl").workspace_type(g).force()
    );
    expect_chain!(
        ["client", "-f", "-t", "tpl", "-T", "graph"],
        p4.client().workspace_type(g).force().template("tpl")
    );
    expect_chain!(
        ["client", "-f", "-t", "tpl", "-T", "graph"],
        p4.client().workspace_type(g).template("tpl").force()
    );

    // Every documented `-T type` value.
    for (ty, value) in [
        (WorkSpaceType::Writable, "writeable"),
        (WorkSpaceType::Readonly, "readonly"),
        (WorkSpaceType::Partitioned, "partitioned"),
        (WorkSpaceType::PartitionedJnl, "partitioned-jnl"),
        (WorkSpaceType::Graph, "graph"),
    ] {
        expect_args(&p4.client().workspace_type(ty), &["client", "-T", value]);
    }

    // The `set_*` builder variants mutate an already-selected option.
    let mut cmd = p4
        .client()
        .template("tmp")
        .workspace_type(WorkSpaceType::Readonly);
    cmd.set_template("tpl").set_workspace_type(g);
    assert_eq!(cmd.get_template(), "tpl");
    assert_eq!(cmd.get_workspace_type(), g);
    expect_args(&cmd, &["client", "-t", "tpl", "-T", "graph"]);

    // The trailing `name` operand is optional; it is appended last on
    // spawn, both with the unit parameter and the one-arg parameter.
    let mut cmd = echo_client();
    assert_eq!(spawned_args_unit(&mut cmd), ["client"]);
    let mut cmd = echo_client();
    assert_eq!(spawned_args(&mut cmd, ("ws-name",)), ["client", "ws-name"]);

    let mut cmd = echo_client().force().template("tpl").workspace_type(g);
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-f", "-t", "tpl", "-T", "graph", "ws-name"]
    );
}

// Form 2 -------------------------------------------------------------------
// `p4 client -d [-f [-Fs|-Fd]] name`
//
// `-f` must accompany `-Fs`/`-Fd`, but the documented form imposes no
// selection order: every path below must reach the same command line.

fn form_2_delete() {
    let p4 = P4Cli::default();

    expect_chain!(["client", "-d"], p4.client().delete());

    // `-d -f`: either order of `-d` and `-f`.
    expect_chain!(["client", "-d", "-f"], p4.client().delete().force());
    expect_chain!(["client", "-d", "-f"], p4.client().force().delete());

    // `-d -f -Fs`: all five legal selection orders.
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.client().delete().force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.client().force().delete().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.client().force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.client().keep_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.client().delete().keep_shelved().force()
    );

    // `-d -f -Fd`: all five legal selection orders.
    expect_chain!(
        ["client", "-d", "-f", "-Fd"],
        p4.client().delete().force().delete_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fd"],
        p4.client().force().delete().delete_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fd"],
        p4.client().force().delete_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fd"],
        p4.client().delete_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fd"],
        p4.client().delete().delete_shelved().force()
    );

    // Getters reflect the terminal selection.
    let cmd = p4.client().delete_shelved().force();
    assert_eq!(cmd.get_shelved(), Some(ShelvedHandling::Delete));
    let cmd = p4.client().keep_shelved().force();
    assert_eq!(cmd.get_shelved(), Some(ShelvedHandling::Keep));

    // `name` is mandatory and is the trailing operand for every variant.
    let mut cmd = echo_client().delete();
    assert_eq!(
        spawned_args(&mut cmd, ("old-ws",)),
        ["client", "-d", "old-ws"]
    );

    let mut cmd = echo_client().delete_shelved().force();
    assert_eq!(
        spawned_args(&mut cmd, ("old-ws",)),
        ["client", "-d", "-f", "-Fd", "old-ws"]
    );

    let mut cmd = echo_client().keep_shelved().force();
    assert_eq!(
        spawned_args(&mut cmd, ("old-ws",)),
        ["client", "-d", "-f", "-Fs", "old-ws"]
    );
}

// Form 3 -------------------------------------------------------------------
// `p4 client -o [-t template] [-T type] [name]`
//
// `-t`/`-T` may be selected before or after entering the `-o` state, in
// any order; `-f` is not part of this form.

fn form_3_stdout() {
    let p4 = P4Cli::default();
    let y = WorkSpaceType::Partitioned;

    expect_chain!(["client", "-o"], p4.client().stdout());

    // `-t` and `-T`, selected before `.stdout()`.
    expect_chain!(
        ["client", "-o", "-t", "tpl"],
        p4.client().template("tpl").stdout()
    );
    expect_chain!(
        ["client", "-o", "-T", "partitioned"],
        p4.client().workspace_type(y).stdout()
    );
    expect_chain!(
        ["client", "-o", "-t", "tpl", "-T", "partitioned"],
        p4.client().template("tpl").workspace_type(y).stdout()
    );
    expect_chain!(
        ["client", "-o", "-t", "tpl", "-T", "partitioned"],
        p4.client().workspace_type(y).template("tpl").stdout()
    );

    // `-t` and `-T`, selected after `.stdout()`.
    expect_chain!(
        ["client", "-o", "-t", "tpl"],
        p4.client().stdout().template("tpl")
    );
    expect_chain!(
        ["client", "-o", "-T", "partitioned"],
        p4.client().stdout().workspace_type(y)
    );
    expect_chain!(
        ["client", "-o", "-t", "tpl", "-T", "partitioned"],
        p4.client().stdout().template("tpl").workspace_type(y)
    );
    expect_chain!(
        ["client", "-o", "-t", "tpl", "-T", "partitioned"],
        p4.client().stdout().workspace_type(y).template("tpl")
    );

    // One option before `.stdout()` and one after it, in both orders.
    expect_chain!(
        ["client", "-o", "-t", "tpl", "-T", "partitioned"],
        p4.client().template("tpl").stdout().workspace_type(y)
    );
    expect_chain!(
        ["client", "-o", "-t", "tpl", "-T", "partitioned"],
        p4.client().workspace_type(y).stdout().template("tpl")
    );

    // Optional `name`.
    let mut cmd = echo_client().stdout();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-o", "ws-name"]
    );
    let mut cmd = echo_client().template("tpl").workspace_type(y).stdout();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-o", "-t", "tpl", "-T", "partitioned", "ws-name"]
    );
}

// Form 4 -------------------------------------------------------------------
// `p4 client -S stream [[-c change] -o] [name]`
//
// `-o` optionally follows `-S stream`; `-c change` is only meaningful
// together with `-o` and is always injected before it.

fn form_4_stream() {
    let p4 = P4Cli::default();

    let cmd = p4.client().stream("//stream/main");
    assert_eq!(cmd.get_stream(), "//stream/main");
    expect_args(&cmd, &["client", "-S", "//stream/main"]);

    // `set_stream` mutates the selected stream.
    let mut cmd = p4.client().stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd, &["client", "-S", "//stream/dev"]);

    expect_chain!(
        ["client", "-S", "//stream/main", "-o"],
        p4.client().stream("//stream/main").stdout()
    );

    let mut cmd = p4.client().stream("//stream/main").stdout();
    assert_eq!(cmd.get_change(), None);
    cmd.set_change("4321");
    assert_eq!(cmd.get_change(), Some("4321"));
    expect_args(&cmd, &["client", "-S", "//stream/main", "-c", "4321", "-o"]);
    expect_chain!(
        ["client", "-S", "//stream/main", "-c", "4321", "-o"],
        p4.client().stream("//stream/main").stdout().change("4321")
    );

    // Optional `name` for each of the three variants.
    let mut cmd = echo_client().stream("//stream/main");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-S", "//stream/main", "ws-name"]
    );
    let mut cmd = echo_client().stream("//stream/main").stdout();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-S", "//stream/main", "-o", "ws-name"]
    );
    let mut cmd = echo_client()
        .stream("//stream/main")
        .stdout()
        .change("4321");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        [
            "client",
            "-S",
            "//stream/main",
            "-c",
            "4321",
            "-o",
            "ws-name"
        ]
    );
}

// Form 5 -------------------------------------------------------------------
// `p4 client -s [-f] -S stream [name]`
//
// Two entry paths, symmetric with form 6:
//
// 1. direct:   `switch_view_stream(stream)` — `-f` may precede or follow
// 2. two-step: `stream(stream)` then `switch_view()` — the intermediate
//    `-S stream` form rejects `-f`, so force can only follow `switch_view`
//
// `name` is optional on every variant (unit or one-arg spawn).

fn form_5_switch_stream() {
    let p4 = P4Cli::default();

    // --- Path 1: direct `switch_view_stream`, no `-f` ---

    let cmd = p4.client().switch_view_stream("//stream/dev");
    assert_eq!(cmd.get_stream(), "//stream/dev");
    expect_args(&cmd, &["client", "-s", "-S", "//stream/dev"]);

    let mut cmd = p4.client().switch_view_stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd, &["client", "-s", "-S", "//stream/dev"]);

    // `-f` before or after the switch transition, same command line.
    expect_chain!(
        ["client", "-s", "-f", "-S", "//stream/dev"],
        p4.client().force().switch_view_stream("//stream/dev")
    );
    expect_chain!(
        ["client", "-s", "-f", "-S", "//stream/dev"],
        p4.client().switch_view_stream("//stream/dev").force()
    );
    // `set_stream` then `-f` as well.
    let mut cmd = p4.client().switch_view_stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd.force(), &["client", "-s", "-f", "-S", "//stream/dev"]);

    // The value stays editable (set / builder / getter) once `-f` is set.
    let mut cmd = p4.client().switch_view_stream("old").force();
    cmd.set_stream("//stream/dev");
    expect_args(&cmd, &["client", "-s", "-f", "-S", "//stream/dev"]);
    expect_chain!(
        ["client", "-s", "-f", "-S", "//stream/dev"],
        p4.client()
            .switch_view_stream("old")
            .force()
            .stream("//stream/dev")
    );
    let cmd = p4.client().force().switch_view_stream("//stream/dev");
    assert_eq!(cmd.get_stream(), "//stream/dev");

    // --- Path 2: `stream(..).switch_view()` ---

    // Without `-f`: selecting `-s` injects no `-f`.
    expect_chain!(
        ["client", "-s", "-S", "//stream/dev"],
        p4.client().stream("//stream/dev").switch_view()
    );

    // The stream value is still changeable on the intermediate `-S`
    // form before `-s` is added.
    let mut cmd = p4.client().stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd.switch_view(), &["client", "-s", "-S", "//stream/dev"]);
    expect_chain!(
        ["client", "-s", "-S", "//stream/dev"],
        p4.client()
            .stream("old")
            .stream("//stream/dev")
            .switch_view()
    );

    // `-f` comes after `switch_view`; the intermediate `-S` form is the
    // form 4 state, which rejects `-f`.
    expect_chain!(
        ["client", "-s", "-f", "-S", "//stream/dev"],
        p4.client().stream("//stream/dev").switch_view().force()
    );
    let cmd = p4.client().stream("//stream/dev").switch_view();
    assert_eq!(cmd.get_stream(), "//stream/dev");

    // --- Spawns: path 1, `name` omitted (unit), all three `-f` positions ---

    let mut cmd = echo_client().switch_view_stream("//stream/dev");
    assert_eq!(
        spawned_args_unit(&mut cmd),
        ["client", "-s", "-S", "//stream/dev"]
    );
    let mut cmd = echo_client().force().switch_view_stream("//stream/dev");
    assert_eq!(
        spawned_args_unit(&mut cmd),
        ["client", "-s", "-f", "-S", "//stream/dev"]
    );
    let mut cmd = echo_client().switch_view_stream("//stream/dev").force();
    assert_eq!(
        spawned_args_unit(&mut cmd),
        ["client", "-s", "-f", "-S", "//stream/dev"]
    );

    // --- Spawns: path 1, trailing `name` ---

    let mut cmd = echo_client().switch_view_stream("//stream/dev");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-S", "//stream/dev", "ws-name"]
    );
    let mut cmd = echo_client().force().switch_view_stream("//stream/dev");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-f", "-S", "//stream/dev", "ws-name"]
    );
    let mut cmd = echo_client().switch_view_stream("//stream/dev").force();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-f", "-S", "//stream/dev", "ws-name"]
    );

    // --- Spawns: path 2, `name` omitted and trailing, without/with `-f` ---

    let mut cmd = echo_client().stream("//stream/dev").switch_view();
    assert_eq!(
        spawned_args_unit(&mut cmd),
        ["client", "-s", "-S", "//stream/dev"]
    );
    let mut cmd = echo_client().stream("//stream/dev").switch_view().force();
    assert_eq!(
        spawned_args_unit(&mut cmd),
        ["client", "-s", "-f", "-S", "//stream/dev"]
    );

    let mut cmd = echo_client().stream("//stream/dev").switch_view();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-S", "//stream/dev", "ws-name"]
    );
    let mut cmd = echo_client().stream("//stream/dev").switch_view().force();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-f", "-S", "//stream/dev", "ws-name"]
    );
}

// Form 6 -------------------------------------------------------------------
// `p4 client -s [-f] -t template [name]`
//
// Reached either directly with `switch_view_template`, or by selecting
// `-t template` in the general form and then `switch_view`.

fn form_6_switch_template() {
    let p4 = P4Cli::default();

    let cmd = p4.client().switch_view_template("tpl");
    assert_eq!(cmd.get_template(), "tpl");
    expect_args(&cmd, &["client", "-s", "-t", "tpl"]);

    let mut cmd = p4.client().switch_view_template("old");
    cmd.set_template("tpl");
    expect_args(&cmd, &["client", "-s", "-t", "tpl"]);

    expect_chain!(
        ["client", "-s", "-t", "tpl"],
        p4.client().template("tpl").switch_view()
    );

    // `-f` in every legal position relative to the two entry paths.
    expect_chain!(
        ["client", "-s", "-f", "-t", "tpl"],
        p4.client().force().switch_view_template("tpl")
    );
    expect_chain!(
        ["client", "-s", "-f", "-t", "tpl"],
        p4.client().switch_view_template("tpl").force()
    );
    expect_chain!(
        ["client", "-s", "-f", "-t", "tpl"],
        p4.client().force().template("tpl").switch_view()
    );
    expect_chain!(
        ["client", "-s", "-f", "-t", "tpl"],
        p4.client().template("tpl").switch_view().force()
    );

    // Optional `name`, with and without `-f`.
    let mut cmd = echo_client().switch_view_template("tpl");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-t", "tpl", "ws-name"]
    );
    let mut cmd = echo_client().force().switch_view_template("tpl");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-f", "-t", "tpl", "ws-name"]
    );
}

// Form 7 -------------------------------------------------------------------
// `p4 client -i [-f]`
//
// The specification arrives on standard input; this form takes no `name`
// operand and spawns with a stdin configuration instead.

fn form_7_stdin() {
    let p4 = P4Cli::default();

    expect_chain!(["client", "-i"], p4.client().stdin());
    expect_chain!(["client", "-i", "-f"], p4.client().force().stdin());
    expect_chain!(["client", "-i", "-f"], p4.client().stdin().force());

    // Spawn uses piped stdin (`()` parameter) or an explicit `Stdio`.
    let mut cmd = echo_client().stdin();
    assert_eq!(spawned_args_unit(&mut cmd), ["client", "-i"]);

    let mut cmd = echo_client().force().stdin();
    let output = cmd
        .spawn_with((Stdio::null(),))
        .expect("spawn echo helper")
        .wait_with_output()
        .expect("wait for echo helper");
    assert_eq!(parse_echo(&output.stdout), ["client", "-i", "-f"]);
}

// Form 8 -------------------------------------------------------------------
// `p4 client -d -f --serverid=X [-Fs] name`
//
// `--serverid=X` combines with no shelved option or with `-Fs` (never
// `-Fd`); again, the documented form fixes neither entry path nor option
// order, so every legal permutation must converge.

fn form_8_server_id() {
    let p4 = P4Cli::default();
    let x = "edge.1667";

    // `-d -f --serverid=X`: all five legal selection orders.
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.client().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.client().server_id(x).force()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.client().delete().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.client().force().delete().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.client().delete().server_id(x).force()
    );

    // Getter/setter on the selected value, before force is added.
    let mut cmd = p4.client().server_id(x);
    assert_eq!(cmd.get_server_id(), x);
    assert_eq!(cmd.get_shelved(), None);
    cmd.set_server_id("edge.42");
    let cmd = cmd.force();
    assert_eq!(cmd.get_server_id(), "edge.42");
    expect_args(&cmd, &["client", "-d", "-f", "--serverid=edge.42"]);

    // `-d -f --serverid=X -Fs`: every permutation of
    // {`-f`, `--serverid=X`, `-Fs`} (3! = 6) ...
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().force().server_id(x).keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().force().keep_shelved().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().server_id(x).force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().server_id(x).keep_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().keep_shelved().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().keep_shelved().server_id(x).force()
    );

    // ... and the same six with an explicit `-d` selected first.
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().delete().force().server_id(x).keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().delete().force().keep_shelved().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().delete().server_id(x).force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().delete().server_id(x).keep_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().delete().keep_shelved().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.client().delete().keep_shelved().server_id(x).force()
    );

    // Getters on a final state.
    let cmd = p4.client().keep_shelved().server_id(x).force();
    assert_eq!(cmd.get_server_id(), x);
    assert_eq!(cmd.get_shelved(), Some(ShelvedHandling::Keep));

    // Mandatory trailing `name` operand.
    let mut cmd = echo_client().server_id(x).force();
    assert_eq!(
        spawned_args(&mut cmd, ("old-ws",)),
        ["client", "-d", "-f", "--serverid=edge.1667", "old-ws"]
    );
    let mut cmd = echo_client().keep_shelved().server_id(x).force();
    assert_eq!(
        spawned_args(&mut cmd, ("old-ws",)),
        [
            "client",
            "-d",
            "-f",
            "--serverid=edge.1667",
            "-Fs",
            "old-ws"
        ]
    );
}

// Rejected combinations -----------------------------------------------------
//
// The chains below contradict the documented forms and must fail to
// compile; they are kept as executable documentation of the type-state
// rules. Uncomment any line to verify it is rejected.
//
// // `-Fd` can never be combined with `--serverid=X` (form 8 allows [-Fs] only).
// let _ = P4Cli::default().client().server_id("x").delete_shelved();
// let _ = P4Cli::default().client().delete_shelved().server_id("x");
// let _ = P4Cli::default().client().force().server_id("x").delete_shelved();
// let _ = P4Cli::default().client().force().delete_shelved().server_id("x");
//
// // `-Fs` and `-Fd` are mutually exclusive.
// let _ = P4Cli::default().client().keep_shelved().delete_shelved();
// let _ = P4Cli::default().client().delete_shelved().keep_shelved();
//
// // `-Fs` / `-Fd` / `--serverid` REQUIRE `-f`: selecting them is allowed in
// // any order, but the resulting intermediate state cannot be spawned.
// let mut pending = echo_client().delete_shelved();
// pending.spawn_with(("old-ws",));
// let mut pending = echo_client().server_id("x");
// pending.spawn_with(("old-ws",));
// let mut pending = echo_client().keep_shelved();
// pending.spawn_with(("old-ws",));
//
// // `-o` does not accept `-f`.
// let _ = P4Cli::default().client().force().stdout();
//
// // `-S stream` does not accept `-f`, `-t`, or `-T`.
// let _ = P4Cli::default().client().force().stream("//s");
// let _ = P4Cli::default().client().template("t").stream("//s");
// let _ = P4Cli::default().client().workspace_type(WorkSpaceType::Graph).stream("//s");
//
// // `-c change` requires `-o` on the `-S stream` form.
// let _ = P4Cli::default().client().stream("//s").change("4321");
//
// // Once `-o` is selected on the `-S stream` form, the `-s` switch is no
// // longer available (the `-S stream -o` form has no `-s` flag).
// let _ = P4Cli::default().client().stream("//s").stdout().switch_view();
//
// // The two-step `-s -S stream` path cannot carry a preselected `-f`:
// // the intermediate `-S` form rejects it, so force must follow
// // `switch_view` (use `force().switch_view_stream(s)` for force-first).
// let _ = P4Cli::default().client().force().stream("//s").switch_view();
//
// // `-i` takes no `name` operand: the specification comes from stdin.
// // (`Client<StdinMode>` implements no one-argument spawn.)
//
// // `-d` forms require `name`: `Client<DeleteMode, _>` implements no unit
// // spawn.

fn main() -> io::Result<()> {
    form_1_general();
    form_2_delete();
    form_3_stdout();
    form_4_stream();
    form_5_switch_stream();
    form_6_switch_template();
    form_7_stdin();
    form_8_server_id();
    println!("all p4 client syntax combinations verified");
    Ok(())
}
