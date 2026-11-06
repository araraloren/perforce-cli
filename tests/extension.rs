//! Verification of every documented `p4 extension` syntax form.
//!
//! For each documented command line this walks the legal builder chains
//! through the type-state API, asserting that they all assemble the same
//! arguments and spawn the same argv tail. The reference is the current
//! command reference:
//!
//! ```text
//! p4 [g-opts] extension --sample extName
//! p4 [g-opts] extension --package extName.p4-extension [--sign keyDir]
//! p4 [g-opts] extension --install extName.p4-extension [--yes]
//! p4 [g-opts] extension --install --cert certfile [--comment text]
//! p4 [g-opts] extension --delete extFQN [--name inst-config] [--revision=n] [--path=filespec] [--yes]
//! p4 [g-opts] extension --delete --cert fingerprint
//! p4 [g-opts] extension --configure extFQN [--revision=n] [-i | -o] [--name inst-config]
//! p4 [g-opts] extension --run instName [extArguments]
//! p4 [g-opts] extension --list --type type [--path path]
//! ```
//!
//! The command first appears in 2019.1, so the whole test target is compiled
//! out on older versions. Version notes:
//!
//! - 2019.1: `--sample`, `--package`, `--install`, `--delete`, `--configure`,
//!   and `--list`; `--delete` already has `--yes`/`--revision`/`--path`, and
//!   `--list --type` is limited to `extensions` and `configs`.
//! - 2020.1 adds `--install --yes`, `--delete --name`, `--run`, and the
//!   `global`/`instance` values of `--list --type`.
//! - 2022.2 adds `--sign`, `--install --cert`/`--comment`, `--delete --cert`,
//!   `--allow-unsigned`, `--list --path`, and the `certs` value.

#![cfg(not(feature = "lt2019_1"))]

use perforce_cli::P4Cli;
use perforce_cli::cmd::ExtensionEntry;
use perforce_cli::cmd::extension::Type;
use perforce_cli::global::GlobalOpts;

mod common;
#[allow(unused_imports)]
use common::{echo_bin, expect_args, expect_chain, spawned_args};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Starts a builder whose executable is the local echo helper.
fn echo_extension() -> ExtensionEntry {
    P4Cli::new(echo_bin("extension"), GlobalOpts::new()).cmd_extension()
}

// sample --------------------------------------------------------------------
// `p4 [g-opts] extension --sample extName`

#[test]
fn sample_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--sample", "ExtName"],
        p4.cmd_extension().sample("ExtName")
    );

    let mut cmd = echo_extension().sample("ExtName");
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--sample", "ExtName"]
    );
}

// package -------------------------------------------------------------------
// `p4 [g-opts] extension --package extName.p4-extension [--sign keyDir]`
//
// `--sign` exists from 2022.2.

#[test]
fn package_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--package", "ExtName"],
        p4.cmd_extension().package("ExtName")
    );

    let mut cmd = echo_extension().package("ExtName");
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--package", "ExtName"]
    );

    #[cfg(not(feature = "lt2022_2"))]
    {
        expect_chain!(
            ["extension", "--package", "ExtName", "--sign", "keys"],
            p4.cmd_extension().package("ExtName").sign("keys")
        );

        let mut cmd = echo_extension().package("ExtName").sign("keys");
        assert_eq!(
            spawned_args(&mut cmd, ()),
            ["extension", "--package", "ExtName", "--sign", "keys"]
        );
    }
}

// install -------------------------------------------------------------------
// `p4 [g-opts] extension --install extName.p4-extension [--yes]`
//
// A plain install always assembles without `--yes`; `--yes` exists from
// 2020.1, `--allow-unsigned` from 2022.2.

#[test]
fn install_dry_run() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--install", "ExtName.p4-extension"],
        p4.cmd_extension().install("ExtName.p4-extension")
    );

    let mut cmd = echo_extension().install("ExtName.p4-extension");
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--install", "ExtName.p4-extension"]
    );
}

#[cfg(not(feature = "lt2020_1"))]
#[test]
fn install_with_yes() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--install", "ExtName.p4-extension", "--yes"],
        p4.cmd_extension().install("ExtName.p4-extension").yes()
    );

    // Inside the extension state `--yes` can be toggled off again.
    expect_chain!(
        ["extension", "--install", "ExtName.p4-extension"],
        p4.cmd_extension()
            .install("ExtName.p4-extension")
            .yes()
            .yes(false)
    );

    let mut cmd = echo_extension().install("ExtName.p4-extension").yes();
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--install", "ExtName.p4-extension", "--yes"]
    );

    #[cfg(not(feature = "lt2022_2"))]
    {
        expect_chain!(
            [
                "extension",
                "--install",
                "ExtName.p4-extension",
                "--yes",
                "--allow-unsigned"
            ],
            p4.cmd_extension()
                .install("ExtName.p4-extension")
                .yes()
                .allow_unsigned(true)
        );

        // `--allow-unsigned` alone is accepted as well.
        expect_chain!(
            [
                "extension",
                "--install",
                "ExtName.p4-extension",
                "--allow-unsigned"
            ],
            p4.cmd_extension()
                .install("ExtName.p4-extension")
                .yes()
                .yes(false)
                .allow_unsigned(true)
        );
    }
}

// `p4 [g-opts] extension --install --cert certfile [--comment text]`
//
// Both options exist from 2022.2; `--comment` always implies `--cert`.

#[cfg(not(feature = "lt2022_2"))]
#[test]
fn install_cert_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--install", "cert.txt", "--cert"],
        p4.cmd_extension().install("cert.txt").cert()
    );
    expect_chain!(
        [
            "extension",
            "--install",
            "cert.txt",
            "--cert",
            "--comment",
            "We trust ACTools"
        ],
        p4.cmd_extension()
            .install("cert.txt")
            .comment("We trust ACTools")
    );

    // The echo helper splits its argv tail on whitespace, so the spawned
    // round-trip uses a space-free comment; the exact official example
    // string is asserted by the dry runs above.
    let mut cmd = echo_extension().install("cert.txt").comment("Trusted-CA");
    assert_eq!(
        spawned_args(&mut cmd, ()),
        [
            "extension",
            "--install",
            "cert.txt",
            "--cert",
            "--comment",
            "Trusted-CA"
        ]
    );
}

// delete --------------------------------------------------------------------
// `p4 [g-opts] extension --delete extFQN [--name] [--revision=n] [--path] [--yes]`
//
// The narrow options may be selected in any builder order but inject in the
// documented `--name`, `--revision`, `--path`, `--yes`, `--allow-unsigned`
// order. `--name` exists from 2020.1, `--cert` and `--allow-unsigned` from
// 2022.2.

#[test]
fn delete_dry_run() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--delete", "myCompany::ext"],
        p4.cmd_extension().delete("myCompany::ext")
    );

    let mut cmd = echo_extension().delete("myCompany::ext");
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--delete", "myCompany::ext"]
    );
}

#[test]
fn delete_ext_options() {
    let p4 = P4Cli::default();

    expect_chain!(
        [
            "extension",
            "--delete",
            "myCompany::ext",
            "--revision=2",
            "--path",
            "//depot/...",
            "--yes"
        ],
        p4.cmd_extension()
            .delete("myCompany::ext")
            .yes()
            .path("//depot/...")
            .revision("2")
    );

    let mut cmd = echo_extension()
        .delete("myCompany::ext")
        .revision("2")
        .path("//depot/...")
        .yes(true);
    assert_eq!(
        spawned_args(&mut cmd, ()),
        [
            "extension",
            "--delete",
            "myCompany::ext",
            "--revision=2",
            "--path",
            "//depot/...",
            "--yes"
        ]
    );

    #[cfg(not(feature = "lt2020_1"))]
    {
        expect_chain!(
            [
                "extension",
                "--delete",
                "myCompany::ext",
                "--name",
                "inst",
                "--revision=2",
                "--yes"
            ],
            p4.cmd_extension()
                .delete("myCompany::ext")
                .revision("2")
                .name("inst")
                .yes(true)
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    {
        expect_chain!(
            [
                "extension",
                "--delete",
                "myCompany::ext",
                "--yes",
                "--allow-unsigned"
            ],
            p4.cmd_extension()
                .delete("myCompany::ext")
                .yes()
                .allow_unsigned(true)
        );
    }
}

// `p4 [g-opts] extension --delete --cert fingerprint`
//
// Exists from 2022.2.

#[cfg(not(feature = "lt2022_2"))]
#[test]
fn delete_cert_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--delete", "AA:BB:CC", "--cert"],
        p4.cmd_extension().delete("AA:BB:CC").cert()
    );

    let mut cmd = echo_extension().delete("AA:BB:CC").cert();
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--delete", "AA:BB:CC", "--cert"]
    );
}

// configure -----------------------------------------------------------------
// `p4 [g-opts] extension --configure extFQN [--revision=n] [-i | -o] [--name inst-config]`
//
// `-i` (stdin) and `-o` (stdout) are mutually exclusive states of the form
// output; `--name` always injects last, after the selected state.

#[test]
fn configure_forms() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--configure", "myCompany::ext"],
        p4.cmd_extension().configure("myCompany::ext")
    );
    expect_chain!(
        [
            "extension",
            "--configure",
            "myCompany::ext",
            "--name",
            "inst"
        ],
        p4.cmd_extension().configure("myCompany::ext").name("inst")
    );
    expect_chain!(
        [
            "extension",
            "--configure",
            "myCompany::ext",
            "--revision=2",
            "--name",
            "inst"
        ],
        p4.cmd_extension()
            .configure("myCompany::ext")
            .name("inst")
            .revision("2")
    );
    expect_chain!(
        [
            "extension",
            "--configure",
            "myCompany::ext",
            "--revision=2",
            "-i",
            "--name",
            "inst"
        ],
        p4.cmd_extension()
            .configure("myCompany::ext")
            .revision("2")
            .name("inst")
            .stdin()
    );
    expect_chain!(
        ["extension", "--configure", "myCompany::ext", "-o"],
        p4.cmd_extension().configure("myCompany::ext").stdout()
    );

    let mut cmd = echo_extension()
        .configure("myCompany::ext")
        .revision("2")
        .name("inst")
        .stdin();
    assert_eq!(
        spawned_args(&mut cmd, ()),
        [
            "extension",
            "--configure",
            "myCompany::ext",
            "--revision=2",
            "-i",
            "--name",
            "inst"
        ]
    );
}

// The `-i` state spawns with a caller-supplied standard input configuration
// (`Stdio::piped()` for a writable `child.stdin`, `Stdio::inherit()` to read
// from the parent); the `-o` state spawns with a caller-supplied standard
// output configuration.

#[test]
fn configure_stdin_spawns_with_stdio() {
    use std::process::Stdio;

    use perforce_cli::spawn::ParameterizedSpawn;

    let mut cmd = echo_extension().configure("myCompany::ext").stdin();
    let output = cmd
        .spawn_with((Stdio::null(),))
        .expect("spawn echo helper")
        .wait_with_output()
        .expect("wait for echo helper");
    assert!(output.status.success(), "echo helper exited unsuccessfully");
    assert_eq!(
        common::parse_echo(&output.stdout),
        ["extension", "--configure", "myCompany::ext", "-i"]
    );
}

#[test]
fn configure_stdout_spawns_with_stdio() {
    use std::process::Stdio;

    use perforce_cli::spawn::ParameterizedSpawn;

    let mut cmd = echo_extension().configure("myCompany::ext").stdout();
    let output = cmd
        .spawn_with((Stdio::piped(),))
        .expect("spawn echo helper")
        .wait_with_output()
        .expect("wait for echo helper");
    assert!(output.status.success(), "echo helper exited unsuccessfully");
    assert_eq!(
        common::parse_echo(&output.stdout),
        ["extension", "--configure", "myCompany::ext", "-o"]
    );
}

// run -----------------------------------------------------------------------
// `p4 [g-opts] extension --run instName [extArguments]`
//
// `--run` exists from 2020.1. Without extra arguments the command spawns
// through the plain `ParameterizedSpawn<()>` impl; the arbitrary argument
// list is a separate `ParameterizedSpawn<(S,)>` impl appended at spawn time.

#[cfg(not(feature = "lt2020_1"))]
#[test]
fn run_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--run", "inst"],
        p4.cmd_extension().run("inst")
    );

    let mut cmd = echo_extension().run("inst");
    assert_eq!(spawned_args(&mut cmd, ()), ["extension", "--run", "inst"]);
}

#[cfg(not(feature = "lt2020_1"))]
#[test]
fn run_with_extra_arguments() {
    let mut cmd = echo_extension().run("inst");
    assert_eq!(
        spawned_args(&mut cmd, (["--fast", "--verbose"],)),
        ["extension", "--run", "inst", "--fast", "--verbose"]
    );

    let mut cmd = echo_extension().run("inst");
    assert_eq!(
        spawned_args(&mut cmd, (vec![String::from("a"), String::from("b")],)),
        ["extension", "--run", "inst", "a", "b"]
    );
}

// list ----------------------------------------------------------------------
// `p4 [g-opts] extension --list --type type [--path path]`
//
// `--type` is required and always injects directly after `--list`; the
// `global`/`instance` values exist from 2020.1, `certs` and `--path` from
// 2022.2. The per-type shortcuts build the same command line as `list`.

#[test]
fn list_form() {
    let p4 = P4Cli::default();

    expect_chain!(
        ["extension", "--list", "--type", "extensions"],
        p4.cmd_extension().list(Type::Extensions)
    );
    expect_chain!(
        ["extension", "--list", "--type", "extensions"],
        p4.cmd_extension().list_extensions()
    );
    expect_chain!(
        ["extension", "--list", "--type", "configs"],
        p4.cmd_extension().list_configs()
    );

    #[cfg(not(feature = "lt2020_1"))]
    {
        expect_chain!(
            ["extension", "--list", "--type", "global"],
            p4.cmd_extension().list_global()
        );
        expect_chain!(
            ["extension", "--list", "--type", "instance"],
            p4.cmd_extension().list_instance()
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    {
        expect_chain!(
            ["extension", "--list", "--type", "certs"],
            p4.cmd_extension().list_certs()
        );
        expect_chain!(
            [
                "extension",
                "--list",
                "--type",
                "configs",
                "--path",
                "//depot/..."
            ],
            p4.cmd_extension().list(Type::Configs).path("//depot/...")
        );
    }

    let mut cmd = echo_extension().list(Type::Extensions);
    assert_eq!(
        spawned_args(&mut cmd, ()),
        ["extension", "--list", "--type", "extensions"]
    );
}

// global options ------------------------------------------------------------
// Global options inject once, before the command name.

#[test]
fn global_opts_are_injected_once() {
    let p4 = P4Cli::new("p4", GlobalOpts::new().user("alice"));

    expect_chain!(
        ["-u", "alice", "extension", "--sample", "ExtName"],
        p4.cmd_extension().sample("ExtName")
    );
    expect_chain!(
        ["-u", "alice", "extension", "--list", "--type", "extensions"],
        p4.cmd_extension().list(Type::Extensions)
    );
}
