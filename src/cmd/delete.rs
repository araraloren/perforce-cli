use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Stdio},
};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

#[cfg_attr(
    feature = "lt2016_1",
    doc = "`p4 [g-opts] delete [-c changelist] [-n -k -v] file ...`"
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] delete [-c changelist] [-n -k -v] [--remote=remote] file ...`"
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "`p4 [g-opts] delete [-c PendingChangelist] [-n -k -v] [--remote=remote] FileSpec1 [[FileSpec]2 ...]`"
)]
///
#[cfg_attr(
    feature = "lt2024_1",
    doc = "Open file(s) in a client workspace for deletion from the depot."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "Open files in a client workspace for deletion from the depot."
)]
#[derive(Debug, Clone, Default)]
pub struct Delete {
    bin: PathBuf,

    global_opts: GlobalOpts,

    changelist: Option<String>,

    preview: bool,

    keep_localfile: bool,

    delete_unsynced: bool,

    #[cfg(not(feature = "lt2016_1"))]
    remote: Option<String>,
}

impl SubCommand for Delete {
    fn name(&self) -> &str {
        "delete"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if let Some(changelist) = self.changelist.as_ref() {
            command.arg("-c").arg(changelist);
        }
        if self.preview {
            command.arg("-n");
        }
        if self.keep_localfile {
            command.arg("-k");
        }
        if self.delete_unsynced {
            command.arg("-v");
        }
        #[cfg(not(feature = "lt2016_1"))]
        if let Some(remote) = self.remote.as_ref() {
            command.arg(format!("--remote={remote}"));
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<S, I> ParameterizedSpawn<(S,)> for Delete
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 delete` for the given files as a child process.
    ///
    /// The child process inherits the standard input, output, and error
    /// streams of the current process, and runs asynchronously; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (input,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(input)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl Delete {
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

    #[cfg_attr(
        feature = "lt2024_1",
        doc = "Open file(s) in a client workspace for deletion from the depot."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "Open files in a client workspace for deletion from the depot."
    )]
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
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "-c changelist",
        doc = "",
        doc = "Opens the files for delete within the specified changelist.",
        doc = "",
        doc = "If this flag is not provided, the files are linked to the default changelist."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2014_2")),
        doc = "-c changelist",
        doc = "",
        doc = "Opens the files for delete within the specified changelist.",
        doc = "",
        doc = "If this option is not provided, the files are linked to the default changelist."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "-c PendingChangelist",
        doc = "",
        doc = "Opens files for `delete` and associates the deletion action with the specified pending changelist.",
        doc = "",
        doc = "If no pending changelist is specified, the deletion action is associated with the default changelist."
    )]
    pub fn get_changelist(&self) -> Option<&String> {
        self.changelist.as_ref()
    }

    /// # Description
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "-c changelist",
        doc = "",
        doc = "Opens the files for delete within the specified changelist.",
        doc = "",
        doc = "If this flag is not provided, the files are linked to the default changelist."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2014_2")),
        doc = "-c changelist",
        doc = "",
        doc = "Opens the files for delete within the specified changelist.",
        doc = "",
        doc = "If this option is not provided, the files are linked to the default changelist."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "-c PendingChangelist",
        doc = "",
        doc = "Opens files for `delete` and associates the deletion action with the specified pending changelist.",
        doc = "",
        doc = "If no pending changelist is specified, the deletion action is associated with the default changelist."
    )]
    pub fn set_changelist(&mut self, v: impl Into<String>) -> &mut Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "-c changelist",
        doc = "",
        doc = "Opens the files for delete within the specified changelist.",
        doc = "",
        doc = "If this flag is not provided, the files are linked to the default changelist."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2014_2")),
        doc = "-c changelist",
        doc = "",
        doc = "Opens the files for delete within the specified changelist.",
        doc = "",
        doc = "If this option is not provided, the files are linked to the default changelist."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "-c PendingChangelist",
        doc = "",
        doc = "Opens files for `delete` and associates the deletion action with the specified pending changelist.",
        doc = "",
        doc = "If no pending changelist is specified, the deletion action is associated with the default changelist."
    )]
    pub fn changelist(mut self, v: impl Into<String>) -> Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview which files would be opened for delete, without actually
    /// changing any files or metadata.
    pub fn get_preview(&self) -> bool {
        self.preview
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview which files would be opened for delete, without actually
    /// changing any files or metadata.
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview which files would be opened for delete, without actually
    /// changing any files or metadata.
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -k
    ///
    /// Delete the file on the shared versioning service, but keep a copy of
    /// the deleted file in your workspace.
    pub fn get_keep_localfile(&self) -> bool {
        self.keep_localfile
    }

    /// # Description
    ///
    /// -k
    ///
    /// Delete the file on the shared versioning service, but keep a copy of
    /// the deleted file in your workspace.
    pub fn set_keep_localfile(&mut self, v: bool) -> &mut Self {
        self.keep_localfile = v;
        self
    }

    /// # Description
    ///
    /// -k
    ///
    /// Delete the file on the shared versioning service, but keep a copy of
    /// the deleted file in your workspace.
    pub fn keep_localfile(mut self, v: bool) -> Self {
        self.keep_localfile = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Delete a file that is not synced into the client workspace.
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "To use this flag, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.) To delete a set of files without transferring them to your workstation, use `p4 sync -k file`, followed by `p4 delete -k file`."
    )]
    #[cfg_attr(
        all(feature = "lt2016_1", not(feature = "lt2014_2")),
        doc = "To use this option, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.) To delete a set of files without transferring them to your workstation, use `p4 sync -k file`, followed by `p4 delete -k file`."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "To use this option, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.)",
        doc = "",
        doc = "To delete a set of files without transferring them to your workstation when another version of these files already exists in your workspace, use `p4 sync -k file`, followed by `p4 delete -k file`. This allows you to delete a file that is in the depot without affecting the file (by the same name) in your workspace. For example, if you have `myfile` version 5 in the depot and `myfile` version 6 in your workspace, this sequence of commands, allows you to keep version 6 but delete version 5. If you don't do this and just do `p4 delete -v`, it will delete version 5 in the depot and version 6 in the workspace."
    )]
    pub fn get_delete_unsynced(&self) -> bool {
        self.delete_unsynced
    }

    /// # Description
    ///
    /// -v
    ///
    /// Delete a file that is not synced into the client workspace.
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "To use this flag, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.) To delete a set of files without transferring them to your workstation, use `p4 sync -k file`, followed by `p4 delete -k file`."
    )]
    #[cfg_attr(
        all(feature = "lt2016_1", not(feature = "lt2014_2")),
        doc = "To use this option, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.) To delete a set of files without transferring them to your workstation, use `p4 sync -k file`, followed by `p4 delete -k file`."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "To use this option, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.)",
        doc = "",
        doc = "To delete a set of files without transferring them to your workstation when another version of these files already exists in your workspace, use `p4 sync -k file`, followed by `p4 delete -k file`. This allows you to delete a file that is in the depot without affecting the file (by the same name) in your workspace. For example, if you have `myfile` version 5 in the depot and `myfile` version 6 in your workspace, this sequence of commands, allows you to keep version 6 but delete version 5. If you don't do this and just do `p4 delete -v`, it will delete version 5 in the depot and version 6 in the workspace."
    )]
    pub fn set_delete_unsynced(&mut self, v: bool) -> &mut Self {
        self.delete_unsynced = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Delete a file that is not synced into the client workspace.
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "To use this flag, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.) To delete a set of files without transferring them to your workstation, use `p4 sync -k file`, followed by `p4 delete -k file`."
    )]
    #[cfg_attr(
        all(feature = "lt2016_1", not(feature = "lt2014_2")),
        doc = "To use this option, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.) To delete a set of files without transferring them to your workstation, use `p4 sync -k file`, followed by `p4 delete -k file`."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "To use this option, specify these files in depot syntax; because such files are not synced, client syntax or local syntax can introduce ambiguities in the list of files to delete. (If the files are synced, `p4 delete -v file` removes the files from your workspace in addition to opening them for deletion.)",
        doc = "",
        doc = "To delete a set of files without transferring them to your workstation when another version of these files already exists in your workspace, use `p4 sync -k file`, followed by `p4 delete -k file`. This allows you to delete a file that is in the depot without affecting the file (by the same name) in your workspace. For example, if you have `myfile` version 5 in the depot and `myfile` version 6 in your workspace, this sequence of commands, allows you to keep version 6 but delete version 5. If you don't do this and just do `p4 delete -v`, it will delete version 5 in the depot and version 6 in the workspace."
    )]
    pub fn delete_unsynced(mut self, v: bool) -> Self {
        self.delete_unsynced = v;
        self
    }

    /// # Description
    ///
    /// `--remote=remote`
    ///
    /// Opens the file for delete in your personal server, and additionally —
    /// if the file is of type `+l` — takes a global exclusive lock on the file
    /// in the shared server from which you cloned the file.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_remote(&self) -> Option<&String> {
        self.remote.as_ref()
    }

    /// # Description
    ///
    /// `--remote=remote`
    ///
    /// Opens the file for delete in your personal server, and additionally —
    /// if the file is of type `+l` — takes a global exclusive lock on the file
    /// in the shared server from which you cloned the file.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_remote(&mut self, v: impl Into<String>) -> &mut Self {
        self.remote = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `--remote=remote`
    ///
    /// Opens the file for delete in your personal server, and additionally —
    /// if the file is of type `+l` — takes a global exclusive lock on the file
    /// in the shared server from which you cloned the file.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn remote(mut self, v: impl Into<String>) -> Self {
        self.remote = Some(v.into());
        self
    }
}
