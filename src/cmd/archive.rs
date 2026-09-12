use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};

use crate::cmd::SubCommand;
use crate::global::GlobalOpts;

#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] archive [-n -h -p -q -t] -D depot file[revRange] ...`: archive obsolete revisions to an archive depot."
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] archive [-h -n -p -q -t] -D depot file[revRange] ...`: archive obsolete revisions to an archive depot."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2018_1")),
    doc = "`p4 [g-opts] archive [-h -n -p -q -t] -D depot File Spec[revSpec]`: archive obsolete revisions to an archive depot."
)]
#[cfg_attr(
    not(feature = "lt2019_1"),
    doc = "`p4 [g-opts] archive [-n -h -p -q -t -z] -D depot File Spec[revSpec]`: archive obsolete revisions to an archive depot."
)]
#[derive(Debug, Clone, Default)]
pub struct Archive {
    bin: PathBuf,

    global_opts: GlobalOpts,

    preview: bool,

    skip_head_revisions: bool,

    purge: bool,

    quiet_mode: bool,

    archive_text_files: bool,

    depot: Option<String>,

    #[cfg(not(feature = "lt2019_1"))]
    include_lazy_copies: bool,
}

impl SubCommand for Archive {
    fn name(&self) -> &str {
        "archive"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.skip_head_revisions {
            command.arg("-h");
        }

        if self.preview {
            command.arg("-n");
        }

        if self.purge {
            command.arg("-p");
        }

        if self.quiet_mode {
            command.arg("-q");
        }

        if self.archive_text_files {
            command.arg("-t");
        }

        #[cfg(not(feature = "lt2019_1"))]
        if self.include_lazy_copies {
            command.arg("-z");
        }

        if let Some(depot) = self.depot.as_ref() {
            command.arg("-D").arg(depot);
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl Archive {
    /// Creates a new `p4 archive` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Spawns `p4 archive` for the given file specs as a child process.
    ///
    /// The child process inherits the standard input, output, and error
    /// streams of the current process, and runs asynchronously; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 archive` for the given file specs to completion and captures
    /// its output.
    ///
    /// Unlike [`Self::spawn`], this method blocks until the command exits and
    /// collects the standard output and error into the returned [`Output`].
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
    /// -D depot
    ///
    /// Specify an archive depot to which files are to be archived.
    pub fn get_depot(&self) -> Option<&String> {
        self.depot.as_ref()
    }

    /// # Description
    ///
    /// -D depot
    ///
    /// Specify an archive depot to which files are to be archived.
    pub fn set_depot(&mut self, v: impl Into<String>) -> &mut Self {
        self.depot = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -D depot
    ///
    /// Specify an archive depot to which files are to be archived.
    pub fn depot(mut self, v: impl Into<String>) -> Self {
        self.depot = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -h
    ///
    /// Do not archive head revisions.
    pub fn get_skip_head_revisions(&self) -> bool {
        self.skip_head_revisions
    }

    /// # Description
    ///
    /// -h
    ///
    /// Do not archive head revisions.
    pub fn set_skip_head_revisions(&mut self, v: bool) -> &mut Self {
        self.skip_head_revisions = v;
        self
    }

    /// # Description
    ///
    /// -h
    ///
    /// Do not archive head revisions.
    pub fn skip_head_revisions(mut self, v: bool) -> Self {
        self.skip_head_revisions = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Do not archive revisions; report on which revisions would have been archived."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Do not archive revisions. Instead, report on which revisions would have been",
        doc = "archived."
    )]
    pub fn get_preview(&self) -> bool {
        self.preview
    }

    /// # Description
    ///
    /// -n
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Do not archive revisions; report on which revisions would have been archived."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Do not archive revisions. Instead, report on which revisions would have been",
        doc = "archived."
    )]
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Do not archive revisions; report on which revisions would have been archived."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Do not archive revisions. Instead, report on which revisions would have been",
        doc = "archived."
    )]
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    #[cfg_attr(
        feature = "lt2019_1",
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "(The action for affected revisions is set to `purge` on completion. File",
        doc = "contents are no longer accessible from `p4 restore`.)"
    )]
    #[cfg_attr(
        all(feature = "lt2019_2", not(feature = "lt2019_1")),
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "The action for affected revisions is set to `purge` on completion.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2019_2")),
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "The action for affected revisions is set to `purge` on completion.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "**Tip:** If you want to retain the metadata of purged files, do one of the",
        doc = "following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "For the specified files in the archive depot, remove the physical file",
        doc = "revisions but retain the revision history with the latest action being set",
        doc = "to `purge`.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "**Tip:** If you want to retain the metadata of purged files, do one of the",
        doc = "following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "For the specified files in the archive depot, remove the physical file",
        doc = "revisions but retain the revision history with the latest action being set",
        doc = "to `purge`. File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "If you want to retain the metadata of purged files, do one of the following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    pub fn get_purge(&self) -> bool {
        self.purge
    }

    /// # Description
    ///
    /// -p
    ///
    #[cfg_attr(
        feature = "lt2019_1",
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "(The action for affected revisions is set to `purge` on completion. File",
        doc = "contents are no longer accessible from `p4 restore`.)"
    )]
    #[cfg_attr(
        all(feature = "lt2019_2", not(feature = "lt2019_1")),
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "The action for affected revisions is set to `purge` on completion.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2019_2")),
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "The action for affected revisions is set to `purge` on completion.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "**Tip:** If you want to retain the metadata of purged files, do one of the",
        doc = "following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "For the specified files in the archive depot, remove the physical file",
        doc = "revisions but retain the revision history with the latest action being set",
        doc = "to `purge`.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "**Tip:** If you want to retain the metadata of purged files, do one of the",
        doc = "following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "For the specified files in the archive depot, remove the physical file",
        doc = "revisions but retain the revision history with the latest action being set",
        doc = "to `purge`. File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "If you want to retain the metadata of purged files, do one of the following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    pub fn set_purge(&mut self, v: bool) -> &mut Self {
        self.purge = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    #[cfg_attr(
        feature = "lt2019_1",
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "(The action for affected revisions is set to `purge` on completion. File",
        doc = "contents are no longer accessible from `p4 restore`.)"
    )]
    #[cfg_attr(
        all(feature = "lt2019_2", not(feature = "lt2019_1")),
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "The action for affected revisions is set to `purge` on completion.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2019_2")),
        doc = "Purge any archives of the specified files named in the archive depot.",
        doc = "The action for affected revisions is set to `purge` on completion.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "**Tip:** If you want to retain the metadata of purged files, do one of the",
        doc = "following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "For the specified files in the archive depot, remove the physical file",
        doc = "revisions but retain the revision history with the latest action being set",
        doc = "to `purge`.",
        doc = "",
        doc = "**Warning:** File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "**Tip:** If you want to retain the metadata of purged files, do one of the",
        doc = "following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "For the specified files in the archive depot, remove the physical file",
        doc = "revisions but retain the revision history with the latest action being set",
        doc = "to `purge`. File contents are no longer accessible from `p4 restore`.",
        doc = "",
        doc = "If you want to retain the metadata of purged files, do one of the following:",
        doc = "",
        doc = "- use the `-p` option of `p4 obliterate` (recommended)",
        doc = "- perform the two-step sequence of `p4 archive` followed by",
        doc = "  `p4 archive -p` (time-consuming)"
    )]
    pub fn purge(mut self, v: bool) -> Self {
        self.purge = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress messages about skipped revisions."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses messages about skipped revisions."
    )]
    pub fn get_quiet_mode(&self) -> bool {
        self.quiet_mode
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress messages about skipped revisions."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses messages about skipped revisions."
    )]
    pub fn set_quiet_mode(&mut self, v: bool) -> &mut Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress messages about skipped revisions."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses messages about skipped revisions."
    )]
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Archive text files (or other revisions stored in delta format, such as
    /// files of type `binary+D`)
    pub fn get_archive_text_files(&self) -> bool {
        self.archive_text_files
    }

    /// # Description
    ///
    /// -t
    ///
    /// Archive text files (or other revisions stored in delta format, such as
    /// files of type `binary+D`)
    pub fn set_archive_text_files(&mut self, v: bool) -> &mut Self {
        self.archive_text_files = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Archive text files (or other revisions stored in delta format, such as
    /// files of type `binary+D`)
    pub fn archive_text_files(mut self, v: bool) -> Self {
        self.archive_text_files = v;
        self
    }

    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Can reduce the use of disk space because it includes in the archive any files",
        doc = "that have lazy copies or are lazy copies. (A lazy copy is a reference to the",
        doc = "location of the full file.) With this flag, only Criteria 1 and 2 must be",
        doc = "met. Unless all copies are archived, the original file remains in the depot."
    )]
    #[cfg_attr(
        not(feature = "lt2021_1"),
        doc = "Includes any files that have lazy copies or are lazy copies. See Criteria",
        doc = "with `-z`."
    )]
    #[cfg(not(feature = "lt2019_1"))]
    pub fn get_include_lazy_copies(&self) -> bool {
        self.include_lazy_copies
    }

    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Can reduce the use of disk space because it includes in the archive any files",
        doc = "that have lazy copies or are lazy copies. (A lazy copy is a reference to the",
        doc = "location of the full file.) With this flag, only Criteria 1 and 2 must be",
        doc = "met. Unless all copies are archived, the original file remains in the depot."
    )]
    #[cfg_attr(
        not(feature = "lt2021_1"),
        doc = "Includes any files that have lazy copies or are lazy copies. See Criteria",
        doc = "with `-z`."
    )]
    #[cfg(not(feature = "lt2019_1"))]
    pub fn set_include_lazy_copies(&mut self, v: bool) -> &mut Self {
        self.include_lazy_copies = v;
        self
    }

    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Can reduce the use of disk space because it includes in the archive any files",
        doc = "that have lazy copies or are lazy copies. (A lazy copy is a reference to the",
        doc = "location of the full file.) With this flag, only Criteria 1 and 2 must be",
        doc = "met. Unless all copies are archived, the original file remains in the depot."
    )]
    #[cfg_attr(
        not(feature = "lt2021_1"),
        doc = "Includes any files that have lazy copies or are lazy copies. See Criteria",
        doc = "with `-z`."
    )]
    #[cfg(not(feature = "lt2019_1"))]
    pub fn include_lazy_copies(mut self, v: bool) -> Self {
        self.include_lazy_copies = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 archive` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let archive = Archive::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&archive.setup_command("p4")), ["archive"]);
    }

    #[test]
    fn all_local_options() {
        let mut archive = Archive::new("p4", GlobalOpts::new());
        archive
            .set_skip_head_revisions(true)
            .set_preview(true)
            .set_purge(true)
            .set_quiet_mode(true)
            .set_archive_text_files(true)
            .set_depot("archives");
        #[cfg(not(feature = "lt2019_1"))]
        archive.set_include_lazy_copies(true);

        #[cfg_attr(feature = "lt2019_1", allow(unused_mut))]
        let mut expected = vec!["archive", "-h", "-n", "-p", "-q", "-t"];
        #[cfg(not(feature = "lt2019_1"))]
        expected.push("-z");
        expected.extend(["-D", "archives"]);

        assert_eq!(args_of(&archive.setup_command("p4")), expected);
    }

    #[cfg(not(feature = "lt2019_1"))]
    #[test]
    fn lazy_copies_flag() {
        let archive = Archive::new("p4", GlobalOpts::new())
            .depot("archives")
            .include_lazy_copies(true);

        assert!(archive.get_include_lazy_copies());
        assert_eq!(
            args_of(&archive.setup_command("p4")),
            ["archive", "-z", "-D", "archives"]
        );
    }

    #[test]
    fn builder_style_with_str_and_global_opts() {
        let archive = Archive::new("p4", GlobalOpts::new().port("localhost:1666"))
            .depot("archives")
            .preview(true);

        assert_eq!(archive.get_depot(), Some(&"archives".to_string()));
        assert_eq!(
            args_of(&archive.setup_command("p4")),
            ["-p", "localhost:1666", "archive", "-n", "-D", "archives"]
        );
    }
}
