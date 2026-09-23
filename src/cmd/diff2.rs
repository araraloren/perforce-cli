use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{DiffOptions, ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The `-b branch` sub-mode of the [`DepotContent`] state: diff files in
/// two branched codelines through a branch mapping.
///
/// Entered with [`Diff2::branch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchMode {
    branch: String,
}

impl ExclusiveOption for BranchMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-b").arg(&self.branch);
    }
}

/// The `-S stream` sub-mode of the [`DepotContent`] state: diff a stream
/// with its parent.
///
/// Entered with [`Diff2::stream`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamMode {
    stream: String,

    parent: Option<String>,
}

impl ExclusiveOption for StreamMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-S").arg(&self.stream);

        if let Some(parent) = &self.parent {
            command.arg("-P").arg(parent);
        }
    }
}

/// Stream spec mode of `p4 diff2` (`-As`): diff two arbitrary stream specs
/// against each other.
///
/// Entered with [`Diff2::stream_spec_mode`]; the two stream specs to compare
/// are passed as the arguments of the spawned command. As the `-As` form
/// accepts only `-doptions` besides g-opts, this mode offers neither the
/// depot content options nor a sub-mode.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamSpecMode;

impl ExclusiveOption for StreamSpecMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-As");
    }
}

/// The depot content modes of `p4 diff2`: the default form comparing two
/// depot paths, or the `-b branch` / `-S stream` sub-modes.
///
/// Entered with [`Diff2::differing_only`], [`Diff2::quiet_mode`],
/// [`Diff2::diff_nontext`], [`Diff2::unified_patch`], [`Diff2::branch`], or
/// [`Diff2::stream`].
///
/// The `M` type parameter tracks the sub-mode at compile time. The default
/// [`Unselected`] state diffs the file pair given as spawn arguments;
/// [`Diff2::branch`] transitions to the [`BranchMode`] sub-mode, and
/// [`Diff2::stream`] transitions to the [`StreamMode`] sub-mode. The
/// `[-Od -q -t -u]` options are shared by all sub-modes and therefore
/// stored here.
#[derive(Debug, Clone, Default)]
pub struct DepotContent<M = Unselected> {
    differing_only: bool,

    quiet_mode: bool,

    diff_nontext: bool,

    unified_patch: bool,

    mode: M,
}

impl<M: ExclusiveOption> ExclusiveOption for DepotContent<M> {
    fn inject_args(&self, command: &mut Command) {
        if self.differing_only {
            command.arg("-Od");
        }

        if self.quiet_mode {
            command.arg("-q");
        }

        if self.diff_nontext {
            command.arg("-t");
        }

        if self.unified_patch {
            command.arg("-u");
        }

        self.mode.inject_args(command);
    }
}

/// `p4 [g-opts] diff2 [-doptions] [-Od -q -t -u] file1[rev] file2[rev]`
///
/// `p4 [g-opts] diff2 [-doptions] [-Od -q -t -u] -b branch [[fromfile[rev]] tofile[rev]]`
///
/// `p4 [g-opts] diff2 [-doptions] [-Od -q -t -u] [-S stream] [-P parent] [[fromfile[rev]] tofile[rev]]`
///
/// `p4 [g-opts] diff2 [-doptions] -As streamname1[@change1] streamname2[@change2]`
///
/// Diff utility for comparing the content at two depot paths. (For
/// comparing workspace content to depot content, see `p4 diff`.) Also
/// compares two arbitrary stream specs with the -As option.
///
/// The `M` type parameter tracks the command mode at compile time. The
/// default [`Unselected`] state diffs the two depot paths given as spawn
/// arguments; [`Self::differing_only`], [`Self::quiet_mode`],
/// [`Self::diff_nontext`], and [`Self::unified_patch`] transition to the
/// [`DepotContent`] state, [`Self::branch`] and [`Self::stream`] transition
/// to the depot content [`BranchMode`] and [`StreamMode`] sub-modes, and
/// [`Self::stream_spec_mode`] transitions to the [`StreamSpecMode`] state.
#[derive(Debug, Clone, Default)]
pub struct Diff2<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    diff_opts: Option<DiffOptions>,

    mode: M,
}

impl Diff2<Unselected> {
    /// Creates a new `p4 diff2` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            diff_opts: None,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    /// -Od
    ///
    /// Limit output to only those files that differ.
    ///
    /// Transitions this command to the [`DepotContent`] state with `-Od`
    /// set according to `v`.
    pub fn differing_only(self, v: bool) -> Diff2<DepotContent<Unselected>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: v,
                quiet_mode: false,
                diff_nontext: false,
                unified_patch: false,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -q
    ///
    /// Quiet diff. Display only the header; if `file1` and `file2` are
    /// identical, display only `file1 - no differing files` as the output.
    ///
    /// Transitions this command to the [`DepotContent`] state with `-q` set
    /// according to `v`.
    pub fn quiet_mode(self, v: bool) -> Diff2<DepotContent<Unselected>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: false,
                quiet_mode: v,
                diff_nontext: false,
                unified_patch: false,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the file revisions even if the file(s) are not of type `text`.
    ///
    /// Transitions this command to the [`DepotContent`] state with `-t` set
    /// according to `v`.
    pub fn diff_nontext(self, v: bool) -> Diff2<DepotContent<Unselected>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: false,
                quiet_mode: false,
                diff_nontext: v,
                unified_patch: false,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -u
    ///
    /// Generate unified output format, showing added and deleted lines with
    /// sufficient context for compatibility with the `patch(1)` utility.
    /// Only those files that differ are included. File names and dates
    /// remain in P4 Server syntax.
    ///
    /// Transitions this command to the [`DepotContent`] state with `-u` set
    /// according to `v`.
    pub fn unified_patch(self, v: bool) -> Diff2<DepotContent<Unselected>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: false,
                quiet_mode: false,
                diff_nontext: false,
                unified_patch: v,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// Use a branch mapping to diff files in two branched codelines. The
    /// files that are compared can be limited by file patterns in either
    /// the `from` or `to` file specifications.
    ///
    /// Transitions this command to the [`DepotContent`] state with the
    /// [`BranchMode`] sub-mode selected and the branch mapping set to
    /// `name`.
    pub fn branch(self, name: impl Into<String>) -> Diff2<DepotContent<BranchMode>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: false,
                quiet_mode: false,
                diff_nontext: false,
                unified_patch: false,
                mode: BranchMode {
                    branch: name.into(),
                },
            },
        }
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Diff a stream with its parent. To diff the stream with a stream
    /// other than its configured parent, specify [`Diff2::parent`] or
    /// [`Diff2::set_parent`].
    ///
    /// Transitions this command to the [`DepotContent`] state with the
    /// [`StreamMode`] sub-mode selected and the stream set to `name`.
    pub fn stream(self, name: impl Into<String>) -> Diff2<DepotContent<StreamMode>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: false,
                quiet_mode: false,
                diff_nontext: false,
                unified_patch: false,
                mode: StreamMode {
                    stream: name.into(),
                    parent: None,
                },
            },
        }
    }

    /// # Description
    ///
    /// -As
    ///
    /// Allows two arbitrary stream specs to be diffed against each other.
    /// Can be used with a streamname, or with a streamname at a specific
    /// changelist number.
    ///
    /// Although this option requires the user have at least the list access
    /// to the stream path, it ignores any other entry in the protections
    /// table, including any minus sign (`-`) that would otherwise block the
    /// operation.
    ///
    /// Transitions this command to the [`StreamSpecMode`] state. The two
    /// stream specs to compare are passed as the arguments of the spawned
    /// command ([`ParameterizedSpawn::spawn_with`]). As the `-As` form
    /// accepts only `-doptions` besides g-opts, this transition is
    /// unavailable once the command has entered the [`DepotContent`] state.
    pub fn stream_spec_mode(self) -> Diff2<StreamSpecMode> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: StreamSpecMode,
        }
    }
}

impl<M> Diff2<M> {
    /// # Description
    ///
    /// -doptions
    ///
    /// Pass options to the underlying diff routine (see Usage notes for
    /// details).
    pub fn get_diff_options(&self) -> Option<&DiffOptions> {
        self.diff_opts.as_ref()
    }

    /// # Description
    ///
    /// -doptions
    ///
    /// Pass options to the underlying diff routine (see Usage notes for
    /// details).
    pub fn set_diff_options(&mut self, v: impl Into<DiffOptions>) -> &mut Self {
        self.diff_opts = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -doptions
    ///
    /// Pass options to the underlying diff routine (see Usage notes for
    /// details).
    pub fn diff_options(mut self, v: impl Into<DiffOptions>) -> Self {
        self.diff_opts = Some(v.into());
        self
    }
}

impl<M: ExclusiveOption> Diff2<M> {
    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn set_global_opts(&mut self, v: GlobalOpts) -> &mut Self {
        self.global_opts = v;
        self
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.global_opts = v;
        self
    }
}

impl<M> Diff2<DepotContent<M>> {
    /// # Description
    ///
    /// -Od
    ///
    /// Limit output to only those files that differ.
    pub fn get_differing_only(&self) -> bool {
        self.mode.differing_only
    }

    /// # Description
    ///
    /// -Od
    ///
    /// Limit output to only those files that differ.
    pub fn set_differing_only(&mut self, v: bool) -> &mut Self {
        self.mode.differing_only = v;
        self
    }

    /// # Description
    ///
    /// -Od
    ///
    /// Limit output to only those files that differ.
    pub fn differing_only(mut self, v: bool) -> Self {
        self.mode.differing_only = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    /// Quiet diff. Display only the header; if `file1` and `file2` are
    /// identical, display only `file1 - no differing files` as the output.
    pub fn get_quiet_mode(&self) -> bool {
        self.mode.quiet_mode
    }

    /// # Description
    ///
    /// -q
    ///
    /// Quiet diff. Display only the header; if `file1` and `file2` are
    /// identical, display only `file1 - no differing files` as the output.
    pub fn set_quiet_mode(&mut self, v: bool) -> &mut Self {
        self.mode.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    /// Quiet diff. Display only the header; if `file1` and `file2` are
    /// identical, display only `file1 - no differing files` as the output.
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.mode.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the file revisions even if the file(s) are not of type `text`.
    pub fn get_diff_nontext(&self) -> bool {
        self.mode.diff_nontext
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the file revisions even if the file(s) are not of type `text`.
    pub fn set_diff_nontext(&mut self, v: bool) -> &mut Self {
        self.mode.diff_nontext = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the file revisions even if the file(s) are not of type `text`.
    pub fn diff_nontext(mut self, v: bool) -> Self {
        self.mode.diff_nontext = v;
        self
    }

    /// # Description
    ///
    /// -u
    ///
    /// Generate unified output format, showing added and deleted lines with
    /// sufficient context for compatibility with the `patch(1)` utility.
    /// Only those files that differ are included. File names and dates
    /// remain in P4 Server syntax.
    pub fn get_unified_patch(&self) -> bool {
        self.mode.unified_patch
    }

    /// # Description
    ///
    /// -u
    ///
    /// Generate unified output format, showing added and deleted lines with
    /// sufficient context for compatibility with the `patch(1)` utility.
    /// Only those files that differ are included. File names and dates
    /// remain in P4 Server syntax.
    pub fn set_unified_patch(&mut self, v: bool) -> &mut Self {
        self.mode.unified_patch = v;
        self
    }

    /// # Description
    ///
    /// -u
    ///
    /// Generate unified output format, showing added and deleted lines with
    /// sufficient context for compatibility with the `patch(1)` utility.
    /// Only those files that differ are included. File names and dates
    /// remain in P4 Server syntax.
    pub fn unified_patch(mut self, v: bool) -> Self {
        self.mode.unified_patch = v;
        self
    }
}

impl Diff2<DepotContent<Unselected>> {
    /// # Description
    ///
    /// -b branch
    ///
    /// Use a branch mapping to diff files in two branched codelines. The
    /// files that are compared can be limited by file patterns in either
    /// the `from` or `to` file specifications.
    ///
    /// Transitions this command to the [`DepotContent`] state with the
    /// [`BranchMode`] sub-mode selected and the branch mapping set to
    /// `name`.
    pub fn branch(self, name: impl Into<String>) -> Diff2<DepotContent<BranchMode>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: self.mode.differing_only,
                quiet_mode: self.mode.quiet_mode,
                diff_nontext: self.mode.diff_nontext,
                unified_patch: self.mode.unified_patch,
                mode: BranchMode {
                    branch: name.into(),
                },
            },
        }
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Diff a stream with its parent. To diff the stream with a stream
    /// other than its configured parent, specify [`Diff2::parent`] or
    /// [`Diff2::set_parent`].
    ///
    /// Transitions this command to the [`DepotContent`] state with the
    /// [`StreamMode`] sub-mode selected and the stream set to `name`.
    pub fn stream(self, name: impl Into<String>) -> Diff2<DepotContent<StreamMode>> {
        Diff2 {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: DepotContent {
                differing_only: self.mode.differing_only,
                quiet_mode: self.mode.quiet_mode,
                diff_nontext: self.mode.diff_nontext,
                unified_patch: self.mode.unified_patch,
                mode: StreamMode {
                    stream: name.into(),
                    parent: None,
                },
            },
        }
    }
}

impl Diff2<DepotContent<BranchMode>> {
    /// # Description
    ///
    /// -b branch
    ///
    /// Use a branch mapping to diff files in two branched codelines. The
    /// files that are compared can be limited by file patterns in either
    /// the `from` or `to` file specifications.
    pub fn get_branch(&self) -> &str {
        &self.mode.mode.branch
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// Use a branch mapping to diff files in two branched codelines. The
    /// files that are compared can be limited by file patterns in either
    /// the `from` or `to` file specifications.
    pub fn set_branch(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.mode.branch = v.into();
        self
    }

    /// # Description
    ///
    /// -b branch
    ///
    /// Use a branch mapping to diff files in two branched codelines. The
    /// files that are compared can be limited by file patterns in either
    /// the `from` or `to` file specifications.
    pub fn branch(mut self, v: impl Into<String>) -> Self {
        self.mode.mode.branch = v.into();
        self
    }
}

impl Diff2<DepotContent<StreamMode>> {
    /// # Description
    ///
    /// -S stream
    ///
    /// Diff a stream with its parent. To diff the stream with a stream
    /// other than its configured parent, specify [`Diff2::parent`] or
    /// [`Diff2::set_parent`].
    pub fn get_stream(&self) -> &str {
        &self.mode.mode.stream
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Diff a stream with its parent. To diff the stream with a stream
    /// other than its configured parent, specify [`Diff2::parent`] or
    /// [`Diff2::set_parent`].
    pub fn set_stream(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.mode.stream = v.into();
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Diff a stream with its parent. To diff the stream with a stream
    /// other than its configured parent, specify [`Diff2::parent`] or
    /// [`Diff2::set_parent`].
    pub fn stream(mut self, v: impl Into<String>) -> Self {
        self.mode.mode.stream = v.into();
        self
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// Diff the stream with a stream other than its configured parent.
    pub fn get_parent(&self) -> Option<&str> {
        self.mode.mode.parent.as_deref()
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// Diff the stream with a stream other than its configured parent.
    pub fn set_parent(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.mode.parent = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// Diff the stream with a stream other than its configured parent.
    pub fn parent(mut self, v: impl Into<String>) -> Self {
        self.mode.mode.parent = Some(v.into());
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Diff2<M> {
    fn name(&self) -> &str {
        "diff2"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(diff_opts) = &self.diff_opts {
            diff_opts.inject_arg(command);
        }

        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<A, B> ParameterizedSpawn<(A, B)> for Diff2<Unselected>
where
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2` for the given pair of file arguments as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    ///
    /// Each file argument is a file name, optionally with a revision
    /// specifier (for example `file#2` or `file@34`).
    fn spawn_with(&mut self, (file1, file2): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(file1)
            .arg(file2)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<A, B> ParameterizedSpawn<(A, B)> for Diff2<DepotContent<Unselected>>
where
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2` for the given pair of file arguments as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    ///
    /// Each file argument is a file name, optionally with a revision
    /// specifier (for example `file#2` or `file@34`).
    fn spawn_with(&mut self, (file1, file2): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(file1)
            .arg(file2)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for Diff2<DepotContent<BranchMode>> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -b branch` without file arguments as a child
    /// process with piped standard output and error streams; the whole
    /// branch mapping is diffed. Use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with(&mut self, _: ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I> ParameterizedSpawn<(I,)> for Diff2<DepotContent<BranchMode>>
where
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -b branch [tofile[rev]]` as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    ///
    /// Pass `Some(tofile)` to limit the target side of the branch mapping
    /// to the given file, or `None` to diff the whole branch mapping.
    fn spawn_with(&mut self, (tofile,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(tofile)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<A, B> ParameterizedSpawn<(A, B)> for Diff2<DepotContent<BranchMode>>
where
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -b branch fromfile[rev] tofile[rev]` as a child
    /// process with piped standard output and error streams; the branch
    /// mapping is diffed between the given files. Use the returned
    /// [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (fromfile, tofile): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(fromfile)
            .arg(tofile)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for Diff2<DepotContent<StreamMode>> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -S stream` without file arguments as a child
    /// process with piped standard output and error streams; the whole
    /// stream is diffed with its parent. Use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with(&mut self, _: ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I> ParameterizedSpawn<(I,)> for Diff2<DepotContent<StreamMode>>
where
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -S stream [tofile[rev]]` as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    ///
    /// Pass `Some(tofile)` to limit the target side of the stream diff to
    /// the given file, or `None` to diff the whole stream with its parent.
    fn spawn_with(&mut self, (tofile,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(tofile)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<A, B> ParameterizedSpawn<(A, B)> for Diff2<DepotContent<StreamMode>>
where
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -S stream fromfile[rev] tofile[rev]` as a child
    /// process with piped standard output and error streams; the stream is
    /// diffed with its parent between the given files. Use the returned
    /// [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (fromfile, tofile): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(fromfile)
            .arg(tofile)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<A, B> ParameterizedSpawn<(A, B)> for Diff2<StreamSpecMode>
where
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff2 -As` for the given pair of stream specs as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    ///
    /// Each stream spec is a streamname, optionally at a specific changelist
    /// number: `@head` selects the head version, `@change` the version at a
    /// specific change, and `@=change` the shelved version at a specific
    /// change.
    fn spawn_with(&mut self, (spec1, spec2): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(spec1)
            .arg(spec2)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::DiffOptionsBuilder;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 diff2` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let diff2 = Diff2::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&diff2.setup_command("p4")), ["diff2"]);
    }

    #[test]
    fn depot_content_flags_are_injected() {
        let diff2 = Diff2::new("p4", GlobalOpts::new())
            .differing_only(true)
            .quiet_mode(true)
            .diff_nontext(true)
            .unified_patch(true);

        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-Od", "-q", "-t", "-u"]
        );
    }

    #[test]
    fn depot_content_flags_apply_to_branch_mode() {
        let diff2 = Diff2::new("p4", GlobalOpts::new())
            .quiet_mode(true)
            .branch("branch2");

        assert!(diff2.get_quiet_mode());
        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-q", "-b", "branch2"]
        );
    }

    #[test]
    fn depot_content_flags_apply_after_branch_transition() {
        let diff2 = Diff2::new("p4", GlobalOpts::new())
            .branch("branch2")
            .quiet_mode(true);

        assert!(diff2.get_quiet_mode());
        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-q", "-b", "branch2"]
        );
    }

    #[test]
    fn diff_options_are_injected() {
        let mut diff2 =
            Diff2::new("p4", GlobalOpts::new()).diff_options(DiffOptionsBuilder::unified(None));
        diff2.set_diff_options(DiffOptionsBuilder::summary());

        assert_eq!(args_of(&diff2.setup_command("p4")), ["diff2", "-ds"]);
    }

    #[test]
    fn branch_mode_injects_branch() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).branch("branch2");

        assert_eq!(diff2.get_branch(), "branch2");
        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-b", "branch2"]
        );
    }

    #[test]
    fn branch_mode_with_files() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).branch("branch2");

        // Mirrors `spawn_with`, which appends the file arguments after the
        // assembled command.
        let mut command = diff2.setup_command("p4");
        command.arg(OsStr::new("//depot/rel1/..."));
        command.arg(OsStr::new("//depot/rel2/...#4"));

        assert_eq!(
            args_of(&command),
            [
                "diff2",
                "-b",
                "branch2",
                "//depot/rel1/...",
                "//depot/rel2/...#4"
            ]
        );
    }

    #[test]
    fn branch_mode_with_tofile() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).branch("branch2");

        // Mirrors `spawn_with` for the single-file input, which appends the
        // target file after the assembled command.
        let mut command = diff2.setup_command("p4");
        command.arg(OsStr::new("//depot/rel2/...#4"));

        assert_eq!(
            args_of(&command),
            ["diff2", "-b", "branch2", "//depot/rel2/...#4"]
        );
    }

    #[test]
    fn stream_mode_injects_stream() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).stream("myStream");

        assert_eq!(diff2.get_stream(), "myStream");
        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-S", "myStream"]
        );
    }

    #[test]
    fn stream_mode_with_parent() {
        let mut diff2 = Diff2::new("p4", GlobalOpts::new()).stream("myStream");
        diff2.set_parent("mainStream");

        assert_eq!(diff2.get_parent(), Some("mainStream"));
        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-S", "myStream", "-P", "mainStream"]
        );
    }

    #[test]
    fn stream_mode_with_tofile() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).stream("myStream");

        // Mirrors `spawn_with` for the single-file input, which appends the
        // target file after the assembled command.
        let mut command = diff2.setup_command("p4");
        command.arg(OsStr::new("//depot/rel2/...#4"));

        assert_eq!(
            args_of(&command),
            ["diff2", "-S", "myStream", "//depot/rel2/...#4"]
        );
    }

    #[test]
    fn stream_mode_preserves_diff_options() {
        let diff2 = Diff2::new("p4", GlobalOpts::new())
            .diff_options(DiffOptionsBuilder::summary())
            .stream("myStream");

        assert_eq!(
            args_of(&diff2.setup_command("p4")),
            ["diff2", "-ds", "-S", "myStream"]
        );
    }

    #[test]
    fn stream_spec_mode_bare() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).stream_spec_mode();

        assert_eq!(args_of(&diff2.setup_command("p4")), ["diff2", "-As"]);
    }

    #[test]
    fn stream_spec_mode_with_specs() {
        let diff2 = Diff2::new("p4", GlobalOpts::new()).stream_spec_mode();

        // Mirrors `spawn_with`, which appends the two stream specs after
        // the assembled command.
        let mut command = diff2.setup_command("p4");
        command.arg("myStream@=1");
        command.arg("yourStream@2");

        assert_eq!(
            args_of(&command),
            ["diff2", "-As", "myStream@=1", "yourStream@2"]
        );
    }
}
