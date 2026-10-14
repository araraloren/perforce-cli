use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Stdio},
};

use super::SubCommand;

use crate::{global::GlobalOpts, spawn::ParameterizedSpawn};

/// `p4 [g-opts] clean [-e -a -d -I -l -K -m -n] [file ...]`
///
/// Restore client workspace files to match the state of corresponding
/// depot files. This command is equivalent to `p4 reconcile -w`: files
/// present in the workspace but missing from the depot are deleted;
/// files present in the depot but missing from the workspace are added
/// at the revision synced from the depot; and files modified in the
/// workspace that are not opened for add, edit, delete, or integrate
/// are restored to the last revision synced from the depot.
///
/// Without file arguments, the files in your local working directory
/// are used.
#[derive(Debug, Clone, Default)]
pub struct Clean {
    bin: PathBuf,

    global_opts: GlobalOpts,

    /// -e
    find_edited: bool,

    /// -a
    find_added: bool,

    /// -d
    find_deleted: bool,

    /// -I
    ignore_p4ignore: bool,

    /// -l
    local_file_syntax: bool,

    /// -K
    #[cfg(not(feature = "lt2022_1"))]
    suppress_keyword_expansion: bool,

    /// -m
    #[cfg(not(feature = "lt2018_1"))]
    fast_timestamp_check: bool,

    /// -n
    preview: bool,
}

impl SubCommand for Clean {
    fn name(&self) -> &str {
        "clean"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if self.find_edited {
            command.arg("-e");
        }
        if self.find_added {
            command.arg("-a");
        }
        if self.find_deleted {
            command.arg("-d");
        }
        if self.ignore_p4ignore {
            command.arg("-I");
        }
        if self.local_file_syntax {
            command.arg("-l");
        }
        #[cfg(not(feature = "lt2022_1"))]
        if self.suppress_keyword_expansion {
            command.arg("-K");
        }
        #[cfg(not(feature = "lt2018_1"))]
        if self.fast_timestamp_check {
            command.arg("-m");
        }
        if self.preview {
            command.arg("-n");
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl ParameterizedSpawn<()> for Clean {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 clean` without file arguments as a child process, so the
    /// files in your local working directory are reconciled.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S, I> ParameterizedSpawn<(S,)> for Clean
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 clean` for the given files as a child process.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl Clean {
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

    /// Create a new `p4 clean` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// # Description
    ///
    /// g-opts
    ///
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
    /// -e
    ///
    /// Edited files: Find files in the workspace that have been modified and
    /// restore them to the last file version that has synced from the depot.
    pub fn get_find_edited(&self) -> bool {
        self.find_edited
    }

    /// # Description
    ///
    /// -e
    ///
    /// Edited files: Find files in the workspace that have been modified and
    /// restore them to the last file version that has synced from the depot.
    pub fn set_find_edited(&mut self, v: bool) -> &mut Self {
        self.find_edited = v;
        self
    }

    /// # Description
    ///
    /// -e
    ///
    /// Edited files: Find files in the workspace that have been modified and
    /// restore them to the last file version that has synced from the depot.
    pub fn find_edited(mut self, v: bool) -> Self {
        self.find_edited = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// Added files: Find files in the workspace that have no corresponding
    /// files in the depot and delete them.
    pub fn get_find_added(&self) -> bool {
        self.find_added
    }

    /// # Description
    ///
    /// -a
    ///
    /// Added files: Find files in the workspace that have no corresponding
    /// files in the depot and delete them.
    pub fn set_find_added(&mut self, v: bool) -> &mut Self {
        self.find_added = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// Added files: Find files in the workspace that have no corresponding
    /// files in the depot and delete them.
    pub fn find_added(mut self, v: bool) -> Self {
        self.find_added = v;
        self
    }

    /// # Description
    ///
    /// -d
    ///
    /// Deleted files: Find those files in the depot that do not exist in your
    /// workspace and add them to the workspace.
    pub fn get_find_deleted(&self) -> bool {
        self.find_deleted
    }

    /// # Description
    ///
    /// -d
    ///
    /// Deleted files: Find those files in the depot that do not exist in your
    /// workspace and add them to the workspace.
    pub fn set_find_deleted(&mut self, v: bool) -> &mut Self {
        self.find_deleted = v;
        self
    }

    /// # Description
    ///
    /// -d
    ///
    /// Deleted files: Find those files in the depot that do not exist in your
    /// workspace and add them to the workspace.
    pub fn find_deleted(mut self, v: bool) -> Self {
        self.find_deleted = v;
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Do not perform any ignore checking, which means to ignore any settings
    /// specified by P4IGNORE for added files.
    pub fn get_ignore_p4ignore(&self) -> bool {
        self.ignore_p4ignore
    }

    /// # Description
    ///
    /// -I
    ///
    /// Do not perform any ignore checking, which means to ignore any settings
    /// specified by P4IGNORE for added files.
    pub fn set_ignore_p4ignore(&mut self, v: bool) -> &mut Self {
        self.ignore_p4ignore = v;
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Do not perform any ignore checking, which means to ignore any settings
    /// specified by P4IGNORE for added files.
    pub fn ignore_p4ignore(mut self, v: bool) -> Self {
        self.ignore_p4ignore = v;
        self
    }

    /// # Description
    ///
    /// -l
    ///
    /// Display output in local file syntax with relative paths, similar to
    /// the workspace-centric view of `p4 status`.
    pub fn get_local_file_syntax(&self) -> bool {
        self.local_file_syntax
    }

    /// # Description
    ///
    /// -l
    ///
    /// Display output in local file syntax with relative paths, similar to
    /// the workspace-centric view of `p4 status`.
    pub fn set_local_file_syntax(&mut self, v: bool) -> &mut Self {
        self.local_file_syntax = v;
        self
    }

    /// # Description
    ///
    /// -l
    ///
    /// Display output in local file syntax with relative paths, similar to
    /// the workspace-centric view of `p4 status`.
    pub fn local_file_syntax(mut self, v: bool) -> Self {
        self.local_file_syntax = v;
        self
    }

    /// # Description
    ///
    /// -K
    ///
    /// Suppress keyword expansion when updating `+k` type files on the
    /// client. See File type modifiers.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn get_suppress_keyword_expansion(&self) -> bool {
        self.suppress_keyword_expansion
    }

    /// # Description
    ///
    /// -K
    ///
    /// Suppress keyword expansion when updating `+k` type files on the
    /// client. See File type modifiers.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn set_suppress_keyword_expansion(&mut self, v: bool) -> &mut Self {
        self.suppress_keyword_expansion = v;
        self
    }

    /// # Description
    ///
    /// -K
    ///
    /// Suppress keyword expansion when updating `+k` type files on the
    /// client. See File type modifiers.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn suppress_keyword_expansion(mut self, v: bool) -> Self {
        self.suppress_keyword_expansion = v;
        self
    }

    /// # Description
    ///
    /// -m
    ///
    /// Compare the file sync or submit time (in the depot) with the file
    /// modification time (in the workspace) to help determine whether the
    /// file has changed. Normally P4 Server uses file digests to determine
    /// whether files in the workspace differ from the head revisions of
    /// these files in the depot. This can be time consuming for large files.
    /// But when the timestamps are the same, the costly digest comparisons
    /// can be skipped. This option is only relevant if you are using clean
    /// to find changed files rather than files that were deleted or added.
    #[cfg(not(feature = "lt2018_1"))]
    pub fn get_fast_timestamp_check(&self) -> bool {
        self.fast_timestamp_check
    }

    /// # Description
    ///
    /// -m
    ///
    /// Compare the file sync or submit time (in the depot) with the file
    /// modification time (in the workspace) to help determine whether the
    /// file has changed. Normally P4 Server uses file digests to determine
    /// whether files in the workspace differ from the head revisions of
    /// these files in the depot. This can be time consuming for large files.
    /// But when the timestamps are the same, the costly digest comparisons
    /// can be skipped. This option is only relevant if you are using clean
    /// to find changed files rather than files that were deleted or added.
    #[cfg(not(feature = "lt2018_1"))]
    pub fn set_fast_timestamp_check(&mut self, v: bool) -> &mut Self {
        self.fast_timestamp_check = v;
        self
    }

    /// # Description
    ///
    /// -m
    ///
    /// Compare the file sync or submit time (in the depot) with the file
    /// modification time (in the workspace) to help determine whether the
    /// file has changed. Normally P4 Server uses file digests to determine
    /// whether files in the workspace differ from the head revisions of
    /// these files in the depot. This can be time consuming for large files.
    /// But when the timestamps are the same, the costly digest comparisons
    /// can be skipped. This option is only relevant if you are using clean
    /// to find changed files rather than files that were deleted or added.
    #[cfg(not(feature = "lt2018_1"))]
    pub fn fast_timestamp_check(mut self, v: bool) -> Self {
        self.fast_timestamp_check = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview the results of the operation without performing any action.
    pub fn get_preview(&self) -> bool {
        self.preview
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview the results of the operation without performing any action.
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview the results of the operation without performing any action.
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut command = Clean::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run check of the assembled `p4 clean` command line; no process is
    /// spawned.
    #[test]
    fn without_options() {
        let clean = Clean::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&clean.setup_command("p4")), ["clean"]);
    }

    #[test]
    fn all_local_options() {
        let mut clean = Clean::new("p4", GlobalOpts::new());
        clean
            .set_find_edited(true)
            .set_find_added(true)
            .set_find_deleted(true)
            .set_ignore_p4ignore(true)
            .set_local_file_syntax(true)
            .set_preview(true);

        #[cfg(not(feature = "lt2022_1"))]
        clean.set_suppress_keyword_expansion(true);

        #[cfg(not(feature = "lt2018_1"))]
        clean.set_fast_timestamp_check(true);

        let mut expected = vec!["clean", "-e", "-a", "-d", "-I", "-l"];
        #[cfg(not(feature = "lt2022_1"))]
        expected.push("-K");
        #[cfg(not(feature = "lt2018_1"))]
        expected.push("-m");
        expected.push("-n");

        assert_eq!(args_of(&clean.setup_command("p4")), expected);
    }

    #[test]
    fn all_accessors_report_state() {
        let clean = Clean::new("p4", GlobalOpts::new())
            .find_edited(true)
            .find_added(true)
            .find_deleted(true)
            .ignore_p4ignore(true)
            .local_file_syntax(true)
            .preview(true);

        #[cfg(not(feature = "lt2022_1"))]
        let clean = clean.suppress_keyword_expansion(true);

        #[cfg(not(feature = "lt2018_1"))]
        let clean = clean.fast_timestamp_check(true);

        assert!(clean.get_find_edited());
        assert!(clean.get_find_added());
        assert!(clean.get_find_deleted());
        assert!(clean.get_ignore_p4ignore());
        assert!(clean.get_local_file_syntax());
        assert!(clean.get_preview());

        #[cfg(not(feature = "lt2022_1"))]
        assert!(clean.get_suppress_keyword_expansion());

        #[cfg(not(feature = "lt2018_1"))]
        assert!(clean.get_fast_timestamp_check());
    }

    #[test]
    fn global_opts_are_injected_once() {
        let clean = Clean::new("p4", GlobalOpts::new().port("localhost:1666"));

        assert_eq!(
            args_of(&clean.setup_command("p4")),
            ["-p", "localhost:1666", "clean"]
        );
    }
}
