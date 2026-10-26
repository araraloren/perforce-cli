use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Selection marker for the `-a` option: report statistics for all tables.
///
/// Because `-a` takes no positional arguments, this state carries no table
/// list; spawn it with `()`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DisplayAllTables;

impl ExclusiveOption for DisplayAllTables {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");
    }
}

/// The `-s` form: `p4 [g-opts] dbstat -s`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size;

/// The scan form.
///
#[cfg_attr(
    feature = "lt2016_1",
    doc = "`p4 [g-opts] dbstat [-h] {-a | dbtable ...}`."
)]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "`p4 [g-opts] dbstat [-h][-f] {-a | dbtable ...}`."
)]
///
/// The `M` type parameter tracks the table selection: [`Unselected`] spawns
/// with a list of `dbtable` arguments; [`DisplayAllTables`] supplies `-a` and
/// spawns without arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scan<M = Unselected> {
    show_histogram: bool,

    #[cfg(not(feature = "lt2016_1"))]
    show_free_pages: bool,

    mode: M,
}

impl ExclusiveOption for Scan<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        if self.show_histogram {
            command.arg("-h");
        }

        #[cfg(not(feature = "lt2016_1"))]
        if self.show_free_pages {
            command.arg("-f");
        }
    }
}

impl ExclusiveOption for Scan<DisplayAllTables> {
    fn inject_args(&self, command: &mut Command) {
        if self.show_histogram {
            command.arg("-h");
        }

        #[cfg(not(feature = "lt2016_1"))]
        if self.show_free_pages {
            command.arg("-f");
        }

        command.arg("-a");
    }
}

/// The checkpoint form:
/// `p4 [g-opts] dbstat -p [-w worklevel] [-n numberOfFiles] {-a | dbtable ...}`.
///
/// Mutually exclusive with the scan form: `-p` cannot be combined with `-h`
/// or `-f`.
#[cfg(not(feature = "lt2023_1"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckPoint<M = Unselected> {
    worklevel: Option<u64>,

    number_files: Option<u64>,

    mode: M,
}

#[cfg(not(feature = "lt2023_1"))]
impl ExclusiveOption for CheckPoint<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-p");

        if let Some(worklevel) = self.worklevel {
            command.arg("-w").arg(worklevel.to_string());
        }

        if let Some(number_files) = self.number_files {
            command.arg("-n").arg(number_files.to_string());
        }
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl ExclusiveOption for CheckPoint<DisplayAllTables> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-p");

        if let Some(worklevel) = self.worklevel {
            command.arg("-w").arg(worklevel.to_string());
        }

        if let Some(number_files) = self.number_files {
            command.arg("-n").arg(number_files.to_string());
        }

        command.arg("-a");
    }
}

impl ExclusiveOption for Size {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s");
    }
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2016_1",
    doc = "`p4 [g-opts] dbstat [-h] {-a | dbtable ...}`"
)]
#[cfg_attr(
    all(feature = "lt2023_1", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] dbstat [-h][-f] {-a | dbtable ...}`"
)]
#[cfg_attr(
    not(feature = "lt2023_1"),
    doc = "`p4 [g-opts] dbstat [-h][-f] | [-p [-w worklevel] [-n numberOfFiles]] {-a | dbtable ...}`"
)]
///
/// `p4 [g-opts] dbstat -s`
///
/// Displays size or simple statistics for one or more database tables. The
/// `dbtable` corresponds to the `db.*` files in your server's root directory.
///
/// The `M` type parameter tracks the command form at compile time, starting
/// as [`Unselected`] from [`DBStat::new`]. The available states and
/// transitions are:
/// - [`Unselected`] / [`DisplayAllTables`]: the bare `{-a | dbtable ...}`
///   selection tail; spawn the former with a list of table names, or use
///   [`Self::all`] to supply `-a` and spawn the latter with `()`. Both states
///   can enter the scan and checkpoint forms; the transitions defined on
///   `DisplayAllTables` keep `-a`.
#[cfg_attr(
    feature = "lt2016_1",
    doc = "- [`Scan`] (entered via [`Self::show_histogram`]): the histogram option followed by a table selection; [`Self::all`] supplies `-a` and transitions to the [`DisplayAllTables`] state."
)]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "- [`Scan`] (entered via [`Self::show_histogram`] or [`Self::show_free_pages`]): histogram and free-page options followed by a table selection; [`Self::all`] supplies `-a` and transitions to the [`DisplayAllTables`] state."
)]
#[cfg_attr(
    not(feature = "lt2023_1"),
    doc = "- [`CheckPoint`] (entered via [`Self::checkpoint`]): `-p` with optional `-w`/`-n`; mutually exclusive with the scan form."
)]
/// - [`Size`] (entered via [`Self::size`]): the `-s` form.
#[derive(Debug, Clone, Default)]
pub struct DBStat<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

// ---------------------------------------------------------------------------
// Constructor and form transitions
// ---------------------------------------------------------------------------

impl DBStat<Unselected> {
    /// Creates a new `p4 dbstat` command.
    ///
    /// `bin` is the path to the Perforce command-line executable. Call one of
    /// the form transitions to select a command form.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: Unselected,
        }
    }

    /// Enters the scan form with `-h` enabled:
    #[cfg_attr(feature = "lt2016_1", doc = "`p4 dbstat [-h] {-a | dbtable ...}`.")]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "`p4 dbstat [-h][-f] {-a | dbtable ...}`."
    )]
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Display a histogram showing distances between leaf pages."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Display a histogram showing distances between leaf pages. This enables you to anticipate disk seeks required for sequential database scans. This scans only the internal node pages of the file, not the leaf or overflow pages, and thus reads only 1-5% of the file.",
        doc = "",
        doc = "> **Note**",
        doc = ">",
        doc = "> This option blocks write access to the database while it scans the tables. It is best to get guidance from [Perforce Support](https://www.perforce.com/support?product=Helix%20Core) before using this option."
    )]
    ///
    /// Use the scan state's `all` transition to supply `-a`; otherwise
    /// spawn with a list of table names.
    pub fn show_histogram(self) -> DBStat<Scan<Unselected>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Scan {
                show_histogram: true,
                #[cfg(not(feature = "lt2016_1"))]
                show_free_pages: false,
                mode: Unselected,
            },
        }
    }

    /// Enters the scan form with `-f` enabled:
    /// `p4 dbstat [-h][-f] {-a | dbtable ...}`.
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Displays a page count, free pages, and percent free data for the specified tables."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Displays a page count, free pages, and percent free data for the specified tables. Using the `-f` with the `-h` option, this command shows a histogram of free page distribution."
    )]
    ///
    /// Use the scan state's `all` transition to supply `-a`; otherwise
    /// spawn with a list of table names.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn show_free_pages(self) -> DBStat<Scan<Unselected>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Scan {
                show_histogram: false,
                show_free_pages: true,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -a
    ///
    /// Display statistics for all tables.
    ///
    /// Transitions to the [`DisplayAllTables`] state, which spawns without
    /// table arguments.
    pub fn all(self) -> DBStat<DisplayAllTables> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DisplayAllTables,
        }
    }

    /// Enters the checkpoint form with `-w worklevel` set:
    /// `p4 dbstat -p -w worklevel {-a | dbtable ...}`.
    ///
    /// The checkpoint form is mutually exclusive with the scan form.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn worklevel(self, v: u64) -> DBStat<CheckPoint<Unselected>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: Some(v),
                number_files: None,
                mode: Unselected,
            },
        }
    }

    /// Enters the checkpoint form with `-n numberOfFiles` set:
    /// `p4 dbstat -p -n numberOfFiles {-a | dbtable ...}`.
    ///
    /// The checkpoint form is mutually exclusive with the scan form.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn number_files(self, v: u64) -> DBStat<CheckPoint<Unselected>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: None,
                number_files: Some(v),
                mode: Unselected,
            },
        }
    }

    /// Enters the checkpoint form:
    /// `p4 dbstat -p [-w worklevel] [-n numberOfFiles] {-a | dbtable ...}`.
    ///
    /// Use [`Self::worklevel`] or [`Self::number_files`] to enter the form
    /// with `-w` or `-n` already set; use [`Self::all`] to supply `-a`,
    /// otherwise spawn with a list of table names.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn checkpoint(self) -> DBStat<CheckPoint<Unselected>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: None,
                number_files: None,
                mode: Unselected,
            },
        }
    }

    /// Enters the size form: `p4 dbstat -s`.
    pub fn size(self) -> DBStat<Size> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Size,
        }
    }
}

// ---------------------------------------------------------------------------
// Transitions from the `-a` state
// ---------------------------------------------------------------------------

impl DBStat<DisplayAllTables> {
    /// Enters the scan form with `-h` set and `-a` retained:
    /// `p4 dbstat [-h][-f] -a`.
    pub fn show_histogram(self) -> DBStat<Scan<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Scan {
                show_histogram: true,
                #[cfg(not(feature = "lt2016_1"))]
                show_free_pages: false,
                mode: DisplayAllTables,
            },
        }
    }

    /// Enters the scan form with `-f` set and `-a` retained:
    /// `p4 dbstat [-h][-f] -a`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn show_free_pages(self) -> DBStat<Scan<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Scan {
                show_histogram: false,
                show_free_pages: true,
                mode: DisplayAllTables,
            },
        }
    }

    /// Enters the checkpoint form with `-a` retained: `p4 dbstat -p -a`.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn checkpoint(self) -> DBStat<CheckPoint<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: None,
                number_files: None,
                mode: DisplayAllTables,
            },
        }
    }

    /// Enters the checkpoint form with `-w worklevel` set and `-a` retained:
    /// `p4 dbstat -p -w worklevel -a`.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn worklevel(self, v: u64) -> DBStat<CheckPoint<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: Some(v),
                number_files: None,
                mode: DisplayAllTables,
            },
        }
    }

    /// Enters the checkpoint form with `-n numberOfFiles` set and `-a`
    /// retained: `p4 dbstat -p -n numberOfFiles -a`.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn number_files(self, v: u64) -> DBStat<CheckPoint<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: None,
                number_files: Some(v),
                mode: DisplayAllTables,
            },
        }
    }
}

impl<M> DBStat<Scan<M>> {
    /// # Description
    ///
    /// -h
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Display a histogram showing distances between leaf pages."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Display a histogram showing distances between leaf pages. This enables you to anticipate disk seeks required for sequential database scans. This scans only the internal node pages of the file, not the leaf or overflow pages, and thus reads only 1-5% of the file.",
        doc = "",
        doc = "> **Note**",
        doc = ">",
        doc = "> This option blocks write access to the database while it scans the tables. It is best to get guidance from [Perforce Support](https://www.perforce.com/support?product=Helix%20Core) before using this option."
    )]
    pub fn get_show_histogram(&self) -> bool {
        self.mode.show_histogram
    }

    /// # Description
    ///
    /// -h
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Display a histogram showing distances between leaf pages."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Display a histogram showing distances between leaf pages. This enables you to anticipate disk seeks required for sequential database scans. This scans only the internal node pages of the file, not the leaf or overflow pages, and thus reads only 1-5% of the file.",
        doc = "",
        doc = "> **Note**",
        doc = ">",
        doc = "> This option blocks write access to the database while it scans the tables. It is best to get guidance from [Perforce Support](https://www.perforce.com/support?product=Helix%20Core) before using this option."
    )]
    pub fn set_show_histogram(&mut self, v: bool) -> &mut Self {
        self.mode.show_histogram = v;
        self
    }

    /// # Description
    ///
    /// -h
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Display a histogram showing distances between leaf pages."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Display a histogram showing distances between leaf pages. This enables you to anticipate disk seeks required for sequential database scans. This scans only the internal node pages of the file, not the leaf or overflow pages, and thus reads only 1-5% of the file.",
        doc = "",
        doc = "> **Note**",
        doc = ">",
        doc = "> This option blocks write access to the database while it scans the tables. It is best to get guidance from [Perforce Support](https://www.perforce.com/support?product=Helix%20Core) before using this option."
    )]
    pub fn show_histogram(mut self, v: bool) -> Self {
        self.set_show_histogram(v);
        self
    }

    /// # Description
    ///
    /// -f
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Displays a page count, free pages, and percent free data for the specified tables."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Displays a page count, free pages, and percent free data for the specified tables. Using the `-f` with the `-h` option, this command shows a histogram of free page distribution."
    )]
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_show_free_pages(&self) -> bool {
        self.mode.show_free_pages
    }

    /// # Description
    ///
    /// -f
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Displays a page count, free pages, and percent free data for the specified tables."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Displays a page count, free pages, and percent free data for the specified tables. Using the `-f` with the `-h` option, this command shows a histogram of free page distribution."
    )]
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_show_free_pages(&mut self, v: bool) -> &mut Self {
        self.mode.show_free_pages = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Displays a page count, free pages, and percent free data for the specified tables."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Displays a page count, free pages, and percent free data for the specified tables. Using the `-f` with the `-h` option, this command shows a histogram of free page distribution."
    )]
    #[cfg(not(feature = "lt2016_1"))]
    pub fn show_free_pages(mut self, v: bool) -> Self {
        self.set_show_free_pages(v);
        self
    }
}

impl DBStat<Scan<Unselected>> {
    /// # Description
    ///
    /// -a
    ///
    /// Display statistics for all tables.
    ///
    /// Transitions to the [`DisplayAllTables`] state, which spawns without
    /// table arguments.
    pub fn all(self) -> DBStat<Scan<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Scan {
                show_histogram: self.mode.show_histogram,
                #[cfg(not(feature = "lt2016_1"))]
                show_free_pages: self.mode.show_free_pages,
                mode: DisplayAllTables,
            },
        }
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl<M> DBStat<CheckPoint<M>> {
    /// # Description
    ///
    /// -w worklevel
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Overrides the [db.checkpoint.worklevel](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.worklevel) configurable so you can specify the `worklevel` at the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "The `worklevel` determines which level of the table should be used to split the table. This option overrides the [db.checkpoint.worklevel](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.worklevel) configurable so you can specify the `worklevel` at the command line."
    )]
    pub fn get_worklevel(&self) -> Option<u64> {
        self.mode.worklevel
    }

    /// # Description
    ///
    /// -w worklevel
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Overrides the [db.checkpoint.worklevel](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.worklevel) configurable so you can specify the `worklevel` at the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "The `worklevel` determines which level of the table should be used to split the table. This option overrides the [db.checkpoint.worklevel](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.worklevel) configurable so you can specify the `worklevel` at the command line."
    )]
    pub fn set_worklevel(&mut self, v: u64) -> &mut Self {
        self.mode.worklevel = Some(v);
        self
    }

    /// # Description
    ///
    /// -w worklevel
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Overrides the [db.checkpoint.worklevel](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.worklevel) configurable so you can specify the `worklevel` at the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "The `worklevel` determines which level of the table should be used to split the table. This option overrides the [db.checkpoint.worklevel](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.worklevel) configurable so you can specify the `worklevel` at the command line."
    )]
    pub fn worklevel(mut self, v: u64) -> Self {
        self.set_worklevel(v);
        self
    }

    /// # Description
    ///
    /// -n numberOfFiles
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Overrides the [db.checkpoint.numfiles](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.numfiles) configurable so you can specify `numberOfFiles` at the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "This option determines, along with the depth of the database table, the number of checkpoint files that would be generated. A large table might contain multiple pages. The `-n numberOfFiles` requested makes it possible divide the page count to split each chunk of work into similar sizes. This option overrides the [db.checkpoint.numfiles](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.numfiles) configurable so you can specify `numberOfFiles` at the command line."
    )]
    pub fn get_number_files(&self) -> Option<u64> {
        self.mode.number_files
    }

    /// # Description
    ///
    /// -n numberOfFiles
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Overrides the [db.checkpoint.numfiles](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.numfiles) configurable so you can specify `numberOfFiles` at the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "This option determines, along with the depth of the database table, the number of checkpoint files that would be generated. A large table might contain multiple pages. The `-n numberOfFiles` requested makes it possible divide the page count to split each chunk of work into similar sizes. This option overrides the [db.checkpoint.numfiles](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.numfiles) configurable so you can specify `numberOfFiles` at the command line."
    )]
    pub fn set_number_files(&mut self, v: u64) -> &mut Self {
        self.mode.number_files = Some(v);
        self
    }

    /// # Description
    ///
    /// -n numberOfFiles
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Overrides the [db.checkpoint.numfiles](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.numfiles) configurable so you can specify `numberOfFiles` at the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "This option determines, along with the depth of the database table, the number of checkpoint files that would be generated. A large table might contain multiple pages. The `-n numberOfFiles` requested makes it possible divide the page count to split each chunk of work into similar sizes. This option overrides the [db.checkpoint.numfiles](https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/configurables.alphabetical.html#db.checkpoint.numfiles) configurable so you can specify `numberOfFiles` at the command line."
    )]
    pub fn number_files(mut self, v: u64) -> Self {
        self.set_number_files(v);
        self
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl DBStat<CheckPoint<Unselected>> {
    /// # Description
    ///
    /// -a
    ///
    /// Display statistics for all tables.
    ///
    /// Transitions to the [`DisplayAllTables`] state, which spawns without
    /// table arguments.
    pub fn all(self) -> DBStat<CheckPoint<DisplayAllTables>> {
        DBStat {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CheckPoint {
                worklevel: self.mode.worklevel,
                number_files: self.mode.number_files,
                mode: DisplayAllTables,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Accessors and command assembly
// ---------------------------------------------------------------------------

impl<M> DBStat<M> {
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

impl<M> SubCommand for DBStat<M>
where
    M: ExclusiveOption,
{
    fn name(&self) -> &str {
        "dbstat"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<S, I> ParameterizedSpawn<(S,)> for DBStat<Unselected>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat` for the given tables as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle to
    /// wait for it or interact with it.
    fn spawn_with(&mut self, (tables,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(tables)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for DBStat<DisplayAllTables> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat -a` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for DBStat<Scan<DisplayAllTables>> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat -a` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S, I> ParameterizedSpawn<(S,)> for DBStat<Scan<Unselected>>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat` for the given tables as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with(&mut self, (tables,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(tables)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl ParameterizedSpawn<()> for DBStat<CheckPoint<DisplayAllTables>> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat -p -a` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl<S, I> ParameterizedSpawn<(S,)> for DBStat<CheckPoint<Unselected>>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat -p` for the given tables as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (tables,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(tables)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for DBStat<Size> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbstat -s` as a child process with piped standard output
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

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut command = DBStat::new("p4", GlobalOpts::new()).show_histogram();
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 dbstat` command line; no process
    /// is spawned.
    #[test]
    fn plain_form_without_options() {
        let dbstat = DBStat::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat"]);
    }

    #[test]
    fn plain_form_all() {
        let dbstat = DBStat::new("p4", GlobalOpts::new()).all();

        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-a"]);
    }

    #[test]
    fn show_histogram_enters_scan_form() {
        let dbstat = DBStat::new("p4", GlobalOpts::new()).show_histogram();

        assert!(dbstat.get_show_histogram());
        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-h"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn show_free_pages_enters_scan_form() {
        let dbstat = DBStat::new("p4", GlobalOpts::new()).show_free_pages();

        assert!(dbstat.get_show_free_pages());
        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-f"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn scan_form_with_histogram_and_free_pages() {
        let dbstat = DBStat::new("p4", GlobalOpts::new())
            .show_histogram()
            .show_free_pages(true);

        assert!(dbstat.get_show_histogram());
        assert!(dbstat.get_show_free_pages());
        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-h", "-f"]);
    }

    #[test]
    fn scan_form_all() {
        let dbstat = DBStat::new("p4", GlobalOpts::new()).show_histogram().all();

        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-h", "-a"]);
    }

    #[test]
    fn size_form() {
        let dbstat = DBStat::new("p4", GlobalOpts::new()).size();

        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-s"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn checkpoint_form_with_worklevel_and_number_files() {
        let mut dbstat = DBStat::new("p4", GlobalOpts::new()).checkpoint();
        dbstat.set_worklevel(2);
        let dbstat = dbstat.number_files(3);

        assert_eq!(dbstat.get_worklevel(), Some(2));
        assert_eq!(dbstat.get_number_files(), Some(3));
        assert_eq!(
            args_of(&dbstat.setup_command("p4")),
            ["dbstat", "-p", "-w", "2", "-n", "3"]
        );
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn checkpoint_form_without_options() {
        let dbstat = DBStat::new("p4", GlobalOpts::new()).checkpoint();

        assert_eq!(dbstat.get_worklevel(), None);
        assert_eq!(dbstat.get_number_files(), None);
        assert_eq!(args_of(&dbstat.setup_command("p4")), ["dbstat", "-p"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn checkpoint_form_all_keeps_options() {
        let dbstat = DBStat::new("p4", GlobalOpts::new())
            .checkpoint()
            .worklevel(2)
            .all();

        assert_eq!(
            args_of(&dbstat.setup_command("p4")),
            ["dbstat", "-p", "-w", "2", "-a"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let dbstat = DBStat::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .show_histogram()
        .all();

        assert_eq!(
            args_of(&dbstat.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "dbstat", "-h", "-a"]
        );
    }
}
