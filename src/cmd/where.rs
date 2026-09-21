use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

#[cfg_attr(feature = "lt2015_1", doc = "`p4 [g-opts] where [file ...]`")]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] where [file …]`"
)]
#[cfg_attr(not(feature = "lt2016_1"), doc = "`p4 [g-opts] where [file ...]`")]
///
#[cfg_attr(
    feature = "lt2023_2",
    doc = "Show where a particular file is located, as determined by the client view.",
    doc = "",
    doc = "`p4 where` uses the client view and root (as set in `p4 client`) to print files'",
    doc = "locations relative to the top of the depot, relative to the top of the client",
    doc = "workspace, and relative to the top of the local OS directory tree. The command",
    doc = "does not check to see if the file exists; it merely reports where the file",
    doc = "*would be* located if it *did* exist.",
    doc = "",
    doc = "For each file provided as a parameter, a set of mappings is output. Each set",
    doc = "of mappings is composed of lines consisting of three parts: the first part",
    doc = "is the filename expressed in depot syntax, the second part is the filename",
    doc = "expressed in client syntax, and the third is the local OS path of the file."
)]
#[cfg_attr(
    not(feature = "lt2023_2"),
    doc = "Show how the specified files are mapped by the client view.",
    doc = "",
    doc = "`p4 where` uses the client view and root to print files locations relative to",
    doc = "the top of the depot, relative to the top of the client workspace, and",
    doc = "relative to the top of the local OS directory tree. The client mappings are",
    doc = "set by the `p4 client` command. The `p4 where` command does not check to see",
    doc = "whether the file exists. Instead, it reports where the file *would be*",
    doc = "located if it *did* exist.",
    doc = "",
    doc = "The command accepts wildcards, such as `p4 where *.html`",
    doc = "",
    doc = "For each file provided as a parameter, a set of mappings is output. Each set",
    doc = "of mappings consists of:",
    doc = "",
    doc = "- the filename in depot syntax. such as `//depot/project1/my-file.html`",
    doc = "- the filename in client syntax, such as `//maria/depot/project1/my-file.html`",
    doc = "- the filename in local syntax, which is the local operating system path,",
    doc = "such as `C:\\Users\\maria\\project1\\my-file.html`"
)]
#[derive(Debug, Clone, Default)]
pub struct Where {
    bin: PathBuf,

    global_opts: GlobalOpts,
}

impl SubCommand for Where {
    fn name(&self) -> &str {
        "where"
    }

    fn inject_local_args(&self, _: &mut Command) {}

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl ParameterizedSpawn for Where {
    type Input<'a> = &'a [&'a OsStr];
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 where` for the given files as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    ///
    /// For each file provided as a parameter, a set of mappings is output.
    fn spawn_with<'a>(&mut self, files: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl Where {
    /// Creates a new `p4 where` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run check of the assembled `p4 where` command line; no process is
    /// spawned.
    #[test]
    fn without_options() {
        let r#where = Where::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&r#where.setup_command("p4")), ["where"]);
    }

    #[test]
    fn with_global_opts() {
        let r#where = Where::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        );

        assert_eq!(
            args_of(&r#where.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "where"]
        );
    }

    #[test]
    fn with_files() {
        let r#where = Where::new("p4", GlobalOpts::new());

        assert_eq!(
            args_of(r#where.setup_command("p4").args(["file.c"])),
            ["where", "file.c"]
        );
    }
}
