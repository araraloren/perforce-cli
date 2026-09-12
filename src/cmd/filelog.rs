use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Output, Stdio},
};

use crate::{
    cmd::{LongOutput, SubCommand},
    global::GlobalOpts,
};

///
/// Print detailed information about the revisions of files.
#[derive(Debug, Clone, Default)]
pub struct FileLog {
    bin: PathBuf,

    global_opts: GlobalOpts,

    changelist: Option<String>,

    content_history: bool,

    follow_branches: bool,

    long_output: Option<LongOutput>,

    max: Option<u32>,

    skip_promoted: bool,

    shortened: bool,

    time: bool,
}

impl FileLog {
    /// Creates a new `p4 filelog` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Self::default()
        }
    }

    /// Runs `p4 filelog` for the given files, inheriting the parent process's
    /// standard streams.
    ///
    /// At least one file or file pattern must be provided.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 filelog` for the given files to completion and captures its
    /// output.
    ///
    /// Unlike [`Self::spawn`], this method blocks until the command exits and
    /// collects the standard output and error into the returned [`Output`].
    ///
    /// At least one file or file pattern must be provided.
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
    /// `-h`
    ///
    /// Display file content history instead of file name history.
    pub fn get_content_history(&self) -> bool {
        self.content_history
    }

    /// # Description
    ///
    /// `-h`
    ///
    /// Display file content history instead of file name history.
    pub fn set_content_history(&mut self, v: bool) -> &mut Self {
        self.content_history = v;
        self
    }

    /// # Description
    ///
    /// `-h`
    ///
    /// Display file content history instead of file name history.
    pub fn content_history(mut self, v: bool) -> Self {
        self.content_history = v;
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
    /// `-l` / `-L`
    ///
    /// Control how much of each changelist description is shown. By default
    /// only the first 30 characters are shown. [`LongOutput::Default`] shows
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
    /// List only the first `max` changes per file output.
    pub fn get_max(&self) -> Option<u32> {
        self.max
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the first `max` changes per file output.
    pub fn set_max(&mut self, v: u32) -> &mut Self {
        self.max = Some(v);
        self
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// List only the first `max` changes per file output.
    pub fn max(mut self, v: u32) -> Self {
        self.max = Some(v);
        self
    }

    /// # Description
    ///
    /// `-p`
    ///
    /// When used with the `-h` option, do not follow content of promoted task
    /// streams.
    pub fn get_skip_promoted(&self) -> bool {
        self.skip_promoted
    }

    /// # Description
    ///
    /// `-p`
    ///
    /// When used with the `-h` option, do not follow content of promoted task
    /// streams.
    pub fn set_skip_promoted(&mut self, v: bool) -> &mut Self {
        self.skip_promoted = v;
        self
    }

    /// # Description
    ///
    /// `-p`
    ///
    /// When used with the `-h` option, do not follow content of promoted task
    /// streams.
    pub fn skip_promoted(mut self, v: bool) -> Self {
        self.skip_promoted = v;
        self
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Display a shortened form of output by ignoring non-contributory
    /// integrations.
    pub fn get_shortened(&self) -> bool {
        self.shortened
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Display a shortened form of output by ignoring non-contributory
    /// integrations.
    pub fn set_shortened(&mut self, v: bool) -> &mut Self {
        self.shortened = v;
        self
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Display a shortened form of output by ignoring non-contributory
    /// integrations.
    pub fn shortened(mut self, v: bool) -> Self {
        self.shortened = v;
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date.
    pub fn get_time(&self) -> bool {
        self.time
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date.
    pub fn set_time(&mut self, v: bool) -> &mut Self {
        self.time = v;
        self
    }

    /// # Description
    ///
    /// `-t`
    ///
    /// Display the time as well as the date.
    pub fn time(mut self, v: bool) -> Self {
        self.time = v;
        self
    }
}

impl SubCommand for FileLog {
    fn name(&self) -> &str {
        "filelog"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if let Some(ref changelist) = self.changelist {
            command.arg("-c").arg(changelist);
        }
        if self.content_history {
            command.arg("-h");
        }
        if self.follow_branches {
            command.arg("-i");
        }
        if let Some(long_output) = self.long_output {
            command.arg(long_output.as_str());
        }
        if let Some(max) = self.max {
            command.arg("-m").arg(max.to_string());
        }
        if self.skip_promoted {
            command.arg("-p");
        }
        if self.shortened {
            command.arg("-s");
        }
        if self.time {
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
        let filelog = FileLog::new("p4", GlobalOpts::default()).content_history(true);
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
    fn long_output_full() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).long_output_full();
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-l"]);
    }

    #[test]
    fn long_output_truncated() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).long_output_truncated();
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-L"]);
    }

    #[test]
    fn max() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).max(5);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-m", "5"]);
    }

    #[test]
    fn skip_promoted() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).skip_promoted(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-p"]);
    }

    #[test]
    fn shortened() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).shortened(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-s"]);
    }

    #[test]
    fn time() {
        let filelog = FileLog::new("p4", GlobalOpts::default()).time(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["filelog", "-t"]);
    }

    #[test]
    fn all_options_order() {
        let filelog = FileLog::new("p4", GlobalOpts::default())
            .changelist("100")
            .content_history(true)
            .follow_branches(true)
            .long_output_full()
            .max(5)
            .skip_promoted(true)
            .shortened(true)
            .time(true);
        let cmd = filelog.setup_command("p4");
        assert_eq!(
            args_of(&cmd),
            vec![
                "filelog", "-c", "100", "-h", "-i", "-l", "-m", "5", "-p", "-s", "-t",
            ]
        );
    }
}
