//! Compile-time verification of every documented `p4 counter` syntax form.
//!
//! For each documented command line this walks the legal builder chains
//! through the type-state API, asserting that they all assemble the same
//! arguments. The reference is the current command reference:
//!
//! ```text
//! p4 [g-opts] counter countername
//! p4 [g-opts] counter [-f -v] countername value
//! p4 [g-opts] counter [-f] -d countername
//! p4 [g-opts] counter [-f -v] -i countername
//! p4 [g-opts] counter [-f] -m [pair list]
//! p4 [g-opts] counter --from oldvalue --to newvalue countername
//! ```

use std::io;

use perforce_cli::P4Cli;
use perforce_cli::cmd::Counter;
use perforce_cli::global::GlobalOpts;

mod common;
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_counter() -> Counter {
    Counter::new(echo_bin("counter"), GlobalOpts::new())
}

// Form 1 -------------------------------------------------------------------
// `p4 counter countername`
//
// The bare query form reads the value of a counter; it accepts no options
// and is the state returned by `cmd_counter`.

fn form_1_query() {
    let p4 = P4Cli::default();

    expect_chain!(["counter"], p4.cmd_counter());

    let mut cmd = echo_counter();
    assert_eq!(
        spawned_args(&mut cmd, ("mycounter",)),
        ["counter", "mycounter"]
    );
}

// Form 2 -------------------------------------------------------------------
// `p4 counter [-f -v] countername value`
//
// The set form is entered with `set`, `force`, or `show_previous`; the two
// flags may be selected in either order and inject as `-f -v`.

fn form_2_set() {
    let p4 = P4Cli::default();

    expect_chain!(["counter"], p4.cmd_counter().set());
    expect_chain!(["counter"], p4.cmd_counter().force(false));
    expect_chain!(["counter", "-f"], p4.cmd_counter().force(true));
    #[cfg(not(feature = "lt2016_1"))]
    {
        expect_chain!(["counter", "-v"], p4.cmd_counter().show_previous());
        expect_chain!(
            ["counter", "-f", "-v"],
            p4.cmd_counter().set().force(true).show_previous()
        );
        expect_chain!(
            ["counter", "-f", "-v"],
            p4.cmd_counter().set().show_previous().force(true)
        );
        expect_chain!(
            ["counter", "-f", "-v"],
            p4.cmd_counter().force(true).show_previous()
        );
        expect_chain!(
            ["counter", "-v"],
            p4.cmd_counter().force(true).show_previous().force(false)
        );
    }

    // Each set form spawns with the counter name and value operands.
    let mut cmd = echo_counter().set();
    assert_eq!(
        spawned_args(&mut cmd, ("mycounter", "123")),
        ["counter", "mycounter", "123"]
    );

    #[cfg(not(feature = "lt2016_1"))]
    {
        let mut cmd = echo_counter().force(true).show_previous();
        assert_eq!(
            spawned_args(&mut cmd, ("mycounter", "123")),
            ["counter", "-f", "-v", "mycounter", "123"]
        );
    }
}

// Form 3 -------------------------------------------------------------------
// `p4 counter [-f] -d countername`
//
// The delete form is entered from the bare command or from the set state;
// the latter preserves `-f`. The `-v` state cannot reach it.

fn form_3_delete() {
    let p4 = P4Cli::default();

    expect_chain!(["counter", "-d"], p4.cmd_counter().delete());
    expect_chain!(
        ["counter", "-f", "-d"],
        p4.cmd_counter().force(true).delete()
    );
    expect_chain!(
        ["counter", "-f", "-d"],
        p4.cmd_counter().delete().force(true)
    );
    expect_chain!(
        ["counter", "-f", "-d"],
        p4.cmd_counter().set().force(true).delete()
    );

    let mut cmd = echo_counter().force(true).delete();
    assert_eq!(
        spawned_args(&mut cmd, ("mycounter",)),
        ["counter", "-f", "-d", "mycounter"]
    );
}

// Form 4 -------------------------------------------------------------------
// `p4 counter [-f -v] -i countername`
//
// The increment form accepts both flags; `-f` is preserved through the
// transitions and `-v` may be supplied before or after entering the form.

fn form_4_increment() {
    let p4 = P4Cli::default();

    expect_chain!(["counter", "-i"], p4.cmd_counter().increment());
    expect_chain!(
        ["counter", "-f", "-i"],
        p4.cmd_counter().increment().force(true)
    );
    expect_chain!(
        ["counter", "-f", "-i"],
        p4.cmd_counter().force(true).increment()
    );
    #[cfg(not(feature = "lt2016_1"))]
    {
        expect_chain!(
            ["counter", "-v", "-i"],
            p4.cmd_counter().increment().show_previous()
        );
        expect_chain!(
            ["counter", "-v", "-i"],
            p4.cmd_counter().show_previous().increment()
        );
        expect_chain!(
            ["counter", "-v", "-i"],
            p4.cmd_counter().set().show_previous().increment()
        );
        expect_chain!(
            ["counter", "-f", "-v", "-i"],
            p4.cmd_counter().force(true).show_previous().increment()
        );
        expect_chain!(
            ["counter", "-f", "-v", "-i"],
            p4.cmd_counter().increment().force(true).show_previous()
        );
    }

    let mut cmd = echo_counter().increment();
    assert_eq!(
        spawned_args(&mut cmd, ("mycounter",)),
        ["counter", "-i", "mycounter"]
    );

    #[cfg(not(feature = "lt2016_1"))]
    {
        let mut cmd = echo_counter().force(true).show_previous().increment();
        assert_eq!(
            spawned_args(&mut cmd, ("mycounter",)),
            ["counter", "-f", "-v", "-i", "mycounter"]
        );
    }
}

// Form 5 -------------------------------------------------------------------
// `p4 counter [-f] -m [pair list]`
//
// The multi form receives the whole pair list as the positional arguments
// that follow `-m`; `-f` may be supplied directly or preserved from the set
// form.

fn form_5_multi() {
    let p4 = P4Cli::default();

    expect_chain!(["counter", "-m"], p4.cmd_counter().multi());
    expect_chain!(
        ["counter", "-f", "-m"],
        p4.cmd_counter().multi().force(true)
    );
    expect_chain!(
        ["counter", "-f", "-m"],
        p4.cmd_counter().force(true).multi()
    );
    expect_chain!(
        ["counter", "-f", "-m"],
        p4.cmd_counter().set().force(true).multi()
    );

    let mut cmd = echo_counter().multi();
    assert_eq!(
        spawned_args(&mut cmd, (["firstcounter", "5", "secondcounter", "4"],)),
        ["counter", "-m", "firstcounter", "5", "secondcounter", "4"]
    );

    let mut cmd = echo_counter().force(true).multi();
    assert_eq!(
        spawned_args(&mut cmd, (["-", "xset", "-", "yset"],)),
        ["counter", "-f", "-m", "-", "xset", "-", "yset"]
    );
}

// Form 6 -------------------------------------------------------------------
// `p4 counter --from oldvalue --to newvalue countername`
//
// The compare-and-set form stores both values at the transition; the
// accessors can replace them afterwards.

#[cfg(not(feature = "lt2016_1"))]
fn form_6_cas() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["counter", "--from", "3", "--to", "4"],
        p4.cmd_counter().cas("3", "4")
    );
    expect_chain!(
        ["counter", "--from", "5", "--to", "6"],
        p4.cmd_counter().cas("3", "4").old_value("5").new_value("6")
    );
    expect_chain!(
        ["counter", "--from", "7", "--to", "8"],
        p4.cmd_counter().cas("3", "4").new_value("8").old_value("7")
    );

    let mut cmd = p4.cmd_counter().cas("3", "4");
    assert_eq!(cmd.get_old_value(), "3");
    assert_eq!(cmd.get_new_value(), "4");
    cmd.set_old_value("5").set_new_value("6");
    expect_args(&cmd, &["counter", "--from", "5", "--to", "6"]);

    let mut cmd = echo_counter().cas("3", "4");
    assert_eq!(
        spawned_args(&mut cmd, ("mycounter",)),
        ["counter", "--from", "3", "--to", "4", "mycounter"]
    );
}

// Global options ------------------------------------------------------------
// Every form accepts the global options, which are injected before the
// subcommand name.

fn global_options() {
    let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

    let query = Counter::new("p4", global_opts.clone());
    expect_args(&query, &["-p", "localhost:1666", "-q", "counter"]);

    let set = Counter::new("p4", global_opts.clone()).set().force(true);
    expect_args(&set, &["-p", "localhost:1666", "-q", "counter", "-f"]);

    let delete = Counter::new("p4", global_opts.clone()).delete();
    expect_args(&delete, &["-p", "localhost:1666", "-q", "counter", "-d"]);

    let increment = Counter::new("p4", global_opts.clone()).increment();
    expect_args(&increment, &["-p", "localhost:1666", "-q", "counter", "-i"]);

    let multi = Counter::new("p4", global_opts.clone()).multi();
    expect_args(&multi, &["-p", "localhost:1666", "-q", "counter", "-m"]);

    #[cfg(not(feature = "lt2016_1"))]
    {
        let cas = Counter::new("p4", global_opts).cas("3", "4");
        expect_args(
            &cas,
            &[
                "-p",
                "localhost:1666",
                "-q",
                "counter",
                "--from",
                "3",
                "--to",
                "4",
            ],
        );
    }
}

// Rejected combinations -----------------------------------------------------
//
// The chains below contradict the documented forms and must fail to
// compile; they are kept as executable documentation of the type-state
// rules. Uncomment any line to verify it is rejected.
//
// // `-v` cannot be combined with `-d`.
// let _ = P4Cli::default().cmd_counter().show_previous().delete();
//
// // `-v` is not accepted by the delete or multi forms.
// let _ = P4Cli::default().cmd_counter().delete().show_previous();
// let _ = P4Cli::default().cmd_counter().multi().show_previous();
// let _ = P4Cli::default().cmd_counter().show_previous().multi();
//
// // The compare-and-set form accepts neither `-f` nor `-v`.
// let _ = P4Cli::default().cmd_counter().cas("3", "4").force(true);
// let _ = P4Cli::default().cmd_counter().cas("3", "4").show_previous();
//
// // The query form spawns with one operand only; the set form with two.
// let _ = P4Cli::default().cmd_counter().spawn_with(("a", "b"));
// let _ = P4Cli::default().cmd_counter().set().spawn_with(("a",));

fn main() -> io::Result<()> {
    form_1_query();
    form_2_set();
    form_3_delete();
    form_4_increment();
    form_5_multi();
    #[cfg(not(feature = "lt2016_1"))]
    form_6_cas();
    global_options();
    println!("all p4 counter syntax combinations verified");
    Ok(())
}
