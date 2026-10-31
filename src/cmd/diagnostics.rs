use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};
use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Journal state (-J [-L limit])
// ---------------------------------------------------------------------------

/// The journal and log copy state of `p4 diagnostics`.
///
#[cfg_attr(
    feature = "lt2025_2",
    doc = "Carries the optional `-L limit` that caps the downloaded journal and log copy, which the basic form always includes."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "Downloads a copy of the current journal file and log file (`-J`), optionally capped by `-L limit`."
)]
///
/// Entered with [`Diagnostics::journal`].
#[derive(Debug, Clone, Default)]
pub struct Journal {
    limit: Option<String>,
}

impl ExclusiveOption for Journal {
    fn inject_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2025_2"))]
        command.arg("-J");

        if let Some(limit) = &self.limit {
            command.arg("-L").arg(limit);
        }
    }
}

// ---------------------------------------------------------------------------
// Strace state (-s [--strace-runtime secs])
// ---------------------------------------------------------------------------

/// The `strace` state of the advance form of `p4 diagnostics`.
///
#[cfg_attr(
    feature = "lt2025_2",
    doc = "Runs an `strace` command on the server processes for 20 seconds (`-s`)."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "Runs an `strace` command on the server processes (`-s`), for 20 seconds by default or for the number of seconds given by `--strace-runtime secs`."
)]
///
/// Entered with [`Diagnostics::run_strace`] or
/// [`Diagnostics::run_strace_with`].
#[derive(Debug, Clone, Default)]
pub struct Strace {
    #[cfg(not(feature = "lt2025_2"))]
    runtime: Option<u64>,
}

impl ExclusiveOption for Strace {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s");

        #[cfg(not(feature = "lt2025_2"))]
        if let Some(runtime) = self.runtime {
            command.arg("--strace-runtime").arg(runtime.to_string());
        }
    }
}

// ---------------------------------------------------------------------------
// Basic form state (-j, -J [-L limit])
// ---------------------------------------------------------------------------

/// The basic form of `p4 diagnostics`.
///
#[cfg_attr(
    feature = "lt2025_2",
    doc = "Selected by `-L limit`; [`Diagnostics::basic`] enters this form explicitly."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "Selected by `-j`, `-J`, or `-L limit`; [`Diagnostics::basic`] and [`Diagnostics::json_format`] enter this form."
)]
///
/// The `J` type parameter tracks the [`Journal`] state, starting as
/// [`Unselected`].
#[derive(Debug, Clone, Default)]
pub struct Basic<J = Unselected> {
    #[cfg(not(feature = "lt2025_2"))]
    json_format: bool,

    journal: J,
}

impl<J: ExclusiveOption> ExclusiveOption for Basic<J> {
    fn inject_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2025_2"))]
        if self.json_format {
            command.arg("-j");
        }

        self.journal.inject_args(command);
    }
}

// ---------------------------------------------------------------------------
// Basic-files state of the advance form (-a [-J [-L limit]])
// ---------------------------------------------------------------------------

/// The basic diagnostic files state of the advance form of `p4 diagnostics`.
///
/// Requests that the basic diagnostic files are produced alongside the
/// advance-form data (`-a`). Entered with
/// [`Diagnostics::advance_with_basic`] or [`Diagnostics::with_basic`];
/// [`Diagnostics::journal`] then selects whether the journal and log copy is
/// also downloaded.
///
/// The `J` type parameter tracks the [`Journal`] state, starting as
/// [`Unselected`].
#[derive(Debug, Clone, Default)]
pub struct WithBasic<J = Unselected> {
    journal: J,
}

impl<J: ExclusiveOption> ExclusiveOption for WithBasic<J> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");

        self.journal.inject_args(command);
    }
}

// ---------------------------------------------------------------------------
// Advance form state (-l, -s, -p, -t, -a)
// ---------------------------------------------------------------------------

/// The advance form of `p4 diagnostics`.
///
#[cfg_attr(
    feature = "lt2025_2",
    doc = "Selected by `-l`, `-s`, or `-a`; [`Diagnostics::advance`] and [`Diagnostics::advance_with_basic`] enter this form explicitly."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "Selected by `-l`, `-s`, `-p`, `-t`, or `-a`; [`Diagnostics::advance`] and [`Diagnostics::advance_with_basic`] enter this form explicitly."
)]
///
/// The `S` type parameter tracks the [`Strace`] state and the `B` type
/// parameter tracks the [`WithBasic`] state, each starting as
/// [`Unselected`].
#[derive(Debug, Clone, Default)]
pub struct Advance<S = Unselected, B = Unselected> {
    run_lsof: bool,

    #[cfg(not(feature = "lt2025_2"))]
    run_ps: bool,

    #[cfg(not(feature = "lt2025_2"))]
    run_top: bool,

    strace: S,

    basic: B,
}

impl<S: ExclusiveOption, B: ExclusiveOption> ExclusiveOption for Advance<S, B> {
    fn inject_args(&self, command: &mut Command) {
        if self.run_lsof {
            command.arg("-l");
        }

        self.strace.inject_args(command);

        #[cfg(not(feature = "lt2025_2"))]
        if self.run_ps {
            command.arg("-p");
        }

        #[cfg(not(feature = "lt2025_2"))]
        if self.run_top {
            command.arg("-t");
        }

        self.basic.inject_args(command);
    }
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2025_2",
    doc = "`p4 [g-opts] diagnostics [-L limit]`\n\n\
           `p4 [g-opts] diagnostics [-l] [-s] [-a [-L limit]]`"
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "`p4 [g-opts] diagnostics [-D] [-j] [-J [-L limit]]`\n\n\
           `p4 [g-opts] diagnostics [-D] [-l] [-s [--strace-runtime secs]] [-p] [-t] [-a [-J [-L limit]]]`"
)]
///
/// Generate diagnostic data about the P4 Server.
///
#[cfg_attr(
    feature = "lt2025_2",
    doc = "This command is a Technology Preview feature. Features offered in Technology Preview are experimental and not guaranteed to always work as expected. If you have feedback and functionality suggestions, email techpreview@perforce.com.",
    doc = "",
    doc = "The output format is subject to change. This command is currently intended for use under guidance by Perforce Support."
)]
#[cfg_attr(
    feature = "lt2025_2",
    doc = "",
    doc = "The first form of the command, `p4 [g-opts] diagnostics [-L limit]`, generates the basic diagnostic data and downloads it to a number of fixed named files in the current directory:",
    doc = "",
    doc = "- `diags` - basic configuration details, configuration, configuration history, server setting, and checkpoint history.",
    doc = "- `monitor` - the monitor records, if monitoring is enabled.",
    doc = "- `topology` - a textual description of the network that contains this server.",
    doc = "- A copy of the current `journal` and `log` files, using the configured names of these files.",
    doc = "",
    doc = "The second form of the command, `p4 [g-opts] diagnostics [-l] [-s] [-a [-L limit]]`, provides information that might take more time and require more server resources."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "",
    doc = "The first form of the command generates the basic diagnostic data, and downloads a compressed tar file (`.tgz`) that contains these files:",
    doc = "",
    doc = "- `diags` - basic configuration details, configuration, configuration history, server setting.",
    doc = "- `info` - tagged output from the `p4 info` and `p4 servers -J` commands, and a list of all non-default configurables.",
    doc = "- `monitor` - the monitor records, if monitoring is enabled.",
    doc = "- `topology` - a textual description of the network that contains this server. Servers that do not have a serverID are ignored.",
    doc = "",
    doc = "The second form of the command provides information that might take more time and require more server resources."
)]
///
#[cfg_attr(
    feature = "lt2025_2",
    doc = "The `M` type parameter tracks the command form at compile time. The default [`Unselected`] state selects the basic form with no local options; `journal` and `basic` enter the [`Basic`] state, while `advance`, `advance_with_basic`, and the `run_*` transitions enter the [`Advance`] state."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "The `M` type parameter tracks the command form at compile time. The default [`Unselected`] state selects the basic form with no local options; `journal`, `json_format`, and `basic` enter the [`Basic`] state, while `advance`, `advance_with_basic`, and the `run_*` transitions enter the [`Advance`] state."
)]
#[derive(Debug, Clone, Default)]
pub struct Diagnostics<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    #[cfg(not(feature = "lt2025_2"))]
    uncompressed: bool,

    mode: M,
}

impl Diagnostics<Unselected> {
    /// Creates a new `p4 diagnostics` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: false,
            mode: Unselected,
        }
    }

    /// Enters the [`Basic`] form of the command with no local options
    /// selected; equivalent to the default [`Unselected`] state.
    pub fn basic(self) -> Diagnostics<Basic<Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Basic {
                #[cfg(not(feature = "lt2025_2"))]
                json_format: false,
                journal: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -j
    ///
    /// Request that the `diags` and `monitor` files are encoded in JSON
    /// format. These output files are named `diags.json` and `monitor.json`.
    /// Any non-UFT8 sequences in a string are replaced by U+FFFD in the JSON
    /// output.
    ///
    /// Enters the [`Basic`] form of the command.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn json_format(self) -> Diagnostics<Basic<Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            uncompressed: self.uncompressed,
            mode: Basic {
                json_format: true,
                journal: Unselected,
            },
        }
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2025_2", doc = "-L limit")]
    #[cfg_attr(not(feature = "lt2025_2"), doc = "-J [-L limit]")]
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Caps the journal and log copy, which the basic form always includes, at the last `limit` bytes."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Downloads a copy of the current journal file and log file, using the configured names of these files."
    )]
    ///
    /// Enters the [`Basic`] form of the command with the [`Journal`] state
    /// selected.
    pub fn journal(self) -> Diagnostics<Basic<Journal>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Basic {
                #[cfg(not(feature = "lt2025_2"))]
                json_format: false,
                journal: Journal::default(),
            },
        }
    }

    /// Enters the [`Advance`] form of the command with no local options
    /// selected.
    pub fn advance(self) -> Diagnostics<Advance<Unselected, Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: false,
                strace: Unselected,
                basic: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -a
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Requests that the basic diagnostic files are produced. Without this option, those files are omitted with the `-l` and `-s` options."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Requests that the basic diagnostic files are produced. Without this option, those files are omitted with the `-l`, `-t`, `-p`, and `-s` options."
    )]
    ///
    /// Enters the [`Advance`] form of the command with the [`WithBasic`]
    /// state selected.
    pub fn advance_with_basic(self) -> Diagnostics<Advance<Unselected, WithBasic<Unselected>>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: false,
                strace: Unselected,
                basic: WithBasic::default(),
            },
        }
    }

    /// # Description
    ///
    /// -l
    ///
    /// Request that the `/usr/bin/lsof` be run to provide locking details of
    /// all files in use by P4 Servers on this system.
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "The data from this operation is downloaded into a file named `manuallocks` in the current directory."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "The data from this operation is downloaded into a file named `manuallocks`."
    )]
    /// *This option is not supported on Windows.*
    ///
    /// Enters the [`Advance`] form of the command.
    pub fn run_lsof(self) -> Diagnostics<Advance<Unselected, Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: true,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: false,
                strace: Unselected,
                basic: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -p
    ///
    /// Run the `ps` program and returns its output.
    ///
    /// Enters the [`Advance`] form of the command.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn run_ps(self) -> Diagnostics<Advance<Unselected, Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: false,
                run_ps: true,
                run_top: false,
                strace: Unselected,
                basic: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -t
    ///
    /// Run the `top` program and returns its output.
    ///
    /// Enters the [`Advance`] form of the command.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn run_top(self) -> Diagnostics<Advance<Unselected, Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: false,
                run_ps: false,
                run_top: true,
                strace: Unselected,
                basic: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -s
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Run an `strace` command on the server processes for 20 seconds and downloads the trace output to a file named `strace` in the current directory. *This option is not yet supported on any platform.*"
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Run an `strace` command on the server processes for 20 seconds. The data from this operation is downloaded into a file named `strace`. *The `strace` Linux command is not supported on Windows.*"
    )]
    ///
    /// Enters the [`Advance`] form of the command with the [`Strace`] state
    /// selected.
    pub fn run_strace(self) -> Diagnostics<Advance<Strace, Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: false,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: false,
                strace: Strace::default(),
                basic: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// `-s --strace-runtime secs`
    ///
    /// Run an `strace` command on the server processes for `secs` seconds
    /// instead of the default 20. The data from this operation is
    /// downloaded into a file named `strace`.
    /// *The `strace` Linux command is not supported on Windows.*
    ///
    /// Enters the [`Advance`] form of the command with the [`Strace`] state
    /// selected.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn run_strace_with(self, runtime: u64) -> Diagnostics<Advance<Strace, Unselected>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: false,
                run_ps: false,
                run_top: false,
                strace: Strace {
                    runtime: Some(runtime),
                },
                basic: Unselected,
            },
        }
    }
}

impl<J: ExclusiveOption> Diagnostics<Basic<J>> {
    /// # Description
    ///
    /// -j
    ///
    /// Request that the `diags` and `monitor` files are encoded in JSON
    /// format. These output files are named `diags.json` and `monitor.json`.
    /// Any non-UFT8 sequences in a string are replaced by U+FFFD in the JSON
    /// output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_json_format(&self) -> bool {
        self.mode.json_format
    }

    /// # Description
    ///
    /// -j
    ///
    /// Request that the `diags` and `monitor` files are encoded in JSON
    /// format. These output files are named `diags.json` and `monitor.json`.
    /// Any non-UFT8 sequences in a string are replaced by U+FFFD in the JSON
    /// output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_json_format(&mut self, v: bool) -> &mut Self {
        self.mode.json_format = v;
        self
    }

    /// # Description
    ///
    /// -j
    ///
    /// Request that the `diags` and `monitor` files are encoded in JSON
    /// format. These output files are named `diags.json` and `monitor.json`.
    /// Any non-UFT8 sequences in a string are replaced by U+FFFD in the JSON
    /// output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn json_format(mut self, v: bool) -> Self {
        self.mode.json_format = v;
        self
    }
}

impl Diagnostics<Basic<Unselected>> {
    /// # Description
    ///
    #[cfg_attr(feature = "lt2025_2", doc = "-L limit")]
    #[cfg_attr(not(feature = "lt2025_2"), doc = "-J [-L limit]")]
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Caps the journal and log copy, which the basic form always includes, at the last `limit` bytes."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Downloads a copy of the current journal file and log file, using the configured names of these files."
    )]
    pub fn journal(self) -> Diagnostics<Basic<Journal>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Basic {
                #[cfg(not(feature = "lt2025_2"))]
                json_format: self.mode.json_format,
                journal: Journal::default(),
            },
        }
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2025_2", doc = "-L limit")]
    #[cfg_attr(not(feature = "lt2025_2"), doc = "-J [-L limit]")]
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Caps the journal and log copy, which the basic form always includes, at the last `limit` bytes."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Downloads a copy of the current journal file and log file, using the configured names of these files."
    )]
    ///
    /// Enters the [`Basic`] form of the command with the [`Journal`] state
    /// selected and the download limited to `limit`.
    pub fn journal_with(self, limit: impl Into<String>) -> Diagnostics<Basic<Journal>> {
        self.journal().limit(limit)
    }
}

impl Diagnostics<Basic<Journal>> {
    /// # Description
    ///
    /// -L limit
    ///
    /// Limit the download size of the journal and log files to the last
    /// `limit` number of bytes of these files, subject to buffer rounding. The
    /// `limit` is in bytes and can be suffixed with a single character to
    /// designate a unit multiplier: `k` or `K` for kilobytes, `m` or `M` for
    /// megabytes, `g` or `G` for gigabytes, and `t` or `T` for terabytes.
    pub fn get_limit(&self) -> Option<&str> {
        self.mode.journal.limit.as_deref()
    }

    /// # Description
    ///
    /// -L limit
    ///
    /// Limit the download size of the journal and log files to the last
    /// `limit` number of bytes of these files, subject to buffer rounding. The
    /// `limit` is in bytes and can be suffixed with a single character to
    /// designate a unit multiplier: `k` or `K` for kilobytes, `m` or `M` for
    /// megabytes, `g` or `G` for gigabytes, and `t` or `T` for terabytes.
    pub fn set_limit(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.journal.limit = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -L limit
    ///
    /// Limit the download size of the journal and log files to the last
    /// `limit` number of bytes of these files, subject to buffer rounding. The
    /// `limit` is in bytes and can be suffixed with a single character to
    /// designate a unit multiplier: `k` or `K` for kilobytes, `m` or `M` for
    /// megabytes, `g` or `G` for gigabytes, and `t` or `T` for terabytes.
    pub fn limit(mut self, v: impl Into<String>) -> Self {
        self.mode.journal.limit = Some(v.into());
        self
    }
}

impl<S: ExclusiveOption, B: ExclusiveOption> Diagnostics<Advance<S, B>> {
    /// # Description
    ///
    /// -l
    ///
    /// Request that the `/usr/bin/lsof` be run to provide locking details of
    /// all files in use by P4 Servers on this system.
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "The data from this operation is downloaded into a file named `manuallocks` in the current directory."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "The data from this operation is downloaded into a file named `manuallocks`."
    )]
    /// *This option is not supported on Windows.*
    pub fn get_run_lsof(&self) -> bool {
        self.mode.run_lsof
    }

    /// # Description
    ///
    /// -l
    ///
    /// Request that the `/usr/bin/lsof` be run to provide locking details of
    /// all files in use by P4 Servers on this system.
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "The data from this operation is downloaded into a file named `manuallocks` in the current directory."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "The data from this operation is downloaded into a file named `manuallocks`."
    )]
    /// *This option is not supported on Windows.*
    pub fn set_run_lsof(&mut self, v: bool) -> &mut Self {
        self.mode.run_lsof = v;
        self
    }

    /// # Description
    ///
    /// -l
    ///
    /// Request that the `/usr/bin/lsof` be run to provide locking details of
    /// all files in use by P4 Servers on this system.
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "The data from this operation is downloaded into a file named `manuallocks` in the current directory."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "The data from this operation is downloaded into a file named `manuallocks`."
    )]
    /// *This option is not supported on Windows.*
    pub fn run_lsof(mut self, v: bool) -> Self {
        self.mode.run_lsof = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    /// Run the `ps` program and returns its output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_run_ps(&self) -> bool {
        self.mode.run_ps
    }

    /// # Description
    ///
    /// -p
    ///
    /// Run the `ps` program and returns its output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_run_ps(&mut self, v: bool) -> &mut Self {
        self.mode.run_ps = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    /// Run the `ps` program and returns its output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn run_ps(mut self, v: bool) -> Self {
        self.mode.run_ps = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Run the `top` program and returns its output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_run_top(&self) -> bool {
        self.mode.run_top
    }

    /// # Description
    ///
    /// -t
    ///
    /// Run the `top` program and returns its output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_run_top(&mut self, v: bool) -> &mut Self {
        self.mode.run_top = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Run the `top` program and returns its output.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn run_top(mut self, v: bool) -> Self {
        self.mode.run_top = v;
        self
    }

    /// # Description
    ///
    /// -s
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Run an `strace` command on the server processes for 20 seconds and downloads the trace output to a file named `strace` in the current directory. *This option is not yet supported on any platform.*"
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Run an `strace` command on the server processes for 20 seconds. The data from this operation is downloaded into a file named `strace`. *The `strace` Linux command is not supported on Windows.*"
    )]
    pub fn run_strace(self) -> Diagnostics<Advance<Strace, B>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: self.mode.run_lsof,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: self.mode.run_ps,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: self.mode.run_top,
                strace: Strace::default(),
                basic: self.mode.basic,
            },
        }
    }

    /// # Description
    ///
    /// `-s --strace-runtime secs`
    ///
    /// Run an `strace` command on the server processes for `secs` seconds
    /// instead of the default 20. The data from this operation is
    /// downloaded into a file named `strace`.
    /// *The `strace` Linux command is not supported on Windows.*
    #[cfg(not(feature = "lt2025_2"))]
    pub fn run_strace_with(self, runtime: u64) -> Diagnostics<Advance<Strace, B>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: self.mode.run_lsof,
                run_ps: self.mode.run_ps,
                run_top: self.mode.run_top,
                strace: Strace {
                    runtime: Some(runtime),
                },
                basic: self.mode.basic,
            },
        }
    }
}

impl<S: ExclusiveOption> Diagnostics<Advance<S, Unselected>> {
    /// # Description
    ///
    /// -a
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Requests that the basic diagnostic files are produced. Without this option, those files are omitted with the `-l` and `-s` options."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Requests that the basic diagnostic files are produced. Without this option, those files are omitted with the `-l`, `-t`, `-p`, and `-s` options."
    )]
    ///
    /// Transitions this command to the [`WithBasic`] state.
    pub fn with_basic(self) -> Diagnostics<Advance<S, WithBasic<Unselected>>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: self.mode.run_lsof,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: self.mode.run_ps,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: self.mode.run_top,
                strace: self.mode.strace,
                basic: WithBasic::default(),
            },
        }
    }
}

impl<S: ExclusiveOption> Diagnostics<Advance<S, WithBasic<Unselected>>> {
    /// # Description
    ///
    #[cfg_attr(feature = "lt2025_2", doc = "-L limit")]
    #[cfg_attr(not(feature = "lt2025_2"), doc = "-J [-L limit]")]
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Caps the journal and log copy, which is included with the basic diagnostic files, at the last `limit` bytes."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Downloads a copy of the current journal file and log file, using the configured names of these files."
    )]
    ///
    /// Transitions this command to the [`Journal`] state.
    pub fn journal(self) -> Diagnostics<Advance<S, WithBasic<Journal>>> {
        Diagnostics {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2025_2"))]
            uncompressed: self.uncompressed,
            mode: Advance {
                run_lsof: self.mode.run_lsof,
                #[cfg(not(feature = "lt2025_2"))]
                run_ps: self.mode.run_ps,
                #[cfg(not(feature = "lt2025_2"))]
                run_top: self.mode.run_top,
                strace: self.mode.strace,
                basic: WithBasic {
                    journal: Journal::default(),
                },
            },
        }
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2025_2", doc = "-L limit")]
    #[cfg_attr(not(feature = "lt2025_2"), doc = "-J [-L limit]")]
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Caps the journal and log copy, which is included with the basic diagnostic files, at the last `limit` bytes."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Downloads a copy of the current journal file and log file, using the configured names of these files."
    )]
    ///
    /// Transitions this command to the [`Journal`] state with the download
    /// limited to `limit`.
    pub fn journal_with(
        self,
        limit: impl Into<String>,
    ) -> Diagnostics<Advance<S, WithBasic<Journal>>> {
        self.journal().limit(limit)
    }
}

impl<S: ExclusiveOption> Diagnostics<Advance<S, WithBasic<Journal>>> {
    /// # Description
    ///
    /// -L limit
    ///
    /// Limit the download size of the journal and log files to the last
    /// `limit` number of bytes of these files, subject to buffer rounding. The
    /// `limit` is in bytes and can be suffixed with a single character to
    /// designate a unit multiplier: `k` or `K` for kilobytes, `m` or `M` for
    /// megabytes, `g` or `G` for gigabytes, and `t` or `T` for terabytes.
    pub fn get_limit(&self) -> Option<&str> {
        self.mode.basic.journal.limit.as_deref()
    }

    /// # Description
    ///
    /// -L limit
    ///
    /// Limit the download size of the journal and log files to the last
    /// `limit` number of bytes of these files, subject to buffer rounding. The
    /// `limit` is in bytes and can be suffixed with a single character to
    /// designate a unit multiplier: `k` or `K` for kilobytes, `m` or `M` for
    /// megabytes, `g` or `G` for gigabytes, and `t` or `T` for terabytes.
    pub fn set_limit(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.basic.journal.limit = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -L limit
    ///
    /// Limit the download size of the journal and log files to the last
    /// `limit` number of bytes of these files, subject to buffer rounding. The
    /// `limit` is in bytes and can be suffixed with a single character to
    /// designate a unit multiplier: `k` or `K` for kilobytes, `m` or `M` for
    /// megabytes, `g` or `G` for gigabytes, and `t` or `T` for terabytes.
    pub fn limit(mut self, v: impl Into<String>) -> Self {
        self.mode.basic.journal.limit = Some(v.into());
        self
    }
}

impl<B: ExclusiveOption> Diagnostics<Advance<Strace, B>> {
    /// # Description
    ///
    /// `--strace-runtime secs`
    ///
    /// Run the `strace` command for `secs` seconds instead of the default 20.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_strace_runtime(&self) -> Option<u64> {
        self.mode.strace.runtime
    }

    /// # Description
    ///
    /// `--strace-runtime secs`
    ///
    /// Run the `strace` command for `secs` seconds instead of the default 20.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_strace_runtime(&mut self, v: u64) -> &mut Self {
        self.mode.strace.runtime = Some(v);
        self
    }

    /// # Description
    ///
    /// `--strace-runtime secs`
    ///
    /// Run the `strace` command for `secs` seconds instead of the default 20.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn strace_runtime(mut self, v: u64) -> Self {
        self.mode.strace.runtime = Some(v);
        self
    }
}

impl<M: ExclusiveOption> Diagnostics<M> {
    /// Get the Perforce command path.
    pub fn get_bin(&self) -> &Path {
        &self.bin
    }

    /// Set the Perforce command path.
    pub fn set_bin(&mut self, bin: impl Into<PathBuf>) -> &mut Self {
        self.bin = bin.into();
        self
    }

    /// Set the Perforce command path.
    pub fn bin(mut self, bin: impl Into<PathBuf>) -> Self {
        self.set_bin(bin);
        self
    }

    /// # Description
    ///
    /// -D
    ///
    /// Create a new directory and download the requested uncompressed
    /// individual files into this new directory. Without this option,
    /// download a single compressed tar file.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_uncompressed(&self) -> bool {
        self.uncompressed
    }

    /// # Description
    ///
    /// -D
    ///
    /// Create a new directory and download the requested uncompressed
    /// individual files into this new directory. Without this option,
    /// download a single compressed tar file.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_uncompressed(&mut self, v: bool) -> &mut Self {
        self.uncompressed = v;
        self
    }

    /// # Description
    ///
    /// -D
    ///
    /// Create a new directory and download the requested uncompressed
    /// individual files into this new directory. Without this option,
    /// download a single compressed tar file.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn uncompressed(mut self, v: bool) -> Self {
        self.uncompressed = v;
        self
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn set_global_opts(&mut self, v: GlobalOpts) -> &mut Self {
        self.global_opts = v;
        self
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.global_opts = v;
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Diagnostics<M> {
    fn name(&self) -> &str {
        "diagnostics"
    }

    fn inject_local_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2025_2"))]
        if self.uncompressed {
            command.arg("-D");
        }

        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption> ParameterizedSpawn<()> for Diagnostics<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diagnostics` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    fn args(cmd: &impl SubCommand) -> Vec<String> {
        args_of(&cmd.setup_command("p4"))
    }

    #[test]
    fn plain_basic_form() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new());
        assert_eq!(args(&cmd), ["diagnostics"]);
        assert_eq!(args(&cmd.basic()), ["diagnostics"]);
    }

    #[test]
    fn journal_with_limit() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new())
            .journal()
            .limit("10m");

        assert_eq!(cmd.get_limit(), Some("10m"));

        #[cfg(not(feature = "lt2025_2"))]
        assert_eq!(args(&cmd), ["diagnostics", "-J", "-L", "10m"]);

        #[cfg(feature = "lt2025_2")]
        assert_eq!(args(&cmd), ["diagnostics", "-L", "10m"]);
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn json_format_with_journal() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new())
            .json_format()
            .journal()
            .limit("1g");

        assert!(cmd.get_json_format());
        assert_eq!(args(&cmd), ["diagnostics", "-j", "-J", "-L", "1g"]);
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn uncompressed_after_subcommand() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new()).uncompressed(true);
        assert_eq!(args(&cmd), ["diagnostics", "-D"]);

        let cmd = cmd.journal();
        assert_eq!(args(&cmd), ["diagnostics", "-D", "-J"]);
    }

    #[test]
    fn advance_lsof_and_strace() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new())
            .run_lsof()
            .run_strace();

        assert!(cmd.get_run_lsof());
        assert_eq!(args(&cmd), ["diagnostics", "-l", "-s"]);
    }

    #[test]
    fn advance_with_basic_journal_limit() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new())
            .advance_with_basic()
            .journal()
            .limit("10m");

        assert_eq!(cmd.get_limit(), Some("10m"));

        #[cfg(not(feature = "lt2025_2"))]
        assert_eq!(args(&cmd), ["diagnostics", "-a", "-J", "-L", "10m"]);

        #[cfg(feature = "lt2025_2")]
        assert_eq!(args(&cmd), ["diagnostics", "-a", "-L", "10m"]);
    }

    #[test]
    fn with_basic_after_strace() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new())
            .run_strace()
            .with_basic();

        assert_eq!(args(&cmd), ["diagnostics", "-s", "-a"]);
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn strace_runtime() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new()).run_strace_with(30);

        assert_eq!(cmd.get_strace_runtime(), Some(30));
        assert_eq!(args(&cmd), ["diagnostics", "-s", "--strace-runtime", "30"]);
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn all_advance_options_in_syntax_order() {
        let cmd = Diagnostics::new("p4", GlobalOpts::new())
            .uncompressed(true)
            .advance()
            .run_lsof(true)
            .run_ps(true)
            .run_top(true)
            .run_strace_with(15);

        assert_eq!(
            args(&cmd),
            [
                "diagnostics",
                "-D",
                "-l",
                "-s",
                "--strace-runtime",
                "15",
                "-p",
                "-t"
            ]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let cmd = Diagnostics::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .run_lsof();

        assert_eq!(
            args(&cmd),
            ["-p", "localhost:1666", "-q", "diagnostics", "-l"]
        );
    }
}
