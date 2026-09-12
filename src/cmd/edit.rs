use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Output, Stdio},
};

use crate::{cmd::SubCommand, global::GlobalOpts};

#[cfg_attr(
    feature = "lt2019_1",
    doc = "`p4 [g-opts] edit [-c changelist] [-k -n] [-t type] [--remote=remote] file ...`"
)]
#[cfg_attr(
    not(feature = "lt2019_1"),
    doc = "`p4 [g-opts] edit [-c changelist] [-k -n] [-t type] [--remote=remote] file ...`\n\n\
           `p4 [g-opts] edit -So [-c changelist]`"
)]
///
/// Opens files in a client workspace for edit, or open the current stream
/// spec.
#[derive(Debug, Clone, Default)]
pub struct Edit {
    bin: PathBuf,

    global_opts: GlobalOpts,

    change_list: Option<String>,

    keep_workspace: bool,

    preview: bool,

    remote: Option<String>,

    filetype: Option<String>,

    #[cfg(not(feature = "lt2019_1"))]
    stream_spec: bool,
}

impl Edit {
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Self::default()
        }
    }

    /// Runs `p4 edit` for the given files, inheriting the parent process's
    /// standard streams.
    ///
    /// This corresponds to the file form of the command:
    /// `p4 edit [options] file ...`.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 edit` for the given files to completion and captures its
    /// output.
    ///
    /// Unlike [`Self::spawn`], this method blocks until the command exits and
    /// collects the standard output and error into the returned [`Output`].
    ///
    /// This corresponds to the file form of the command:
    /// `p4 edit [options] file ...`.
    pub fn output<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Output, std::io::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

    /// Runs `p4 edit -So` to open the current stream spec for edit, inheriting
    /// the parent process's standard streams.
    ///
    /// This corresponds to the stream spec form of the command:
    /// `p4 edit -So [-c changelist]`, which takes no file arguments.
    ///
    /// The `-So` flag is set automatically; do not call this method together
    /// with file arguments.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn spawn_stream_spec(&self) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).spawn()
    }

    /// Runs `p4 edit -So` to open the current stream spec for edit, waits for
    /// it to complete, and captures its output.
    ///
    /// Unlike [`Self::spawn_stream_spec`], this method blocks until the
    /// command exits and collects the standard output and error into the
    /// returned [`Output`].
    ///
    /// This corresponds to the stream spec form of the command:
    /// `p4 edit -So [-c changelist]`, which takes no file arguments.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn output_stream_spec(&self) -> Result<Output, std::io::Error> {
        self.setup_command(&self.bin)
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
    /// `-c changelist`
    ///
    /// Opens the files for edit within the specified changelist. If this
    /// option is not provided, the files are linked to the default changelist.
    pub fn get_change_list(&self) -> Option<&String> {
        self.change_list.as_ref()
    }

    /// # Description
    ///
    /// `-c changelist`
    ///
    /// Opens the files for edit within the specified changelist. If this
    /// option is not provided, the files are linked to the default changelist.
    pub fn set_change_list(&mut self, v: impl Into<String>) -> &mut Self {
        self.change_list = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-c changelist`
    ///
    /// Opens the files for edit within the specified changelist. If this
    /// option is not provided, the files are linked to the default changelist.
    pub fn change_list(mut self, v: impl Into<String>) -> Self {
        self.change_list = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-k`
    ///
    /// Keep existing workspace files; mark the file as open for edit even if
    /// the file is not in the client view. Use `p4 edit -k` only in the
    /// context of reconciling work performed while disconnected from the
    /// shared versioning service.
    pub fn get_keep_workspace(&self) -> bool {
        self.keep_workspace
    }

    /// # Description
    ///
    /// `-k`
    ///
    /// Keep existing workspace files; mark the file as open for edit even if
    /// the file is not in the client view. Use `p4 edit -k` only in the
    /// context of reconciling work performed while disconnected from the
    /// shared versioning service.
    pub fn set_keep_workspace(&mut self, v: bool) -> &mut Self {
        self.keep_workspace = v;
        self
    }

    /// # Description
    ///
    /// `-k`
    ///
    /// Keep existing workspace files; mark the file as open for edit even if
    /// the file is not in the client view. Use `p4 edit -k` only in the
    /// context of reconciling work performed while disconnected from the
    /// shared versioning service.
    pub fn keep_workspace(mut self, v: bool) -> Self {
        self.keep_workspace = v;
        self
    }

    /// # Description
    ///
    /// `-n`
    ///
    /// Preview which files would be opened for edit, without actually changing
    /// any files or metadata.
    pub fn get_preview(&self) -> bool {
        self.preview
    }

    /// # Description
    ///
    /// `-n`
    ///
    /// Preview which files would be opened for edit, without actually changing
    /// any files or metadata.
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// `-n`
    ///
    /// Preview which files would be opened for edit, without actually changing
    /// any files or metadata.
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// `--remote=remote`
    ///
    /// Opens the file for edit in your personal server, and additionally — if
    /// the file is of type `+l` — takes a global exclusive lock on the file in
    /// the shared server from which you cloned the file.
    pub fn get_remote(&self) -> Option<&String> {
        self.remote.as_ref()
    }

    /// # Description
    ///
    /// `--remote=remote`
    ///
    /// Opens the file for edit in your personal server, and additionally — if
    /// the file is of type `+l` — takes a global exclusive lock on the file in
    /// the shared server from which you cloned the file.
    pub fn set_remote(&mut self, v: impl Into<String>) -> &mut Self {
        self.remote = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `--remote=remote`
    ///
    /// Opens the file for edit in your personal server, and additionally — if
    /// the file is of type `+l` — takes a global exclusive lock on the file in
    /// the shared server from which you cloned the file.
    pub fn remote(mut self, v: impl Into<String>) -> Self {
        self.remote = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-t type`
    ///
    /// Stores the new file revision as the specified type, overriding the file
    /// type of the previous revision of the same file. To forcibly re-detect a
    /// file's filetype upon editing a file, use `p4 edit -t auto`. This assigns
    /// a file type as if the file were being newly added.
    ///
    #[cfg_attr(feature = "lt2024_1", doc = "See File types for a list of file types.")]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "See File types as well as the lbr.autocompress configurable."
    )]
    pub fn get_filetype(&self) -> Option<&String> {
        self.filetype.as_ref()
    }

    /// # Description
    ///
    /// `-t type`
    ///
    /// Stores the new file revision as the specified type, overriding the file
    /// type of the previous revision of the same file. To forcibly re-detect a
    /// file's filetype upon editing a file, use `p4 edit -t auto`. This assigns
    /// a file type as if the file were being newly added.
    ///
    #[cfg_attr(feature = "lt2024_1", doc = "See File types for a list of file types.")]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "See File types as well as the lbr.autocompress configurable."
    )]
    pub fn set_filetype(&mut self, v: impl Into<String>) -> &mut Self {
        self.filetype = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-t type`
    ///
    /// Stores the new file revision as the specified type, overriding the file
    /// type of the previous revision of the same file. To forcibly re-detect a
    /// file's filetype upon editing a file, use `p4 edit -t auto`. This assigns
    /// a file type as if the file were being newly added.
    ///
    #[cfg_attr(feature = "lt2024_1", doc = "See File types for a list of file types.")]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "See File types as well as the lbr.autocompress configurable."
    )]
    pub fn filetype(mut self, v: impl Into<String>) -> Self {
        self.filetype = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-So`
    ///
    /// Can be used with `-c changelist` to open the client's stream spec for
    /// edit. No list of files is allowed. `p4 edit -So` is an alias for
    /// `p4 stream edit` (see also `p4 help streamcmds`).
    #[cfg(not(feature = "lt2019_1"))]
    pub fn get_stream_spec(&self) -> bool {
        self.stream_spec
    }

    /// # Description
    ///
    /// `-So`
    ///
    /// Can be used with `-c changelist` to open the client's stream spec for
    /// edit. No list of files is allowed. `p4 edit -So` is an alias for
    /// `p4 stream edit` (see also `p4 help streamcmds`).
    #[cfg(not(feature = "lt2019_1"))]
    pub fn set_stream_spec(&mut self, v: bool) -> &mut Self {
        self.stream_spec = v;
        self
    }

    /// # Description
    ///
    /// `-So`
    ///
    /// Can be used with `-c changelist` to open the client's stream spec for
    /// edit. No list of files is allowed. `p4 edit -So` is an alias for
    /// `p4 stream edit` (see also `p4 help streamcmds`).
    #[cfg(not(feature = "lt2019_1"))]
    pub fn stream_spec(mut self, v: bool) -> Self {
        self.stream_spec = v;
        self
    }
}

impl SubCommand for Edit {
    fn name(&self) -> &str {
        "edit"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if let Some(ref change_list) = self.change_list {
            command.arg("-c").arg(change_list);
        }
        if self.keep_workspace {
            command.arg("-k");
        }
        if self.preview {
            command.arg("-n");
        }
        if let Some(ref remote) = self.remote {
            command.arg(format!("--remote={remote}"));
        }
        if let Some(ref filetype) = self.filetype {
            command.arg("-t").arg(filetype);
        }
        #[cfg(not(feature = "lt2019_1"))]
        if self.stream_spec {
            command.arg("-So");
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
        let edit = Edit::new("p4", GlobalOpts::default());
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(args_of(&cmd), vec!["edit", "//depot/file.txt"]);
    }

    #[test]
    fn change_list() {
        let edit = Edit::new("p4", GlobalOpts::default()).change_list("14");
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(args_of(&cmd), vec!["edit", "-c", "14", "//depot/file.txt"]);
    }

    #[test]
    fn keep_workspace_and_preview() {
        let edit = Edit::new("p4", GlobalOpts::default())
            .keep_workspace(true)
            .preview(true);
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(args_of(&cmd), vec!["edit", "-k", "-n", "//depot/file.txt"]);
    }

    #[test]
    fn remote_option_uses_equals_sign() {
        let edit = Edit::new("p4", GlobalOpts::default()).remote("origin");
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(
            args_of(&cmd),
            vec!["edit", "--remote=origin", "//depot/file.txt"]
        );
    }

    #[test]
    fn filetype() {
        let edit = Edit::new("p4", GlobalOpts::default()).filetype("text+k");
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(
            args_of(&cmd),
            vec!["edit", "-t", "text+k", "//depot/file.txt"]
        );
    }

    #[cfg(not(feature = "lt2019_1"))]
    #[test]
    fn stream_spec() {
        let edit = Edit::new("p4", GlobalOpts::default()).stream_spec(true);
        let cmd = edit.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["edit", "-So"]);
    }

    #[cfg(not(feature = "lt2019_1"))]
    #[test]
    fn stream_spec_with_change_list() {
        let edit = Edit::new("p4", GlobalOpts::default())
            .stream_spec(true)
            .change_list("14");
        let cmd = edit.setup_command("p4");
        assert_eq!(args_of(&cmd), vec!["edit", "-c", "14", "-So"]);
    }

    #[test]
    fn all_options_order() {
        let edit = Edit::new("p4", GlobalOpts::default())
            .change_list("14")
            .keep_workspace(true)
            .preview(true)
            .remote("origin")
            .filetype("binary");
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(
            args_of(&cmd),
            vec![
                "edit",
                "-c",
                "14",
                "-k",
                "-n",
                "--remote=origin",
                "-t",
                "binary",
                "//depot/file.txt"
            ]
        );
    }

    #[test]
    fn set_style_with_global_opts() {
        let mut edit = Edit::new("p4", GlobalOpts::default());
        edit.set_change_list("14").set_preview(true);
        let mut cmd = edit.setup_command("p4");
        cmd.arg("//depot/file.txt");
        assert_eq!(
            args_of(&cmd),
            vec!["edit", "-c", "14", "-n", "//depot/file.txt"]
        );
    }
}
