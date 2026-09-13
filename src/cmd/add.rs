use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Output, Stdio},
};

use crate::{cmd::SubCommand, global::GlobalOpts};

/// `p4 [g-opts] add [-c changelist] [-d -f -I -n] [-t filetype] file ...`
///
/// Open files in a client workspace for addition to the depot.
#[derive(Debug, Clone, Default)]
pub struct Add {
    bin: PathBuf,

    global_opts: GlobalOpts,

    change_list: Option<String>,

    downgrade: bool,

    force_literal_filenames: bool,

    skip_ignore: bool,

    preview: bool,

    filetype: Option<String>,
}

impl SubCommand for Add {
    fn name(&self) -> &str {
        "add"
    }

    fn inject_local_args(&self, command: &mut std::process::Command) {
        if let Some(change_list) = self.change_list.as_ref() {
            command.arg("-c").arg(change_list);
        }
        if self.downgrade {
            command.arg("-d");
        }
        if self.force_literal_filenames {
            command.arg("-f");
        }
        if self.skip_ignore {
            command.arg("-I");
        }
        if self.preview {
            command.arg("-n");
        }
        if let Some(filetype) = self.filetype.as_ref() {
            command.arg("-t").arg(filetype);
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl Add {
    /// Open files in a client workspace for addition to the depot.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Spawns `p4 add` for the given files as a child process.
    ///
    /// The child process inherits the standard input, output, and error
    /// streams of the current process, and runs asynchronously; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 add` for the given files to completion and captures its
    /// output.
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
    /// -c changelist
    ///
    /// Opens the files for add within the specified changelist. If this
    #[cfg_attr(feature = "lt2014_2", doc = "flag is")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "option is")]
    /// not used, the files are linked to the default changelist.
    pub fn get_change_list(&self) -> Option<&String> {
        self.change_list.as_ref()
    }

    /// # Description
    ///
    /// -c changelist
    ///
    /// Opens the files for add within the specified changelist. If this
    #[cfg_attr(feature = "lt2014_2", doc = "flag is")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "option is")]
    /// not used, the files are linked to the default changelist.
    pub fn set_change_list(&mut self, v: impl Into<String>) -> &mut Self {
        self.change_list = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -c changelist
    ///
    /// Opens the files for add within the specified changelist. If this
    #[cfg_attr(feature = "lt2014_2", doc = "flag is")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "option is")]
    /// not used, the files are linked to the default changelist.
    pub fn change_list(mut self, v: impl Into<String>) -> Self {
        self.change_list = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -d
    ///
    /// Downgrade file open status to simple add.
    pub fn get_downgrade(&self) -> bool {
        self.downgrade
    }
    /// # Description
    ///
    /// -d
    ///
    /// Downgrade file open status to simple add.
    pub fn set_downgrade(&mut self, v: bool) -> &mut Self {
        self.downgrade = v;
        self
    }
    /// # Description
    ///
    /// -d
    ///
    /// Downgrade file open status to simple add.
    pub fn downgrade(mut self, v: bool) -> Self {
        self.downgrade = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Use the -f
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flag to force inclusion of wildcards in filenames. See the",
        doc = "File Specifications chapter for details."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "option to force inclusion of wildcards in filenames. See the",
        doc = "“File Specifications” chapter for details."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "option to force inclusion of wildcards in filenames. See",
        doc = "“File Specifications” for details."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "Specifications for details."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2018_2")),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "specifications for details."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "specifications for details.",
        doc = "",
        doc = "Filenames that contain the special characters '@', '#', '%', or '*' are",
        doc = "reformatted to encode the characters using ASCII hexadecimal",
        doc = "representation. After the files are added, refer to them using the",
        doc = "reformatted file name instead of the local file system name."
    )]
    pub fn get_force_literal_filenames(&self) -> bool {
        self.force_literal_filenames
    }

    /// # Description
    ///
    /// -f
    ///
    /// Use the -f
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flag to force inclusion of wildcards in filenames. See the",
        doc = "File Specifications chapter for details."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "option to force inclusion of wildcards in filenames. See the",
        doc = "“File Specifications” chapter for details."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "option to force inclusion of wildcards in filenames. See",
        doc = "“File Specifications” for details."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "Specifications for details."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2018_2")),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "specifications for details."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "specifications for details.",
        doc = "",
        doc = "Filenames that contain the special characters '@', '#', '%', or '*' are",
        doc = "reformatted to encode the characters using ASCII hexadecimal",
        doc = "representation. After the files are added, refer to them using the",
        doc = "reformatted file name instead of the local file system name."
    )]
    pub fn set_force_literal_filenames(&mut self, v: bool) -> &mut Self {
        self.force_literal_filenames = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Use the -f
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flag to force inclusion of wildcards in filenames. See the",
        doc = "File Specifications chapter for details."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "option to force inclusion of wildcards in filenames. See the",
        doc = "“File Specifications” chapter for details."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "option to force inclusion of wildcards in filenames. See",
        doc = "“File Specifications” for details."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "Specifications for details."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2018_2")),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "specifications for details."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "option to force inclusion of wildcards in filenames. See File",
        doc = "specifications for details.",
        doc = "",
        doc = "Filenames that contain the special characters '@', '#', '%', or '*' are",
        doc = "reformatted to encode the characters using ASCII hexadecimal",
        doc = "representation. After the files are added, refer to them using the",
        doc = "reformatted file name instead of the local file system name."
    )]
    pub fn force_literal_filenames(mut self, v: bool) -> Self {
        self.force_literal_filenames = v;
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Do not perform any ignore checking; ignore any settings specified by
    /// P4IGNORE.
    pub fn get_skip_ignore(&self) -> bool {
        self.skip_ignore
    }
    /// # Description
    ///
    /// -I
    ///
    /// Do not perform any ignore checking; ignore any settings specified by
    /// P4IGNORE.
    pub fn set_skip_ignore(&mut self, v: bool) -> &mut Self {
        self.skip_ignore = v;
        self
    }
    /// # Description
    ///
    /// -I
    ///
    /// Do not perform any ignore checking; ignore any settings specified by
    /// P4IGNORE.
    pub fn skip_ignore(mut self, v: bool) -> Self {
        self.skip_ignore = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview which files would be opened for add, without actually changing
    /// any files or metadata.
    pub fn get_preview(&self) -> bool {
        self.preview
    }
    /// # Description
    ///
    /// -n
    ///
    /// Preview which files would be opened for add, without actually changing
    /// any files or metadata.
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }
    /// # Description
    ///
    /// -n
    ///
    /// Preview which files would be opened for add, without actually changing
    /// any files or metadata.
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -t filetype
    ///
    /// Adds the file as the specified filetype, overriding any settings in the
    /// typemap table.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Please see the File Types chapter for a list of Perforce",
        doc = "file types."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "Please see the “File Types” chapter for a list of",
        doc = "Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "See “File Types” for a list of Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2017_1")),
        doc = "See File Types for a list of Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_2")),
        doc = "See File Types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_2")),
        doc = "See File types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "See File types for a list of Helix server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "See File types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2024_1")),
        doc = "See File types as well as the lbr.autocompress",
        doc = "configurable."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "See Base file types and modifiers as well as the",
        doc = "lbr.autocompress configurable."
    )]
    pub fn get_filetype(&self) -> Option<&String> {
        self.filetype.as_ref()
    }

    /// # Description
    ///
    /// -t filetype
    ///
    /// Adds the file as the specified filetype, overriding any settings in the
    /// typemap table.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Please see the File Types chapter for a list of Perforce",
        doc = "file types."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "Please see the “File Types” chapter for a list of",
        doc = "Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "See “File Types” for a list of Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2017_1")),
        doc = "See File Types for a list of Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_2")),
        doc = "See File Types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_2")),
        doc = "See File types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "See File types for a list of Helix server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "See File types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2024_1")),
        doc = "See File types as well as the lbr.autocompress",
        doc = "configurable."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "See Base file types and modifiers as well as the",
        doc = "lbr.autocompress configurable."
    )]
    pub fn set_filetype(&mut self, v: impl Into<String>) -> &mut Self {
        self.filetype = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -t filetype
    ///
    /// Adds the file as the specified filetype, overriding any settings in the
    /// typemap table.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Please see the File Types chapter for a list of Perforce",
        doc = "file types."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "Please see the “File Types” chapter for a list of",
        doc = "Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2017_1", not(feature = "lt2015_1")),
        doc = "See “File Types” for a list of Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2017_1")),
        doc = "See File Types for a list of Perforce file types."
    )]
    #[cfg_attr(
        all(feature = "lt2018_2", not(feature = "lt2017_2")),
        doc = "See File Types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_2")),
        doc = "See File types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "See File types for a list of Helix server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "See File types for a list of Helix Server file types."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2024_1")),
        doc = "See File types as well as the lbr.autocompress",
        doc = "configurable."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "See Base file types and modifiers as well as the",
        doc = "lbr.autocompress configurable."
    )]
    pub fn filetype(mut self, v: impl Into<String>) -> Self {
        self.filetype = Some(v.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run check of the assembled `p4 add` command line; no process is
    /// spawned.
    #[test]
    fn without_options() {
        let add = Add::new("p4", GlobalOpts::new());

        let command = add.setup_command("p4");

        assert_eq!(command.get_program(), OsStr::new("p4"));
        assert_eq!(args_of(&command), ["add"]);
    }

    #[test]
    fn all_local_options() {
        let mut add = Add::new("p4", GlobalOpts::new());
        add.set_change_list("42")
            .set_downgrade(true)
            .set_force_literal_filenames(true)
            .set_skip_ignore(true)
            .set_preview(true)
            .set_filetype("text");

        assert_eq!(
            args_of(&add.setup_command("p4")),
            ["add", "-c", "42", "-d", "-f", "-I", "-n", "-t", "text"]
        );
    }
}
