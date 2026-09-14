use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Output, Stdio},
};

use crate::{
    cmd::{ExclusiveOption, SubCommand, Unselected},
    global::GlobalOpts,
};

/// `-l`: full text of each changelist description.
#[derive(Debug, Clone, Copy, Default)]
pub struct FullDescription;

impl ExclusiveOption for FullDescription {
    fn inject_args(&self, command: &mut std::process::Command) {
        command.arg("-l");
    }
}

/// `-L`: full text truncated at 250 characters.
#[derive(Debug, Clone, Copy, Default)]
pub struct TruncatedDescription;

impl ExclusiveOption for TruncatedDescription {
    fn inject_args(&self, command: &mut std::process::Command) {
        command.arg("-L");
    }
}

/// User filter for the `-u` / `--me` options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum User {
    /// List only changes made from the named user (`-u user`).
    User(String),
    /// Equivalent to `-u $P4USER` (`--me`).
    #[cfg(not(feature = "lt2016_1"))]
    Me,
}

/// Status of a changelist as accepted by the `-s` option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Pending changelists.
    Pending,
    /// Submitted changelists.
    Submitted,
    /// Shelved changelists.
    Shelved,
}

impl Status {
    /// Returns the command-line representation of this status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Submitted => "submitted",
            Status::Shelved => "shelved",
        }
    }
}

#[cfg_attr(
    feature = "lt2015_2",
    doc = "`p4 [g-opts] changes [-i -t -l -L -f] [-c client] [-m max] [-s status] [-u user] [file[RevRange] ...]`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_2")),
    doc = "`p4 [g-opts] changes [-i -t -l -L -f] [-c client] [ -e changelist#][-m max] [-s status] [-u user][file[RevRange] ...]`"
)]
#[cfg_attr(
    all(feature = "lt2017_2", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] changes [-i -t -l -L -f] [-c client] [ -e changelist#][-m max] [-s status] [-u user | --me][file[RevRange] ...]`"
)]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2017_2")),
    doc = "`p4 [g-opts] changes [-i -t -l -L -f] [-c client] [ -e changelist#][-m max] [-r] [-s status] [-u user | --me] [file[RevRange] ...]`"
)]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "`p4 [g-opts] changes [-i -t -l -L -f] [-c client] [ -e changelist#][-m max] [-r] [-s status] [-u user | --me] [file[RevRange] ...] [--stream | --nostream]`"
)]
///
/// List submitted and pending changelists.
///
/// The command `p4 changelists` is an alias for `p4 changes`.
///
/// The `L` type parameter tracks the `-l` / `-L` long output mode at
/// compile time; see [`FullDescription`], [`TruncatedDescription`],
/// [`Self::long_output_full`], and [`Self::long_output_truncated`].
#[derive(Debug, Clone, Default)]
pub struct Changes<L = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    #[cfg(not(feature = "lt2015_2"))]
    min_change_list: Option<String>,

    include_restricted: bool,

    include_integrated: bool,

    include_time: bool,

    long_output: L,

    limit: Option<u64>,

    #[cfg(not(feature = "lt2017_2"))]
    reverse_order: bool,

    filter_status: Option<Status>,

    filter_users: Option<Vec<User>>,

    filter_clients: Option<Vec<String>>,

    #[cfg(not(feature = "lt2025_1"))]
    client_case_insensitive: bool,

    #[cfg(not(feature = "lt2022_2"))]
    stream: Option<bool>,
}

impl Changes<Unselected> {
    /// Creates a new `p4 changes` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Self::default()
        }
    }

    /// # Description
    ///
    /// `-l`
    ///
    /// List long output, with the full text of each changelist description.
    ///
    /// Transitions this command to the [`FullDescription`] state.
    pub fn long_output_full(self) -> Changes<FullDescription> {
        Changes {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2015_2"))]
            min_change_list: self.min_change_list,
            include_restricted: self.include_restricted,
            include_integrated: self.include_integrated,
            include_time: self.include_time,
            long_output: FullDescription,
            limit: self.limit,
            #[cfg(not(feature = "lt2017_2"))]
            reverse_order: self.reverse_order,
            filter_status: self.filter_status,
            filter_users: self.filter_users,
            filter_clients: self.filter_clients,
            #[cfg(not(feature = "lt2025_1"))]
            client_case_insensitive: self.client_case_insensitive,
            #[cfg(not(feature = "lt2022_2"))]
            stream: self.stream,
        }
    }

    /// # Description
    ///
    /// `-L`
    ///
    /// List long output, with the full text of each changelist description
    /// truncated at 250 characters.
    ///
    /// Transitions this command to the [`TruncatedDescription`] state.
    pub fn long_output_truncated(self) -> Changes<TruncatedDescription> {
        Changes {
            bin: self.bin,
            global_opts: self.global_opts,
            #[cfg(not(feature = "lt2015_2"))]
            min_change_list: self.min_change_list,
            include_restricted: self.include_restricted,
            include_integrated: self.include_integrated,
            include_time: self.include_time,
            long_output: TruncatedDescription,
            limit: self.limit,
            #[cfg(not(feature = "lt2017_2"))]
            reverse_order: self.reverse_order,
            filter_status: self.filter_status,
            filter_users: self.filter_users,
            filter_clients: self.filter_clients,
            #[cfg(not(feature = "lt2025_1"))]
            client_case_insensitive: self.client_case_insensitive,
            #[cfg(not(feature = "lt2022_2"))]
            stream: self.stream,
        }
    }
}

impl<L: ExclusiveOption> Changes<L> {
    /// Runs `p4 changes` for the given files, inheriting the parent process's
    /// standard streams.
    ///
    /// If files are specified, only changelists that affect those files are
    /// listed. Pass an empty slice to list all changelists.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 changes` for the given files to completion and captures its
    /// output.
    ///
    /// Unlike [`Self::spawn`], this method blocks until the command exits and
    /// collects the standard output and error into the returned [`Output`].
    ///
    /// If files are specified, only changelists that affect those files are
    /// listed. Pass an empty slice to list all changelists.
    pub fn output<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Output, std::io::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
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
    /// `-c client`
    ///
    /// List only changes made from the named client workspace. This option
    /// can be repeated to filter for multiple clients.
    pub fn get_clients(&self) -> Option<&[String]> {
        self.filter_clients.as_deref()
    }

    /// # Description
    ///
    /// `-c client`
    ///
    /// List only changes made from the named client workspace. This option
    /// can be repeated to filter for multiple clients.
    pub fn set_client(&mut self, v: impl Into<String>) -> &mut Self {
        self.filter_clients
            .get_or_insert_with(Vec::new)
            .push(v.into());
        self
    }

    /// # Description
    ///
    /// `-c client`
    ///
    /// List only changes made from the named client workspace. This option
    /// can be repeated to filter for multiple clients.
    pub fn client(mut self, v: impl Into<String>) -> Self {
        self.filter_clients
            .get_or_insert_with(Vec::new)
            .push(v.into());
        self
    }

    /// # Description
    ///
    /// `-e changelist#`
    ///
    /// Display only changes where the changelist number is equal to, or
    /// higher than, the specified changelist number.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn get_min_change_list(&self) -> Option<&String> {
        self.min_change_list.as_ref()
    }

    /// # Description
    ///
    /// `-e changelist#`
    ///
    /// Display only changes where the changelist number is equal to, or
    /// higher than, the specified changelist number.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn set_min_change_list(&mut self, v: impl Into<String>) -> &mut Self {
        self.min_change_list = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-e changelist#`
    ///
    /// Display only changes where the changelist number is equal to, or
    /// higher than, the specified changelist number.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn min_change_list(mut self, v: impl Into<String>) -> Self {
        self.min_change_list = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// View restricted changes (requires admin permission).
    pub fn get_include_restricted(&self) -> bool {
        self.include_restricted
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// View restricted changes (requires admin permission).
    pub fn set_include_restricted(&mut self, v: bool) -> &mut Self {
        self.include_restricted = v;
        self
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// View restricted changes (requires admin permission).
    pub fn include_restricted(mut self, v: bool) -> Self {
        self.include_restricted = v;
        self
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Include changelists that affected files that were integrated with the
    /// specified files.
    pub fn get_include_integrated(&self) -> bool {
        self.include_integrated
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Include changelists that affected files that were integrated with the
    /// specified files.
    pub fn set_include_integrated(&mut self, v: bool) -> &mut Self {
        self.include_integrated = v;
        self
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Include changelists that affected files that were integrated with the
    /// specified files.
    pub fn include_integrated(mut self, v: bool) -> Self {
        self.include_integrated = v;
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the highest numbered `max` changes.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the highest numbered `max` changes.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the highest numbered `max` changes.
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reverse the order of the list, earliest first instead of most recent
    /// first.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn get_reverse_order(&self) -> bool {
        self.reverse_order
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reverse the order of the list, earliest first instead of most recent
    /// first.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn set_reverse_order(&mut self, v: bool) -> &mut Self {
        self.reverse_order = v;
        self
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reverse the order of the list, earliest first instead of most recent
    /// first.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn reverse_order(mut self, v: bool) -> Self {
        self.reverse_order = v;
        self
    }

    /// # Description
    ///
    /// `-s status`
    ///
    /// Limit the list to the changelists with the specified status:
    /// `pending`, `submitted`, or `shelved`.
    pub fn get_status(&self) -> Option<Status> {
        self.filter_status
    }

    /// # Description
    ///
    /// `-s status`
    ///
    /// Limit the list to the changelists with the specified status:
    /// `pending`, `submitted`, or `shelved`.
    pub fn set_status(&mut self, v: Status) -> &mut Self {
        self.filter_status = Some(v);
        self
    }

    /// # Description
    ///
    /// `-s status`
    ///
    /// Limit the list to the changelists with the specified status:
    /// `pending`, `submitted`, or `shelved`.
    pub fn status(mut self, v: Status) -> Self {
        self.filter_status = Some(v);
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date of each change.
    pub fn get_include_time(&self) -> bool {
        self.include_time
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date of each change.
    pub fn set_include_time(&mut self, v: bool) -> &mut Self {
        self.include_time = v;
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date of each change.
    pub fn include_time(mut self, v: bool) -> Self {
        self.include_time = v;
        self
    }

    /// # Description
    ///
    /// `-u user` / `--me`
    ///
    /// List only changes made from the named user, or, with
    /// [`User::Me`], the current user (equivalent to `-u $P4USER`).
    /// This option can be repeated to filter for multiple users.
    pub fn get_users(&self) -> Option<&[User]> {
        self.filter_users.as_deref()
    }

    /// # Description
    ///
    /// `-u user`
    ///
    /// List only changes made from the named user. This option can be
    /// repeated to filter for multiple users.
    pub fn set_user_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.filter_users
            .get_or_insert_with(Vec::new)
            .push(User::User(v.into()));
        self
    }

    /// # Description
    ///
    /// `-u user`
    ///
    /// List only changes made from the named user. This option can be
    /// repeated to filter for multiple users.
    pub fn user_name(mut self, v: impl Into<String>) -> Self {
        self.filter_users
            .get_or_insert_with(Vec::new)
            .push(User::User(v.into()));
        self
    }

    /// # Description
    ///
    /// `--me`
    ///
    /// Equivalent to `-u $P4USER`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_me(&mut self) -> &mut Self {
        self.filter_users
            .get_or_insert_with(Vec::new)
            .push(User::Me);
        self
    }

    /// # Description
    ///
    /// `--me`
    ///
    /// Equivalent to `-u $P4USER`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn me(mut self) -> Self {
        self.filter_users
            .get_or_insert_with(Vec::new)
            .push(User::Me);
        self
    }

    /// # Description
    ///
    /// `--client-case-insensitive`
    ///
    /// Makes the `-c client` search pattern case-insensitive, even on a
    /// case-sensitive server.
    #[cfg(not(feature = "lt2025_1"))]
    pub fn get_client_case_insensitive(&self) -> bool {
        self.client_case_insensitive
    }

    /// # Description
    ///
    /// `--client-case-insensitive`
    ///
    /// Makes the `-c client` search pattern case-insensitive, even on a
    /// case-sensitive server.
    #[cfg(not(feature = "lt2025_1"))]
    pub fn set_client_case_insensitive(&mut self, v: bool) -> &mut Self {
        self.client_case_insensitive = v;
        self
    }

    /// # Description
    ///
    /// `--client-case-insensitive`
    ///
    /// Makes the `-c client` search pattern case-insensitive, even on a
    /// case-sensitive server.
    #[cfg(not(feature = "lt2025_1"))]
    pub fn client_case_insensitive(mut self, v: bool) -> Self {
        self.client_case_insensitive = v;
        self
    }

    /// # Description
    ///
    /// `--stream` / `--nostream`
    ///
    /// With `Some(true)`, display only changes that contain a stream spec
    /// (`--stream`). With `Some(false)`, display only changes that do not
    /// contain a stream spec (`--nostream`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_stream(&self) -> Option<bool> {
        self.stream
    }

    /// # Description
    ///
    /// `--stream` / `--nostream`
    ///
    /// With `Some(true)`, display only changes that contain a stream spec
    /// (`--stream`). With `Some(false)`, display only changes that do not
    /// contain a stream spec (`--nostream`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_stream(&mut self, v: Option<bool>) -> &mut Self {
        self.stream = v;
        self
    }

    /// # Description
    ///
    /// `--stream` / `--nostream`
    ///
    /// With `Some(true)`, display only changes that contain a stream spec
    /// (`--stream`). With `Some(false)`, display only changes that do not
    /// contain a stream spec (`--nostream`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn stream(mut self, v: Option<bool>) -> Self {
        self.stream = v;
        self
    }
}

impl<L: ExclusiveOption> SubCommand for Changes<L> {
    fn name(&self) -> &str {
        "changes"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if let Some(ref clients) = self.filter_clients {
            for client in clients {
                command.arg("-c").arg(client);
            }
        }
        #[cfg(not(feature = "lt2025_1"))]
        if self.client_case_insensitive {
            command.arg("--client-case-insensitive");
        }
        #[cfg(not(feature = "lt2015_2"))]
        if let Some(ref min_change) = self.min_change_list {
            command.arg("-e").arg(min_change);
        }
        if self.include_restricted {
            command.arg("-f");
        }
        if self.include_integrated {
            command.arg("-i");
        }
        self.long_output.inject_args(command);
        if let Some(max) = self.limit {
            command.arg("-m").arg(max.to_string());
        }
        #[cfg(not(feature = "lt2017_2"))]
        if self.reverse_order {
            command.arg("-r");
        }
        if let Some(status) = self.filter_status {
            command.arg("-s").arg(status.as_str());
        }
        if self.include_time {
            command.arg("-t");
        }
        if let Some(ref users) = self.filter_users {
            for user in users {
                match user {
                    User::User(name) => {
                        command.arg("-u").arg(name);
                    }
                    #[cfg(not(feature = "lt2016_1"))]
                    User::Me => {
                        command.arg("--me");
                    }
                }
            }
        }
        #[cfg(not(feature = "lt2022_2"))]
        if let Some(stream) = self.stream {
            if stream {
                command.arg("--stream");
            } else {
                command.arg("--nostream");
            }
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    #[test]
    fn without_options() {
        let changes = Changes::new("p4", GlobalOpts::default());
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes"]);
    }

    #[test]
    fn with_files() {
        let changes = Changes::new("p4", GlobalOpts::default());
        let mut cmd = changes.setup_command("p4");
        cmd.arg("//depot/project/...");
        assert_eq!(args_of(&cmd), vec!["changes", "//depot/project/..."]);
    }

    #[test]
    fn client() {
        let changes = Changes::new("p4", GlobalOpts::default()).client("eds_elm");
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-c", "eds_elm"]);
    }

    #[test]
    fn multiple_clients() {
        let changes = Changes::new("p4", GlobalOpts::default())
            .client("eds_elm")
            .client("build_ws");
        let cmd = changes.setup_command("p4");
        assert_eq!(
            args_of(&cmd),
            vec!["changes", "-c", "eds_elm", "-c", "build_ws"]
        );
    }

    #[cfg(not(feature = "lt2015_2"))]
    #[test]
    fn min_change_list() {
        let changes = Changes::new("p4", GlobalOpts::default()).min_change_list("800");
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-e", "800"]);
    }

    #[test]
    fn include_restricted() {
        let changes = Changes::new("p4", GlobalOpts::default()).include_restricted(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-f"]);
    }

    #[test]
    fn include_integrated() {
        let changes = Changes::new("p4", GlobalOpts::default()).include_integrated(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-i"]);
    }

    #[test]
    fn long_output_full() {
        let changes = Changes::new("p4", GlobalOpts::default()).long_output_full();
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-l"]);
    }

    #[test]
    fn long_output_truncated() {
        let changes = Changes::new("p4", GlobalOpts::default()).long_output_truncated();
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-L"]);
    }

    #[test]
    fn limit() {
        let changes = Changes::new("p4", GlobalOpts::default()).limit(5);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-m", "5"]);
    }

    #[cfg(not(feature = "lt2017_2"))]
    #[test]
    fn reverse_order() {
        let changes = Changes::new("p4", GlobalOpts::default()).reverse_order(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-r"]);
    }

    #[test]
    fn status_pending() {
        let changes = Changes::new("p4", GlobalOpts::default()).status(Status::Pending);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-s", "pending"]);
    }

    #[test]
    fn status_submitted() {
        let changes = Changes::new("p4", GlobalOpts::default()).status(Status::Submitted);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-s", "submitted"]);
    }

    #[test]
    fn status_shelved() {
        let changes = Changes::new("p4", GlobalOpts::default()).status(Status::Shelved);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-s", "shelved"]);
    }

    #[test]
    fn include_time() {
        let changes = Changes::new("p4", GlobalOpts::default()).include_time(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-t"]);
    }

    #[test]
    fn user_name() {
        let changes = Changes::new("p4", GlobalOpts::default()).user_name("edk");
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-u", "edk"]);
    }

    #[test]
    fn multiple_users() {
        let changes = Changes::new("p4", GlobalOpts::default())
            .user_name("maria")
            .user_name("edk");
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-u", "maria", "-u", "edk"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn user_me() {
        let changes = Changes::new("p4", GlobalOpts::default()).me();
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "--me"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn multiple_users_with_me() {
        let changes = Changes::new("p4", GlobalOpts::default())
            .user_name("maria")
            .me();
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-u", "maria", "--me"]);
    }

    #[cfg(not(feature = "lt2025_1"))]
    #[test]
    fn client_case_insensitive() {
        let changes = Changes::new("p4", GlobalOpts::default())
            .client("eds_elm")
            .client_case_insensitive(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(
            args_of(&cmd),
            vec!["changes", "-c", "eds_elm", "--client-case-insensitive"]
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn stream_spec() {
        let changes = Changes::new("p4", GlobalOpts::default()).stream(Some(true));
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "--stream"]);
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn no_stream_spec() {
        let changes = Changes::new("p4", GlobalOpts::default()).stream(Some(false));
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "--nostream"]);
    }

    #[test]
    fn all_options_order() {
        let changes = Changes::new("p4", GlobalOpts::default())
            .client("eds_elm")
            .include_restricted(true)
            .include_integrated(true)
            .long_output_full()
            .limit(5)
            .status(Status::Submitted)
            .include_time(true)
            .user_name("edk");
        #[cfg(not(feature = "lt2015_2"))]
        let changes = changes.min_change_list("800");
        #[cfg(not(feature = "lt2017_2"))]
        let changes = changes.reverse_order(true);
        #[cfg(not(feature = "lt2022_2"))]
        let changes = changes.stream(Some(true));
        let cmd = changes.setup_command("p4");
        let mut expected = vec!["changes", "-c", "eds_elm"];
        #[cfg(not(feature = "lt2015_2"))]
        expected.extend(["-e", "800"]);
        expected.extend(["-f", "-i", "-l", "-m", "5"]);
        #[cfg(not(feature = "lt2017_2"))]
        expected.push("-r");
        expected.extend(["-s", "submitted", "-t", "-u", "edk"]);
        #[cfg(not(feature = "lt2022_2"))]
        expected.push("--stream");
        assert_eq!(args_of(&cmd), expected);
    }

    #[test]
    fn long_output_full_preserves_other_options() {
        let changes = Changes::new("p4", GlobalOpts::default())
            .include_restricted(true)
            .include_time(true)
            .long_output_full();
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-f", "-l", "-t"]);
    }
}
