use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

#[cfg(not(feature = "lt2017_1"))]
use super::depot::DepotType;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The `-e` / `-E` name filter: list only depots whose name matches the
/// filter pattern.
///
/// Created by [`Depots::filter`]; the `case_insensitive` parameter selects
/// [`Filter::CaseSensitiveMode`] (`-e`) or [`Filter::InCaseSensitiveMode`]
/// (`-E`).
#[cfg(not(feature = "lt2017_2"))]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Filter {
    /// `-e`: match the pattern using the server's normal case-sensitivity
    /// rules.
    CaseSensitiveMode(String),

    /// `-E`: match the pattern case-insensitively, even on a case-sensitive
    /// server.
    InCaseSensitiveMode(String),
}

#[cfg(not(feature = "lt2017_2"))]
impl ExclusiveOption for Filter {
    fn inject_args(&self, command: &mut Command) {
        match self {
            Filter::CaseSensitiveMode(pattern) => {
                command.arg("-e").arg(pattern);
            }
            Filter::InCaseSensitiveMode(pattern) => {
                command.arg("-E").arg(pattern);
            }
        }
    }
}

/// List-all mode of `p4 depots` (`-a`): lists depots of `all` types,
/// included hidden depots.
///
/// Entered with [`Depots::list_all`].
#[cfg(not(feature = "lt2024_1"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListAll {}

#[cfg(not(feature = "lt2024_1"))]
impl ExclusiveOption for ListAll {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");
    }
}

/// List-specified mode of `p4 depots` (`[-t type]`): list only the depots
/// of the specified type.
///
/// Entered with [`Depots::depot_type`].
#[cfg(not(feature = "lt2017_1"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListSpecified {
    depot_type: Option<DepotType>,
}

#[cfg(not(feature = "lt2017_1"))]
impl ExclusiveOption for ListSpecified {
    fn inject_args(&self, command: &mut Command) {
        if let Some(depot_type) = &self.depot_type {
            #[cfg(feature = "lt2017_2")]
            command.arg(format!("--depot-type={}", depot_type.as_str()));
            #[cfg(not(feature = "lt2017_2"))]
            command.arg("-t").arg(depot_type.as_str());
        }
    }
}

#[cfg_attr(feature = "lt2017_1", doc = "`p4 [g-opts] depots`")]
#[cfg_attr(
    all(feature = "lt2017_2", not(feature = "lt2017_1")),
    doc = "`p4 [g-opts] depots [--depot-type=type]`"
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2017_2")),
    doc = "`p4 [g-opts] depots [[-e | -E] nameFilter] [-t type]`"
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "`p4 [g-opts] depots [[-e | -E] nameFilter] [-t type] [-a]`"
)]
///
#[cfg_attr(
    feature = "lt2017_2",
    doc = "Display a list of depots known to the Perforce versioning service."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2017_2")),
    doc = "Display a list of depots known to the Helix Server."
)]
#[cfg_attr(
    all(feature = "lt2021_1", not(feature = "lt2019_1")),
    doc = "Display a list of depots known to the Helix server."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2021_1")),
    doc = "Display a list of depots known to the Helix Server."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2024_1")),
    doc = "Display a list of depots known to the Helix Core Server."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "Display a list of depots known to the P4 Server."
)]
///
#[cfg_attr(
    feature = "lt2017_1",
    doc = "Lists all the remote and local depots known to the Perforce service, in the form:",
    doc = "",
    doc = "`Depot name date type address map description`",
    doc = "",
    doc = "where name, date, type, address, map, and description are as defined in the [`p4 depot`](crate::cmd::Depot) form.",
    doc = "",
    doc = "If a depot is excluded in the protections table for a given user, the user denied access will no longer see the depot in the output of `p4 depots`.",
    doc = "",
    doc = "`Operator` users may now run the `p4 depots` command."
)]
#[cfg_attr(not(feature = "lt2017_1"), doc = "Display a list of depots.")]
#[cfg_attr(
    all(feature = "lt2022_1", not(feature = "lt2020_2")),
    doc = "",
    doc = "The list excludes the default graph depot named `repo` unless you use the command, `p4 depot -t graph`."
)]
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "",
    doc = "The list excludes",
    doc = "",
    doc = "- the default graph depot named `repo` unless you use the `p4 depots -t graph` command",
    doc = "- the extension depot unless a user with the `super` protection uses the `p4 depots -t extension` command"
)]
#[cfg_attr(
    not(feature = "lt2017_1"),
    doc = "",
    doc = "If a depot is excluded in the protections table for a given user, that user does not see the depot in the output of this command.",
    doc = "",
    doc = "`Operator` users can run this command. (To learn about `operator` users, see the `p4 user` usage notes.)"
)]
///
/// The `M` type parameter tracks the command form at compile time. The
/// default [`Unselected`] state is the bare listing form with no local
/// options
#[cfg_attr(feature = "lt2017_1", doc = ".")]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2017_1")),
    doc = "; [`Self::depot_type`] transitions to the [`ListSpecified`] mode."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "; [`Self::depot_type`] and [`Self::list_all`] transition to the [`ListSpecified`] and [`ListAll`] modes."
)]
#[derive(Debug, Clone, Default)]
pub struct Depots<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    #[cfg(not(feature = "lt2017_2"))]
    filter: Option<Filter>,

    mode: M,
}

impl Depots<Unselected> {
    /// Creates a new `p4 depots` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            #[cfg(not(feature = "lt2017_2"))]
            filter: None,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2017_2", doc = "--depot-type=type")]
    #[cfg_attr(not(feature = "lt2017_2"), doc = "-t type")]
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "List only the depots of the specified *type*"
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "List only the depots of the specified *type*."
    )]
    ///
    /// Transitions this command to the [`ListSpecified`] mode.
    #[cfg(not(feature = "lt2017_1"))]
    pub fn depot_type(self, v: DepotType) -> Depots<ListSpecified> {
        Depots {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2017_2"))]
            filter: self.filter,
            mode: ListSpecified {
                depot_type: Some(v),
            },
        }
    }

    /// # Description
    ///
    /// -a
    ///
    /// Lists depots of `all` types, included hidden depots. This is
    /// especially useful when the user needs to check map paths for all
    /// depots.
    ///
    /// Transitions this command to the [`ListAll`] mode.
    #[cfg(not(feature = "lt2024_1"))]
    pub fn list_all(self) -> Depots<ListAll> {
        Depots {
            bin: self.bin,
            global_opts: self.global_opts,
            filter: self.filter,
            mode: ListAll {},
        }
    }
}

#[cfg(not(feature = "lt2017_1"))]
impl Depots<ListSpecified> {
    /// # Description
    ///
    #[cfg_attr(feature = "lt2017_2", doc = "--depot-type=type")]
    #[cfg_attr(not(feature = "lt2017_2"), doc = "-t type")]
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "List only the depots of the specified *type*"
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "List only the depots of the specified *type*."
    )]
    pub fn get_depot_type(&self) -> Option<DepotType> {
        self.mode.depot_type
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2017_2", doc = "--depot-type=type")]
    #[cfg_attr(not(feature = "lt2017_2"), doc = "-t type")]
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "List only the depots of the specified *type*"
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "List only the depots of the specified *type*."
    )]
    pub fn set_depot_type(&mut self, v: DepotType) -> &mut Self {
        self.mode.depot_type = Some(v);
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2017_2", doc = "--depot-type=type")]
    #[cfg_attr(not(feature = "lt2017_2"), doc = "-t type")]
    ///
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "List only the depots of the specified *type*"
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "List only the depots of the specified *type*."
    )]
    pub fn depot_type(mut self, v: DepotType) -> Self {
        self.mode.depot_type = Some(v);
        self
    }
}

impl<M: ExclusiveOption> Depots<M> {
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
    /// -e nameFilter
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "Lists depot specs with a name that matches the *nameFilter* pattern. `p4 depots -e h*x*` finds depots with names like `helix` and `hxadm`. This option follows the case-sensitivity of the server."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Lists depot specs with a name that matches the *nameFilter* pattern. `p4 depots -e \"h*x*\"` finds depots with names like `helix` and `hxadm`. If you use the `*` wildcard, enclose the search pattern in double quotes (\"). This option follows the case-sensitivity of the server."
    )]
    ///
    /// -E nameFilter
    ///
    /// Makes the matching case-insensitive, even on a case-sensitive server.
    ///
    /// The `case_insensitive` parameter selects `-E` over `-e`.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn set_filter(&mut self, v: impl Into<String>, case_insensitive: bool) -> &mut Self {
        self.filter = Some(if case_insensitive {
            Filter::InCaseSensitiveMode(v.into())
        } else {
            Filter::CaseSensitiveMode(v.into())
        });
        self
    }

    /// # Description
    ///
    /// -e nameFilter
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "Lists depot specs with a name that matches the *nameFilter* pattern. `p4 depots -e h*x*` finds depots with names like `helix` and `hxadm`. This option follows the case-sensitivity of the server."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Lists depot specs with a name that matches the *nameFilter* pattern. `p4 depots -e \"h*x*\"` finds depots with names like `helix` and `hxadm`. If you use the `*` wildcard, enclose the search pattern in double quotes (\"). This option follows the case-sensitivity of the server."
    )]
    ///
    /// -E nameFilter
    ///
    /// Makes the matching case-insensitive, even on a case-sensitive server.
    ///
    /// The `case_insensitive` parameter selects `-E` over `-e`.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn filter(mut self, v: impl Into<String>, case_insensitive: bool) -> Self {
        self.set_filter(v, case_insensitive);
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

impl<M: ExclusiveOption> SubCommand for Depots<M> {
    fn name(&self) -> &str {
        "depots"
    }

    fn inject_local_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2017_2"))]
        if let Some(filter) = &self.filter {
            filter.inject_args(command);
        }
        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption> ParameterizedSpawn<()> for Depots<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 depots` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}
