use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Name filter (-e / -E)
// ---------------------------------------------------------------------------

/// The `-e` / `-E` name filter: list only client workspaces whose name
/// matches the filter pattern.
///
/// Created by [`Clients::filter`]; the `case_insensitive` parameter selects
/// [`Filter::CaseSensitiveMode`] (`-e`) or [`Filter::InCaseSensitiveMode`]
/// (`-E`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// `-e`: match the pattern using the server's normal case-sensitivity
    /// rules.
    CaseSensitiveMode(String),

    /// `-E`: match the pattern case-insensitively, even on a case-sensitive
    /// server.
    InCaseSensitiveMode(String),
}

impl ExclusiveOption for Filter {
    fn inject_args(&self, command: &mut Command) {
        match self {
            Filter::CaseSensitiveMode(name) => {
                command.arg("-e").arg(name);
            }
            Filter::InCaseSensitiveMode(name) => {
                command.arg("-E").arg(name);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Owner filter states (-u user / --me)
// ---------------------------------------------------------------------------

/// The `-u user` state: list only client workspaces owned by the named user.
///
/// Created by [`Clients::user`]; call [`Clients::case_insensitive`] on the
/// entered state to add the `--user-case-insensitive` suboption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    user: String,

    #[cfg(not(feature = "lt2024_2"))]
    case_insensitive: bool,
}

impl ExclusiveOption for User {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-u").arg(&self.user);
        #[cfg(not(feature = "lt2024_2"))]
        if self.case_insensitive {
            command.arg("--user-case-insensitive");
        }
    }
}

/// The `--me` state: list only client workspaces owned by the current user
/// (equivalent to `-u $P4USER`).
///
/// Created by [`Clients::filter_me`].
#[cfg(not(feature = "lt2016_1"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Me {}

#[cfg(not(feature = "lt2016_1"))]
impl ExclusiveOption for Me {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--me");
    }
}

// ---------------------------------------------------------------------------
// List category states (-a / -s serverID)
// ---------------------------------------------------------------------------

/// The `-a` state: list all client workspaces, not just workspaces bound to
/// this server.
///
/// Created by [`Clients::all`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAll {}

impl ExclusiveOption for ListAll {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");
    }
}

/// The `-s serverID` state: list only client workspaces bound to the
/// specified server.
///
/// Created by [`Clients::server_id`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSpecified {
    server_id: String,
}

impl ExclusiveOption for ListSpecified {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s").arg(&self.server_id);
    }
}

// ---------------------------------------------------------------------------
// Mode structs
// ---------------------------------------------------------------------------

/// The regular listing form: `p4 clients [-t] [-u user | --me] [[-e|-E]
/// filter] [-m max] [-S stream] [-a | -s serverID]`.
///
/// Entered from [`Clients<Unselected>`](Clients) by setting any regular
/// option or filter, for example [`Clients::display_time`],
/// [`Clients::filter`], or [`Clients::user`].
///
/// The `F`, `U`, and `C` type parameters track the mutually exclusive option
/// groups at compile time, each starting as [`Unselected`]:
/// - `F`: the `-e` / `-E` name filter, entered via [`Clients::filter`]
///   ([`Filter`]).
/// - `U`: the `-u user` / `--me` owner filter, entered via [`Clients::user`]
///   ([`User`]) or [`Clients::filter_me`] ([`Me`]).
/// - `C`: the `-a` / `-s serverID` list category, entered via
///   [`Clients::all`] ([`ListAll`]) or [`Clients::server_id`]
///   ([`ListSpecified`]).
///
/// Each group can be set at most once; the remaining options stay available
/// afterwards.
#[derive(Debug, Clone)]
pub struct RegularMode<F = Unselected, U = Unselected, C = Unselected> {
    display_time: bool,

    limit: Option<u64>,

    stream: Option<String>,

    name_filter: F,

    user_filter: U,

    list_category: C,
}

/// The unloaded listing form: `p4 clients -U`.
///
/// List only client workspaces unloaded with `p4 unload`. This form is
/// entered via [`Clients::unloaded`] and accepts no other options.
#[derive(Debug, Clone, Default)]
pub struct UnloadedMode {}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] clients [-t] [ -u user ] [ [-e|-E] filter -m max ] -S stream [-a | -s serverID]`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] clients [-t] [-u user] [[-e|-E] filter] [-m max] [-S stream] [-a | -s serverID]`"
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] clients [-t] [-u user | --me] [[-e|-E] filter] [-m max] [-S stream] [-a | -s serverID]`"
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "`p4 [g-opts] clients [-t] [-u user [--user-case-insensitive] | --me] [[-e|-E] filter] [-m max] [-S stream] [-a | -s serverID]`"
)]
///
/// `p4 [g-opts] clients -U`
///
/// List all client workspaces currently known to the system.
///
/// The command `p4 workspaces` is an alias for `p4 clients`.
///
/// The `M` type parameter tracks the command form at compile time, starting
/// as [`Unselected`] from [`Clients::new`], which spawns the bare `p4
/// clients` report. Set any regular option or filter (for example
/// [`Self::display_time`], [`Self::filter`], or [`Self::user`]) to enter
/// [`RegularMode`], or call [`Self::unloaded`] (`-U`) to enter
/// [`UnloadedMode`].
#[derive(Debug, Clone, Default)]
pub struct Clients<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

impl Clients<Unselected> {
    /// Creates a new `p4 clients` command with no option selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. The
    /// command spawns the bare `p4 clients` report as is; set any regular
    /// option or filter to enter [`RegularMode`], or call
    /// [`Self::unloaded`] to enter [`UnloadedMode`].
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// # Description
    ///
    /// -U
    ///
    /// List only client workspaces unloaded with `p4 unload`.
    ///
    /// This is a standalone syntax form (`p4 clients -U`) that accepts no
    /// other options.
    ///
    /// Transitions this command to the [`UnloadedMode`] state.
    pub fn unloaded(self) -> Clients<UnloadedMode> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: UnloadedMode {},
        }
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the
    /// workspace.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn display_time(self, v: bool) -> Clients<RegularMode> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: v,
                limit: None,
                stream: None,
                name_filter: Unselected,
                user_filter: Unselected,
                list_category: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` client workspaces.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn limit(self, v: u64) -> Clients<RegularMode> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: Some(v),
                stream: None,
                name_filter: Unselected,
                user_filter: Unselected,
                list_category: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List client workspaces associated with the specified stream.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn stream(self, v: impl Into<String>) -> Clients<RegularMode> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: None,
                stream: Some(v.into()),
                name_filter: Unselected,
                user_filter: Unselected,
                list_category: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -e filter / -E filter
    ///
    /// `v` is the client workspace name filter pattern; lists only client
    /// workspaces whose name matches it.
    ///
    /// When `case_insensitive` is `false` (`-e`), the pattern is matched
    /// using the server's normal case-sensitivity rules. When
    /// `case_insensitive` is `true` (`-E`), the pattern is matched
    /// case-insensitively, even on a case-sensitive server.
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "",
        doc = "`--client-case-insensitive` is equivalent to `-E`."
    )]
    ///
    /// Transitions this command to the [`Filter`] state.
    pub fn filter(
        self,
        v: impl Into<String>,
        case_insensitive: bool,
    ) -> Clients<RegularMode<Filter, Unselected, Unselected>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: None,
                stream: None,
                name_filter: if case_insensitive {
                    Filter::InCaseSensitiveMode(v.into())
                } else {
                    Filter::CaseSensitiveMode(v.into())
                },
                user_filter: Unselected,
                list_category: Unselected,
            },
        }
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2024_2", doc = "-u user")]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "-u user [--user-case-insensitive]")]
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "List only client workspaces owned by `v`."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "List client workspaces that are owned by `v`. This can include `*` wildcards to form a search pattern. If wildcards are used, enclose the search pattern in double quotes (`\"`).",
        doc = "",
        doc = "For example:",
        doc = "",
        doc = "`p4 clients -u \"O*hara\"`",
        doc = "",
        doc = "Call [`Clients::case_insensitive`] on the entered state to add the `--user-case-insensitive` suboption."
    )]
    ///
    /// Transitions this command to the [`User`] state.
    pub fn user(self, v: impl Into<String>) -> Clients<RegularMode<Unselected, User, Unselected>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: None,
                stream: None,
                name_filter: Unselected,
                user_filter: User {
                    user: v.into(),
                    #[cfg(not(feature = "lt2024_2"))]
                    case_insensitive: false,
                },
                list_category: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// --me
    ///
    /// Equivalent to `-u $P4USER`.
    ///
    /// Transitions this command to the [`Me`] state.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn filter_me(self) -> Clients<RegularMode<Unselected, Me, Unselected>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: None,
                stream: None,
                name_filter: Unselected,
                user_filter: Me {},
                list_category: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -a
    ///
    /// List all client workspaces, not just workspaces bound to this server.
    ///
    /// Transitions this command to the [`ListAll`] state.
    pub fn all(self) -> Clients<RegularMode<Unselected, Unselected, ListAll>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: None,
                stream: None,
                name_filter: Unselected,
                user_filter: Unselected,
                list_category: ListAll {},
            },
        }
    }

    /// # Description
    ///
    /// -s serverID
    ///
    /// List only client workspaces bound to the specified `serverID`. On an
    /// edge server, the `-s` option defaults to the edge server's serverID.
    ///
    /// Transitions this command to the [`ListSpecified`] state.
    pub fn server_id(
        self,
        v: impl Into<String>,
    ) -> Clients<RegularMode<Unselected, Unselected, ListSpecified>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: false,
                limit: None,
                stream: None,
                name_filter: Unselected,
                user_filter: Unselected,
                list_category: ListSpecified {
                    server_id: v.into(),
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Shared accessors
// ---------------------------------------------------------------------------

impl<M> Clients<M> {
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
// Regular form accessors
// ---------------------------------------------------------------------------

impl<F, U, C> Clients<RegularMode<F, U, C>> {
    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` client workspaces.
    pub fn get_limit(&self) -> Option<u64> {
        self.mode.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` client workspaces.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.mode.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` client workspaces.
    pub fn limit(mut self, v: u64) -> Self {
        self.mode.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the
    /// workspace.
    pub fn get_display_time(&self) -> bool {
        self.mode.display_time
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the
    /// workspace.
    pub fn set_display_time(&mut self, v: bool) -> &mut Self {
        self.mode.display_time = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the
    /// workspace.
    pub fn display_time(mut self, v: bool) -> Self {
        self.mode.display_time = v;
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List client workspaces associated with the specified stream.
    pub fn get_stream(&self) -> Option<&String> {
        self.mode.stream.as_ref()
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List client workspaces associated with the specified stream.
    pub fn set_stream(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.stream = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List client workspaces associated with the specified stream.
    pub fn stream(mut self, v: impl Into<String>) -> Self {
        self.mode.stream = Some(v.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Name filter transitions
// ---------------------------------------------------------------------------

impl<U, C> Clients<RegularMode<Unselected, U, C>> {
    /// # Description
    ///
    /// -e filter / -E filter
    ///
    /// `v` is the client workspace name filter pattern; lists only client
    /// workspaces whose name matches it.
    ///
    /// When `case_insensitive` is `false` (`-e`), the pattern is matched
    /// using the server's normal case-sensitivity rules. When
    /// `case_insensitive` is `true` (`-E`), the pattern is matched
    /// case-insensitively, even on a case-sensitive server.
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "",
        doc = "`--client-case-insensitive` is equivalent to `-E`."
    )]
    ///
    /// Transitions this command to the [`Filter`] state.
    pub fn filter(
        self,
        v: impl Into<String>,
        case_insensitive: bool,
    ) -> Clients<RegularMode<Filter, U, C>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: self.mode.display_time,
                limit: self.mode.limit,
                stream: self.mode.stream,
                user_filter: self.mode.user_filter,
                list_category: self.mode.list_category,
                name_filter: if case_insensitive {
                    Filter::InCaseSensitiveMode(v.into())
                } else {
                    Filter::CaseSensitiveMode(v.into())
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Owner filter transitions
// ---------------------------------------------------------------------------

impl<F, C> Clients<RegularMode<F, Unselected, C>> {
    /// # Description
    ///
    #[cfg_attr(feature = "lt2024_2", doc = "-u user")]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "-u user [--user-case-insensitive]")]
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "List only client workspaces owned by `v`."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "List client workspaces that are owned by `v`. This can include `*` wildcards to form a search pattern. If wildcards are used, enclose the search pattern in double quotes (`\"`).",
        doc = "",
        doc = "For example:",
        doc = "",
        doc = "`p4 clients -u \"O*hara\"`",
        doc = "",
        doc = "Call [`Clients::case_insensitive`] on the entered state to add the `--user-case-insensitive` suboption."
    )]
    ///
    /// Transitions this command to the [`User`] state.
    pub fn user(self, v: impl Into<String>) -> Clients<RegularMode<F, User, C>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: self.mode.display_time,
                limit: self.mode.limit,
                stream: self.mode.stream,
                name_filter: self.mode.name_filter,
                list_category: self.mode.list_category,
                user_filter: User {
                    user: v.into(),
                    #[cfg(not(feature = "lt2024_2"))]
                    case_insensitive: false,
                },
            },
        }
    }

    /// # Description
    ///
    /// --me
    ///
    /// Equivalent to `-u $P4USER`.
    ///
    /// Transitions this command to the [`Me`] state.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn filter_me(self) -> Clients<RegularMode<F, Me, C>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: self.mode.display_time,
                limit: self.mode.limit,
                stream: self.mode.stream,
                name_filter: self.mode.name_filter,
                list_category: self.mode.list_category,
                user_filter: Me {},
            },
        }
    }
}

impl<F, C> Clients<RegularMode<F, User, C>> {
    /// # Description
    ///
    /// --user-case-insensitive
    ///
    /// Indicates that the `-u` user value is a case-insensitive search
    /// pattern, even on a case-sensitive server.
    ///
    /// For example:
    ///
    /// `p4 clients -u "O*hara" --user-case-insensitive`
    #[cfg(not(feature = "lt2024_2"))]
    pub fn case_insensitive(self) -> Clients<RegularMode<F, User, C>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: self.mode.display_time,
                limit: self.mode.limit,
                stream: self.mode.stream,
                name_filter: self.mode.name_filter,
                list_category: self.mode.list_category,
                user_filter: User {
                    user: self.mode.user_filter.user,
                    case_insensitive: true,
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// List category transitions
// ---------------------------------------------------------------------------

impl<F, U> Clients<RegularMode<F, U, Unselected>> {
    /// # Description
    ///
    /// -a
    ///
    /// List all client workspaces, not just workspaces bound to this server.
    ///
    /// Transitions this command to the [`ListAll`] state.
    pub fn all(self) -> Clients<RegularMode<F, U, ListAll>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: self.mode.display_time,
                limit: self.mode.limit,
                stream: self.mode.stream,
                name_filter: self.mode.name_filter,
                user_filter: self.mode.user_filter,
                list_category: ListAll {},
            },
        }
    }

    /// # Description
    ///
    /// -s serverID
    ///
    /// List only client workspaces bound to the specified `serverID`. On an
    /// edge server, the `-s` option defaults to the edge server's serverID.
    ///
    /// Transitions this command to the [`ListSpecified`] state.
    pub fn server_id(self, v: impl Into<String>) -> Clients<RegularMode<F, U, ListSpecified>> {
        Clients {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                display_time: self.mode.display_time,
                limit: self.mode.limit,
                stream: self.mode.stream,
                name_filter: self.mode.name_filter,
                user_filter: self.mode.user_filter,
                list_category: ListSpecified {
                    server_id: v.into(),
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl SubCommand for Clients<Unselected> {
    fn name(&self) -> &str {
        "clients"
    }

    fn inject_local_args(&self, _: &mut Command) {}

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<F: ExclusiveOption, U: ExclusiveOption, C: ExclusiveOption> SubCommand
    for Clients<RegularMode<F, U, C>>
{
    fn name(&self) -> &str {
        "clients"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.mode.display_time {
            command.arg("-t");
        }
        self.mode.user_filter.inject_args(command);
        self.mode.name_filter.inject_args(command);
        if let Some(max) = self.mode.limit {
            command.arg("-m").arg(max.to_string());
        }
        if let Some(ref stream) = self.mode.stream {
            command.arg("-S").arg(stream);
        }
        self.mode.list_category.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl SubCommand for Clients<UnloadedMode> {
    fn name(&self) -> &str {
        "clients"
    }

    fn inject_local_args(&self, command: &mut Command) {
        command.arg("-U");
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

impl ParameterizedSpawn<()> for Clients<Unselected> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 clients` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it
    /// or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<F: ExclusiveOption, U: ExclusiveOption, C: ExclusiveOption> ParameterizedSpawn<()>
    for Clients<RegularMode<F, U, C>>
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 clients` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it
    /// or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for Clients<UnloadedMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 clients -U` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it
    /// or interact with it.
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
        let mut command = Clients::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    // Compile-time rejection: -e and -E are mutually exclusive by
    // construction (the Filter enum holds exactly one variant, and no
    // transition exists on Clients<RegularMode<Filter, ..>>).
    // Compile-time rejection: -u and --me are mutually exclusive (user and
    // filter_me exist only on the Unselected owner-filter state).
    // Compile-time rejection: case_insensitive exists only on the User
    // state, never on Me or Unselected.
    // Compile-time rejection: -a and -s are mutually exclusive (all and
    // server_id exist only on the Unselected list-category state).
    // Compile-time rejection: -U is only reachable from Clients<Unselected>,
    // and Clients<UnloadedMode> offers no other options.
    // Each filter can be set at most once; limit, display_time, stream, and
    // the other filters stay available afterwards.

    #[test]
    fn without_options() {
        let clients = Clients::new("p4", GlobalOpts::new());
        assert_eq!(args_of(&clients.setup_command("p4")), ["clients"]);
    }

    #[test]
    fn limit() {
        let clients = Clients::new("p4", GlobalOpts::new()).limit(5);
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-m", "5"]
        );
    }

    #[test]
    fn display_time() {
        let clients = Clients::new("p4", GlobalOpts::new()).display_time(true);
        assert_eq!(args_of(&clients.setup_command("p4")), ["clients", "-t"]);
    }

    #[test]
    fn display_time_can_be_unset() {
        let clients = Clients::new("p4", GlobalOpts::new()).display_time(false);
        assert_eq!(args_of(&clients.setup_command("p4")), ["clients"]);
    }

    #[test]
    fn stream() {
        let clients = Clients::new("p4", GlobalOpts::new()).stream("//stream/main");
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-S", "//stream/main"]
        );
    }

    #[test]
    fn filter_case_sensitive() {
        let clients = Clients::new("p4", GlobalOpts::new()).filter("p4vBeta*", false);
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-e", "p4vBeta*"]
        );
    }

    #[test]
    fn filter_case_insensitive() {
        let clients = Clients::new("p4", GlobalOpts::new()).filter("*Us*rs", true);
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-E", "*Us*rs"]
        );
    }

    #[test]
    fn user() {
        let clients = Clients::new("p4", GlobalOpts::new()).user("O*hara");
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-u", "O*hara"]
        );
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn user_case_insensitive() {
        let clients = Clients::new("p4", GlobalOpts::new())
            .user("O*hara")
            .case_insensitive();
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-u", "O*hara", "--user-case-insensitive"]
        );
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn filter_me() {
        let clients = Clients::new("p4", GlobalOpts::new()).filter_me();
        assert_eq!(args_of(&clients.setup_command("p4")), ["clients", "--me"]);
    }

    #[test]
    fn all() {
        let clients = Clients::new("p4", GlobalOpts::new()).all();
        assert_eq!(args_of(&clients.setup_command("p4")), ["clients", "-a"]);
    }

    #[test]
    fn server_id() {
        let clients = Clients::new("p4", GlobalOpts::new()).server_id("edge1");
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-s", "edge1"]
        );
    }

    #[test]
    fn unloaded() {
        let clients = Clients::new("p4", GlobalOpts::new()).unloaded();
        assert_eq!(args_of(&clients.setup_command("p4")), ["clients", "-U"]);
    }

    #[test]
    fn transitions_preserve_other_options() {
        let clients = Clients::new("p4", GlobalOpts::new())
            .user("O*hara")
            .filter("p4v*", false)
            .limit(5)
            .display_time(true)
            .stream("//stream/main")
            .all();
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            [
                "clients",
                "-t",
                "-u",
                "O*hara",
                "-e",
                "p4v*",
                "-m",
                "5",
                "-S",
                "//stream/main",
                "-a"
            ]
        );

        let clients = Clients::new("p4", GlobalOpts::new())
            .server_id("edge1")
            .filter("*Us*rs", true);
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["clients", "-E", "*Us*rs", "-s", "edge1"]
        );
    }

    #[test]
    fn setters_update_regular_options() {
        let mut clients = Clients::new("p4", GlobalOpts::new()).filter("p4v*", false);
        assert_eq!(clients.get_limit(), None);
        clients.set_limit(10);
        assert_eq!(clients.get_limit(), Some(10));
        assert!(!clients.get_display_time());
        clients.set_display_time(true);
        assert!(clients.get_display_time());
        assert_eq!(clients.get_stream(), None);
        clients.set_stream("//stream/dev");
        assert_eq!(clients.get_stream(), Some(&"//stream/dev".to_string()));
    }

    #[test]
    fn all_options_order() {
        let clients = Clients::new("p4", GlobalOpts::new())
            .display_time(true)
            .user("O*hara")
            .filter("*Us*rs", true)
            .limit(5)
            .stream("//stream/main")
            .server_id("edge1");
        assert_eq!(
            args_of(&clients.setup_command("p4")),
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
                "edge1"
            ]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let clients = Clients::new("p4", GlobalOpts::new().port("localhost:1666"));
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["-p", "localhost:1666", "clients"]
        );

        let clients = Clients::new("p4", GlobalOpts::new().port("localhost:1666")).limit(1);
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["-p", "localhost:1666", "clients", "-m", "1"]
        );

        let clients = Clients::new("p4", GlobalOpts::new().port("localhost:1666")).unloaded();
        assert_eq!(
            args_of(&clients.setup_command("p4")),
            ["-p", "localhost:1666", "clients", "-U"]
        );
    }
}
