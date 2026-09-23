use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Full description output of `p4 filelog` (`-l`): list long output, with
/// the full text of each changelist description.
///
/// Entered with [`FileLog::full_description`].
#[derive(Debug, Clone, Copy, Default)]
pub struct FullDescription;

impl ExclusiveOption for FullDescription {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-l");
    }
}

/// Truncated description output of `p4 filelog` (`-L`): list long output,
/// with the full text of each changelist description truncated at 250
/// characters.
///
/// Entered with [`FileLog::truncated_description`].
#[derive(Debug, Clone, Copy, Default)]
pub struct TruncatedDescription;

impl ExclusiveOption for TruncatedDescription {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-L");
    }
}

/// Content history mode of `p4 filelog` (`-h`): display file content
/// history instead of file name history.
///
/// This is the only state in which the `-p` option
/// ([`skip_promoted_tasks`](FileLog::get_skip_promoted_tasks)) is
/// meaningful. Entered with [`FileLog::content_history`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayContentHistory {
    skip_promoted_tasks: bool,
}

impl ExclusiveOption for DisplayContentHistory {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-h");

        if self.skip_promoted_tasks {
            command.arg("-p");
        }
    }
}

///
/// Print detailed information about the revisions of files.
///
/// The `L` type parameter tracks the changelist description output at
/// compile time: [`Self::full_description`] transitions to the
/// [`FullDescription`] state and [`Self::truncated_description`]
/// transitions to the [`TruncatedDescription`] state. The `H` type
/// parameter tracks whether file content history is displayed:
/// [`Self::content_history`] transitions to the [`DisplayContentHistory`]
/// state.
#[derive(Debug, Clone, Default)]
pub struct FileLog<L = Unselected, H = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    changelist: Option<String>,

    content_history: H,

    follow_branches: bool,

    long_output: L,

    limit: Option<u64>,

    ignore_non_contributory: bool,

    include_time: bool,
}

impl FileLog<Unselected, Unselected> {
    /// Creates a new `p4 filelog` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            changelist: None,
            content_history: Unselected,
            follow_branches: false,
            long_output: Unselected,
            limit: None,
            ignore_non_contributory: false,
            include_time: false,
        }
    }
}

impl<H: ExclusiveOption> FileLog<Unselected, H> {
    /// # Description
    ///
    /// -l
    ///
    /// List long output, with the full text of each changelist description.
    ///
    /// Transitions this command to the [`FullDescription`] state.
    pub fn full_description(self) -> FileLog<FullDescription, H> {
        FileLog {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            content_history: self.content_history,
            follow_branches: self.follow_branches,
            long_output: FullDescription,
            limit: self.limit,
            ignore_non_contributory: self.ignore_non_contributory,
            include_time: self.include_time,
        }
    }

    /// # Description
    ///
    /// -L
    ///
    /// List long output, with the full text of each changelist description
    /// truncated at 250 characters.
    ///
    /// Transitions this command to the [`TruncatedDescription`] state.
    pub fn truncated_description(self) -> FileLog<TruncatedDescription, H> {
        FileLog {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            content_history: self.content_history,
            follow_branches: self.follow_branches,
            long_output: TruncatedDescription,
            limit: self.limit,
            ignore_non_contributory: self.ignore_non_contributory,
            include_time: self.include_time,
        }
    }
}

impl<L: ExclusiveOption> FileLog<L, Unselected> {
    /// # Description
    ///
    /// -h
    ///
    /// Display file content history instead of file name history.
    ///
    /// Transitions this command to the [`DisplayContentHistory`] state,
    /// which unlocks the `-p` option.
    pub fn content_history(self) -> FileLog<L, DisplayContentHistory> {
        FileLog {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            content_history: DisplayContentHistory {
                skip_promoted_tasks: false,
            },
            follow_branches: self.follow_branches,
            long_output: self.long_output,
            limit: self.limit,
            ignore_non_contributory: self.ignore_non_contributory,
            include_time: self.include_time,
        }
    }
}

impl<L: ExclusiveOption, H: ExclusiveOption, S, I> ParameterizedSpawn<(S,)> for FileLog<L, H>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 filelog` for the given files as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    ///
    /// At least one file or file pattern must be provided.
    fn spawn_with(&mut self, (files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<L: ExclusiveOption, H: ExclusiveOption> FileLog<L, H> {
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
    /// `-c change`
    ///
    /// Display only files submitted at the specified changelist number.
    pub fn get_changelist(&self) -> Option<&String> {
        self.changelist.as_ref()
    }

    /// # Description
    ///
    /// `-c change`
    ///
    /// Display only files submitted at the specified changelist number.
    pub fn set_changelist(&mut self, v: impl Into<String>) -> &mut Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-c change`
    ///
    /// Display only files submitted at the specified changelist number.
    pub fn changelist(mut self, v: impl Into<String>) -> Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Follow file history across branches.
    pub fn get_follow_branches(&self) -> bool {
        self.follow_branches
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Follow file history across branches.
    pub fn set_follow_branches(&mut self, v: bool) -> &mut Self {
        self.follow_branches = v;
        self
    }

    /// # Description
    ///
    /// `-i`
    ///
    /// Follow file history across branches.
    pub fn follow_branches(mut self, v: bool) -> Self {
        self.follow_branches = v;
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the first `max` changes per file output.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the first `max` changes per file output.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the first `max` changes per file output.
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Display a shortened form of output by ignoring non-contributory
    /// integrations.
    pub fn get_ignore_non_contributory(&self) -> bool {
        self.ignore_non_contributory
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Display a shortened form of output by ignoring non-contributory
    /// integrations.
    pub fn set_ignore_non_contributory(&mut self, v: bool) -> &mut Self {
        self.ignore_non_contributory = v;
        self
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Display a shortened form of output by ignoring non-contributory
    /// integrations.
    pub fn ignore_non_contributory(mut self, v: bool) -> Self {
        self.ignore_non_contributory = v;
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date.
    pub fn get_include_time(&self) -> bool {
        self.include_time
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date.
    pub fn set_include_time(&mut self, v: bool) -> &mut Self {
        self.include_time = v;
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date.
    pub fn include_time(mut self, v: bool) -> Self {
        self.include_time = v;
        self
    }
}

impl<L: ExclusiveOption> FileLog<L, DisplayContentHistory> {
    /// # Description
    ///
    /// -p
    ///
    /// When used with the `-h` option, do not follow content of promoted task
    /// streams.
    pub fn get_skip_promoted_tasks(&self) -> bool {
        self.content_history.skip_promoted_tasks
    }

    /// # Description
    ///
    /// -p
    ///
    /// When used with the `-h` option, do not follow content of promoted task
    /// streams.
    pub fn set_skip_promoted_tasks(&mut self, v: bool) -> &mut Self {
        self.content_history.skip_promoted_tasks = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    /// When used with the `-h` option, do not follow content of promoted task
    /// streams.
    pub fn skip_promoted_tasks(mut self, v: bool) -> Self {
        self.content_history.skip_promoted_tasks = v;
        self
    }
}

impl<L: ExclusiveOption, H: ExclusiveOption> SubCommand for FileLog<L, H> {
    fn name(&self) -> &str {
        "filelog"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(changelist) = &self.changelist {
            command.arg("-c").arg(changelist);
        }

        self.content_history.inject_args(command);

        if self.follow_branches {
            command.arg("-i");
        }

        self.long_output.inject_args(command);

        if let Some(max) = self.limit {
            command.arg("-m").arg(max.to_string());
        }

        if self.ignore_non_contributory {
            command.arg("-s");
        }

        if self.include_time {
            command.arg("-t");
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
    fn with_files() {
        let filelog = FileLog::new("p4", GlobalOpts::default());
        let mut cmd = filelog.setup_command("p4");
        cmd.arg("//depot/project/...");
        assert_eq!(args_of(&cmd), vec!["filelog", "//depot/project/..."]);
    }

    #[test]
    fn changelist() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).changelist("100");
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-c", "100"]);
    }

    #[test]
    fn content_history() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).content_history();
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-h"]);
    }

    #[test]
    fn follow_branches() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).follow_branches(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-i"]);
    }

    #[test]
    fn full_description() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).full_description();
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-l"]);
    }

    #[test]
    fn truncated_description() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).truncated_description();
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-L"]);
    }

    #[test]
    fn limit() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).limit(5);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-m", "5"]);
    }

    #[test]
    fn skip_promoted_tasks() {
        let filelog = FileLog::new("p4", GlobalOpts::default())
            .content_history()
            .skip_promoted_tasks(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-h", "-p"]);
    }

    #[test]
    fn skip_promoted_tasks_accessors() {
        let mut filelog = FileLog::new("p4", GlobalOpts::default())
            .full_description()
            .content_history();
        filelog.set_skip_promoted_tasks(true);

        assert!(filelog.get_skip_promoted_tasks());
        assert_eq!(
            args_of(&filelog.setup_command("p4")),
            ["filelog", "-h", "-p", "-l"]
        );
    }

    #[test]
    fn ignore_non_contributory() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).ignore_non_contributory(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-s"]);
    }

    #[test]
    fn include_time() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).include_time(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-t"]);
    }

    #[test]
    fn all_options_order() {
        let filelog = FileLog::new("p4", GlobalOpts::default())
            .changelist("100")
            .content_history()
            .follow_branches(true)
            .full_description()
            .limit(5)
            .skip_promoted_tasks(true)
            .ignore_non_contributory(true)
            .include_time(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(
            args_of(&cmd),
            vec![
                "filelog", "-c", "100", "-h", "-p", "-i", "-l", "-m", "5", "-s", "-t",
            ]
        );
    }
}
