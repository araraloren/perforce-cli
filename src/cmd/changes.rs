use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Output, Stdio},
};

use crate::{
    cmd::{LongOutput, SubCommand},
    global::GlobalOpts,
};

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
#[derive(Debug, Clone, Default)]
pub struct Changes {
    bin: PathBuf,

    global_opts: GlobalOpts,

    client: Option<String>,

    #[cfg(not(feature = "lt2015_2"))]
    min_change_list: Option<String>,

    restricted: bool,

    integrated: bool,

    long_output: Option<LongOutput>,

    max: Option<u32>,

    #[cfg(not(feature = "lt2017_2"))]
    reverse: bool,

    status: Option<Status>,

    time: bool,

    user: Option<User>,

    #[cfg(not(feature = "lt2022_2"))]
    stream: Option<bool>,
}

impl Changes {
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
    /// List only changes made from the named client workspace.
    pub fn get_client(&self) -> Option<&String> {
        self.client.as_ref()
    }

    /// # Description
    ///
    /// `-c client`
    ///
    /// List only changes made from the named client workspace.
    pub fn set_client(&mut self, v: impl Into<String>) -> &mut Self {
        self.client = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-c client`
    ///
    /// List only changes made from the named client workspace.
    pub fn client(mut self, v: impl Into<String>) -> Self {
        self.client = Some(v.into());
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
    pub fn get_restricted(&self) -> bool {
        self.restricted
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// View restricted changes (requires admin permission).
    pub fn set_restricted(&mut self, v: bool) -> &mut Self {
        self.restricted = v;
        self
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// View restricted changes (requires admin permission).
    pub fn restricted(mut self, v: bool) -> Self {
        self.restricted = v;
        self
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Include changelists that affected files that were integrated with the
    /// specified files.
    pub fn get_integrated(&self) -> bool {
        self.integrated
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Include changelists that affected files that were integrated with the
    /// specified files.
    pub fn set_integrated(&mut self, v: bool) -> &mut Self {
        self.integrated = v;
        self
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Include changelists that affected files that were integrated with the
    /// specified files.
    pub fn integrated(mut self, v: bool) -> Self {
        self.integrated = v;
        self
    }

    /// # Description
    ///
    /// `-l` / `-L`
    ///
    /// Control how much of each changelist description is shown. By default
    /// only the first 31 characters are shown. [`LongOutput::Default`] shows
    /// the full text (`-l`), while [`LongOutput::Truncated`] truncates at 250
    /// characters (`-L`).
    pub fn get_long_output(&self) -> Option<LongOutput> {
        self.long_output
    }

    /// # Description
    ///
    /// `-l`
    ///
    /// List long output, with the full text of each changelist description.
    pub fn set_long_output_full(&mut self) -> &mut Self {
        self.long_output = Some(LongOutput::Default);
        self
    }

    /// # Description
    ///
    /// `-l`
    ///
    /// List long output, with the full text of each changelist description.
    pub fn long_output_full(mut self) -> Self {
        self.long_output = Some(LongOutput::Default);
        self
    }

    /// # Description
    ///
    /// `-L`
    ///
    /// List long output, with the full text of each changelist description
    /// truncated at 250 characters.
    pub fn set_long_output_truncated(&mut self) -> &mut Self {
        self.long_output = Some(LongOutput::Truncated);
        self
    }

    /// # Description
    ///
    /// `-L`
    ///
    /// List long output, with the full text of each changelist description
    /// truncated at 250 characters.
    pub fn long_output_truncated(mut self) -> Self {
        self.long_output = Some(LongOutput::Truncated);
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the highest numbered `max` changes.
    pub fn get_max(&self) -> Option<u32> {
        self.max
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the highest numbered `max` changes.
    pub fn set_max(&mut self, v: u32) -> &mut Self {
        self.max = Some(v);
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the highest numbered `max` changes.
    pub fn max(mut self, v: u32) -> Self {
        self.max = Some(v);
        self
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reverse the order of the list, earliest first instead of most recent
    /// first.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn get_reverse(&self) -> bool {
        self.reverse
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reverse the order of the list, earliest first instead of most recent
    /// first.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn set_reverse(&mut self, v: bool) -> &mut Self {
        self.reverse = v;
        self
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reverse the order of the list, earliest first instead of most recent
    /// first.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = v;
        self
    }

    /// # Description
    ///
    /// `-s status`
    ///
    /// Limit the list to the changelists with the specified status:
    /// `pending`, `submitted`, or `shelved`.
    pub fn get_status(&self) -> Option<Status> {
        self.status
    }

    /// # Description
    ///
    /// `-s status`
    ///
    /// Limit the list to the changelists with the specified status:
    /// `pending`, `submitted`, or `shelved`.
    pub fn set_status(&mut self, v: Status) -> &mut Self {
        self.status = Some(v);
        self
    }

    /// # Description
    ///
    /// `-s status`
    ///
    /// Limit the list to the changelists with the specified status:
    /// `pending`, `submitted`, or `shelved`.
    pub fn status(mut self, v: Status) -> Self {
        self.status = Some(v);
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date of each change.
    pub fn get_time(&self) -> bool {
        self.time
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date of each change.
    pub fn set_time(&mut self, v: bool) -> &mut Self {
        self.time = v;
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date of each change.
    pub fn time(mut self, v: bool) -> Self {
        self.time = v;
        self
    }

    /// # Description
    ///
    /// `-u user` / `--me`
    ///
    /// List only changes made from the named user, or, with
    /// [`User::Me`], the current user (equivalent to `-u $P4USER`).
    pub fn get_user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    /// # Description
    ///
    /// `-u user`
    ///
    /// List only changes made from the named user.
    pub fn set_user_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.user = Some(User::User(v.into()));
        self
    }

    /// # Description
    ///
    /// `-u user`
    ///
    /// List only changes made from the named user.
    pub fn user_name(mut self, v: impl Into<String>) -> Self {
        self.user = Some(User::User(v.into()));
        self
    }

    /// # Description
    ///
    /// `--me`
    ///
    /// Equivalent to `-u $P4USER`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_me(&mut self) -> &mut Self {
        self.user = Some(User::Me);
        self
    }

    /// # Description
    ///
    /// `--me`
    ///
    /// Equivalent to `-u $P4USER`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn me(mut self) -> Self {
        self.user = Some(User::Me);
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

impl SubCommand for Changes {
    fn name(&self) -> &str {
        "changes"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if let Some(ref client) = self.client {
            command.arg("-c").arg(client);
        }
        #[cfg(not(feature = "lt2015_2"))]
        if let Some(ref min_change) = self.min_change_list {
            command.arg("-e").arg(min_change);
        }
        if self.restricted {
            command.arg("-f");
        }
        if self.integrated {
            command.arg("-i");
        }
        if let Some(long_output) = self.long_output {
            command.arg(long_output.as_str());
        }
        if let Some(max) = self.max {
            command.arg("-m").arg(max.to_string());
        }
        #[cfg(not(feature = "lt2017_2"))]
        if self.reverse {
            command.arg("-r");
        }
        if let Some(status) = self.status {
            command.arg("-s").arg(status.as_str());
        }
        if self.time {
            command.arg("-t");
        }
        if let Some(ref user) = self.user {
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

    #[cfg(not(feature = "lt2015_2"))]
    #[test]
    fn min_change_list() {
        let changes = Changes::new("p4", GlobalOpts::default()).min_change_list("800");
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-e", "800"]);
    }

    #[test]
    fn restricted() {
        let changes = Changes::new("p4", GlobalOpts::default()).restricted(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-f"]);
    }

    #[test]
    fn integrated() {
        let changes = Changes::new("p4", GlobalOpts::default()).integrated(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-i"]);
    }

    #[test]
    fn long_output_default() {
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
    fn max() {
        let changes = Changes::new("p4", GlobalOpts::default()).max(5);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-m", "5"]);
    }

    #[cfg(not(feature = "lt2017_2"))]
    #[test]
    fn reverse() {
        let changes = Changes::new("p4", GlobalOpts::default()).reverse(true);
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
    fn time() {
        let changes = Changes::new("p4", GlobalOpts::default()).time(true);
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-t"]);
    }

    #[test]
    fn user_name() {
        let changes = Changes::new("p4", GlobalOpts::default()).user_name("edk");
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "-u", "edk"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn user_me() {
        let changes = Changes::new("p4", GlobalOpts::default()).me();
        let cmd = changes.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["changes", "--me"]);
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
            .restricted(true)
            .integrated(true)
            .long_output_full()
            .max(5)
            .status(Status::Submitted)
            .time(true)
            .user_name("edk");
        #[cfg(not(feature = "lt2015_2"))]
        let changes = changes.min_change_list("800");
        #[cfg(not(feature = "lt2017_2"))]
        let changes = changes.reverse(true);
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
}
