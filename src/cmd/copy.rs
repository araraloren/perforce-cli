use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Mode structs
// ---------------------------------------------------------------------------

/// The `-r` (reverse) state of the branch form.
///
/// When selected, the mappings in the branch view are reversed, integrating
/// from the target files to the source files.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BranchReverse;

/// The branch form: `p4 copy -b branch [-r] [toFile[rev] ...]`.
///
/// Uses a branch view to determine source and target files. The `B` type
/// parameter tracks whether `-r` has been supplied: [`Unselected`] is the
/// forward direction; [`BranchReverse`] reverses the mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch<B = Unselected> {
    branch: String,
    reverse: B,
}

impl ExclusiveOption for Branch<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-b").arg(&self.branch);
    }
}

impl ExclusiveOption for Branch<BranchReverse> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-b").arg(&self.branch).arg("-r");
    }
}

/// The branch-override form: `p4 copy -b branch -s fromFile[rev] [toFile ...]`.
///
/// Treats `fromFile` as the source and both sides of the branch view as the
/// target. Overrides the `-r` option if specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchOverride {
    branch: String,
    source: String,
}

impl ExclusiveOption for BranchOverride {
    fn inject_args(&self, command: &mut Command) {
        command
            .arg("-b")
            .arg(&self.branch)
            .arg("-s")
            .arg(&self.source);
    }
}

/// The `-Af` (files only) state of the stream form.
#[cfg(not(feature = "lt2020_2"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CopyFilesOnly;

/// The `-As` (stream spec only) state of the stream form.
#[cfg(not(feature = "lt2020_2"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CopyStreamSpecOnly;

/// The stream form: `p4 copy -S stream [-Af | -As] [-P parent] [-F] [-r]
/// [toFile[rev] ...]`.
///
/// Uses a generated branch view that maps the stream to its parent and copies
/// from a stream to its parent.
#[cfg_attr(
    not(feature = "lt2020_2"),
    doc = "The `S` type parameter tracks whether `-Af` (files only) or `-As` (stream spec only) has been supplied: [`Unselected`] copies both; [`CopyFilesOnly`] copies only the files; [`CopyStreamSpecOnly`] copies only the stream spec."
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream<S = Unselected> {
    stream: String,

    parent: Option<String>,

    reverse: bool,

    force_stream_flow: bool,

    strategy: S,
}

impl<S> Stream<S> {
    fn inject_stream_args(&self, command: &mut Command) {
        command.arg("-S").arg(&self.stream);
        if let Some(parent) = &self.parent {
            command.arg("-P").arg(parent);
        }
        if self.force_stream_flow {
            command.arg("-F");
        }
        if self.reverse {
            command.arg("-r");
        }
    }
}

impl ExclusiveOption for Stream<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        self.inject_stream_args(command);
    }
}

#[cfg(not(feature = "lt2020_2"))]
impl ExclusiveOption for Stream<CopyFilesOnly> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-Af");
        self.inject_stream_args(command);
    }
}

#[cfg(not(feature = "lt2020_2"))]
impl ExclusiveOption for Stream<CopyStreamSpecOnly> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-As");
        self.inject_stream_args(command);
    }
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 copy`.
///
/// Copy files and/or the stream spec from one location in the depot to
/// another.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] copy [-c change] [-n -f -K -v -q] [-m max] fromFile[rev] toFile
/// p4 [g-opts] copy [-c change] [-n -f -K -v -q] [-m max] -b branch [-r] [toFile[rev] ...]
/// p4 [g-opts] copy [-c change] [-n -f -K -v -q] [-m max] -b branch -s fromFile[rev] [toFile ...]
/// p4 [g-opts] copy [-c change] [-n -f -K -v -q] [-m max] -S stream [-P parent] [-F] [-r] [toFile[rev] ...]
/// p4 [g-opts] copy [options] -S stream [-Af | -As] [-P parent] [-F] [-r] [toFile[rev] ...]
/// ```
///
/// The `M` type parameter tracks the command form at compile time, starting
/// as [`Unselected`] from [`Copy::new`]. The [`Unselected`] state is the
/// direct form `p4 copy fromFile[rev] toFile`: spawn it with the two file
/// arguments to copy without a branch or stream mapping. Enter a branch or
/// stream form with one of [`Self::branch`] or [`Self::stream`]:
/// - [`Branch`] (entered via [`Self::branch`]): use a branch view;
///   `Copy::reverse` supplies `-r` and transitions to the [`BranchReverse`]
///   state; `Copy::source` supplies `-s fromFile[rev]` and transitions to the
///   [`BranchOverride`] state.
#[cfg_attr(
    feature = "lt2020_2",
    doc = "- [`Stream`] (entered via [`Self::stream`]): use a stream's generated branch view."
)]
#[cfg_attr(
    not(feature = "lt2020_2"),
    doc = "- [`Stream`] (entered via [`Self::stream`]): use a stream's generated branch view; `Copy::files_only` supplies `-Af` and `Copy::stream_spec_only` supplies `-As`, each transitioning to the corresponding state."
)]
#[derive(Debug, Clone, Default)]
pub struct Copy<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    changelist: Option<String>,

    limit: Option<u64>,

    preview: bool,

    force: bool,

    quiet: bool,

    virtual_integration: bool,

    #[cfg(not(feature = "lt2022_1"))]
    suppress_keyword_expansion: bool,

    mode: M,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

impl Copy<Unselected> {
    /// Creates a new `p4 copy` command in the direct form
    /// `p4 copy fromFile[rev] toFile`.
    ///
    /// `bin` is the path to the Perforce command-line executable. In this
    /// state, spawn with the `fromFile[rev]` and `toFile` arguments via
    /// [`ParameterizedSpawn::spawn_with`]; use [`Self::branch`] or
    /// [`Self::stream`] to enter a branch or stream form.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Enters the branch form: `p4 copy -b branch [-r] [toFile[rev] ...]`.
    ///
    /// Uses the named branch view to determine source and target files. Use
    /// `Copy::reverse` to supply `-r`, or `Copy::source` to supply `-s
    /// fromFile[rev]` (which transitions to the [`BranchOverride`] form).
    pub fn branch(self, v: impl Into<String>) -> Copy<Branch<Unselected>> {
        Copy {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            limit: self.limit,
            preview: self.preview,
            force: self.force,
            quiet: self.quiet,
            virtual_integration: self.virtual_integration,
            #[cfg(not(feature = "lt2022_1"))]
            suppress_keyword_expansion: self.suppress_keyword_expansion,
            mode: Branch {
                branch: v.into(),
                reverse: Unselected,
            },
        }
    }

    /// Enters the stream form: `p4 copy -S stream [-P parent] [-F] [-r]
    /// [toFile[rev] ...]`.
    ///
    #[cfg_attr(feature = "lt2020_2", doc = "Uses the stream's generated branch view.")]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Uses a generated branch view that maps the stream to its parent. Use `Copy::files_only` (`-Af`) or `Copy::stream_spec_only` (`-As`) to restrict what is copied."
    )]
    pub fn stream(self, v: impl Into<String>) -> Copy<Stream<Unselected>> {
        Copy {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            limit: self.limit,
            preview: self.preview,
            force: self.force,
            quiet: self.quiet,
            virtual_integration: self.virtual_integration,
            #[cfg(not(feature = "lt2022_1"))]
            suppress_keyword_expansion: self.suppress_keyword_expansion,
            mode: Stream {
                stream: v.into(),
                parent: None,
                reverse: false,
                force_stream_flow: false,
                strategy: Unselected,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Shared accessors (available in every mode)
// ---------------------------------------------------------------------------

impl<M> Copy<M> {
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
    /// -c change
    ///
    /// Open the files in the specified pending changelist rather than in the
    /// default changelist.
    pub fn get_changelist(&self) -> Option<&String> {
        self.changelist.as_ref()
    }

    /// # Description
    ///
    /// -c change
    ///
    /// Open the files in the specified pending changelist rather than in the
    /// default changelist.
    pub fn set_changelist(&mut self, v: impl Into<String>) -> &mut Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -c change
    ///
    /// Open the files in the specified pending changelist rather than in the
    /// default changelist.
    pub fn changelist(mut self, v: impl Into<String>) -> Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Specify the maximum number of files to copy, to limit the size of the
    /// operation.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Specify the maximum number of files to copy, to limit the size of the
    /// operation.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Specify the maximum number of files to copy, to limit the size of the
    /// operation.
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview the copy.
    pub fn get_preview(&self) -> bool {
        self.preview
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview the copy.
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Preview the copy.
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the creation of extra revisions in order to explicitly record
    /// that files have been copied. Deleted source files are copied if they do
    /// not exist in the target, and files that are already identical are
    /// copied if they are not connected by existing integration records.
    pub fn get_force(&self) -> bool {
        self.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the creation of extra revisions in order to explicitly record
    /// that files have been copied. Deleted source files are copied if they do
    /// not exist in the target, and files that are already identical are
    /// copied if they are not connected by existing integration records.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the creation of extra revisions in order to explicitly record
    /// that files have been copied. Deleted source files are copied if they do
    /// not exist in the target, and files that are already identical are
    /// copied if they are not connected by existing integration records.
    pub fn force(mut self, v: bool) -> Self {
        self.force = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress normal output messages about the list of",
        doc = "files being integrated, copied, or merged. Messages regarding",
        doc = "errors or exceptional conditions are displayed."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses normal output messages about the list of",
        doc = "files being integrated, copied, or merged. Messages regarding errors",
        doc = "or exceptional conditions are displayed."
    )]
    pub fn get_quiet(&self) -> bool {
        self.quiet
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress normal output messages about the list of",
        doc = "files being integrated, copied, or merged. Messages regarding",
        doc = "errors or exceptional conditions are displayed."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses normal output messages about the list of",
        doc = "files being integrated, copied, or merged. Messages regarding errors",
        doc = "or exceptional conditions are displayed."
    )]
    pub fn set_quiet(&mut self, v: bool) -> &mut Self {
        self.quiet = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress normal output messages about the list of",
        doc = "files being integrated, copied, or merged. Messages regarding",
        doc = "errors or exceptional conditions are displayed."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses normal output messages about the list of",
        doc = "files being integrated, copied, or merged. Messages regarding errors",
        doc = "or exceptional conditions are displayed."
    )]
    pub fn quiet(mut self, v: bool) -> Self {
        self.quiet = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Do not sync the target files. By default, p4 copy syncs the target files.",
        doc = "If a large number of files is involved and you do not require",
        doc = "the files to be present in your workspace, you can minimize",
        doc = "overhead and network traffic by specifying `-v`."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Performs a \"virtual\" integration that does not modify client workspace",
        doc = "files unless target files need to be resolved. After submitting a",
        doc = "\"virtual integration\", `p4 sync` can be used to update the workspace."
    )]
    pub fn get_virtual_integration(&self) -> bool {
        self.virtual_integration
    }

    /// # Description
    ///
    /// -v
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Do not sync the target files. By default, p4 copy syncs the target files.",
        doc = "If a large number of files is involved and you do not require",
        doc = "the files to be present in your workspace, you can minimize",
        doc = "overhead and network traffic by specifying `-v`."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Performs a \"virtual\" integration that does not modify client workspace",
        doc = "files unless target files need to be resolved. After submitting a",
        doc = "\"virtual integration\", `p4 sync` can be used to update the workspace."
    )]
    pub fn set_virtual_integration(&mut self, v: bool) -> &mut Self {
        self.virtual_integration = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    #[cfg_attr(
        feature = "lt2025_2",
        doc = "Do not sync the target files. By default, p4 copy syncs the target files.",
        doc = "If a large number of files is involved and you do not require",
        doc = "the files to be present in your workspace, you can minimize",
        doc = "overhead and network traffic by specifying `-v`."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "Performs a \"virtual\" integration that does not modify client workspace",
        doc = "files unless target files need to be resolved. After submitting a",
        doc = "\"virtual integration\", `p4 sync` can be used to update the workspace."
    )]
    pub fn virtual_integration(mut self, v: bool) -> Self {
        self.virtual_integration = v;
        self
    }

    /// # Description
    ///
    /// -K
    ///
    /// Suppress keyword expansion when updating +k type files on the client.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn get_suppress_keyword_expansion(&self) -> bool {
        self.suppress_keyword_expansion
    }

    /// # Description
    ///
    /// -K
    ///
    /// Suppress keyword expansion when updating +k type files on the client.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn set_suppress_keyword_expansion(&mut self, v: bool) -> &mut Self {
        self.suppress_keyword_expansion = v;
        self
    }

    /// # Description
    ///
    /// -K
    ///
    /// Suppress keyword expansion when updating +k type files on the client.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn suppress_keyword_expansion(mut self, v: bool) -> Self {
        self.suppress_keyword_expansion = v;
        self
    }
}

// ---------------------------------------------------------------------------
// Branch<Unselected>: reverse + source transitions + branch accessor
// ---------------------------------------------------------------------------

impl Copy<Branch<Unselected>> {
    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn get_branch(&self) -> &str {
        &self.mode.branch
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn set_branch(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.branch = v.into();
        self
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn branch(mut self, v: impl Into<String>) -> Self {
        self.mode.branch = v.into();
        self
    }

    /// # Description
    ///
    /// -r
    ///
    /// Reverse the mappings in the branch view, integrating from the target
    /// files to the source files.
    #[cfg_attr(feature = "lt2014_2", doc = "Requires the `-b` flag.")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "Requires the `-b` option.")]
    ///
    /// Transitions this command to the [`BranchReverse`] state.
    pub fn reverse(self) -> Copy<Branch<BranchReverse>> {
        Copy {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            limit: self.limit,
            preview: self.preview,
            force: self.force,
            quiet: self.quiet,
            virtual_integration: self.virtual_integration,
            #[cfg(not(feature = "lt2022_1"))]
            suppress_keyword_expansion: self.suppress_keyword_expansion,
            mode: Branch {
                branch: self.mode.branch,
                reverse: BranchReverse,
            },
        }
    }

    /// # Description
    ///
    /// -s `fromFile[rev]`
    ///
    /// Treat `fromFile` as the source and both sides of the branch view as the
    /// target. To restrict the scope of the target further, specify the
    /// optional `toFile` parameter.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Overrides the `-r` flag, if specified. Requires `-b`."
    )]
    #[cfg_attr(
        not(feature = "lt2014_2"),
        doc = "Overrides the `-r` option, if specified. Requires `-b`."
    )]
    ///
    /// Transitions this command to the [`BranchOverride`] state.
    pub fn source(self, v: impl Into<String>) -> Copy<BranchOverride> {
        Copy {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            limit: self.limit,
            preview: self.preview,
            force: self.force,
            quiet: self.quiet,
            virtual_integration: self.virtual_integration,
            #[cfg(not(feature = "lt2022_1"))]
            suppress_keyword_expansion: self.suppress_keyword_expansion,
            mode: BranchOverride {
                branch: self.mode.branch,
                source: v.into(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Branch<BranchReverse>: branch accessor
// ---------------------------------------------------------------------------

impl Copy<Branch<BranchReverse>> {
    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn get_branch(&self) -> &str {
        &self.mode.branch
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn set_branch(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.branch = v.into();
        self
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn branch(mut self, v: impl Into<String>) -> Self {
        self.mode.branch = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// BranchOverride: branch + source accessors
// ---------------------------------------------------------------------------

impl Copy<BranchOverride> {
    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn get_branch(&self) -> &str {
        &self.mode.branch
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn set_branch(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.branch = v.into();
        self
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// The branch view used to determine source and target files.
    pub fn branch(mut self, v: impl Into<String>) -> Self {
        self.mode.branch = v.into();
        self
    }

    /// # Description
    ///
    /// -s `fromFile[rev]`
    ///
    /// Treat `fromFile` as the source and both sides of the branch view as the
    /// target. To restrict the scope of the target further, specify the
    /// optional `toFile` parameter.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Overrides the `-r` flag, if specified. Requires `-b`."
    )]
    #[cfg_attr(
        not(feature = "lt2014_2"),
        doc = "Overrides the `-r` option, if specified. Requires `-b`."
    )]
    pub fn get_source(&self) -> &str {
        &self.mode.source
    }

    /// # Description
    ///
    /// -s `fromFile[rev]`
    ///
    /// Treat `fromFile` as the source and both sides of the branch view as the
    /// target. To restrict the scope of the target further, specify the
    /// optional `toFile` parameter.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Overrides the `-r` flag, if specified. Requires `-b`."
    )]
    #[cfg_attr(
        not(feature = "lt2014_2"),
        doc = "Overrides the `-r` option, if specified. Requires `-b`."
    )]
    pub fn set_source(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.source = v.into();
        self
    }

    /// # Description
    ///
    /// -s `fromFile[rev]`
    ///
    /// Treat `fromFile` as the source and both sides of the branch view as the
    /// target. To restrict the scope of the target further, specify the
    /// optional `toFile` parameter.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Overrides the `-r` flag, if specified. Requires `-b`."
    )]
    #[cfg_attr(
        not(feature = "lt2014_2"),
        doc = "Overrides the `-r` option, if specified. Requires `-b`."
    )]
    pub fn source(mut self, v: impl Into<String>) -> Self {
        self.mode.source = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// Stream<S>: shared stream accessors + strategy transitions
// ---------------------------------------------------------------------------

impl<S> Copy<Stream<S>> {
    /// # Description
    ///
    /// -S stream
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Specify the source stream. Changes are copied to its parent. You can use the",
        doc = "`-r` flag to reverse direction. To submit copied stream files,",
        doc = "the current client must be switched to the target stream or to a virtual child",
        doc = "stream of the target stream."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2014_2")),
        doc = "Specify the source stream. Changes are copied to its parent. You",
        doc = "can use the `-r` option to reverse direction. To",
        doc = "submit copied stream files, the current client must be switched",
        doc = "to the target stream or to a virtual child stream of the target",
        doc = "stream."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Causes p4 copy to use a generated branch view that maps the stream to its parent and copy from a stream to its parent. To reverse the copy direction, use `-r` with `-S`.",
        doc = "",
        doc = "Copies the source stream spec propagatable fields into the target stream spec and opens for edit both the target stream spec and the stream files."
    )]
    pub fn get_stream(&self) -> &str {
        &self.mode.stream
    }

    /// # Description
    ///
    /// -S stream
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Specify the source stream. Changes are copied to its parent. You can use the",
        doc = "`-r` flag to reverse direction. To submit copied stream files,",
        doc = "the current client must be switched to the target stream or to a virtual child",
        doc = "stream of the target stream."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2014_2")),
        doc = "Specify the source stream. Changes are copied to its parent. You",
        doc = "can use the `-r` option to reverse direction. To",
        doc = "submit copied stream files, the current client must be switched",
        doc = "to the target stream or to a virtual child stream of the target",
        doc = "stream."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Causes p4 copy to use a generated branch view that maps the stream to its parent and copy from a stream to its parent. To reverse the copy direction, use `-r` with `-S`.",
        doc = "",
        doc = "Copies the source stream spec propagatable fields into the target stream spec and opens for edit both the target stream spec and the stream files."
    )]
    pub fn set_stream(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.stream = v.into();
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Specify the source stream. Changes are copied to its parent. You can use the",
        doc = "`-r` flag to reverse direction. To submit copied stream files,",
        doc = "the current client must be switched to the target stream or to a virtual child",
        doc = "stream of the target stream."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2014_2")),
        doc = "Specify the source stream. Changes are copied to its parent. You",
        doc = "can use the `-r` option to reverse direction. To",
        doc = "submit copied stream files, the current client must be switched",
        doc = "to the target stream or to a virtual child stream of the target",
        doc = "stream."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Causes p4 copy to use a generated branch view that maps the stream to its parent and copy from a stream to its parent. To reverse the copy direction, use `-r` with `-S`.",
        doc = "",
        doc = "Copies the source stream spec propagatable fields into the target stream spec and opens for edit both the target stream spec and the stream files."
    )]
    pub fn stream(mut self, v: impl Into<String>) -> Self {
        self.mode.stream = v.into();
        self
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// Specify a target stream other than the parent of the source stream.
    /// Requires `-S`.
    pub fn get_parent(&self) -> Option<&String> {
        self.mode.parent.as_ref()
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// Specify a target stream other than the parent of the source stream.
    /// Requires `-S`.
    pub fn set_parent(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.parent = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// Specify a target stream other than the parent of the source stream.
    /// Requires `-S`.
    pub fn parent(mut self, v: impl Into<String>) -> Self {
        self.mode.parent = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -r
    ///
    /// Reverse the copy direction. With `-S`, copies from the parent stream to
    /// the source stream.
    pub fn get_reverse(&self) -> bool {
        self.mode.reverse
    }

    /// # Description
    ///
    /// -r
    ///
    /// Reverse the copy direction. With `-S`, copies from the parent stream to
    /// the source stream.
    pub fn set_reverse(&mut self, v: bool) -> &mut Self {
        self.mode.reverse = v;
        self
    }

    /// # Description
    ///
    /// -r
    ///
    /// Reverse the copy direction. With `-S`, copies from the parent stream to
    /// the source stream.
    pub fn reverse(mut self, v: bool) -> Self {
        self.mode.reverse = v;
        self
    }

    /// # Description
    ///
    /// -F
    ///
    /// Force copy operation; perform the operation when the target stream is
    /// not configured to accept a copy of the source.
    pub fn get_force_stream_flow(&self) -> bool {
        self.mode.force_stream_flow
    }

    /// # Description
    ///
    /// -F
    ///
    /// Force copy operation; perform the operation when the target stream is
    /// not configured to accept a copy of the source.
    pub fn set_force_stream_flow(&mut self, v: bool) -> &mut Self {
        self.mode.force_stream_flow = v;
        self
    }

    /// # Description
    ///
    /// -F
    ///
    /// Force copy operation; perform the operation when the target stream is
    /// not configured to accept a copy of the source.
    pub fn force_stream_flow(mut self, v: bool) -> Self {
        self.mode.force_stream_flow = v;
        self
    }
}

// Stream<Unselected>: strategy transitions
#[cfg(not(feature = "lt2020_2"))]
impl Copy<Stream<Unselected>> {
    /// # Description
    ///
    /// -Af
    ///
    /// Copy the files only (not the stream spec).
    ///
    /// Transitions this command to the [`CopyFilesOnly`] state.
    pub fn files_only(self) -> Copy<Stream<CopyFilesOnly>> {
        Copy {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            limit: self.limit,
            preview: self.preview,
            force: self.force,
            quiet: self.quiet,
            virtual_integration: self.virtual_integration,
            #[cfg(not(feature = "lt2022_1"))]
            suppress_keyword_expansion: self.suppress_keyword_expansion,
            mode: Stream {
                stream: self.mode.stream,
                parent: self.mode.parent,
                reverse: self.mode.reverse,
                force_stream_flow: self.mode.force_stream_flow,
                strategy: CopyFilesOnly,
            },
        }
    }

    /// # Description
    ///
    /// -As
    ///
    /// Copy the stream spec only (not the files).
    ///
    /// Transitions this command to the [`CopyStreamSpecOnly`] state.
    pub fn stream_spec_only(self) -> Copy<Stream<CopyStreamSpecOnly>> {
        Copy {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: self.changelist,
            limit: self.limit,
            preview: self.preview,
            force: self.force,
            quiet: self.quiet,
            virtual_integration: self.virtual_integration,
            #[cfg(not(feature = "lt2022_1"))]
            suppress_keyword_expansion: self.suppress_keyword_expansion,
            mode: Stream {
                stream: self.mode.stream,
                parent: self.mode.parent,
                reverse: self.mode.reverse,
                force_stream_flow: self.mode.force_stream_flow,
                strategy: CopyStreamSpecOnly,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> SubCommand for Copy<M> {
    fn name(&self) -> &str {
        "copy"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(changelist) = &self.changelist {
            command.arg("-c").arg(changelist);
        }

        if self.preview {
            command.arg("-n");
        }

        if self.force {
            command.arg("-f");
        }

        #[cfg(not(feature = "lt2022_1"))]
        if self.suppress_keyword_expansion {
            command.arg("-K");
        }

        if self.virtual_integration {
            command.arg("-v");
        }

        if self.quiet {
            command.arg("-q");
        }

        if let Some(limit) = self.limit {
            command.arg("-m").arg(limit.to_string());
        }

        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

// Direct form: exactly 2 args (fromFile, toFile).
impl<A, B> ParameterizedSpawn<(A, B)> for Copy<Unselected>
where
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy fromFile[rev] toFile` with the given source and target
    /// file arguments as a child process with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (from_file, to_file): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(from_file)
            .arg(to_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Branch forms (forward + reverse): 0 or more toFile args.
impl<B> ParameterizedSpawn<()> for Copy<Branch<B>>
where
    Branch<B>: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy -b branch [-r]` without target file arguments as a
    /// child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<B, S, I> ParameterizedSpawn<(S,)> for Copy<Branch<B>>
where
    Branch<B>: ExclusiveOption,
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy -b branch [-r] [toFile[rev] ...]` for the given target
    /// files as a child process with piped standard output and error streams;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (to_files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(to_files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// BranchOverride form: 0 or more toFile args.
impl ParameterizedSpawn<()> for Copy<BranchOverride> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy -b branch -s fromFile[rev]` without target file
    /// arguments as a child process with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S, I> ParameterizedSpawn<(S,)> for Copy<BranchOverride>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy -b branch -s fromFile[rev] [toFile ...]` for the given
    /// target files as a child process with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (to_files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(to_files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Stream forms: 0 or more toFile args.
impl<S> ParameterizedSpawn<()> for Copy<Stream<S>>
where
    Stream<S>: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy -S stream [...]` without target file arguments as a
    /// child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S, T, I> ParameterizedSpawn<(T,)> for Copy<Stream<S>>
where
    Stream<S>: ExclusiveOption,
    T: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 copy -S stream [...] [toFile[rev] ...]` for the given
    /// target files as a child process with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (to_files,): (T,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(to_files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut command = Copy::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    #[cfg(not(feature = "lt2022_1"))]
    #[test]
    fn direct_form_assembles_options_and_args() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .changelist("123")
            .limit(10)
            .preview(true)
            .force(true)
            .quiet(true)
            .virtual_integration(true)
            .suppress_keyword_expansion(true);
        let mut cmd = copy.setup_command("p4");
        cmd.arg("//depot/main/...").arg("//depot/dev/...");
        assert_eq!(
            args_of(&cmd),
            [
                "copy",
                "-c",
                "123",
                "-n",
                "-f",
                "-K",
                "-v",
                "-q",
                "-m",
                "10",
                "//depot/main/...",
                "//depot/dev/..."
            ]
        );
    }

    #[cfg(feature = "lt2022_1")]
    #[test]
    fn direct_form_assembles_options_and_args() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .changelist("123")
            .limit(10)
            .preview(true)
            .force(true)
            .quiet(true)
            .virtual_integration(true);
        let mut cmd = copy.setup_command("p4");
        cmd.arg("//depot/main/...").arg("//depot/dev/...");
        assert_eq!(
            args_of(&cmd),
            [
                "copy",
                "-c",
                "123",
                "-n",
                "-f",
                "-v",
                "-q",
                "-m",
                "10",
                "//depot/main/...",
                "//depot/dev/..."
            ]
        );
    }

    #[test]
    fn branch_form() {
        let copy = Copy::new("p4", GlobalOpts::new()).branch("mybranch");
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-b", "mybranch"]
        );
    }

    #[test]
    fn branch_reverse_form() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .branch("mybranch")
            .reverse();
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-b", "mybranch", "-r"]
        );
    }

    #[test]
    fn branch_override_form() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .branch("mybranch")
            .source("//depot/main/...");
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-b", "mybranch", "-s", "//depot/main/..."]
        );
    }

    #[test]
    fn stream_form() {
        let copy = Copy::new("p4", GlobalOpts::new()).stream("//stream/dev");
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-S", "//stream/dev"]
        );
    }

    #[test]
    fn stream_with_parent_and_flags() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .stream("//stream/dev")
            .parent("//stream/main")
            .reverse(true)
            .force_stream_flow(true);
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            [
                "copy",
                "-S",
                "//stream/dev",
                "-P",
                "//stream/main",
                "-F",
                "-r"
            ]
        );
    }

    #[cfg(not(feature = "lt2020_2"))]
    #[test]
    fn stream_files_only() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .stream("//stream/dev")
            .files_only();
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-Af", "-S", "//stream/dev"]
        );
    }

    #[cfg(not(feature = "lt2020_2"))]
    #[test]
    fn stream_spec_only() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .stream("//stream/dev")
            .stream_spec_only();
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-As", "-S", "//stream/dev"]
        );
    }

    #[test]
    fn common_options_preserved_across_transitions() {
        let copy = Copy::new("p4", GlobalOpts::new())
            .changelist("5")
            .preview(true)
            .branch("mybranch");
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["copy", "-c", "5", "-n", "-b", "mybranch"]
        );
    }

    #[test]
    fn branch_accessors() {
        let mut copy = Copy::new("p4", GlobalOpts::new()).branch("old");
        assert_eq!(copy.get_branch(), "old");
        copy.set_branch("new");
        assert_eq!(copy.get_branch(), "new");
        let copy = copy.branch("newer");
        assert_eq!(copy.get_branch(), "newer");
    }

    #[test]
    fn stream_accessors() {
        let mut copy = Copy::new("p4", GlobalOpts::new())
            .stream("//stream/dev")
            .parent("//stream/main");
        assert_eq!(copy.get_stream(), "//stream/dev");
        assert_eq!(copy.get_parent(), Some(&String::from("//stream/main")));
        assert!(!copy.get_reverse());
        assert!(!copy.get_force_stream_flow());

        copy.set_reverse(true);
        copy.set_force_stream_flow(true);
        assert!(copy.get_reverse());
        assert!(copy.get_force_stream_flow());
    }

    #[test]
    fn global_opts_are_injected_once() {
        let copy = Copy::new("p4", GlobalOpts::new().port("localhost:1666"));
        assert_eq!(
            args_of(&copy.setup_command("p4")),
            ["-p", "localhost:1666", "copy"]
        );
    }
}
