use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{DiffOptions, ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Display options passed via the `-soptions` flag, producing a shorthand
/// list of files that match the filter instead of diffs.
///
/// Exactly one filter may be selected; the enum makes the alternatives
/// unavailable at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayOptions {
    /// `-sa`
    ///
    /// Show only the names of opened files that are different from the
    /// revision in the depot, or are missing.
    OpenedChangedOrMissing,

    /// `-sb`
    ///
    /// Show only the names of files opened for integrate that have been
    /// resolved, but that have been modified after being resolved.
    ResolvedThenModified,

    /// `-sd`
    ///
    /// Show only the names of unopened files that are missing from the
    /// client workspace, but present in the depot.
    UnopenedMissing,

    /// `-se`
    ///
    /// Show only the names of unopened files in the client workspace that
    /// are different than the revision in the depot.
    UnopenedChanged,

    /// `-sl file ...`
    ///
    /// Every unopened `file` is compared with the depot, and listed with a
    /// status of `same`, `diff`, or `missing`.
    ///
    /// If you use the `-f` option together with the `-sl` option, files that
    /// are open for edit are also compared and their status is listed.
    ///
    /// The compared files are passed as the file arguments of the spawned
    /// command.
    ListWithStatus,

    /// `-sr`
    ///
    /// Show only the names of opened files in the client workspace that are
    /// identical to the revision in the depot.
    OpenedIdentical,
}

impl DisplayOptions {
    /// Returns the command-line flag for this display option.
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayOptions::OpenedChangedOrMissing => "-sa",
            DisplayOptions::ResolvedThenModified => "-sb",
            DisplayOptions::UnopenedMissing => "-sd",
            DisplayOptions::UnopenedChanged => "-se",
            DisplayOptions::ListWithStatus => "-sl",
            DisplayOptions::OpenedIdentical => "-sr",
        }
    }
}

/// The `-soptions` state of [`WorkspaceMode`]: a display filter is selected,
/// making `-m max` unavailable.
///
/// Entered with [`Diff::display_options`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceDisplayMode {
    display_opts: DisplayOptions,
}

impl ExclusiveOption for WorkspaceDisplayMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg(self.display_opts.as_str());
    }
}

/// The `-m max` state of [`WorkspaceMode`]: a file limit is selected, making
/// `-soptions` unavailable.
///
/// Entered with [`Diff::limit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceRegularMode {
    limit: u64,
}

impl ExclusiveOption for WorkspaceRegularMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-m").arg(self.limit.to_string());
    }
}

/// Workspace content mode of `p4 diff`, comparing files in the client
/// workspace to revisions in the depot.
///
/// Entered with [`Diff::force`], [`Diff::differing_only`], or
/// [`Diff::diff_nontext`].
///
/// The `M` type parameter isolates `-m max` and `-soptions` at compile time:
/// [`Diff::limit`] transitions to the [`WorkspaceRegularMode`] state, while
/// [`Diff::display_options`] transitions to the [`WorkspaceDisplayMode`]
/// state.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceMode<M = Unselected> {
    force: bool,

    differing_only: bool,

    diff_nontext: bool,

    mode: M,
}

impl<M: ExclusiveOption> ExclusiveOption for WorkspaceMode<M> {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }

        if self.diff_nontext {
            command.arg("-t");
        }

        if self.differing_only {
            command.arg("-Od");
        }

        self.mode.inject_args(command);
    }
}

/// Stream spec mode of `p4 diff` (`-As`): diff a privately edited stream
/// spec against another version of the same stream spec, or diff two
/// arbitrary stream specs against each other.
///
/// Entered with [`Diff::stream_spec_mode`]; the stream spec to compare
/// against is passed as the argument of the spawned command.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamSpecMode;

impl ExclusiveOption for StreamSpecMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-As");
    }
}

/// `p4 [g-opts] diff [-doptions] [-f -t -Od] [-m max] [-soptions] [file[rev] ...]`
///
/// `p4 [g-opts] diff [-doptions] -As [streamname[@change]]`
///
/// Diff utility for comparing workspace content to depot content. (For
/// comparing two depot paths, see `p4 diff2`.) Also for stream spec
/// comparison.
///
/// The `M` type parameter tracks the command mode at compile time. The
/// default [`Unselected`] state offers neither the workspace content options
/// nor `-As`; [`Self::force`], [`Self::differing_only`], and
/// [`Self::diff_nontext`] transition to the [`WorkspaceMode`] state, while
/// [`Self::stream_spec_mode`] transitions to the [`StreamSpecMode`] state.
#[derive(Debug, Clone, Default)]
pub struct Diff<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    diff_opts: Option<DiffOptions>,

    mode: M,
}

impl Diff<Unselected> {
    /// Creates a new `p4 diff` command.
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
    /// -f
    ///
    /// Force the diff (if no revision is specified, against the head
    /// revision), even when the client file is not open for `edit`.
    ///
    /// Transitions this command to the [`WorkspaceMode`] state with `-f` set
    /// according to `v`.
    pub fn force(self, v: bool) -> Diff<WorkspaceMode<Unselected>> {
        Diff {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: WorkspaceMode {
                force: v,
                differing_only: false,
                diff_nontext: false,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -Od
    ///
    /// Limit output to only those files that differ.
    ///
    /// Transitions this command to the [`WorkspaceMode`] state with `-Od`
    /// set according to `v`.
    pub fn differing_only(self, v: bool) -> Diff<WorkspaceMode<Unselected>> {
        Diff {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: WorkspaceMode {
                force: false,
                differing_only: v,
                diff_nontext: false,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the revisions even if the files are not of type `text`.
    ///
    /// Transitions this command to the [`WorkspaceMode`] state with `-t` set
    /// according to `v`.
    pub fn diff_nontext(self, v: bool) -> Diff<WorkspaceMode<Unselected>> {
        Diff {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: WorkspaceMode {
                force: false,
                differing_only: false,
                diff_nontext: v,
                mode: Unselected,
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
    /// Transitions this command to the [`StreamSpecMode`] state. The stream
    /// spec to compare against is passed as the argument of the spawned
    /// command ([`ParameterizedSpawn::spawn_with`]); without one, the opened
    /// stream spec is diffed against its have version.
    pub fn stream_spec_mode(self) -> Diff<StreamSpecMode> {
        Diff {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: StreamSpecMode,
        }
    }
}

impl<M> Diff<M> {
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

impl<M: ExclusiveOption> Diff<M> {
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

impl<M> Diff<WorkspaceMode<M>> {
    /// # Description
    ///
    /// -f
    ///
    /// Force the diff (if no revision is specified, against the head
    /// revision), even when the client file is not open for `edit`.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the diff (if no revision is specified, against the head
    /// revision), even when the client file is not open for `edit`.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the diff (if no revision is specified, against the head
    /// revision), even when the client file is not open for `edit`.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }

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
    /// -t
    ///
    /// Diff the revisions even if the files are not of type `text`.
    pub fn get_diff_nontext(&self) -> bool {
        self.mode.diff_nontext
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the revisions even if the files are not of type `text`.
    pub fn set_diff_nontext(&mut self, v: bool) -> &mut Self {
        self.mode.diff_nontext = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Diff the revisions even if the files are not of type `text`.
    pub fn diff_nontext(mut self, v: bool) -> Self {
        self.mode.diff_nontext = v;
        self
    }
}

impl Diff<WorkspaceMode<Unselected>> {
    /// # Description
    ///
    /// -soptions
    ///
    /// Pass display options to the underlying diff routine (see Usage notes
    /// for details).
    ///
    /// Transitions this command to the [`WorkspaceDisplayMode`] state with
    /// the display `options` set; `-m max` is unavailable in this state.
    pub fn display_options(self, v: DisplayOptions) -> Diff<WorkspaceMode<WorkspaceDisplayMode>> {
        Diff {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: WorkspaceMode {
                force: self.mode.force,
                differing_only: self.mode.differing_only,
                diff_nontext: self.mode.diff_nontext,
                mode: WorkspaceDisplayMode { display_opts: v },
            },
        }
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limit output to diffs (or status) of only the first `max` files,
    /// unless the `-s` option is used, in which case the `-m` option is
    /// ignored.
    ///
    /// Transitions this command to the [`WorkspaceRegularMode`] state with
    /// the limit set to `max`; `-soptions` is unavailable in this state.
    pub fn limit(self, max: u64) -> Diff<WorkspaceMode<WorkspaceRegularMode>> {
        Diff {
            bin: self.bin,
            global_opts: self.global_opts,
            diff_opts: self.diff_opts,
            mode: WorkspaceMode {
                force: self.mode.force,
                differing_only: self.mode.differing_only,
                diff_nontext: self.mode.diff_nontext,
                mode: WorkspaceRegularMode { limit: max },
            },
        }
    }
}

impl Diff<WorkspaceMode<WorkspaceDisplayMode>> {
    /// # Description
    ///
    /// -soptions
    ///
    /// Pass display options to the underlying diff routine (see Usage notes
    /// for details).
    pub fn get_display_options(&self) -> &DisplayOptions {
        &self.mode.mode.display_opts
    }

    /// # Description
    ///
    /// -soptions
    ///
    /// Pass display options to the underlying diff routine (see Usage notes
    /// for details).
    pub fn set_display_options(&mut self, v: DisplayOptions) -> &mut Self {
        self.mode.mode.display_opts = v;
        self
    }
}

impl Diff<WorkspaceMode<WorkspaceRegularMode>> {
    /// # Description
    ///
    /// -m max
    ///
    /// Limit output to diffs (or status) of only the first `max` files,
    /// unless the `-s` option is used, in which case the `-m` option is
    /// ignored.
    pub fn get_limit(&self) -> u64 {
        self.mode.mode.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limit output to diffs (or status) of only the first `max` files,
    /// unless the `-s` option is used, in which case the `-m` option is
    /// ignored.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.mode.mode.limit = v;
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Diff<M> {
    fn name(&self) -> &str {
        "diff"
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

impl<S, I> ParameterizedSpawn<(S,)> for Diff<Unselected>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff` for the given file arguments as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<M: ExclusiveOption, S, I> ParameterizedSpawn<(S,)> for Diff<WorkspaceMode<M>>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff` for the given file arguments as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I> ParameterizedSpawn<(I,)> for Diff<StreamSpecMode>
where
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff -As` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it
    /// or interact with it.
    ///
    /// The stream spec is a streamname, optionally at a specific changelist
    /// number: `@head` selects the head version, `@change` the version at a
    /// specific change, and `@=change` the shelved version at a specific
    /// change.
    ///
    /// Use [`spawn()`](SpawnExt::spawn) (no arguments) to diff the opened
    /// stream spec against its have version.
    fn spawn_with(&mut self, (stream_spec,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(stream_spec)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for Diff<StreamSpecMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 diff -As` without a stream spec as a child process with
    /// piped standard output and error streams; the opened stream spec is
    /// diffed against its have version. Use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with(&mut self, _: ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
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

    /// Dry-run checks of the assembled `p4 diff` command line; no process is
    /// spawned.
    #[test]
    fn without_options() {
        let diff = Diff::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&diff.setup_command("p4")), ["diff"]);
    }

    #[test]
    fn workspace_mode_via_force() {
        let diff = Diff::new("p4", GlobalOpts::new()).force(true);

        assert!(diff.get_force());
        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-f"]);
    }

    #[test]
    fn workspace_mode_via_differing_only() {
        let diff = Diff::new("p4", GlobalOpts::new()).differing_only(true);

        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-Od"]);
    }

    #[test]
    fn workspace_mode_via_diff_nontext() {
        let diff = Diff::new("p4", GlobalOpts::new()).diff_nontext(true);

        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-t"]);
    }

    #[test]
    fn workspace_mode_combines_options() {
        let mut diff = Diff::new("p4", GlobalOpts::new()).diff_nontext(true);
        diff.set_force(true).set_differing_only(true);

        assert_eq!(
            args_of(&diff.setup_command("p4")),
            ["diff", "-f", "-t", "-Od"]
        );
    }

    #[test]
    fn workspace_mode_preserves_diff_options() {
        let mut diff = Diff::new("p4", GlobalOpts::new())
            .diff_options(DiffOptionsBuilder::unified(None))
            .force(true);
        diff.set_diff_options(DiffOptionsBuilder::summary());

        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-ds", "-f"]);
    }

    #[test]
    fn display_mode_injects_filter() {
        let diff = Diff::new("p4", GlobalOpts::new())
            .force(true)
            .display_options(DisplayOptions::UnopenedChanged);

        assert_eq!(diff.get_display_options(), &DisplayOptions::UnopenedChanged);
        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-f", "-se"]);
    }

    #[test]
    fn display_mode_flag_mapping() {
        assert_eq!(DisplayOptions::OpenedChangedOrMissing.as_str(), "-sa");
        assert_eq!(DisplayOptions::ResolvedThenModified.as_str(), "-sb");
        assert_eq!(DisplayOptions::UnopenedMissing.as_str(), "-sd");
        assert_eq!(DisplayOptions::UnopenedChanged.as_str(), "-se");
        assert_eq!(DisplayOptions::ListWithStatus.as_str(), "-sl");
        assert_eq!(DisplayOptions::OpenedIdentical.as_str(), "-sr");
    }

    #[test]
    fn regular_mode_injects_limit() {
        let diff = Diff::new("p4", GlobalOpts::new()).force(false).limit(10);

        assert_eq!(diff.get_limit(), 10);
        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-m", "10"]);
    }

    #[test]
    fn stream_spec_mode_bare() {
        let diff = Diff::new("p4", GlobalOpts::new()).stream_spec_mode();

        assert_eq!(args_of(&diff.setup_command("p4")), ["diff", "-As"]);
    }

    #[test]
    fn stream_spec_mode_with_spec() {
        let diff = Diff::new("p4", GlobalOpts::new()).stream_spec_mode();

        // Mirrors `spawn_with`, which appends the stream spec after the
        // assembled command.
        let mut command = diff.setup_command("p4");
        command.arg("myStream@head");

        assert_eq!(args_of(&command), ["diff", "-As", "myStream@head"]);
    }

    #[test]
    fn stream_spec_mode_preserves_diff_options() {
        let diff = Diff::new("p4", GlobalOpts::new())
            .diff_options(DiffOptionsBuilder::unified(None))
            .stream_spec_mode();

        // Mirrors `spawn_with`, which appends the stream spec after the
        // assembled command.
        let mut command = diff.setup_command("p4");
        command.arg("myStream");

        assert_eq!(args_of(&command), ["diff", "-du", "-As", "myStream"]);
    }
}
