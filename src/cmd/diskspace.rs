use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Predefined targets
// ---------------------------------------------------------------------------

/// The `P4ROOT` target: report disk space for the filesystem holding
/// [`P4ROOT`](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/P4ROOT.html).
///
/// Entered with [`DiskSpace::report_p4root`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct P4Root;

/// The `P4JOURNAL` target: report disk space for the filesystem holding
/// [`P4JOURNAL`](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/P4JOURNAL.html).
///
/// Entered with [`DiskSpace::report_p4journal`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct P4Journal;

/// The `P4LOG` target: report disk space for the filesystem holding
/// [`P4LOG`](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/P4LOG.html)
/// (the server log).
///
/// Entered with [`DiskSpace::report_p4log`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct P4Log;

/// The `TEMP` target: report disk space for the filesystem holding temporary
/// files. If `TEMP` is not defined, `P4ROOT` is used on Windows and `/tmp` on
/// Unix.
///
/// Entered with [`DiskSpace::report_temp`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Temp;

// ---------------------------------------------------------------------------
// Argument injection
// ---------------------------------------------------------------------------

impl ExclusiveOption for P4Root {
    fn inject_args(&self, command: &mut Command) {
        command.arg("P4ROOT");
    }
}

impl ExclusiveOption for P4Journal {
    fn inject_args(&self, command: &mut Command) {
        command.arg("P4JOURNAL");
    }
}

impl ExclusiveOption for P4Log {
    fn inject_args(&self, command: &mut Command) {
        command.arg("P4LOG");
    }
}

impl ExclusiveOption for Temp {
    fn inject_args(&self, command: &mut Command) {
        command.arg("TEMP");
    }
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 diskspace`.
///
/// `p4 df` is an alias for `p4 diskspace`.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] diskspace [P4ROOT | P4JOURNAL | P4LOG | TEMP | journalPrefix | depot]
/// ```
///
/// Shows summary information about the current availability of disk space on
/// the server.
///
/// The output of `p4 diskspace` is in the form:
///
/// ```text
/// name (type type) : xxx GB free, yyy GB used, zzz GB total (ff % full)
/// ```
///
#[cfg_attr(
    feature = "lt2017_2",
    doc = "Where `name` can be either `P4ROOT`, `P4JOURNAL`, `P4LOG`, `TEMP`, a prefix to a non-default Perforce journal file location, or the name of a Perforce depot. The filesystem `type` is that reported by the operating system."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2017_2")),
    doc = "Where `name` can be either `P4ROOT`, `P4JOURNAL`, `P4LOG`, `TEMP`, a prefix to a non-default Helix Server journal file location, or the name of a Helix Server depot. The filesystem `type` is that reported by the operating system."
)]
#[cfg_attr(
    all(feature = "lt2021_1", not(feature = "lt2019_1")),
    doc = "Where `name` can be either `P4ROOT`, `P4JOURNAL`, `P4LOG`, `TEMP`, a prefix to a non-default Helix server journal file location, or the name of a Helix server depot. The filesystem `type` is that reported by the operating system."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2021_1")),
    doc = "Where `name` can be either `P4ROOT`, `P4JOURNAL`, `P4LOG`, `TEMP`, a prefix to a non-default Helix Server journal file location, or the name of a Helix Server depot. The filesystem `type` is that reported by the operating system."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2024_1")),
    doc = "Where `name` can be either `P4ROOT`, `P4JOURNAL`, `P4LOG`, `TEMP`, a prefix to a non-default Helix Core Server journal file location, or the name of a Helix Core Server depot. The filesystem `type` is that reported by the operating system."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "Where `name` can be either `P4ROOT`, `P4JOURNAL`, `P4LOG`, `TEMP`, a prefix to a non-default P4 Server journal file location, or the name of a P4 Server depot. The filesystem `type` is that reported by the operating system."
)]
///
/// If no arguments are specified, disk space information is displayed for all
/// objects.
///
#[cfg_attr(
    all(not(feature = "lt2016_1"), feature = "lt2017_2"),
    doc = "By default, Perforce rejects commands when free space on the filesystems housing the `P4ROOT`, `P4JOURNAL`, `P4LOG`, or `TEMP` falls below 10 megabytes. To change this behavior, set the `filesys.P4ROOT.min` (and corresponding) configurables to your desired limits."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2019_1",
        not(feature = "lt2017_2")
    ),
    doc = "By default, Helix Server rejects commands when free space on the filesystems housing the `P4ROOT`, `P4JOURNAL`, `P4LOG`, or `TEMP` falls below 10 megabytes. To change this behavior, set the `filesys.P4ROOT.min` (and corresponding) configurables to your desired limits."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2021_1",
        not(feature = "lt2019_1")
    ),
    doc = "By default, Helix server rejects commands when free space on the filesystems housing the `P4ROOT`, `P4JOURNAL`, `P4LOG`, or `TEMP` falls below 10 megabytes. To change this behavior, set the `filesys.P4ROOT.min` (and corresponding) configurables to your desired limits."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2024_1",
        not(feature = "lt2021_1")
    ),
    doc = "By default, Helix Server rejects commands when free space on the filesystems housing the `P4ROOT`, `P4JOURNAL`, `P4LOG`, or `TEMP` falls below 10 megabytes. To change this behavior, set the `filesys.P4ROOT.min` (and corresponding) configurables to your desired limits."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2024_2",
        not(feature = "lt2024_1")
    ),
    doc = "By default, Helix Core Server rejects commands when free space on the filesystems housing the `P4ROOT`, `P4JOURNAL`, `P4LOG`, or `TEMP` falls below 10 megabytes. To change this behavior, set the `filesys.P4ROOT.min` (and corresponding) configurables to your desired limits."
)]
#[cfg_attr(
    all(not(feature = "lt2016_1"), not(feature = "lt2024_2")),
    doc = "By default, P4 Server rejects commands when free space on the filesystems housing the `P4ROOT`, `P4JOURNAL`, `P4LOG`, or `TEMP` falls below 10 megabytes. To change this behavior, set the `filesys.P4ROOT.min` (and corresponding) configurables to your desired limits."
)]
#[cfg_attr(not(feature = "lt2016_1"), doc = "")]
#[cfg_attr(
    all(not(feature = "lt2016_1"), feature = "lt2017_2"),
    doc = "If the user account that runs the Perforce versioning service is subject to disk quotas, the `filesys.*.min` configurables reflect those quotas, regardless of how much physical space actually remains on the filesystem(s) in question."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2019_1",
        not(feature = "lt2017_2")
    ),
    doc = "If the user account that runs the Helix Server is subject to disk quotas, the `filesys.*.min` configurables reflect those quotas, regardless of how much physical space actually remains on the filesystem(s) in question."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2021_1",
        not(feature = "lt2019_1")
    ),
    doc = "If the user account that runs the Helix server is subject to disk quotas, the `filesys.*.min` configurables reflect those quotas, regardless of how much physical space actually remains on the filesystem(s) in question."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2024_1",
        not(feature = "lt2021_1")
    ),
    doc = "If the user account that runs the Helix Server is subject to disk quotas, the `filesys.*.min` configurables reflect those quotas, regardless of how much physical space actually remains on the filesystem(s) in question."
)]
#[cfg_attr(
    all(
        not(feature = "lt2016_1"),
        feature = "lt2024_2",
        not(feature = "lt2024_1")
    ),
    doc = "If the user account that runs the Helix Core Server is subject to disk quotas, the `filesys.*.min` configurables reflect those quotas, regardless of how much physical space actually remains on the filesystem(s) in question."
)]
#[cfg_attr(
    all(not(feature = "lt2016_1"), not(feature = "lt2024_2")),
    doc = "If the user account that runs the P4 Server is subject to disk quotas, the `filesys.*.min` configurables reflect those quotas, regardless of how much physical space actually remains on the filesystem(s) in question."
)]
#[cfg_attr(not(feature = "lt2016_1"), doc = "", doc = "# Note", doc = "")]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "Server releases prior to 16.1 included reserved space in the number displayed for used space in the Size column for Unix filesystems. This has been changed so that the size given now excludes reserved space."
)]
///
/// The `M` type parameter tracks the target at compile time. [`Unselected`]
/// (the state returned by [`DiskSpace::new`]) spawns the bare command, which
/// reports disk space for every object, and can also be spawned with an
/// arbitrary target argument through
/// [`ParameterizedSpawn`] with a one-element tuple, or
/// [`SpawnExt1`](crate::spawn::SpawnExt1); that covers a `journalPrefix` or a
/// depot name. Use [`DiskSpace::report_p4root`], [`DiskSpace::report_p4journal`],
/// [`DiskSpace::report_p4log`], or [`DiskSpace::report_temp`] to select one of
/// the predefined targets; those states spawn without arguments.
#[derive(Debug, Clone, Default)]
pub struct DiskSpace<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

// ---------------------------------------------------------------------------
// Constructors and target transitions
// ---------------------------------------------------------------------------

impl DiskSpace<Unselected> {
    /// Creates a new `p4 diskspace` command.
    ///
    /// `bin` is the path to the Perforce command-line executable. The
    /// resulting command has no target argument and reports disk space for
    /// every object.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: Unselected,
        }
    }
}

impl<M> DiskSpace<M> {
    fn with_mode<N>(self, mode: N) -> DiskSpace<N> {
        DiskSpace {
            bin: self.bin,
            global_opts: self.global_opts,
            mode,
        }
    }

    /// # Description
    ///
    /// `P4ROOT`
    ///
    /// Report disk space available for filesystem holding `P4ROOT`.
    ///
    /// Transitions to the [`P4Root`] state, which spawns without arguments.
    pub fn report_p4root(self) -> DiskSpace<P4Root> {
        self.with_mode(P4Root)
    }

    /// # Description
    ///
    /// `P4JOURNAL`
    ///
    /// Report disk space available for filesystem holding `P4JOURNAL`.
    ///
    /// Transitions to the [`P4Journal`] state, which spawns without
    /// arguments.
    pub fn report_p4journal(self) -> DiskSpace<P4Journal> {
        self.with_mode(P4Journal)
    }

    /// # Description
    ///
    /// `P4LOG`
    ///
    /// Report disk space available for filesystem holding `P4LOG` (server
    /// log).
    ///
    /// Transitions to the [`P4Log`] state, which spawns without arguments.
    pub fn report_p4log(self) -> DiskSpace<P4Log> {
        self.with_mode(P4Log)
    }

    /// # Description
    ///
    /// `TEMP`
    ///
    /// Report disk space available for filesystem holding temporary files. If
    /// not defined, uses `P4ROOT` on Windows, and `/tmp` on Unix.
    ///
    /// Transitions to the [`Temp`] state, which spawns without arguments.
    pub fn report_temp(self) -> DiskSpace<Temp> {
        self.with_mode(Temp)
    }
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> DiskSpace<M> {
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
    /// g-opts
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "See the [Global Options](GlobalOpts) section."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "See the [“Global Options”](GlobalOpts) section."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "See [“Global Options”](GlobalOpts)."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "See [Global Options](GlobalOpts)."
    )]
    #[cfg_attr(not(feature = "lt2018_2"), doc = "See [Global options](GlobalOpts).")]
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// # Description
    ///
    /// g-opts
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "See the [Global Options](GlobalOpts) section."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "See the [“Global Options”](GlobalOpts) section."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "See [“Global Options”](GlobalOpts)."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "See [Global Options](GlobalOpts)."
    )]
    #[cfg_attr(not(feature = "lt2018_2"), doc = "See [Global options](GlobalOpts).")]
    pub fn set_global_opts(&mut self, v: GlobalOpts) -> &mut Self {
        self.global_opts = v;
        self
    }

    /// # Description
    ///
    /// g-opts
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "See the [Global Options](GlobalOpts) section."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "See the [“Global Options”](GlobalOpts) section."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "See [“Global Options”](GlobalOpts)."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "See [Global Options](GlobalOpts)."
    )]
    #[cfg_attr(not(feature = "lt2018_2"), doc = "See [Global options](GlobalOpts).")]
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.global_opts = v;
        self
    }
}

// ---------------------------------------------------------------------------
// SubCommand and spawning
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> SubCommand for DiskSpace<M> {
    fn name(&self) -> &str {
        "diskspace"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption> ParameterizedSpawn<()> for DiskSpace<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diskspace` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S: AsRef<OsStr>> ParameterizedSpawn<(S,)> for DiskSpace<Unselected> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diskspace <target>` as a child process with piped standard
    /// output and error streams, where `target` is a `journalPrefix` or a
    /// depot name.
    fn spawn_with(&mut self, (target,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(target)
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

    /// Dry-run checks of the assembled `p4 diskspace` command line; no
    /// process is spawned.
    #[test]
    fn bare_form_has_no_argument() {
        let cmd = DiskSpace::new("p4", GlobalOpts::new());

        assert_eq!(args(&cmd), ["diskspace"]);
    }

    #[test]
    fn predefined_targets() {
        assert_eq!(
            args(&DiskSpace::new("p4", GlobalOpts::new()).report_p4root()),
            ["diskspace", "P4ROOT"]
        );
        assert_eq!(
            args(&DiskSpace::new("p4", GlobalOpts::new()).report_p4journal()),
            ["diskspace", "P4JOURNAL"]
        );
        assert_eq!(
            args(&DiskSpace::new("p4", GlobalOpts::new()).report_p4log()),
            ["diskspace", "P4LOG"]
        );
        assert_eq!(
            args(&DiskSpace::new("p4", GlobalOpts::new()).report_temp()),
            ["diskspace", "TEMP"]
        );
    }

    #[test]
    fn targets_can_be_switched() {
        let cmd = DiskSpace::new("p4", GlobalOpts::new()).report_p4root();
        let cmd = cmd.report_temp();

        assert_eq!(args(&cmd), ["diskspace", "TEMP"]);
    }

    #[test]
    fn global_opts_are_injected_once() {
        let cmd = DiskSpace::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .report_p4root();

        assert_eq!(
            args(&cmd),
            ["-p", "localhost:1666", "-q", "diskspace", "P4ROOT"]
        );
    }

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut cmd = DiskSpace::new("p4", GlobalOpts::new()).report_temp();
        assert_eq!(cmd.get_bin(), Path::new("p4"));

        cmd.set_bin("/usr/local/bin/p4");
        assert_eq!(cmd.get_bin(), Path::new("/usr/local/bin/p4"));

        let cmd = cmd.bin("p4.exe");
        assert_eq!(cmd.get_bin(), Path::new("p4.exe"));
    }
}
