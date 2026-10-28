use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The depot type selected with `-t type`.
///
/// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
#[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
#[cfg_attr(
    not(feature = "lt2017_1"),
    doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
)]
#[cfg(not(feature = "lt2015_2"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepotType {
    /// A `local` depot (the default).
    Local,

    /// A `remote` depot.
    Remote,

    /// A `spec` depot.
    Spec,

    /// A `stream` depot.
    Stream,

    /// An `unload` depot.
    Unload,

    /// An `archive` depot.
    Archive,

    /// A `tangent` depot.
    Tangent,

    /// A `graph` depot, used to store Git repos in the server.
    #[cfg(not(feature = "lt2017_1"))]
    Graph,
}

#[cfg(not(feature = "lt2015_2"))]
impl DepotType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
            Self::Spec => "spec",
            Self::Stream => "stream",
            Self::Unload => "unload",
            Self::Archive => "archive",
            Self::Tangent => "tangent",
            #[cfg(not(feature = "lt2017_1"))]
            Self::Graph => "graph",
        }
    }
}

/// Delete mode of `p4 depot` (`-d [-f]`).
///
/// Entered with [`Depot::delete`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Delete {
    force: bool,
}

impl ExclusiveOption for Delete {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-d");
        if self.force {
            command.arg("-f");
        }
    }
}

/// Stdout mode of `p4 depot` (`-o [-t type]`): write a depot specification
/// to standard output.
///
/// Entered with [`Depot::stdout`], or from the regular form with
/// [`Depot::stdout`](Depot::stdout).
#[derive(Debug, Clone, Default)]
pub struct Stdout {
    #[cfg(not(feature = "lt2015_2"))]
    depot_type: Option<DepotType>,
}

impl ExclusiveOption for Stdout {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-o");
        #[cfg(not(feature = "lt2015_2"))]
        if let Some(depot_type) = &self.depot_type {
            command.arg("-t").arg(depot_type.as_str());
        }
    }
}

/// Stdin mode of `p4 depot` (`-i`): read a depot specification from standard
/// input.
///
/// Entered with [`Depot::stdin`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Stdin {}

impl ExclusiveOption for Stdin {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-i");
    }
}

/// Regular mode of `p4 depot` (`[-t type] depotname`): create or edit a
/// depot specification.
///
/// Entered with [`Depot::depot_type`]; [`Depot::stdout`](Depot::stdout)
/// transitions to the [`Stdout`] mode, preserving the selected depot type.
#[cfg(not(feature = "lt2015_2"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Regular {
    depot_type: Option<DepotType>,
}

#[cfg(not(feature = "lt2015_2"))]
impl ExclusiveOption for Regular {
    fn inject_args(&self, command: &mut Command) {
        if let Some(depot_type) = &self.depot_type {
            command.arg("-t").arg(depot_type.as_str());
        }
    }
}

#[cfg_attr(
    feature = "lt2015_2",
    doc = "`p4 [g-opts] depot depotname`\n\n\
           `p4 [g-opts] depot -d [-f] depotname`\n\n\
           `p4 [g-opts] depot -o depotname`\n\n\
           `p4 [g-opts] depot -i`"
)]
#[cfg_attr(
    not(feature = "lt2015_2"),
    doc = "`p4 [g-opts] depot [-t type] depotname`\n\n\
           `p4 [g-opts] depot -d [-f] depotname`\n\n\
           `p4 [g-opts] depot -o [-t type] depotname`\n\n\
           `p4 [g-opts] depot -i`"
)]
///
#[cfg_attr(feature = "lt2022_1", doc = "Create or edit a depot specification.")]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2022_1")),
    doc = "Create, edit, or delete a depot specification.",
    doc = "",
    doc = "A branch, depot, label, and workspace cannot have the same name."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2022_2")),
    doc = "Create, edit, or delete a depot specification."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "Create, edit, or delete a depot specification.",
    doc = "",
    doc = "A branch, depot, label, and workspace cannot share the same name."
)]
///
/// The `M` type parameter tracks the command form at compile time. The
/// default [`Unselected`] state is the regular form with no options;
/// [`Self::delete`], [`Self::stdout`], and [`Self::stdin`] transition to the
/// [`Delete`], [`Stdout`], and [`Stdin`] modes
#[cfg_attr(feature = "lt2015_2", doc = ".")]
#[cfg_attr(
    not(feature = "lt2015_2"),
    doc = ", while [`Self::depot_type`] transitions to the [`Regular`] mode carrying `-t type`."
)]
#[derive(Debug, Clone, Default)]
pub struct Depot<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

impl Depot<Unselected> {
    /// Creates a new `p4 depot` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    /// -d depotname
    ///
    /// Delete the depot *depotname*.
    ///
    #[cfg_attr(
        feature = "lt2015_2",
        doc = "The depot must not contain any files; the Perforce superuser can remove files with `p4 obliterate`.",
        doc = "",
        doc = "If the depot is `remote`, `p4 obliterate` must still be run: no files are deleted, but any outstanding client or label records referring to that depot are eliminated."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2015_2")),
        doc = "The depot must not contain any files; the Perforce superuser can remove files with `p4 obliterate`.",
        doc = "",
        doc = "If the depot is `remote`, `p4 obliterate` must still be run: no files are deleted, but any outstanding client or label records referring to that depot are eliminated.",
        doc = "",
        doc = "The name of a depot may not be the same as the name of a branch, client workspace, or label."
    )]
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2017_2")),
        doc = "The depot must not contain any files; the Helix Server superuser can remove files with `p4 obliterate`.",
        doc = "",
        doc = "If the depot is `remote`, `p4 obliterate` must still be run: no files are deleted, but any outstanding client or label records referring to that depot are eliminated.",
        doc = "",
        doc = "The name of a depot may not be the same as the name of a branch, client workspace, or label."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2022_1")),
        doc = "A classic depot must not contain any files. The Helix Core Server user with the `super` access level can remove files with `p4 obliterate`.",
        doc = "",
        doc = "A stream depot must not contain any stream specs. A Helix Core user with the `admin` access level can remove a stream with the `--obliterate` option of `p4 stream`.",
        doc = "",
        doc = "If the depot is `remote`, `p4 obliterate` must still be run. No files are deleted, but any outstanding client or label records referring to that depot are eliminated."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2022_2")),
        doc = "A classic depot must not contain any files. The Helix Core Server user with the `super` access level can permanently remove files with `p4 obliterate`.",
        doc = "",
        doc = "A stream depot must not contain any streams. A Helix Core Server user with the `admin` access level can can use the `--obliterate` option of `p4 stream` to permanently remove a stream.",
        doc = "",
        doc = "If the depot is `remote`, `p4 obliterate` must still be run: no files are deleted, but any outstanding client or label records referring to that depot are eliminated.",
        doc = "",
        doc = "The name of a depot cannot be the same as the name of a branch, client workspace, or label."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "A classic depot must not contain any files. The P4 Server user with the `super` access level can permanently remove files with `p4 obliterate`.",
        doc = "",
        doc = "A stream depot must not contain any streams. A P4 Server user with the `admin` access level can can use the `--obliterate` option of `p4 stream` to permanently remove a stream.",
        doc = "",
        doc = "If the depot is `remote`, `p4 obliterate` must still be run: no files are deleted, but any outstanding client or label records referring to that depot are eliminated.",
        doc = "",
        doc = "The name of a depot cannot be the same as the name of a branch, client workspace, or label."
    )]
    ///
    /// Transitions this command to the [`Delete`] mode.
    pub fn delete(self) -> Depot<Delete> {
        Depot {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Delete { force: false },
        }
    }

    /// # Description
    ///
    /// -o depotname
    ///
    /// Write a depot specification to standard output.
    ///
    /// Transitions this command to the [`Stdout`] mode.
    pub fn stdout(self) -> Depot<Stdout> {
        Depot {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Stdout {
                #[cfg(not(feature = "lt2015_2"))]
                depot_type: None,
            },
        }
    }

    /// # Description
    ///
    /// -i
    ///
    #[cfg_attr(
        feature = "lt2019_1",
        doc = "Read a depot specification from standard input."
    )]
    #[cfg_attr(
        not(feature = "lt2019_1"),
        doc = "Read a depot specification from standard input, or from a file in the case of `p4 depot -o -t stream mystreamdepot2 > filename` where content of filename is the depot spec."
    )]
    ///
    /// Transitions this command to the [`Stdin`] mode.
    pub fn stdin(self) -> Depot<Stdin> {
        Depot {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Stdin {},
        }
    }

    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    ///
    /// Transitions this command to the [`Regular`] mode with the depot type
    /// set.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn depot_type(self, v: DepotType) -> Depot<Regular> {
        Depot {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Regular {
                depot_type: Some(v),
            },
        }
    }
}

impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Depot<Unselected> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depot depotname` with the given depot name as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (depotname,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(depotname)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Depot<Delete> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depot -d [-f] depotname` with the given depot name as a
    /// child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (depotname,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(depotname)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Depot<Stdout> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depot -o [-t type] depotname` with the given depot name as
    /// a child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (depotname,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(depotname)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for Depot<Stdin> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depot -i` with a piped standard input stream (and piped
    /// standard output and error streams); write the depot specification to
    /// the returned [`Child`]'s stdin handle.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<(Stdio,)> for Depot<Stdin> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depot -i` with the given standard input configuration and
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    ///
    /// Pass `Stdio::piped()` to obtain a writable `child.stdin` handle and
    /// feed the depot specification into it; pass `Stdio::inherit()` to read
    /// the specification from the spawning process's own standard input.
    fn spawn_with(&mut self, (stdio,): (Stdio,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(stdio)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(not(feature = "lt2015_2"))]
impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Depot<Regular> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depot [-t type] depotname` with the given depot name as a
    /// child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (depotname,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(depotname)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl Depot<Delete> {
    /// # Description
    ///
    /// -f
    ///
    /// By default, when you delete a depot, the directory specified by the
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "`Map:` field (typically under `P4ROOT`) must be empty. Use the `-f` flag to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2014_2")),
        doc = "`Map:` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2024_1")),
        doc = "`Map` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "`Map` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty.",
        doc = "",
        doc = "The P4 Server will not delete files on disk if they are referenced elsewhere (due to being branched or integrated to another depot as a lazy copy). You should not remove referenced files from disk manually using operating system commands. For more information, see Deleting depots."
    )]
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// By default, when you delete a depot, the directory specified by the
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "`Map:` field (typically under `P4ROOT`) must be empty. Use the `-f` flag to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2014_2")),
        doc = "`Map:` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2024_1")),
        doc = "`Map` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "`Map` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty.",
        doc = "",
        doc = "The P4 Server will not delete files on disk if they are referenced elsewhere (due to being branched or integrated to another depot as a lazy copy). You should not remove referenced files from disk manually using operating system commands. For more information, see Deleting depots."
    )]
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// By default, when you delete a depot, the directory specified by the
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "`Map:` field (typically under `P4ROOT`) must be empty. Use the `-f` flag to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2014_2")),
        doc = "`Map:` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2024_1")),
        doc = "`Map` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "`Map` field (typically under `P4ROOT`) must be empty. Use the `-f` option to remove all files even if the directory is not empty.",
        doc = "",
        doc = "The P4 Server will not delete files on disk if they are referenced elsewhere (due to being branched or integrated to another depot as a lazy copy). You should not remove referenced files from disk manually using operating system commands. For more information, see Deleting depots."
    )]
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }
}

#[cfg(not(feature = "lt2015_2"))]
impl Depot<Regular> {
    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    pub fn get_depot_type(&self) -> Option<DepotType> {
        self.mode.depot_type
    }

    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    pub fn set_depot_type(&mut self, v: DepotType) -> &mut Self {
        self.mode.depot_type = Some(v);
        self
    }

    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    pub fn depot_type(mut self, v: DepotType) -> Self {
        self.mode.depot_type = Some(v);
        self
    }

    /// # Description
    ///
    /// -o depotname
    ///
    /// Write a depot specification to standard output.
    ///
    /// Transitions this command to the [`Stdout`] mode, preserving the
    /// selected depot type.
    pub fn stdout(self) -> Depot<Stdout> {
        Depot {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Stdout {
                depot_type: self.mode.depot_type,
            },
        }
    }
}

impl Depot<Stdout> {
    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    #[cfg(not(feature = "lt2015_2"))]
    pub fn get_depot_type(&self) -> Option<DepotType> {
        self.mode.depot_type
    }

    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    #[cfg(not(feature = "lt2015_2"))]
    pub fn set_depot_type(&mut self, v: DepotType) -> &mut Self {
        self.mode.depot_type = Some(v);
        self
    }

    /// # Description
    ///
    /// -t type
    ///
    /// The type of the depot: `local`, `remote`, `spec`, `stream`, `unload`,
    #[cfg_attr(feature = "lt2017_1", doc = "`archive` or `tangent`.")]
    #[cfg_attr(
        not(feature = "lt2017_1"),
        doc = "`archive`, or `tangent`. To support Git workflows, a special type is `graph`."
    )]
    #[cfg(not(feature = "lt2015_2"))]
    pub fn depot_type(mut self, v: DepotType) -> Self {
        self.mode.depot_type = Some(v);
        self
    }
}

impl<M: ExclusiveOption> Depot<M> {
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

impl<M: ExclusiveOption> SubCommand for Depot<M> {
    fn name(&self) -> &str {
        "depot"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}
