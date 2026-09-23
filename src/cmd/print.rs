use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Standard print mode of `p4 print`: print the contents of depot file
/// revisions.
///
/// Entered by setting any standard option (such as [`Print::all_revisions`])
/// from the [`Unselected`] state.
#[derive(Debug, Clone, Default)]
pub struct StandardPrintMode {
    all_revisions: bool,

    from_archive_depots: bool,

    suppress_keyword_expansion: bool,

    #[cfg(not(feature = "lt2023_1"))]
    line_ending: Option<LineEnding>,

    #[cfg(not(feature = "lt2023_1"))]
    charset: Option<String>,

    #[cfg(not(feature = "lt2023_1"))]
    utf8bom: Option<Utf8Bom>,

    limit: Option<u64>,

    #[cfg(not(feature = "lt2022_1"))]
    offset: Option<u64>,

    #[cfg(not(feature = "lt2022_1"))]
    size: Option<u64>,

    redirect_output: Option<PathBuf>,

    quiet_mode: bool,

    #[cfg(not(feature = "lt2026_1"))]
    ignore_changeview: bool,
}

impl ExclusiveOption for StandardPrintMode {
    fn inject_args(&self, command: &mut Command) {
        if self.all_revisions {
            command.arg("-a");
        }

        if self.from_archive_depots {
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
                command.arg("-B").arg(utf8bom.as_str());
            }

            if let Some(line_ending) = &self.line_ending {
                command.arg("-L").arg(line_ending.as_str());
            }
        }

        #[cfg(not(feature = "lt2026_1"))]
        {
            if self.ignore_changeview {
                command.arg("--ignore-changeview");
            }
        }
    }
}

/// Unload depot mode of `p4 print` (`-U`): look for the specified files
/// in the unload depot.
///
/// Entered with [`Print::from_unload_depot`]. No other local option can be
/// combined with `-U`.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnloadDepotMode;

impl ExclusiveOption for UnloadDepotMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-U");
    }
}

/// Attribute trait mode of `p4 print` (`-T attribute`): print the value of
/// the specified non-encoded attribute of the specified file.
///
/// Only `-a`, `-q`, `-o`, and (before 2026.1) `--ignore-changeview` can be
/// combined with `-T`. Entered with [`Print::extract_attribute`].
#[cfg(not(feature = "lt2024_2"))]
#[derive(Debug, Clone, Default)]
pub struct AttributeTraitMode {
    all_revisions: bool,

    redirect_output: Option<PathBuf>,

    quiet_mode: bool,

    attribute: String,

    #[cfg(not(feature = "lt2026_1"))]
    ignore_changeview: bool,
}

#[cfg(not(feature = "lt2024_2"))]
impl ExclusiveOption for AttributeTraitMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-T").arg(&self.attribute);

        if self.all_revisions {
            command.arg("-a");
        }

        if let Some(output) = &self.redirect_output {
            command.arg("-o").arg(output);
        }

        if self.quiet_mode {
            command.arg("-q");
        }

        #[cfg(not(feature = "lt2026_1"))]
        {
            if self.ignore_changeview {
                command.arg("--ignore-changeview");
            }
        }
    }
}

// https://help.perforce.com/helix-core/integrations-plugins/p4jenkins/current/Content/P4Jenkins/unicode.html
/// The `-B` setting controlling the byte-order-mark in utf8 files.
#[cfg(not(feature = "lt2023_1"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf8Bom {
    /// Do not write a BOM (`-B 0`).
    No,
    /// Write utf8 files with a BOM (`-B 1`).
    Yes,
    /// Write the BOM only on Windows (`-B 2`).
    WindowsOnly,
}

#[cfg(not(feature = "lt2023_1"))]
impl Utf8Bom {
    /// Returns the command-line value for this BOM setting.
    pub fn as_str(&self) -> &'static str {
        match self {
            Utf8Bom::No => "0",
            Utf8Bom::Yes => "1",
            Utf8Bom::WindowsOnly => "2",
        }
    }
}

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
///
#[cfg_attr(
    feature = "lt2024_2",
    doc = "The `M` type parameter tracks which of the two forms of the command is in use at compile time. The default [`Unselected`] state prints files without local options; setting any standard option such as [`Self::all_revisions`] transitions to [`StandardPrintMode`], and [`Self::from_unload_depot`] transitions to [`UnloadDepotMode`]."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "The `M` type parameter tracks which of the three forms of the command is in use at compile time. The default [`Unselected`] state prints files without local options; setting any standard option such as [`Self::all_revisions`] transitions to [`StandardPrintMode`], [`Self::from_unload_depot`] transitions to [`UnloadDepotMode`], and [`Self::extract_attribute`] transitions to [`AttributeTraitMode`]."
)]
#[derive(Debug, Clone, Default)]
pub struct Print<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

impl Print<Unselected> {
    /// Creates a new `p4 print` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    /// -U
    ///
    /// Look for the specified file or files in the unload depot. Data about an
    /// unloaded client, label, or task stream can be printed.
    ///
    /// Transitions this command to the [`UnloadDepotMode`] state; no other
    /// local option can be combined with `-U`.
    pub fn from_unload_depot(self) -> Print<UnloadDepotMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: UnloadDepotMode,
        }
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
    ///
    /// Transitions this command to the [`AttributeTraitMode`] state.
    #[cfg(not(feature = "lt2024_2"))]
    pub fn extract_attribute(self, name: impl Into<String>) -> Print<AttributeTraitMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: AttributeTraitMode {
                all_revisions: false,
                redirect_output: None,
                quiet_mode: false,
                attribute: name.into(),
                #[cfg(not(feature = "lt2026_1"))]
                ignore_changeview: false,
            },
        }
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    pub fn all_revisions(self, v: bool) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                all_revisions: v,
                ..StandardPrintMode::default()
            },
        }
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
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    pub fn archive_depots(self, v: bool) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                from_archive_depots: v,
                ..StandardPrintMode::default()
            },
        }
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
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    pub fn suppress_keyword_expansion(self, v: bool) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                suppress_keyword_expansion: v,
                ..StandardPrintMode::default()
            },
        }
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
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    pub fn redirect_output(self, v: impl Into<PathBuf>) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                redirect_output: Some(v.into()),
                ..StandardPrintMode::default()
            },
        }
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
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    pub fn quiet_mode(self, v: bool) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                quiet_mode: v,
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    pub fn limit(self, v: u64) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                limit: Some(v),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// --offset bytesToSkip
    ///
    /// (Optional) Skip the specified number of bytes and only print what
    /// follows. Can be used with `--size`.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn offset(self, v: u64) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                offset: Some(v),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// --size bytesToPrint
    ///
    /// (Optional) Print the specified number of bytes from the offset. If
    /// `--offset` is not explicitly set, prints the specified number of bytes
    /// from the beginning of the file.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn size(self, v: u64) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                size: Some(v),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -Q charset
    ///
    /// Allow the charset for unicode type files to be explicitly specified,
    /// overriding the connection's `P4CHARSET` but not overriding the charset
    /// of unicode files with versioned charsets.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn charset(self, v: impl Into<String>) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                charset: Some(v.into()),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -B 1 / -B 0
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files. With `true`,
    /// write utf8 files with a BOM (`-B 1`); with `false`, do not write a
    /// BOM (`-B 0`).
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn write_utf8bom(self, v: bool) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                utf8bom: Some(if v { Utf8Bom::Yes } else { Utf8Bom::No }),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -B 2
    ///
    /// Override the client-side `filesys.utf8bom` setting to write the BOM
    /// only on Windows.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn write_utf8bom_windows_only(self) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                utf8bom: Some(Utf8Bom::WindowsOnly),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -L unix
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn line_ending_unix(self) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                line_ending: Some(LineEnding::Unix),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -L win
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn line_ending_win(self) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                line_ending: Some(LineEnding::Win),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// -L mac
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn line_ending_mac(self) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                line_ending: Some(LineEnding::Mac),
                ..StandardPrintMode::default()
            },
        }
    }

    /// # Description
    ///
    /// --ignore-changeview
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// `p4 client`.
    ///
    /// Transitions this command to the [`StandardPrintMode`] state.
    #[cfg(not(feature = "lt2026_1"))]
    pub fn ignore_changeview(self, v: bool) -> Print<StandardPrintMode> {
        Print {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StandardPrintMode {
                ignore_changeview: v,
                ..StandardPrintMode::default()
            },
        }
    }
}

impl<M: ExclusiveOption, S, I> ParameterizedSpawn<(S,)> for Print<M>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 print` for the given files as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with(&mut self, (files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<M: ExclusiveOption> Print<M> {
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

impl Print<StandardPrintMode> {
    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn get_all_revisions(&self) -> bool {
        self.mode.all_revisions
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn set_all_revisions(&mut self, v: bool) -> &mut Self {
        self.mode.all_revisions = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn all_revisions(mut self, v: bool) -> Self {
        self.mode.all_revisions = v;
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
        self.mode.from_archive_depots
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
        self.mode.from_archive_depots = v;
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
        self.mode.from_archive_depots = v;
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
        self.mode.suppress_keyword_expansion
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
        self.mode.suppress_keyword_expansion = v;
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
        self.mode.suppress_keyword_expansion = v;
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
        self.mode.redirect_output.as_ref()
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
        self.mode.redirect_output = Some(v.into());
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
        self.mode.redirect_output = Some(v.into());
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
        self.mode.quiet_mode
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
        self.mode.quiet_mode = v;
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
        self.mode.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    pub fn get_limit(&self) -> Option<u64> {
        self.mode.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.mode.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Print only the first *max* files.
    pub fn limit(mut self, v: u64) -> Self {
        self.mode.limit = Some(v);
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
        self.mode.offset
    }

    /// # Description
    ///
    /// --offset bytesToSkip
    ///
    /// (Optional) Skip the specified number of bytes and only print what
    /// follows. Can be used with `--size`.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn set_offset(&mut self, v: u64) -> &mut Self {
        self.mode.offset = Some(v);
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
        self.mode.offset = Some(v);
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
        self.mode.size
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
        self.mode.size = Some(v);
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
        self.mode.size = Some(v);
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
        self.mode.charset.as_deref()
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
        self.mode.charset = Some(v.into());
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
        self.mode.charset = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -B utf8bom
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_utf8bom(&self) -> Option<Utf8Bom> {
        self.mode.utf8bom
    }

    /// # Description
    ///
    /// -B 1 / -B 0
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files. With `true`,
    /// write utf8 files with a BOM (`-B 1`); with `false`, do not write a
    /// BOM (`-B 0`).
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_write_utf8bom(&mut self, v: bool) -> &mut Self {
        self.mode.utf8bom = Some(if v { Utf8Bom::Yes } else { Utf8Bom::No });
        self
    }

    /// # Description
    ///
    /// -B 1 / -B 0
    ///
    /// Override the client-side `filesys.utf8bom` setting, controlling the
    /// presence of the byte-order-mark in utf8 type files. With `true`,
    /// write utf8 files with a BOM (`-B 1`); with `false`, do not write a
    /// BOM (`-B 0`).
    #[cfg(not(feature = "lt2023_1"))]
    pub fn write_utf8bom(mut self, v: bool) -> Self {
        self.mode.utf8bom = Some(if v { Utf8Bom::Yes } else { Utf8Bom::No });
        self
    }

    /// # Description
    ///
    /// -B 2
    ///
    /// Override the client-side `filesys.utf8bom` setting to write the BOM
    /// only on Windows.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_write_utf8bom_windows_only(&mut self) -> &mut Self {
        self.mode.utf8bom = Some(Utf8Bom::WindowsOnly);
        self
    }

    /// # Description
    ///
    /// -B 2
    ///
    /// Override the client-side `filesys.utf8bom` setting to write the BOM
    /// only on Windows.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn write_utf8bom_windows_only(mut self) -> Self {
        self.mode.utf8bom = Some(Utf8Bom::WindowsOnly);
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
        self.mode.line_ending
    }

    /// # Description
    ///
    /// -L unix
    ///
    /// Allow the line ending of textual files to be explicitly specified as
    /// 'unix', 'win', or 'mac'.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_line_ending_unix(&mut self) -> &mut Self {
        self.mode.line_ending = Some(LineEnding::Unix);
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
        self.mode.line_ending = Some(LineEnding::Unix);
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
        self.mode.line_ending = Some(LineEnding::Win);
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
        self.mode.line_ending = Some(LineEnding::Win);
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
        self.mode.line_ending = Some(LineEnding::Mac);
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
        self.mode.line_ending = Some(LineEnding::Mac);
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
        self.mode.ignore_changeview
    }

    /// # Description
    ///
    /// --ignore-changeview
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// `p4 client`.
    #[cfg(not(feature = "lt2026_1"))]
    pub fn set_ignore_changeview(&mut self, v: bool) -> &mut Self {
        self.mode.ignore_changeview = v;
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
        self.mode.ignore_changeview = v;
        self
    }
}

#[cfg(not(feature = "lt2024_2"))]
impl Print<AttributeTraitMode> {
    /// # Description
    ///
    /// -T attribute
    ///
    /// Print the value of the specified non-encoded attribute of the specified
    /// file.
    pub fn get_attribute(&self) -> &str {
        &self.mode.attribute
    }

    /// # Description
    ///
    /// -T attribute
    ///
    /// Print the value of the specified non-encoded attribute of the specified
    /// file.
    pub fn set_attribute(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.attribute = v.into();
        self
    }

    /// # Description
    ///
    /// -T attribute
    ///
    /// Print the value of the specified non-encoded attribute of the specified
    /// file.
    pub fn attribute(mut self, v: impl Into<String>) -> Self {
        self.mode.attribute = v.into();
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn get_all_revisions(&self) -> bool {
        self.mode.all_revisions
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn set_all_revisions(&mut self, v: bool) -> &mut Self {
        self.mode.all_revisions = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, print all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn all_revisions(mut self, v: bool) -> Self {
        self.mode.all_revisions = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "-o outfile")]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "-o localfile")]
    ///
    /// Redirect output to the specified output file on the local disk,
    /// preserving the same file type, attributes, and/or permission bits as
    /// the original file in the depot.
    pub fn get_redirect_output(&self) -> Option<&PathBuf> {
        self.mode.redirect_output.as_ref()
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "-o outfile")]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "-o localfile")]
    ///
    /// Redirect output to the specified output file on the local disk,
    /// preserving the same file type, attributes, and/or permission bits as
    /// the original file in the depot.
    pub fn set_redirect_output(&mut self, v: impl Into<PathBuf>) -> &mut Self {
        self.mode.redirect_output = Some(v.into());
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2023_1", doc = "-o outfile")]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "-o localfile")]
    ///
    /// Redirect output to the specified output file on the local disk,
    /// preserving the same file type, attributes, and/or permission bits as
    /// the original file in the depot.
    pub fn redirect_output(mut self, v: impl Into<PathBuf>) -> Self {
        self.mode.redirect_output = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -q
    ///
    /// Suppress the one-line file header normally added by the Helix Core
    /// Server.
    pub fn get_quiet_mode(&self) -> bool {
        self.mode.quiet_mode
    }

    /// # Description
    ///
    /// -q
    ///
    /// Suppress the one-line file header normally added by the Helix Core
    /// Server.
    pub fn set_quiet_mode(&mut self, v: bool) -> &mut Self {
        self.mode.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    /// Suppress the one-line file header normally added by the Helix Core
    /// Server.
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.mode.quiet_mode = v;
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
        self.mode.ignore_changeview
    }

    /// # Description
    ///
    /// --ignore-changeview
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// `p4 client`.
    #[cfg(not(feature = "lt2026_1"))]
    pub fn set_ignore_changeview(&mut self, v: bool) -> &mut Self {
        self.mode.ignore_changeview = v;
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
        self.mode.ignore_changeview = v;
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Print<M> {
    fn name(&self) -> &str {
        "print"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
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
    fn standard_mode_all_options() {
        let print = Print::new("p4", GlobalOpts::new())
            .all_revisions(true)
            .archive_depots(true)
            .suppress_keyword_expansion(true)
            .redirect_output("out.bin")
            .quiet_mode(true)
            .limit(5);
        #[cfg(not(feature = "lt2022_1"))]
        let print = print.offset(100).size(200);
        #[cfg(not(feature = "lt2023_1"))]
        let print = print.charset("utf8").write_utf8bom(true).line_ending_unix();
        #[cfg(not(feature = "lt2026_1"))]
        let print = print.ignore_changeview(true);

        let mut expected = vec!["print", "-a", "-A"];
        #[cfg(feature = "lt2022_1")]
        expected.push("-k");
        #[cfg(not(feature = "lt2022_1"))]
        expected.push("-K");
        expected.extend(["-o", "out.bin", "-q", "-m", "5"]);
        #[cfg(not(feature = "lt2022_1"))]
        expected.extend(["--offset", "100", "--size", "200"]);
        #[cfg(not(feature = "lt2023_1"))]
        expected.extend(["-Q", "utf8", "-B", "1", "-L", "unix"]);
        #[cfg(not(feature = "lt2026_1"))]
        expected.push("--ignore-changeview");

        assert_eq!(args_of(&print.setup_command("p4")), expected);
    }

    #[test]
    fn standard_mode_set_style() {
        let mut print = Print::new("p4", GlobalOpts::new()).limit(5);
        print.set_all_revisions(true).set_quiet_mode(true);
        #[cfg(not(feature = "lt2022_1"))]
        print.set_offset(100).set_size(200);

        let expected = {
            #[cfg_attr(feature = "lt2022_1", allow(unused_mut))]
            let mut v = vec!["print", "-a", "-q", "-m", "5"];
            #[cfg(not(feature = "lt2022_1"))]
            v.extend(["--offset", "100", "--size", "200"]);
            v
        };

        assert_eq!(args_of(&print.setup_command("p4")), expected);
    }

    #[test]
    fn unload_depot_mode() {
        let print = Print::new("p4", GlobalOpts::new()).from_unload_depot();

        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-U"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn line_ending_is_passed_as_separate_args() {
        let print = Print::new("p4", GlobalOpts::new()).line_ending_win();

        assert_eq!(print.get_line_ending(), Some(LineEnding::Win));
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-L", "win"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn write_utf8bom_bool() {
        let yes = Print::new("p4", GlobalOpts::new()).write_utf8bom(true);
        assert_eq!(yes.get_utf8bom(), Some(Utf8Bom::Yes));
        assert_eq!(args_of(&yes.setup_command("p4")), ["print", "-B", "1"]);

        let no = Print::new("p4", GlobalOpts::new()).write_utf8bom(false);
        assert_eq!(no.get_utf8bom(), Some(Utf8Bom::No));
        assert_eq!(args_of(&no.setup_command("p4")), ["print", "-B", "0"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn write_utf8bom_windows_only() {
        let print = Print::new("p4", GlobalOpts::new()).write_utf8bom_windows_only();

        assert_eq!(print.get_utf8bom(), Some(Utf8Bom::WindowsOnly));
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-B", "2"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn utf8bom_set_style_can_replace() {
        let mut print = Print::new("p4", GlobalOpts::new()).write_utf8bom(true);
        print.set_write_utf8bom(false);
        assert_eq!(print.get_utf8bom(), Some(Utf8Bom::No));

        print.set_write_utf8bom_windows_only();
        assert_eq!(print.get_utf8bom(), Some(Utf8Bom::WindowsOnly));
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-B", "2"]);
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn attribute_trait_mode() {
        let print = Print::new("p4", GlobalOpts::new()).extract_attribute("desc");

        assert_eq!(print.get_attribute(), "desc");
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-T", "desc"]);
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn attribute_trait_mode_with_allowed_options() {
        let print = Print::new("p4", GlobalOpts::new())
            .extract_attribute("desc")
            .all_revisions(true)
            .quiet_mode(true)
            .redirect_output("out.bin");
        #[cfg(not(feature = "lt2026_1"))]
        let print = print.ignore_changeview(true);

        let mut expected = vec!["print", "-T", "desc", "-a", "-o", "out.bin", "-q"];
        #[cfg(not(feature = "lt2026_1"))]
        expected.push("--ignore-changeview");

        assert_eq!(args_of(&print.setup_command("p4")), expected);
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn attribute_name_can_be_replaced() {
        let mut print = Print::new("p4", GlobalOpts::new()).extract_attribute("old");
        print.set_attribute("new");

        assert_eq!(print.get_attribute(), "new");
        assert_eq!(args_of(&print.setup_command("p4")), ["print", "-T", "new"]);
    }

    #[cfg(not(feature = "lt2026_1"))]
    #[test]
    fn ignore_changeview_standard_mode() {
        let print = Print::new("p4", GlobalOpts::new()).ignore_changeview(true);

        assert!(print.get_ignore_changeview());
        assert_eq!(
            args_of(&print.setup_command("p4")),
            ["print", "--ignore-changeview"]
        );
    }
}
