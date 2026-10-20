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

/// The `-e` / `-E` name filter: list only branches whose name matches the
/// filter pattern.
///
/// Created by [`Branches::filter`]; the `case_insensitive` parameter selects
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

/// The `-u user` / `--me` owner filter: list only branches owned by the
/// specified user (or the current user).
///
#[cfg_attr(
    feature = "lt2016_1",
    doc = "Created by [`Branches::user`]. The enum holds exactly one of the two mutually exclusive choices."
)]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "Created by [`Branches::user`] or [`Branches::filter_me`]; the enum holds exactly one of the two mutually exclusive choices."
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum User {
    /// `-u user`: list branches owned by the named user; the flag selects
    /// the `--user-case-insensitive` suboption.
    Named {
        name: String,

        #[cfg(not(feature = "lt2024_2"))]
        case_insensitive: bool,
    },

    /// `--me`: list branches owned by the current user (equivalent to
    /// `-u $P4USER`).
    #[cfg(not(feature = "lt2016_1"))]
    Me,
}

impl ExclusiveOption for User {
    fn inject_args(&self, command: &mut Command) {
        match self {
            User::Named {
                name,
                #[cfg(not(feature = "lt2024_2"))]
                case_insensitive,
            } => {
                command.arg("-u").arg(name);
                #[cfg(not(feature = "lt2024_2"))]
                if *case_insensitive {
                    command.arg("--user-case-insensitive");
                }
            }
            #[cfg(not(feature = "lt2016_1"))]
            User::Me => {
                command.arg("--me");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2016_1",
    doc = "`p4 [g-opts] branches [-t] [-u user] [[-e|-E]filter -mmax]`"
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] branches [[-e | -E]filter] [-mmax][-t] [-u user | --me]`"
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "`p4 [g-opts] branches [[-e | -E] nameFilter] [-m max][-t] [-u user [--user-case-insensitive] | --me]`"
)]
///
/// List existing branch mappings.
///
/// Print the list of all branch mappings currently known to the system.
///
/// The `F` type parameter tracks the `-e` / `-E` name filter at compile
/// time, starting as [`Unselected`]: [`Self::filter`] transitions to the
/// [`Filter`] state, which carries both the pattern and its case
/// sensitivity.
///
/// The `U` type parameter tracks the owner filter at compile time, starting
/// as [`Unselected`]:
#[cfg_attr(feature = "lt2016_1", doc = "[`User`] (entered via [`Self::user`]).")]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "[`User`] (entered via [`Self::user`] or [`Self::filter_me`])."
)]
///
/// Each filter can be set at most once; the remaining options (`-m`, `-t`,
/// and the other filter) stay available afterwards.
#[derive(Debug, Clone, Default)]
pub struct Branches<F = Unselected, U = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    limit: Option<u64>,

    time: bool,

    user_filter: U,

    name_filter: F,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

impl Branches<Unselected, Unselected> {
    /// Creates a new `p4 branches` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Shared accessors
// ---------------------------------------------------------------------------

impl<F, U> Branches<F, U> {
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

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` branch mappings.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` branch mappings.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` branch mappings.
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the branch.
    pub fn get_display_time(&self) -> bool {
        self.time
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the branch.
    pub fn set_display_time(&mut self, v: bool) -> &mut Self {
        self.time = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Display the time as well as the date of the last update to the branch.
    pub fn display_time(mut self, v: bool) -> Self {
        self.time = v;
        self
    }
}

// ---------------------------------------------------------------------------
// Name filter transition
// ---------------------------------------------------------------------------

impl<U> Branches<Unselected, U> {
    /// # Description
    ///
    #[cfg_attr(feature = "lt2024_2", doc = "-e filter / -E filter")]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "-e nameFilter / -E nameFilter")]
    ///
    /// `v` is the branch name filter pattern; lists only branch specs whose
    /// name matches it.
    ///
    /// When `case_insensitive` is `false` (`-e`), the pattern is matched
    /// using the server's normal case-sensitivity rules. When
    /// `case_insensitive` is `true` (`-E`), the pattern is matched
    /// case-insensitively, even on a case-sensitive server.
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "",
        doc = "For example:",
        doc = "",
        doc = "`p4 branches -e \"svr-dev-rel*\"`"
    )]
    ///
    /// Transitions this command to the [`Filter`] state.
    pub fn filter(self, v: impl Into<String>, case_insensitive: bool) -> Branches<Filter, U> {
        Branches {
            bin: self.bin,
            global_opts: self.global_opts,
            limit: self.limit,
            time: self.time,
            user_filter: self.user_filter,
            name_filter: if case_insensitive {
                Filter::InCaseSensitiveMode(v.into())
            } else {
                Filter::CaseSensitiveMode(v.into())
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Owner filter transitions
// ---------------------------------------------------------------------------

impl<F> Branches<F, Unselected> {
    /// # Description
    ///
    /// -u user
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "`v` is the owner user name or search pattern."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "`v` is the owner user name or search pattern; `case_insensitive` selects the `--user-case-insensitive` suboption."
    )]
    ///
    #[cfg_attr(feature = "lt2024_2", doc = "List only branches owned by `user`.")]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "List branch specs owned by the specified user.",
        doc = "",
        doc = "This can include `*` wildcards to form a search pattern. If wildcards are used, enclose the search pattern in double quotes (`\"`).",
        doc = "",
        doc = "For example:",
        doc = "",
        doc = "`p4 branches -u \"*bert\"`",
        doc = "",
        doc = "You can also add the `--user-case-insensitive` suboption to indicate that the `user` value is a case-insensitive search pattern, even for a case-sensitive server."
    )]
    ///
    /// Transitions this command to the [`User`] state.
    pub fn user(self, v: impl Into<String>, case_insensitive: bool) -> Branches<F, User> {
        let _ = case_insensitive;
        Branches {
            bin: self.bin,
            global_opts: self.global_opts,
            limit: self.limit,
            time: self.time,
            name_filter: self.name_filter,
            user_filter: User::Named {
                name: v.into(),
                #[cfg(not(feature = "lt2024_2"))]
                case_insensitive,
            },
        }
    }

    /// # Description
    ///
    /// --me
    ///
    /// Equivalent to `-u $P4USER`.
    ///
    /// Transitions this command to the [`User`] state.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn filter_me(self) -> Branches<F, User> {
        Branches {
            bin: self.bin,
            global_opts: self.global_opts,
            limit: self.limit,
            time: self.time,
            name_filter: self.name_filter,
            user_filter: User::Me,
        }
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl<F: ExclusiveOption, U: ExclusiveOption> SubCommand for Branches<F, U> {
    fn name(&self) -> &str {
        "branches"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.name_filter.inject_args(command);
        if let Some(max) = self.limit {
            command.arg("-m").arg(max.to_string());
        }
        if self.time {
            command.arg("-t");
        }
        self.user_filter.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

impl<F: ExclusiveOption, U: ExclusiveOption> ParameterizedSpawn<()> for Branches<F, U> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branches` as a child process with piped standard output
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
        let mut command = Branches::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    // Compile-time rejection: -e and -E are mutually exclusive by
    // construction (the Filter enum holds exactly one variant).
    // Compile-time rejection: -u and --me are mutually exclusive by
    // construction (the User enum holds exactly one variant).
    // Compile-time rejection: filter can be set only once (no transition or
    // accessor exists on Branches<Filter, U>).
    // Compile-time rejection: user/filter_me can be set only once (no
    // transition or accessor exists on Branches<F, User>).
    // Compile-time rejection: --user-case-insensitive is passed as an
    // argument to user(), never settable on its own.
    // Each filter can be set at most once; limit, display_time, and the
    // other filter stay available afterwards.

    #[test]
    fn without_options() {
        let branches = Branches::new("p4", GlobalOpts::new());
        assert_eq!(args_of(&branches.setup_command("p4")), ["branches"]);
    }

    #[test]
    fn limit() {
        let branches = Branches::new("p4", GlobalOpts::new()).limit(5);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["branches", "-m", "5"]
        );
    }

    #[test]
    fn display_time() {
        let branches = Branches::new("p4", GlobalOpts::new()).display_time(true);
        assert_eq!(args_of(&branches.setup_command("p4")), ["branches", "-t"]);
    }

    #[test]
    fn display_time_can_be_unset() {
        let branches = Branches::new("p4", GlobalOpts::new()).display_time(false);
        assert_eq!(args_of(&branches.setup_command("p4")), ["branches"]);
    }

    #[test]
    fn filter_case_sensitive() {
        let branches = Branches::new("p4", GlobalOpts::new()).filter("svr-dev-rel*", false);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["branches", "-e", "svr-dev-rel*"]
        );
    }

    #[test]
    fn filter_case_insensitive() {
        let branches = Branches::new("p4", GlobalOpts::new()).filter("SVR-DEV-REL*", true);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["branches", "-E", "SVR-DEV-REL*"]
        );
    }

    #[test]
    fn user() {
        let branches = Branches::new("p4", GlobalOpts::new()).user("*bert", false);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["branches", "-u", "*bert"]
        );
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn filter_me() {
        let branches = Branches::new("p4", GlobalOpts::new()).filter_me();
        assert_eq!(args_of(&branches.setup_command("p4")), ["branches", "--me"]);
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn user_case_insensitive() {
        let branches = Branches::new("p4", GlobalOpts::new()).user("*bert", true);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["branches", "-u", "*bert", "--user-case-insensitive"]
        );
    }

    #[test]
    fn filter_transitions_preserve_owner_filter() {
        let branches = Branches::new("p4", GlobalOpts::new())
            .user("*bert", false)
            .filter("svr-dev-rel*", false);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["branches", "-e", "svr-dev-rel*", "-u", "*bert"]
        );

        #[cfg(not(feature = "lt2016_1"))]
        {
            let branches = Branches::new("p4", GlobalOpts::new())
                .filter("SVR*", true)
                .filter_me();
            assert_eq!(
                args_of(&branches.setup_command("p4")),
                ["branches", "-E", "SVR*", "--me"]
            );
        }
    }

    #[test]
    fn setters_update_limit_and_display_time() {
        let mut branches = Branches::new("p4", GlobalOpts::new()).filter("svr-dev*", false);
        assert_eq!(branches.get_limit(), None);
        branches.set_limit(10);
        assert_eq!(branches.get_limit(), Some(10));
        assert!(!branches.get_display_time());
        branches.set_display_time(true);
        assert!(branches.get_display_time());
    }

    #[test]
    fn all_options_order() {
        let branches = Branches::new("p4", GlobalOpts::new())
            .filter("svr-dev-rel*", false)
            .limit(5)
            .display_time(true)
            .user("*bert", false);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            [
                "branches",
                "-e",
                "svr-dev-rel*",
                "-m",
                "5",
                "-t",
                "-u",
                "*bert"
            ]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let branches = Branches::new("p4", GlobalOpts::new().port("localhost:1666")).limit(1);
        assert_eq!(
            args_of(&branches.setup_command("p4")),
            ["-p", "localhost:1666", "branches", "-m", "1"]
        );
    }
}
