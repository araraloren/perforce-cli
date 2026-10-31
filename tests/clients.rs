//! Scratch verification of every documented `p4 clients` syntax form.
//!
//! This is a throwaway example: for each documented command line it walks
//! the legal selection orders through the type-state builder, asserting
//! that they all converge on the same arguments. The reference is the
//! current command reference:
//!
//! ```text
//! p4 [g-opts] clients [-t] [-u user [--user-case-insensitive] | --me] [[-e|-E] filter] [-m max] [-S stream] [-a | -s serverID]
//! p4 [g-opts] clients -U
//! ```
//!
//! (`--me` requires 2016.1+; `--user-case-insensitive` requires 2024.2+.)
//!
//! The three exclusive groups `-u user | --me`, `-e | -E`, and
//! `-a | -s serverID` are independent of each other and of the free
//! options `-t`, `-m max`, and `-S stream`: any selection order converges
//! on the documented injection order `-t -u/--me -e/-E -m -S -a/-s`.

use std::io;

use perforce_cli::P4Cli;
use perforce_cli::cmd::clients::Clients;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_clients() -> Clients {
    Clients::new(echo_bin("clients"), GlobalOpts::new())
}

// Form 1 -------------------------------------------------------------------
// `p4 clients`
//
// The bare command lists every client workspace known to the system; the
// `workspaces` alias assembles the identical command line.

fn form_1_bare() {
    let p4 = P4Cli::default();

    expect_chain!(["clients"], p4.cmd_clients());
    expect_chain!(["clients"], p4.cmd_workspaces());

    let mut cmd = echo_clients();
    assert_eq!(spawned_args(&mut cmd, ()), ["clients"]);
}

// Form 2 -------------------------------------------------------------------
// `p4 clients [-t] [-u user [--user-case-insensitive] | --me] [[-e|-E] filter] [-m max] [-S stream] [-a | -s serverID]`
//
// Every option on its own, entered straight from the bare command.

fn form_2_single_options() {
    let p4 = P4Cli::default();

    expect_chain!(["clients", "-t"], p4.cmd_clients().display_time(true));
    // `display_time(false)` enters the regular form but injects nothing.
    expect_chain!(["clients"], p4.cmd_clients().display_time(false));
    expect_chain!(["clients", "-m", "5"], p4.cmd_clients().limit(5));
    expect_chain!(
        ["clients", "-S", "//stream/main"],
        p4.cmd_clients().stream("//stream/main")
    );
    expect_chain!(
        ["clients", "-e", "p4vBeta*"],
        p4.cmd_clients().filter("p4vBeta*", false)
    );
    expect_chain!(
        ["clients", "-E", "*Us*rs"],
        p4.cmd_clients().filter("*Us*rs", true)
    );
    expect_chain!(["clients", "-u", "O*hara"], p4.cmd_clients().user("O*hara"));
    expect_chain!(["clients", "-a"], p4.cmd_clients().all());
    expect_chain!(
        ["clients", "-s", "edge.1667"],
        p4.cmd_clients().server_id("edge.1667")
    );

    #[cfg(not(feature = "lt2016_1"))]
    expect_chain!(["clients", "--me"], p4.cmd_clients().filter_me());

    #[cfg(not(feature = "lt2024_2"))]
    expect_chain!(
        ["clients", "-u", "O*hara", "--user-case-insensitive"],
        p4.cmd_clients().user("O*hara").case_insensitive()
    );

    // Each single-option form spawns.
    let mut cmd = echo_clients().limit(5);
    assert_eq!(spawned_args(&mut cmd, ()), ["clients", "-m", "5"]);
}

// Form 3 -------------------------------------------------------------------
// The regular form's option groups are orthogonal: the three exclusive
// groups `-u user | --me`, `-e | -E`, and `-a | -s serverID` plus the free
// options `-t`, `-m max`, and `-S stream` may be selected in ANY order;
// every path converges on the documented injection order.

fn form_3_selection_orders() {
    let p4 = P4Cli::default();

    // `-u` and `-E` are independent groups: both selection orders converge.
    expect_chain!(
        ["clients", "-u", "O*hara", "-E", "*Us*rs"],
        p4.cmd_clients().user("O*hara").filter("*Us*rs", true)
    );
    expect_chain!(
        ["clients", "-u", "O*hara", "-E", "*Us*rs"],
        p4.cmd_clients().filter("*Us*rs", true).user("O*hara")
    );

    // `--me` and `-e` are independent groups as well: both selection
    // orders are legal and converge (`--me` belongs to the OWNER group,
    // not to the name-filter group).
    #[cfg(not(feature = "lt2016_1"))]
    {
        expect_chain!(
            ["clients", "--me", "-e", "p4vBeta*"],
            p4.cmd_clients().filter_me().filter("p4vBeta*", false)
        );
        expect_chain!(
            ["clients", "--me", "-e", "p4vBeta*"],
            p4.cmd_clients().filter("p4vBeta*", false).filter_me()
        );
    }

    // `--user-case-insensitive` chains anywhere after `-u` without
    // blocking the remaining groups.
    #[cfg(not(feature = "lt2024_2"))]
    {
        expect_chain!(
            [
                "clients",
                "-u",
                "O*hara",
                "--user-case-insensitive",
                "-s",
                "edge.1667"
            ],
            p4.cmd_clients()
                .user("O*hara")
                .case_insensitive()
                .server_id("edge.1667")
        );
        expect_chain!(
            [
                "clients",
                "-u",
                "O*hara",
                "--user-case-insensitive",
                "-s",
                "edge.1667"
            ],
            p4.cmd_clients()
                .user("O*hara")
                .server_id("edge.1667")
                .case_insensitive()
        );
        expect_chain!(
            [
                "clients",
                "-u",
                "O*hara",
                "--user-case-insensitive",
                "-s",
                "edge.1667"
            ],
            p4.cmd_clients()
                .server_id("edge.1667")
                .user("O*hara")
                .case_insensitive()
        );
    }

    // The full option set in several selection orders converges
    // identically.
    let full = [
        "clients",
        "-t",
        "-u",
        "O*hara",
        "-E",
        "*Us*rs",
        "-m",
        "5",
        "-S",
        "//stream/main",
        "-s",
        "edge.1667",
    ];
    expect_chain!(
        [
            "clients",
            "-t",
            "-u",
            "O*hara",
            "-E",
            "*Us*rs",
            "-m",
            "5",
            "-S",
            "//stream/main",
            "-s",
            "edge.1667",
        ],
        p4.cmd_clients()
            .display_time(true)
            .user("O*hara")
            .filter("*Us*rs", true)
            .limit(5)
            .stream("//stream/main")
            .server_id("edge.1667")
    );
    expect_chain!(
        [
            "clients",
            "-t",
            "-u",
            "O*hara",
            "-E",
            "*Us*rs",
            "-m",
            "5",
            "-S",
            "//stream/main",
            "-s",
            "edge.1667",
        ],
        p4.cmd_clients()
            .server_id("edge.1667")
            .stream("//stream/main")
            .limit(5)
            .filter("*Us*rs", true)
            .user("O*hara")
            .display_time(true)
    );
    expect_chain!(
        [
            "clients",
            "-t",
            "-u",
            "O*hara",
            "-E",
            "*Us*rs",
            "-m",
            "5",
            "-S",
            "//stream/main",
            "-s",
            "edge.1667",
        ],
        p4.cmd_clients()
            .filter("*Us*rs", true)
            .server_id("edge.1667")
            .stream("//stream/main")
            .user("O*hara")
            .limit(5)
            .display_time(true)
    );

    // The same full set with `--me` and `-a`.
    #[cfg(not(feature = "lt2016_1"))]
    {
        expect_chain!(
            [
                "clients",
                "-t",
                "--me",
                "-e",
                "p4vBeta*",
                "-m",
                "5",
                "-S",
                "//stream/main",
                "-a",
            ],
            p4.cmd_clients()
                .all()
                .filter_me()
                .display_time(true)
                .filter("p4vBeta*", false)
                .stream("//stream/main")
                .limit(5)
        );
        expect_chain!(
            [
                "clients",
                "-t",
                "--me",
                "-e",
                "p4vBeta*",
                "-m",
                "5",
                "-S",
                "//stream/main",
                "-a",
            ],
            p4.cmd_clients()
                .filter("p4vBeta*", false)
                .filter_me()
                .all()
                .limit(5)
                .stream("//stream/main")
                .display_time(true)
        );
    }

    // The `set_*` variants mutate an already-entered regular form.
    let mut cmd = p4.cmd_clients().user("O*hara");
    cmd.set_limit(10)
        .set_display_time(true)
        .set_stream("//stream/dev");
    assert_eq!(cmd.get_limit(), Some(10));
    assert!(cmd.get_display_time());
    assert_eq!(cmd.get_stream(), Some(&"//stream/dev".to_string()));
    expect_args(
        &cmd,
        &[
            "clients",
            "-t",
            "-u",
            "O*hara",
            "-m",
            "10",
            "-S",
            "//stream/dev",
        ],
    );

    // The full house spawns with the converged argument order.
    let mut cmd = echo_clients()
        .display_time(true)
        .user("O*hara")
        .filter("*Us*rs", true)
        .limit(5)
        .stream("//stream/main")
        .server_id("edge.1667");
    assert_eq!(spawned_args(&mut cmd, ()), full);
}

// Form 4 -------------------------------------------------------------------
// `p4 clients -U`
//
// A standalone form listing unloaded workspaces; it accepts no other
// option and is reachable only from the bare command.

fn form_4_unloaded() {
    let p4 = P4Cli::default();

    expect_chain!(["clients", "-U"], p4.cmd_clients().unloaded());
    expect_chain!(["clients", "-U"], p4.cmd_workspaces().unloaded());

    let mut cmd = echo_clients().unloaded();
    assert_eq!(spawned_args(&mut cmd, ()), ["clients", "-U"]);
}

// Rejected combinations -----------------------------------------------------
//
// The chains below contradict the documented forms and must fail to
// compile; they are kept as executable documentation of the type-state
// rules. Uncomment any line to verify it is rejected.
//
// // `-u` and `--me` share one syntax group (`[-u user ... | --me]`):
// // setting either removes the other from the type state.
// let _ = P4Cli::default().cmd_clients().user("x").filter_me();
// let _ = P4Cli::default().cmd_clients().filter_me().user("x");
//
// // The name filter is set at most once (`filter` consumes the F state).
// let _ = P4Cli::default().cmd_clients().filter("a", false).filter("b", true);
//
// // `-a` and `-s` are mutually exclusive.
// let _ = P4Cli::default().cmd_clients().all().server_id("x");
// let _ = P4Cli::default().cmd_clients().server_id("x").all();
//
// // `--user-case-insensitive` exists only on the `-u user` state.
// let _ = P4Cli::default().cmd_clients().filter_me().case_insensitive();
// let _ = P4Cli::default().cmd_clients().filter("a", false).case_insensitive();
//
// // `-U` is a standalone form: reachable only from the bare command, and
// // offering no further options.
// let _ = P4Cli::default().cmd_clients().limit(5).unloaded();
// let _ = P4Cli::default().cmd_clients().unloaded().limit(5);

fn main() -> io::Result<()> {
    form_1_bare();
    form_2_single_options();
    form_3_selection_orders();
    form_4_unloaded();
    println!("all p4 clients syntax combinations verified");
    Ok(())
}
