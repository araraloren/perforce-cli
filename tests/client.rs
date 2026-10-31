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
use std::process::Stdio;

use perforce_cli::P4Cli;
#[cfg(not(feature = "lt2016_2"))]
use perforce_cli::cmd::client::WorkSpaceType;
use perforce_cli::cmd::client::{Client, ShelvedHandling};
use perforce_cli::global::GlobalOpts;
use perforce_cli::spawn::ParameterizedSpawn;

mod common;
use common::{echo_bin, expect_args, expect_chain, parse_echo, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_client() -> Client {
    Client::new(echo_bin("client"), GlobalOpts::new())
}

// Form 1 -------------------------------------------------------------------
// `p4 client [-f] [-t template] [-T type] [name]`
//
// `-f`, `-t`, and `-T` are orthogonal: every subset is legal and every
// selection order converges on syntax order.

fn form_1_general() {
    let p4 = P4Cli::default();

    // Every subset of {`-f`, `-t`}.
    expect_chain!(["client"], p4.cmd_client());
    expect_chain!(["client", "-f"], p4.cmd_client().force());
    expect_chain!(["client", "-t", "tpl"], p4.cmd_client().template("tpl"));
    expect_chain!(
        ["client", "-f", "-t", "tpl"],
        p4.cmd_client().force().template("tpl")
    );

    // The `set_*` builder variants mutate an already-selected option.
    let mut cmd = p4.cmd_client().template("tmp");
    cmd.set_template("tpl");
    assert_eq!(cmd.get_template(), "tpl");
    expect_args(&cmd, &["client", "-t", "tpl"]);

    #[cfg(not(feature = "lt2016_2"))]
    {
        let g = WorkSpaceType::Partitioned;

        // Every remaining subset/permutation of {`-f`, `-t`, `-T`}.
        expect_chain!(
            ["client", "-T", "partitioned"],
            p4.cmd_client().workspace_type(g)
        );
        expect_chain!(
            ["client", "-f", "-T", "partitioned"],
            p4.cmd_client().force().workspace_type(g)
        );
        expect_chain!(
            ["client", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().template("tpl").workspace_type(g)
        );
        expect_chain!(
            ["client", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().workspace_type(g).template("tpl")
        );

        // Every permutation (3!) of the full option set converges identically.
        expect_chain!(
            ["client", "-f", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().force().template("tpl").workspace_type(g)
        );
        expect_chain!(
            ["client", "-f", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().force().workspace_type(g).template("tpl")
        );
        expect_chain!(
            ["client", "-f", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().template("tpl").force().workspace_type(g)
        );
        expect_chain!(
            ["client", "-f", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().template("tpl").workspace_type(g).force()
        );
        expect_chain!(
            ["client", "-f", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().workspace_type(g).force().template("tpl")
        );
        expect_chain!(
            ["client", "-f", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().workspace_type(g).template("tpl").force()
        );

        // Every documented `-T type` value.
        for (ty, value) in [
            (WorkSpaceType::Writable, "writeable"),
            (WorkSpaceType::Readonly, "readonly"),
            (WorkSpaceType::Partitioned, "partitioned"),
            #[cfg(not(feature = "lt2024_1"))]
            (WorkSpaceType::PartitionedJnl, "partitioned-jnl"),
            #[cfg(not(feature = "lt2024_1"))]
            (WorkSpaceType::Graph, "graph"),
        ] {
            expect_args(
                &p4.cmd_client().workspace_type(ty),
                &["client", "-T", value],
            );
        }

        // The `set_*` builder variants mutate an already-selected option.
        let mut cmd = p4
            .cmd_client()
            .template("tmp")
            .workspace_type(WorkSpaceType::Readonly);
        cmd.set_template("tpl").set_workspace_type(g);
        assert_eq!(cmd.get_template(), "tpl");
        assert_eq!(cmd.get_workspace_type(), g);
        expect_args(&cmd, &["client", "-t", "tpl", "-T", "partitioned"]);

        // Spawn with the full option set.
        let mut cmd = echo_client().force().template("tpl").workspace_type(g);
        assert_eq!(
            spawned_args(&mut cmd, ("ws-name",)),
            ["client", "-f", "-t", "tpl", "-T", "partitioned", "ws-name"]
        );
    }

    // The trailing `name` operand is optional; it is appended last on
    // spawn, both with the unit parameter and the one-arg parameter.
    let mut cmd = echo_client();
    assert_eq!(spawned_args(&mut cmd, ()), ["client"]);
    let mut cmd = echo_client();
    assert_eq!(spawned_args(&mut cmd, ("ws-name",)), ["client", "ws-name"]);
}

// Form 2 -------------------------------------------------------------------
// `p4 client -d [-f [-Fs|-Fd]] name`
//
// `-f` must accompany `-Fs`/`-Fd`, but the documented form imposes no
// selection order: every path below must reach the same command line.

fn form_2_delete() {
    let p4 = P4Cli::default();

    expect_chain!(["client", "-d"], p4.cmd_client().delete());

    // `-d -f`: either order of `-d` and `-f`.
    expect_chain!(["client", "-d", "-f"], p4.cmd_client().delete().force());
    expect_chain!(["client", "-d", "-f"], p4.cmd_client().force().delete());

    // `-d -f -Fs`: all five legal selection orders.
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.cmd_client().delete().force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.cmd_client().force().delete().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.cmd_client().force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.cmd_client().keep_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "-Fs"],
        p4.cmd_client().delete().keep_shelved().force()
    );

    #[cfg(not(feature = "lt2024_1"))]
    {
        // `-d -f -Fd`: all five legal selection orders.
        expect_chain!(
            ["client", "-d", "-f", "-Fd"],
            p4.cmd_client().delete().force().delete_shelved()
        );
        expect_chain!(
            ["client", "-d", "-f", "-Fd"],
            p4.cmd_client().force().delete().delete_shelved()
        );
        expect_chain!(
            ["client", "-d", "-f", "-Fd"],
            p4.cmd_client().force().delete_shelved()
        );
        expect_chain!(
            ["client", "-d", "-f", "-Fd"],
            p4.cmd_client().delete_shelved().force()
        );
        expect_chain!(
            ["client", "-d", "-f", "-Fd"],
            p4.cmd_client().delete().delete_shelved().force()
        );

        // The getter reflects the terminal selection.
        let cmd = p4.cmd_client().delete_shelved().force();
        assert_eq!(cmd.get_shelved(), Some(ShelvedHandling::Delete));

        // `name` is mandatory and is the trailing operand.
        let mut cmd = echo_client().delete_shelved().force();
        assert_eq!(
            spawned_args(&mut cmd, ("old-ws",)),
            ["client", "-d", "-f", "-Fd", "old-ws"]
        );
    }

    // The getter reflects the terminal selection.
    let cmd = p4.cmd_client().keep_shelved().force();
    assert_eq!(cmd.get_shelved(), Some(ShelvedHandling::Keep));

    // `name` is mandatory and is the trailing operand for every variant.
    let mut cmd = echo_client().delete();
    assert_eq!(
        spawned_args(&mut cmd, ("old-ws",)),
        ["client", "-d", "old-ws"]
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

    expect_chain!(["client", "-o"], p4.cmd_client().stdout());

    // `-t`, selected before `.stdout()`.
    expect_chain!(
        ["client", "-o", "-t", "tpl"],
        p4.cmd_client().template("tpl").stdout()
    );

    // `-t`, selected after `.stdout()`.
    expect_chain!(
        ["client", "-o", "-t", "tpl"],
        p4.cmd_client().stdout().template("tpl")
    );

    #[cfg(not(feature = "lt2016_2"))]
    {
        let y = WorkSpaceType::Partitioned;

        // `-T`, selected before `.stdout()`.
        expect_chain!(
            ["client", "-o", "-T", "partitioned"],
            p4.cmd_client().workspace_type(y).stdout()
        );

        // `-t` and `-T`, both selected before `.stdout()`.
        expect_chain!(
            ["client", "-o", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().template("tpl").workspace_type(y).stdout()
        );
        expect_chain!(
            ["client", "-o", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().workspace_type(y).template("tpl").stdout()
        );

        // `-T`, selected after `.stdout()`.
        expect_chain!(
            ["client", "-o", "-T", "partitioned"],
            p4.cmd_client().stdout().workspace_type(y)
        );

        // `-t` and `-T`, both selected after `.stdout()`.
        expect_chain!(
            ["client", "-o", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().stdout().template("tpl").workspace_type(y)
        );
        expect_chain!(
            ["client", "-o", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().stdout().workspace_type(y).template("tpl")
        );

        // One option before `.stdout()` and one after it, in both orders.
        expect_chain!(
            ["client", "-o", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().template("tpl").stdout().workspace_type(y)
        );
        expect_chain!(
            ["client", "-o", "-t", "tpl", "-T", "partitioned"],
            p4.cmd_client().workspace_type(y).stdout().template("tpl")
        );

        // Optional `name` with both options.
        let mut cmd = echo_client().template("tpl").workspace_type(y).stdout();
        assert_eq!(
            spawned_args(&mut cmd, ("ws-name",)),
            ["client", "-o", "-t", "tpl", "-T", "partitioned", "ws-name"]
        );
    }

    // Optional `name`.
    let mut cmd = echo_client().stdout();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-o", "ws-name"]
    );
}

// Form 4 -------------------------------------------------------------------
// `p4 client -S stream [[-c change] -o] [name]`
//
// `-o` optionally follows `-S stream`; `-c change` is only meaningful
// together with `-o` and is always injected before it.

fn form_4_stream() {
    let p4 = P4Cli::default();

    let cmd = p4.cmd_client().stream("//stream/main");
    assert_eq!(cmd.get_stream(), "//stream/main");
    expect_args(&cmd, &["client", "-S", "//stream/main"]);

    // `set_stream` mutates the selected stream.
    let mut cmd = p4.cmd_client().stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd, &["client", "-S", "//stream/dev"]);

    expect_chain!(
        ["client", "-S", "//stream/main", "-o"],
        p4.cmd_client().stream("//stream/main").stdout()
    );

    let mut cmd = p4.cmd_client().stream("//stream/main").stdout();
    assert_eq!(cmd.get_change(), None);
    cmd.set_change("4321");
    assert_eq!(cmd.get_change(), Some("4321"));
    expect_args(&cmd, &["client", "-S", "//stream/main", "-c", "4321", "-o"]);
    expect_chain!(
        ["client", "-S", "//stream/main", "-c", "4321", "-o"],
        p4.cmd_client()
            .stream("//stream/main")
            .stdout()
            .change("4321")
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

    let cmd = p4.cmd_client().switch_view_stream("//stream/dev");
    assert_eq!(cmd.get_stream(), "//stream/dev");
    expect_args(&cmd, &["client", "-s", "-S", "//stream/dev"]);

    let mut cmd = p4.cmd_client().switch_view_stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd, &["client", "-s", "-S", "//stream/dev"]);

    #[cfg(not(feature = "lt2022_1"))]
    {
        // `-f` before or after the switch transition, same command line.
        expect_chain!(
            ["client", "-s", "-f", "-S", "//stream/dev"],
            p4.cmd_client().force().switch_view_stream("//stream/dev")
        );
        expect_chain!(
            ["client", "-s", "-f", "-S", "//stream/dev"],
            p4.cmd_client().switch_view_stream("//stream/dev").force()
        );
        // `set_stream` then `-f` as well.
        let mut cmd = p4.cmd_client().switch_view_stream("old");
        cmd.set_stream("//stream/dev");
        expect_args(&cmd.force(), &["client", "-s", "-f", "-S", "//stream/dev"]);

        // The value stays editable (set / builder / getter) once `-f` is set.
        let mut cmd = p4.cmd_client().switch_view_stream("old").force();
        cmd.set_stream("//stream/dev");
        expect_args(&cmd, &["client", "-s", "-f", "-S", "//stream/dev"]);
        expect_chain!(
            ["client", "-s", "-f", "-S", "//stream/dev"],
            p4.cmd_client()
                .switch_view_stream("old")
                .force()
                .stream("//stream/dev")
        );
        let cmd = p4.cmd_client().force().switch_view_stream("//stream/dev");
        assert_eq!(cmd.get_stream(), "//stream/dev");
    }

    // --- Path 2: `stream(..).switch_view()` ---

    // Without `-f`: selecting `-s` injects no `-f`.
    expect_chain!(
        ["client", "-s", "-S", "//stream/dev"],
        p4.cmd_client().stream("//stream/dev").switch_view()
    );

    // The stream value is still changeable on the intermediate `-S`
    // form before `-s` is added.
    let mut cmd = p4.cmd_client().stream("old");
    cmd.set_stream("//stream/dev");
    expect_args(&cmd.switch_view(), &["client", "-s", "-S", "//stream/dev"]);
    expect_chain!(
        ["client", "-s", "-S", "//stream/dev"],
        p4.cmd_client()
            .stream("old")
            .stream("//stream/dev")
            .switch_view()
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
        // `-f` comes after `switch_view`; the intermediate `-S` form is the
        // form 4 state, which rejects `-f`.
        expect_chain!(
            ["client", "-s", "-f", "-S", "//stream/dev"],
            p4.cmd_client().stream("//stream/dev").switch_view().force()
        );
    }
    let cmd = p4.cmd_client().stream("//stream/dev").switch_view();
    assert_eq!(cmd.get_stream(), "//stream/dev");

    // --- Spawns: path 1, `name` omitted (unit), all three `-f` positions ---

    let mut cmd = echo_client().switch_view_stream("//stream/dev");
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["client", "-s", "-S", "//stream/dev"]
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
        let mut cmd = echo_client().force().switch_view_stream("//stream/dev");
        assert_eq!(
            spawned_args(&mut cmd, ()),
            ["client", "-s", "-f", "-S", "//stream/dev"]
        );
        let mut cmd = echo_client().switch_view_stream("//stream/dev").force();
        assert_eq!(
            spawned_args(&mut cmd, ()),
            ["client", "-s", "-f", "-S", "//stream/dev"]
        );
    }

    // --- Spawns: path 1, trailing `name` ---

    let mut cmd = echo_client().switch_view_stream("//stream/dev");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-S", "//stream/dev", "ws-name"]
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
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
    }

    // --- Spawns: path 2, `name` omitted and trailing, without/with `-f` ---

    let mut cmd = echo_client().stream("//stream/dev").switch_view();
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["client", "-s", "-S", "//stream/dev"]
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
        let mut cmd = echo_client().stream("//stream/dev").switch_view().force();
        assert_eq!(
            spawned_args(&mut cmd, ()),
            ["client", "-s", "-f", "-S", "//stream/dev"]
        );
    }

    let mut cmd = echo_client().stream("//stream/dev").switch_view();
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-S", "//stream/dev", "ws-name"]
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
        let mut cmd = echo_client().stream("//stream/dev").switch_view().force();
        assert_eq!(
            spawned_args(&mut cmd, ("ws-name",)),
            ["client", "-s", "-f", "-S", "//stream/dev", "ws-name"]
        );
    }
}

// Form 6 -------------------------------------------------------------------
// `p4 client -s [-f] -t template [name]`
//
// Reached either directly with `switch_view_template`, or by selecting
// `-t template` in the general form and then `switch_view`.

fn form_6_switch_template() {
    let p4 = P4Cli::default();

    let cmd = p4.cmd_client().switch_view_template("tpl");
    assert_eq!(cmd.get_template(), "tpl");
    expect_args(&cmd, &["client", "-s", "-t", "tpl"]);

    let mut cmd = p4.cmd_client().switch_view_template("old");
    cmd.set_template("tpl");
    expect_args(&cmd, &["client", "-s", "-t", "tpl"]);

    expect_chain!(
        ["client", "-s", "-t", "tpl"],
        p4.cmd_client().template("tpl").switch_view()
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
        // `-f` in every legal position relative to the two entry paths.
        expect_chain!(
            ["client", "-s", "-f", "-t", "tpl"],
            p4.cmd_client().force().switch_view_template("tpl")
        );
        expect_chain!(
            ["client", "-s", "-f", "-t", "tpl"],
            p4.cmd_client().switch_view_template("tpl").force()
        );
        expect_chain!(
            ["client", "-s", "-f", "-t", "tpl"],
            p4.cmd_client().force().template("tpl").switch_view()
        );
        expect_chain!(
            ["client", "-s", "-f", "-t", "tpl"],
            p4.cmd_client().template("tpl").switch_view().force()
        );
    }

    // Optional `name`, with and without `-f`.
    let mut cmd = echo_client().switch_view_template("tpl");
    assert_eq!(
        spawned_args(&mut cmd, ("ws-name",)),
        ["client", "-s", "-t", "tpl", "ws-name"]
    );

    #[cfg(not(feature = "lt2022_1"))]
    {
        let mut cmd = echo_client().force().switch_view_template("tpl");
        assert_eq!(
            spawned_args(&mut cmd, ("ws-name",)),
            ["client", "-s", "-f", "-t", "tpl", "ws-name"]
        );
    }
}

// Form 7 -------------------------------------------------------------------
// `p4 client -i [-f]`
//
// The specification arrives on standard input; this form takes no `name`
// operand and spawns with a stdin configuration instead.

fn form_7_stdin() {
    let p4 = P4Cli::default();

    expect_chain!(["client", "-i"], p4.cmd_client().stdin());
    expect_chain!(["client", "-i", "-f"], p4.cmd_client().force().stdin());
    expect_chain!(["client", "-i", "-f"], p4.cmd_client().stdin().force());

    // Spawn uses piped stdin (`()` parameter) or an explicit `Stdio`.
    let mut cmd = echo_client().stdin();
    assert_eq!(spawned_args(&mut cmd, ()), ["client", "-i"]);

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

#[cfg(not(feature = "lt2015_2"))]
fn form_8_server_id() {
    let p4 = P4Cli::default();
    let x = "edge.1667";

    // `-d -f --serverid=X`: all five legal selection orders.
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.cmd_client().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.cmd_client().server_id(x).force()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.cmd_client().delete().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.cmd_client().force().delete().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667"],
        p4.cmd_client().delete().server_id(x).force()
    );

    // Getter/setter on the selected value, before force is added.
    let mut cmd = p4.cmd_client().server_id(x);
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
        p4.cmd_client().force().server_id(x).keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().force().keep_shelved().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().server_id(x).force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().server_id(x).keep_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().keep_shelved().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().keep_shelved().server_id(x).force()
    );

    // ... and the same six with an explicit `-d` selected first.
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().delete().force().server_id(x).keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().delete().force().keep_shelved().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().delete().server_id(x).force().keep_shelved()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().delete().server_id(x).keep_shelved().force()
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().delete().keep_shelved().force().server_id(x)
    );
    expect_chain!(
        ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"],
        p4.cmd_client().delete().keep_shelved().server_id(x).force()
    );

    // Getters on a final state.
    let cmd = p4.cmd_client().keep_shelved().server_id(x).force();
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
// let _ = P4Cli::default().cmd_client().server_id("x").delete_shelved();
// let _ = P4Cli::default().cmd_client().delete_shelved().server_id("x");
// let _ = P4Cli::default().cmd_client().force().server_id("x").delete_shelved();
// let _ = P4Cli::default().cmd_client().force().delete_shelved().server_id("x");
//
// // `-Fs` and `-Fd` are mutually exclusive.
// let _ = P4Cli::default().cmd_client().keep_shelved().delete_shelved();
// let _ = P4Cli::default().cmd_client().delete_shelved().keep_shelved();
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
// let _ = P4Cli::default().cmd_client().force().stdout();
//
// // `-S stream` does not accept `-f`, `-t`, or `-T`.
// let _ = P4Cli::default().cmd_client().force().stream("//s");
// let _ = P4Cli::default().cmd_client().template("t").stream("//s");
// let _ = P4Cli::default().cmd_client().workspace_type(WorkSpaceType::Graph).stream("//s");
//
// // `-c change` requires `-o` on the `-S stream` form.
// let _ = P4Cli::default().cmd_client().stream("//s").change("4321");
//
// // Once `-o` is selected on the `-S stream` form, the `-s` switch is no
// // longer available (the `-S stream -o` form has no `-s` flag).
// let _ = P4Cli::default().cmd_client().stream("//s").stdout().switch_view();
//
// // The two-step `-s -S stream` path cannot carry a preselected `-f`:
// // the intermediate `-S` form rejects it, so force must follow
// // `switch_view` (use `force().switch_view_stream(s)` for force-first).
// let _ = P4Cli::default().cmd_client().force().stream("//s").switch_view();
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
    #[cfg(not(feature = "lt2015_2"))]
    form_8_server_id();
    println!("all p4 client syntax combinations verified");
    Ok(())
}
