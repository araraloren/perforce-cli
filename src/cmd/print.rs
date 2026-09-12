use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
};

use crate::{cmd::SubCommand, global::GlobalOpts};

/// The `-L` line ending for textual files: 'unix', 'win', or 'mac'.
#[cfg(not(feature = "lt2023_1"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Use the `unix` line ending (`-L unix`).
    Unix,
    /// Use the `win` line ending (`-L win`).
    Win,
    /// Use the `mac` line ending (`-L mac`).
    Mac,
}

#[cfg(not(feature = "lt2023_1"))]
impl LineEnding {
    /// Returns the command-line value for this line ending.
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Unix => "unix",
            LineEnding::Win => "win",
            LineEnding::Mac => "mac",
        }
    }
}

#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] print [-a -A -k -o outfile -q -m max -U] file[revRange] ...`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] print [-a -A -k -q] [-m max] [-o outfile] file[revRange] …`",
    doc = "",
    doc = "`p4 [g-opts] print -U unloadfile …`"
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] print [-a -A -k -q] [-m max] [-o outfile] file[revRange] ...`",
    doc = "",
    doc = "`p4 [g-opts] print -U unloadfile ...`"
)]
#[cfg_attr(
    all(feature = "lt2022_1", not(feature = "lt2018_1")),
    doc = "`p4 [g-opts] print [-a -A -k -q] [-m max] [-o outfile] FileSpec[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] print -U unload[FileSpec]`"
)]
#[cfg_attr(
    all(feature = "lt2023_1", not(feature = "lt2022_1")),
    doc = "`p4 [g-opts] print [-a -A -K -q] [-m max] --offset bytesToSkip --size bytesToPrint [-o outfile] FileSpec[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] print -U unload[FileSpec]`"
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2023_1")),
    doc = "`p4 print [-a -A -K -o localFile -q -m max --offset offset --size size -Q charset -B utf8bom -L line-ending] file[revRange] ...`",
    doc = "",
    doc = "`p4 print -U unloadfile ...`"
)]
#[cfg_attr(
    all(feature = "lt2026_1", not(feature = "lt2024_2")),
    doc = "`p4 print [-a -A -K -o localFile -q -m max --offset offset --size size -Q charset -B utf8bom -L line-ending] file[revRange] ...`",
    doc = "",
    doc = "`p4 print -U unloadfile ...`",
    doc = "",
    doc = "`p4 print -T attribute [-a -q -o localFile] file ...`"
)]
#[cfg_attr(
    not(feature = "lt2026_1"),
    doc = "`p4 print [-a -A -K -o localFile -q -m max --offset offset --size size -Q charset -B utf8bom -L line-ending] [--ignore-changeview] file[revRange] ...`",
    doc = "",
    doc = "`p4 print -U unloadfile ...`",
    doc = "",
    doc = "`p4 print -T attribute [-a -q -o localFile] [--ignore-changeview] file ...`"
)]
///
/// Print the contents of a depot file revision.
#[derive(Debug, Clone, Default)]
pub struct Print {
    bin: PathBuf,

    global_opts: GlobalOpts,

    all_revisions: bool,

    archive_depots: bool,

    suppress_keyword_expansion: bool,

    #[cfg(not(feature = "lt2023_1"))]
    line_ending: Option<LineEnding>,

    #[cfg(not(feature = "lt2023_1"))]
    charset: Option<String>,

    #[cfg(not(feature = "lt2023_1"))]
    utf8bom: Option<String>,

    limit: Option<u64>,

    #[cfg(not(feature = "lt2022_1"))]
    offset: Option<u64>,

    #[cfg(not(feature = "lt2022_1"))]
    size: Option<u64>,

    redirect_output: Option<PathBuf>,

    quiet_mode: bool,

    unload_depot: bool,

    #[cfg(not(feature = "lt2024_2"))]
    attribute: Option<String>,

    #[cfg(not(feature = "lt2026_1"))]
    ignore_changeview: bool,
}

impl SubCommand for Print {
    fn name(&self) -> &str {
        "print"
    }

    fn inject_local_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2024_2"))]
        {
            if let Some(attribute) = &self.attribute {
                command.arg("-T").arg(attribute);
            }
        }

        if self.all_revisions {
            command.arg("-a");
        }

        if self.archive_depots {
            command.arg("-A");
        }

        if self.suppress_keyword_expansion {
            #[cfg(feature = "lt2022_1")]
            {
                command.arg("-k");
            }
            #[cfg(not(feature = "lt2022_1"))]
            {
                command.arg("-K");
            }
        }

        if let Some(output) = &self.redirect_output {
            command.arg("-o").arg(output);
        }

        if self.quiet_mode {
            command.arg("-q");
        }

        if let Some(max) = self.limit {
            command.arg("-m").arg(max.to_string());
        }

        #[cfg(not(feature = "lt2022_1"))]
        {
            if let Some(offset) = self.offset {
                command.arg("--offset").arg(offset.to_string());
            }

            if let Some(size) = self.size {
                command.arg("--size").arg(size.to_string());
            }
        }

        #[cfg(not(feature = "lt2023_1"))]
        {
            if let Some(charset) = &self.charset {
                command.arg("-Q").arg(charset);
            }

            if let Some(utf8bom) = &self.utf8bom {
                command.arg("-B").arg(utf8bom);
            }

            if let Some(line_ending) = &self.line_ending {
                command.arg("-L").arg(line_ending.as_str());
            }
        }

        if self.unload_depot {
            command.arg("-U");
        }

        #[cfg(not(feature = "lt2026_1"))]
        {
            if self.ignore_changeview {
                command.arg("--ignore-changeview");
            }
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl Print {
    /// Creates a new `p4 print` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Spawns `p4 print` for the given files as a child process.
    ///
    /// The child process inherits the standard input, output, and error
    /// streams of the current process, and runs asynchronously; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 print` for the given files to completion and captures its
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
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn get_all_revisions(&self) -> bool {
        self.all_revisions
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn set_all_revisions(&mut self, v: bool) -> &mut Self {
        self.all_revisions = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn all_revisions(mut self, v: bool) -> Self {
        self.all_revisions = v;
        self
    }

    /// # Description
    ///
    /// -A
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Attempt to print a file stored in an archive depot."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "Print files in archive depots.")]
    pub fn get_archive_depots(&self) -> bool {
        self.archive_depots
    }

    /// # Description
    ///
    /// -A
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Attempt to print a file stored in an archive depot."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "Print files in archive depots.")]
    pub fn set_archive_depots(&mut self, v: bool) -> &mut Self {
        self.archive_depots = v;
        self
    }

    /// # Description
    ///
    /// -A
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Attempt to print a file stored in an archive depot."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "Print files in archive depots.")]
    pub fn archive_depots(mut self, v: bool) -> Self {
        self.archive_depots = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2022_1", doc = "-k")]
    #[cfg_attr(not(feature = "lt2022_1"), doc = "-K")]
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "Suppress RCS keyword expansion.")]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Suppress RCS keyword expansion. This replaced the `-k` flag in",
        doc = "2022.1, which is now an alias for `-K` for backwards compatibility."
    )]
    pub fn get_suppress_keyword_expansion(&self) -> bool {
        self.suppress_keyword_expansion
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2022_1", doc = "-k")]
    #[cfg_attr(not(feature = "lt2022_1"), doc = "-K")]
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "Suppress RCS keyword expansion.")]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Suppress RCS keyword expansion. This replaced the `-k` flag in",
        doc = "2022.1, which is now an alias for `-K` for backwards compatibility."
    )]
    pub fn set_suppress_keyword_expansion(&mut self, v: bool) -> &mut Self {
        self.suppress_keyword_expansion = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2022_1", doc = "-k")]
    #[cfg_attr(not(feature = "lt2022_1"), doc = "-K")]
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "Suppress RCS keyword expansion.")]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Suppress RCS keyword expansion. This replaced the `-k` flag in",
        doc = "2022.1, which is now an alias for `-K` for backwards compatibility."
    )]
    pub fn suppress_keyword_expansion(mut self, v: bool) -> Self {
        self.suppress_keyword_expansion = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "-o outfile")]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "-o localfile")]
    ///
    #[cfg_attr(
        feature = "lt2018_2",
        doc = "Redirect output to the specified output file on the local disk,",
        doc = "preserving the same file type, attributes, and/or permission bits",
        doc = "as the original file in the depot."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2018_2")),
        doc = "Redirect output to the specified output file (`outfile`) on the",
        doc = "local disk. This preserves the same file type, attributes, and/or",
        doc = "permission bits as the original file (`FileSpec`) in the depot.",
        doc = "Multiple files can be written by using wildcards in the",
        doc = "`localFile` argument that match wildcards in the depot",
        doc = "(`FileSpec`) argument. For example: To print the contents of a",
        doc = "directory and directories under that directory, use the `...`",
        doc = "wildcard: `p4 print -o c:/tmp/main-copy/... //depot/main/...` To",
        doc = "print all files that match readme.txt or readme.pdf, you might",
        doc = "specify `p4 print -o readme.* //depot/readme.*`"
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Redirect output to the specified output file (`localfile`) on the",
        doc = "local disk. This preserves the same file type, attributes, and/or",
        doc = "permission bits as the original file (`FileSpec`) in the depot.",
        doc = "Multiple files can be written by using wildcards in the",
        doc = "`localFile` argument that match wildcards in the depot",
        doc = "(`FileSpec`) argument. For example: To print the contents of a",
        doc = "directory and directories under that directory, use the `...`",
        doc = "wildcard: `p4 print -o c:/tmp/main-copy/... //depot/main/...` To",
        doc = "print all files that match readme.txt or readme.pdf, you might",
        doc = "specify `p4 print -o readme.* //depot/readme.*`"
    )]
    pub fn get_redirect_output(&self) -> Option<&PathBuf> {
        self.redirect_output.as_ref()
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "-o outfile")]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "-o localfile")]
    ///
    #[cfg_attr(
        feature = "lt2018_2",
        doc = "Redirect output to the specified output file on the local disk,",
        doc = "preserving the same file type, attributes, and/or permission bits",
        doc = "as the original file in the depot."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2018_2")),
        doc = "Redirect output to the specified output file (`outfile`) on the",
        doc = "local disk. This preserves the same file type, attributes, and/or",
        doc = "permission bits as the original file (`FileSpec`) in the depot.",
        doc = "Multiple files can be written by using wildcards in the",
        doc = "`localFile` argument that match wildcards in the depot",
        doc = "(`FileSpec`) argument. For example: To print the contents of a",
        doc = "directory and directories under that directory, use the `...`",
        doc = "wildcard: `p4 print -o c:/tmp/main-copy/... //depot/main/...` To",
        doc = "print all files that match readme.txt or readme.pdf, you might",
        doc = "specify `p4 print -o readme.* //depot/readme.*`"
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Redirect output to the specified output file (`localfile`) on the",
        doc = "local disk. This preserves the same file type, attributes, and/or",
        doc = "permission bits as the original file (`FileSpec`) in the depot.",
        doc = "Multiple files can be written by using wildcards in the",
        doc = "`localFile` argument that match wildcards in the depot",
        doc = "(`FileSpec`) argument. For example: To print the contents of a",
        doc = "directory and directories under that directory, use the `...`",
        doc = "wildcard: `p4 print -o c:/tmp/main-copy/... //depot/main/...` To",
        doc = "print all files that match readme.txt or readme.pdf, you might",
        doc = "specify `p4 print -o readme.* //depot/readme.*`"
    )]
    pub fn set_redirect_output(&mut self, v: impl Into<PathBuf>) -> &mut Self {
        self.redirect_output = Some(v.into());
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "-o outfile")]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "-o localfile")]
    ///
    #[cfg_attr(
        feature = "lt2018_2",
        doc = "Redirect output to the specified output file on the local disk,",
        doc = "preserving the same file type, attributes, and/or permission bits",
        doc = "as the original file in the depot."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2018_2")),
        doc = "Redirect output to the specified output file (`outfile`) on the",
        doc = "local disk. This preserves the same file type, attributes, and/or",
        doc = "permission bits as the original file (`FileSpec`) in the depot.",
        doc = "Multiple files can be written by using wildcards in the",
        doc = "`localFile` argument that match wildcards in the depot",
        doc = "(`FileSpec`) argument. For example: To print the contents of a",
        doc = "directory and directories under that directory, use the `...`",
        doc = "wildcard: `p4 print -o c:/tmp/main-copy/... //depot/main/...` To",
        doc = "print all files that match readme.txt or readme.pdf, you might",
        doc = "specify `p4 print -o readme.* //depot/readme.*`"
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Redirect output to the specified output file (`localfile`) on the",
        doc = "local disk. This preserves the same file type, attributes, and/or",
        doc = "permission bits as the original file (`FileSpec`) in the depot.",
        doc = "Multiple files can be written by using wildcards in the",
        doc = "`localFile` argument that match wildcards in the depot",
        doc = "(`FileSpec`) argument. For example: To print the contents of a",
        doc = "directory and directories under that directory, use the `...`",
        doc = "wildcard: `p4 print -o c:/tmp/main-copy/... //depot/main/...` To",
        doc = "print all files that match readme.txt or readme.pdf, you might",
        doc = "specify `p4 print -o readme.* //depot/readme.*`"
    )]
    pub fn redirect_output(mut self, v: impl Into<PathBuf>) -> Self {
        self.redirect_output = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -q
    ///
    /// Suppress the one-line file header normally added by
    #[cfg_attr(feature = "lt2018_1", doc = "Perforce.")]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Helix Server."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server."
    )]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "P4 Server.")]
    pub fn get_quiet_mode(&self) -> bool {
        self.quiet_mode
    }

    /// # Description
    ///
    /// -q
    ///
    /// Suppress the one-line file header normally added by
    #[cfg_attr(feature = "lt2018_1", doc = "Perforce.")]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Helix Server."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server."
    )]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "P4 Server.")]
    pub fn set_quiet_mode(&mut self, v: bool) -> &mut Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    /// Suppress the one-line file header normally added by
    #[cfg_attr(feature = "lt2018_1", doc = "Perforce.")]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Helix Server."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server."
    )]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "P4 Server.")]
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// --offset bytesToSkip
    ///
    /// (Optional) Skip the specified number of bytes and only print what
    /// follows. Can be used with `--size`.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn get_offset(&self) -> Option<u64> {
        self.offset
    }

    /// # Description
    ///
    /// --offset bytesToSkip
    ///
    /// (Optional) Skip the specified number of bytes and only print what
    /// follows. Can be used with `--size`.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn set_offset(&mut self, v: u64) -> &mut Self {
        self.offset = Some(v);
        self
    }

    /// # Description
    ///
    /// --offset bytesToSkip
    ///
    /// (Optional) Skip the specified number of bytes and only print what
    /// follows. Can be used with `--size`.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn offset(mut self, v: u64) -> Self {
        self.offset = Some(v);
        self
    }

    /// # Description
    ///
    /// --size bytesToPrint
    ///
    /// (Optional) Print the specified number of bytes from the offset. If
    /// `--offset` is not explicitly set, prints the specified number of bytes
    /// from the beginning of the file.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn get_size(&self) -> Option<u64> {
        self.size
    }

    /// # Description
    ///
    /// --size bytesToPrint
    ///
    /// (Optional) Print the specified number of bytes from the offset. If
    /// `--offset` is not explicitly set, prints the specified number of bytes
    /// from the beginning of the file.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn set_size(&mut self, v: u64) -> &mut Self {
        self.size = Some(v);
        self
    }

    /// # Description
    ///
    /// --size bytesToPrint
    ///
    /// (Optional) Print the specified number of bytes from the offset. If
    /// `--offset` is not explicitly set, prints the specified number of bytes
    /// from the beginning of the file.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn size(mut self, v: u64) -> Self {
        self.size = Some(v);
        self
    }

    /// # Description
    ///
    /// -Q charset
    ///
    /// Allow the charset for unicode type files to be explicitly specified,
    /// overriding the connection's `P4CHARSET` but not overriding the charset
    /// of unicode files with versioned charsets.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }

    /// # Description
    ///
    /// -Q charset
    ///
    /// Allow the charset for unicode type files to be explicitly specified,
    /// overriding the connection's `P4CHARSET` but not overriding the charset
    /// of unicode files with versioned charsets.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_charset(&mut self, v: impl Into<String>) -> &mut Self {
        self.charset = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -Q charset
    ///
    /// Allow the charset for unicode type files to be explicitly specified,
    /// overriding the connection's `P4CHARSET` but not overriding the charset
    /// of unicode files with versioned charsets.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn charset(mut self, v: impl Into<String>) -> Self {
        self.charset = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -B utf8bom
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_utf8bom(&self) -> Option<&str> {
        self.utf8bom.as_deref()
    }

    /// # Description
    ///
    /// -B utf8bom
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_utf8bom(&mut self, v: impl Into<String>) -> &mut Self {
        self.utf8bom = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -B utf8bom
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn utf8bom(mut self, v: impl Into<String>) -> Self {
        self.utf8bom = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -L line-ending
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    ///
    /// Returns the line ending currently set, or `None` if no line ending is
    /// set.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_line_ending(&self) -> Option<LineEnding> {
        self.line_ending
    }

    /// # Description
    ///
    /// -L unix
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_line_ending_unix(&mut self) -> &mut Self {
        self.line_ending = Some(LineEnding::Unix);
        self
    }

    /// # Description
    ///
    /// -L unix
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn line_ending_unix(mut self) -> Self {
        self.line_ending = Some(LineEnding::Unix);
        self
    }

    /// # Description
    ///
    /// -L win
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_line_ending_win(&mut self) -> &mut Self {
        self.line_ending = Some(LineEnding::Win);
        self
    }

    /// # Description
    ///
    /// -L win
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn line_ending_win(mut self) -> Self {
        self.line_ending = Some(LineEnding::Win);
        self
    }

    /// # Description
    ///
    /// -L mac
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_line_ending_mac(&mut self) -> &mut Self {
        self.line_ending = Some(LineEnding::Mac);
        self
    }

    /// # Description
    ///
    /// -L mac
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn line_ending_mac(mut self) -> Self {
        self.line_ending = Some(LineEnding::Mac);
        self
    }

    /// # Description
    ///
    /// -T attribute
    ///
    /// Print the value of the specified non-encoded attribute of the specified
    /// file. This command, rather than `p4 fstat -Oa`, is appropriate for
    /// non-encoded binary attributes larger than 250 megabytes. The
    /// `p4 fstat -Oa` command might fail with the `Rpc buffer too big to send`
    /// error if attributes exceed 250 megabytes. The `p4 print` command has no
    /// option to show the value of the attribute in hex. For that, use the
    /// `p4 fstat -Oe` command.
    #[cfg(not(feature = "lt2024_2"))]
    pub fn get_attribute(&self) -> Option<&str> {
        self.attribute.as_deref()
    }

    /// # Description
    ///
    /// -T attribute
    ///
    /// Print the value of the specified non-encoded attribute of the specified
    /// file. This command, rather than `p4 fstat -Oa`, is appropriate for
    /// non-encoded binary attributes larger than 250 megabytes. The
    /// `p4 fstat -Oa` command might fail with the `Rpc buffer too big to send`
    /// error if attributes exceed 250 megabytes. The `p4 print` command has no
    /// option to show the value of the attribute in hex. For that, use the
    /// `p4 fstat -Oe` command.
    #[cfg(not(feature = "lt2024_2"))]
    pub fn set_attribute(&mut self, v: impl Into<String>) -> &mut Self {
        self.attribute = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -T attribute
    ///
    /// Print the value of the specified non-encoded attribute of the specified
    /// file. This command, rather than `p4 fstat -Oa`, is appropriate for
    /// non-encoded binary attributes larger than 250 megabytes. The
    /// `p4 fstat -Oa` command might fail with the `Rpc buffer too big to send`
    /// error if attributes exceed 250 megabytes. The `p4 print` command has no
    /// option to show the value of the attribute in hex. For that, use the
    /// `p4 fstat -Oe` command.
    #[cfg(not(feature = "lt2024_2"))]
    pub fn attribute(mut self, v: impl Into<String>) -> Self {
        self.attribute = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -U
    ///
    /// Look for the specified file or files in the unload depot. Data about an
    /// unloaded client, label, or task stream can be printed.
    pub fn get_unload_depot(&self) -> bool {
        self.unload_depot
    }

    /// # Description
    ///
    /// -U
    ///
    /// Look for the specified file or files in the unload depot. Data about an
    /// unloaded client, label, or task stream can be printed.
    pub fn set_unload_depot(&mut self, v: bool) -> &mut Self {
        self.unload_depot = v;
        self
    }

    /// # Description
    ///
    /// -U
    ///
    /// Look for the specified file or files in the unload depot. Data about an
    /// unloaded client, label, or task stream can be printed.
    pub fn unload_depot(mut self, v: bool) -> Self {
        self.unload_depot = v;
        self
    }

    /// # Description
    ///
    /// --ignore-changeview
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// `p4 client`.
    #[cfg(not(feature = "lt2026_1"))]
    pub fn get_ignore_changeview(&self) -> bool {
        self.ignore_changeview
    }

    /// # Description
    ///
    /// --ignore-changeview
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// `p4 client`.
    #[cfg(not(feature = "lt2026_1"))]
    pub fn set_ignore_changeview(&mut self, v: bool) -> &mut Self {
        self.ignore_changeview = v;
        self
    }

    /// # Description
    ///
    /// --ignore-changeview
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// `p4 client`.
    #[cfg(not(feature = "lt2026_1"))]
    pub fn ignore_changeview(mut self, v: bool) -> Self {
        self.ignore_changeview = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 print` command line; no process is
    /// spawned.
    #[test]
    fn without_options() {
        let print = Print::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&print.setup_command("p4")), ["print"]);
    }

    #[test]
    fn all_local_options() {
        let mut print = Print::new("p4", GlobalOpts::new());
        print
            .set_all_revisions(true)
            .set_archive_depots(true)
            .set_suppress_keyword_expansion(true)
            .set_redirect_output("out.bin")
            .set_quiet_mode(true)
            .set_limit(5);
        #[cfg(not(feature = "lt2022_1"))]
        print.set_offset(100).set_size(200);
        #[cfg(not(feature = "lt2023_1"))]
        print.set_charset("utf8").set_utf8bom("on");
        #[cfg(not(feature = "lt2023_1"))]
        print.set_line_ending_unix();
        #[cfg(not(feature = "lt2024_2"))]
        print.set_attribute("attr1");
        print.set_unload_depot(true);
        #[cfg(not(feature = "lt2026_1"))]
        print.set_ignore_changeview(true);

        let mut expected = vec!["print"];
        #[cfg(not(feature = "lt2024_2"))]
        expected.extend(["-T", "attr1"]);
        expected.extend(["-a", "-A"]);
        #[cfg(feature = "lt2022_1")]
        expected.push("-k");
        #[cfg(not(feature = "lt2022_1"))]
        expected.push("-K");
        expected.extend(["-o", "out.bin", "-q", "-m", "5"]);
        #[cfg(not(feature = "lt2022_1"))]
        expected.extend(["--offset", "100", "--size", "200"]);
        #[cfg(not(feature = "lt2023_1"))]
        expected.extend(["-Q", "utf8", "-B", "on", "-L", "unix"]);
        expected.push("-U");
        #[cfg(not(feature = "lt2026_1"))]
        expected.push("--ignore-changeview");

        assert_eq!(args_of(&print.setup_command("p4")), expected);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn line_ending_is_passed_as_separate_args() {
        let mut print = Print::new("p4", GlobalOpts::new());
        print.set_line_ending_win();

        assert_eq!(print.get_line_ending(), Some(LineEnding::Win));
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-L", "win"]);
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn attribute_is_passed_as_separate_args() {
        let mut print = Print::new("p4", GlobalOpts::new());
        print.set_attribute("desc");

        assert_eq!(print.get_attribute(), Some("desc"));
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-T", "desc"]);
    }

    #[cfg(not(feature = "lt2026_1"))]
    #[test]
    fn ignore_changeview_is_passed_as_separate_args() {
        let mut print = Print::new("p4", GlobalOpts::new());
        print.set_ignore_changeview(true);

        assert!(print.get_ignore_changeview());
        assert_eq!(
            args_of(&print.setup_command("p4")),
            ["print", "--ignore-changeview"]
        );
    }
}
