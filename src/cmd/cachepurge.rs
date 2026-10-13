use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Purge-goal dimension: `-a` vs `-f n` vs `-m n` vs `-s n`
// ---------------------------------------------------------------------------

/// The purge goal `-a`: delete all file content.
///
/// This goal reclaims the maximum amount of disk space, but any file
/// content must be retrieved from the target server. Entered with
/// [`CachePurge::purge_all`]; mutually exclusive with [`FreeSpaceMode`],
/// [`RevisionCountMode`], and [`SizeMode`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeAllMode;

/// The purge goal `-f n`: delete sufficient file content to leave `n`
/// bytes of free space for the file system.
///
/// Entered with [`CachePurge::free_space`]; mutually exclusive with the
/// other purge goals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FreeSpaceMode {
    free_space: u64,
}

/// The purge goal `-m n`: delete `n` file revisions. The amount of space
/// this frees up depends on the size of the files.
///
/// Entered with [`CachePurge::revision_count`]; mutually exclusive with the
/// other purge goals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RevisionCountMode {
    revision_count: u64,
}

/// The purge goal `-s n`: delete `n` bytes of file data. This can be
/// helpful in those cases when you can predict the growth rate of file
/// system resources.
///
/// Entered with [`CachePurge::size`]; mutually exclusive with the other
/// purge goals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SizeMode {
    size: u64,
}

// ---------------------------------------------------------------------------
// ExclusiveOption implementations
// ---------------------------------------------------------------------------

impl ExclusiveOption for PurgeAllMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");
    }
}

impl ExclusiveOption for FreeSpaceMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-f").arg(self.free_space.to_string());
    }
}

impl ExclusiveOption for RevisionCountMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-m").arg(self.revision_count.to_string());
    }
}

impl ExclusiveOption for SizeMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s").arg(self.size.to_string());
    }
}

/// Marker for a selected purge goal: one of [`PurgeAllMode`],
/// [`FreeSpaceMode`], [`RevisionCountMode`], or [`SizeMode`].
///
/// Exactly one of `-a` / `-f` / `-m` / `-s` is required, so the command can
/// only be assembled and spawned once [`CachePurge`] has transitioned into
/// one of these states.
pub trait PurgeGoal: ExclusiveOption {}

impl PurgeGoal for PurgeAllMode {}
impl PurgeGoal for FreeSpaceMode {}
impl PurgeGoal for RevisionCountMode {}
impl PurgeGoal for SizeMode {}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 cachepurge`.
///
/// Reclaim disk space on a replicated server by deleting versioned file
/// content from the replica's cache; file content is deleted only from the
/// replica, and purged content is retrieved from the target server on
/// demand. This command requires `super` access.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] cachepurge -a [-i N -n -R -S N -O] [-D file[revRange]]
/// p4 [g-opts] cachepurge -f n [-i N -n -R -S N -O] [-D file[revRange]]
/// p4 [g-opts] cachepurge -m n [-i N -n -R -S N -O] [-D file[revRange]]
/// p4 [g-opts] cachepurge -s n [-i N -n -R -S N -O] [-D file[revRange]]
/// ```
///
/// The `M` type parameter tracks the purge goal at compile time: it starts
/// as [`Unselected`], in which the command has no effect and cannot be
/// spawned. One of [`Self::purge_all`] (`-a`), [`Self::free_space`]
/// (`-f n`), [`Self::revision_count`] (`-m n`), or [`Self::size`] (`-s n`)
/// must be used to select a goal before the command can be spawned. The
/// goal can be switched freely with the same methods; the common options
/// are preserved and the previously selected goal is discarded.
///
/// The deletion-order options `-R` / `-O` are available in every goal
/// state through [`CachePurge::purge_in_order`] and
/// [`CachePurge::purge_oldest_first`].
#[derive(Debug, Clone, Default)]
pub struct CachePurge<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    /// `-i N`
    interval: Option<u64>,

    /// `-n`
    preview: bool,

    /// `-D file[revRange]`
    filter: Option<String>,

    /// `-S N`
    retain_recent_revision: Option<u64>,

    /// -R
    purge_in_order: bool,

    /// -O
    purge_oldest_first: bool,

    mode: M,
}

impl CachePurge<Unselected> {
    /// Creates a new `p4 cachepurge` command with no purge goal selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. The
    /// command has no effect in this state; use [`Self::purge_all`],
    /// [`Self::free_space`], [`Self::revision_count`], or [`Self::size`] to
    /// select the required `-a` / `-f` / `-m` / `-s` goal.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }
}

impl<M> CachePurge<M> {
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

    /// Selects `-a`: delete all file content.
    ///
    /// This transitions into the [`PurgeAllMode`] state, discarding any
    /// previously selected purge goal; the common options are preserved.
    pub fn purge_all(self) -> CachePurge<PurgeAllMode> {
        CachePurge {
            bin: self.bin,
            global_opts: self.global_opts,
            interval: self.interval,
            preview: self.preview,
            filter: self.filter,
            retain_recent_revision: self.retain_recent_revision,
            purge_in_order: self.purge_in_order,
            purge_oldest_first: self.purge_oldest_first,
            mode: PurgeAllMode,
        }
    }

    /// Selects `-f n`: delete sufficient file content to leave `n` bytes of
    /// free space for the file system.
    ///
    /// This transitions into the [`FreeSpaceMode`] state, discarding any
    /// previously selected purge goal; the common options are preserved.
    pub fn free_space(self, n: u64) -> CachePurge<FreeSpaceMode> {
        CachePurge {
            bin: self.bin,
            global_opts: self.global_opts,
            interval: self.interval,
            preview: self.preview,
            filter: self.filter,
            retain_recent_revision: self.retain_recent_revision,
            purge_in_order: self.purge_in_order,
            purge_oldest_first: self.purge_oldest_first,
            mode: FreeSpaceMode { free_space: n },
        }
    }

    /// Selects `-m n`: delete `n` file revisions.
    ///
    /// This transitions into the [`RevisionCountMode`] state, discarding any
    /// previously selected purge goal; the common options are preserved.
    pub fn revision_count(self, n: u64) -> CachePurge<RevisionCountMode> {
        CachePurge {
            bin: self.bin,
            global_opts: self.global_opts,
            interval: self.interval,
            preview: self.preview,
            filter: self.filter,
            retain_recent_revision: self.retain_recent_revision,
            purge_in_order: self.purge_in_order,
            purge_oldest_first: self.purge_oldest_first,
            mode: RevisionCountMode { revision_count: n },
        }
    }

    /// Selects `-s n`: delete `n` bytes of file data.
    ///
    /// This transitions into the [`SizeMode`] state, discarding any
    /// previously selected purge goal; the common options are preserved.
    pub fn size(self, n: u64) -> CachePurge<SizeMode> {
        CachePurge {
            bin: self.bin,
            global_opts: self.global_opts,
            interval: self.interval,
            preview: self.preview,
            filter: self.filter,
            retain_recent_revision: self.retain_recent_revision,
            purge_in_order: self.purge_in_order,
            purge_oldest_first: self.purge_oldest_first,
            mode: SizeMode { size: n },
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
    /// -i N
    ///
    /// Repeat the command every `N` seconds. If you omit this option, the
    /// command runs only once.
    pub fn get_interval(&self) -> Option<u64> {
        self.interval
    }

    /// # Description
    ///
    /// -i N
    ///
    /// Repeat the command every `N` seconds. If you omit this option, the
    /// command runs only once.
    pub fn set_interval(&mut self, v: u64) -> &mut Self {
        self.interval = Some(v);
        self
    }

    /// # Description
    ///
    /// -i N
    ///
    /// Repeat the command every `N` seconds. If you omit this option, the
    /// command runs only once.
    pub fn interval(mut self, v: u64) -> Self {
        self.interval = Some(v);
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Display a preview of the `cachepurge` operation without deleting any
    /// files.
    pub fn get_preview(&self) -> bool {
        self.preview
    }

    /// # Description
    ///
    /// -n
    ///
    /// Display a preview of the `cachepurge` operation without deleting any
    /// files.
    pub fn set_preview(&mut self, v: bool) -> &mut Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Display a preview of the `cachepurge` operation without deleting any
    /// files.
    pub fn preview(mut self, v: bool) -> Self {
        self.preview = v;
        self
    }

    /// # Description
    ///
    /// `-D file[revRange]`
    ///
    /// Limit the action of the command to the specified set of files (and
    /// revision range, if specified).
    pub fn get_filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }

    /// # Description
    ///
    /// `-D file[revRange]`
    ///
    /// Limit the action of the command to the specified set of files (and
    /// revision range, if specified).
    pub fn set_filter(&mut self, v: impl Into<String>) -> &mut Self {
        self.filter = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-D file[revRange]`
    ///
    /// Limit the action of the command to the specified set of files (and
    /// revision range, if specified).
    pub fn filter(mut self, v: impl Into<String>) -> Self {
        self.filter = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -S N
    ///
    /// Do not delete the `N` most recent revisions of each file. For
    /// example, specifying `-S 1` means that the head revision of each file
    /// is retained in the replica's cache if it is already there.
    pub fn get_retain_recent_revision(&self) -> Option<u64> {
        self.retain_recent_revision
    }

    /// # Description
    ///
    /// -S N
    ///
    /// Do not delete the `N` most recent revisions of each file. For
    /// example, specifying `-S 1` means that the head revision of each file
    /// is retained in the replica's cache if it is already there.
    pub fn set_retain_recent_revision(&mut self, v: u64) -> &mut Self {
        self.retain_recent_revision = Some(v);
        self
    }

    /// # Description
    ///
    /// -S N
    ///
    /// Do not delete the `N` most recent revisions of each file. For
    /// example, specifying `-S 1` means that the head revision of each file
    /// is retained in the replica's cache if it is already there.
    pub fn retain_recent_revision(mut self, v: u64) -> Self {
        self.retain_recent_revision = Some(v);
        self
    }
    /// # Description
    ///
    /// -R
    ///
    /// Delete files in the order specified by the `-O` option. If the
    /// `-O` option is not specified, file names are used to determine
    /// order of deletion.
    pub fn get_purge_in_order(&self) -> bool {
        self.purge_in_order
    }

    /// # Description
    ///
    /// -R
    ///
    /// Delete files in the order specified by the `-O` option. If the
    /// `-O` option is not specified, file names are used to determine
    /// order of deletion.
    pub fn set_purge_in_order(&mut self, v: bool) -> &mut Self {
        self.purge_in_order = v;
        self
    }

    /// # Description
    ///
    /// -R
    ///
    /// Delete files in the order specified by the `-O` option. If the
    /// `-O` option is not specified, file names are used to determine
    /// order of deletion.
    pub fn purge_in_order(mut self, v: bool) -> Self {
        self.purge_in_order = v;
        self
    }

    /// # Description
    ///
    /// -O
    ///
    /// Delete the files from the oldest to the newest; that is, delete
    /// older files before deleting newer files.
    pub fn get_purge_oldest_first(&self) -> bool {
        self.purge_oldest_first
    }

    /// # Description
    ///
    /// -O
    ///
    /// Delete the files from the oldest to the newest; that is, delete
    /// older files before deleting newer files.
    pub fn set_purge_oldest_first(&mut self, v: bool) -> &mut Self {
        self.purge_oldest_first = v;
        self
    }

    /// # Description
    ///
    /// -O
    ///
    /// Delete the files from the oldest to the newest; that is, delete
    /// older files before deleting newer files.
    pub fn purge_oldest_first(mut self, v: bool) -> Self {
        self.purge_oldest_first = v;
        self
    }
}

impl CachePurge<FreeSpaceMode> {
    /// The free-space goal `n` selected with `-f`.
    pub fn get_free_space(&self) -> u64 {
        self.mode.free_space
    }
}

impl CachePurge<RevisionCountMode> {
    /// The revision count `n` selected with `-m`.
    pub fn get_revision_count(&self) -> u64 {
        self.mode.revision_count
    }
}

impl CachePurge<SizeMode> {
    /// The byte count `n` selected with `-s`.
    pub fn get_size(&self) -> u64 {
        self.mode.size
    }
}

// ---------------------------------------------------------------------------
// SubCommand: only after a purge goal (`-a` / `-f` / `-m` / `-s`) is selected
// ---------------------------------------------------------------------------

impl<M: PurgeGoal> SubCommand for CachePurge<M> {
    fn name(&self) -> &str {
        "cachepurge"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);

        if let Some(interval) = self.interval {
            command.arg("-i").arg(interval.to_string());
        }

        if self.preview {
            command.arg("-n");
        }

        if self.purge_in_order {
            command.arg("-R");
        }

        if let Some(retain) = self.retain_recent_revision {
            command.arg("-S").arg(retain.to_string());
        }

        if self.purge_oldest_first {
            command.arg("-O");
        }

        if let Some(filter) = &self.filter {
            command.arg("-D").arg(filter);
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning: only after a purge goal (`-a` / `-f` / `-m` / `-s`) is selected
// ---------------------------------------------------------------------------

impl<M: PurgeGoal> ParameterizedSpawn<()> for CachePurge<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 cachepurge` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it
    /// or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
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
        let mut command = CachePurge::new("p4", GlobalOpts::new()).purge_all();
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 cachepurge` command line; no
    /// process is spawned.
    #[test]
    fn purge_all_goal_only() {
        let purge = CachePurge::new("p4", GlobalOpts::new()).purge_all();

        assert_eq!(args_of(&purge.setup_command("p4")), ["cachepurge", "-a"]);
    }

    #[test]
    fn free_space_goal_only() {
        let purge = CachePurge::new("p4", GlobalOpts::new()).free_space(1_073_741_824);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-f", "1073741824"]
        );
        assert_eq!(purge.get_free_space(), 1_073_741_824);
    }

    #[test]
    fn revision_count_goal_only() {
        let purge = CachePurge::new("p4", GlobalOpts::new()).revision_count(100);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-m", "100"]
        );
        assert_eq!(purge.get_revision_count(), 100);
    }

    #[test]
    fn size_goal_only() {
        let purge = CachePurge::new("p4", GlobalOpts::new()).size(2_048);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-s", "2048"]
        );
        assert_eq!(purge.get_size(), 2_048);
    }

    #[test]
    fn order_flags_are_injected() {
        let purge = CachePurge::new("p4", GlobalOpts::new())
            .purge_all()
            .purge_in_order(true);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-a", "-R"]
        );
        assert!(purge.get_purge_in_order());
        assert!(!purge.get_purge_oldest_first());

        let purge = purge.purge_oldest_first(true);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-a", "-R", "-O"]
        );
        assert!(purge.get_purge_in_order());
        assert!(purge.get_purge_oldest_first());
    }

    #[test]
    fn order_flags_available_in_every_goal_state() {
        let purge = CachePurge::new("p4", GlobalOpts::new())
            .free_space(1_024)
            .purge_in_order(true)
            .purge_oldest_first(true);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-f", "1024", "-R", "-O"]
        );

        let purge = purge.revision_count(10);
        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-m", "10", "-R", "-O"]
        );

        let purge = purge.size(2_048);
        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-s", "2048", "-R", "-O"]
        );
    }

    #[test]
    fn all_options_injected_in_syntax_order() {
        let purge = CachePurge::new("p4", GlobalOpts::new())
            .purge_all()
            .purge_in_order(true)
            .purge_oldest_first(true)
            .interval(86_400)
            .preview(true)
            .retain_recent_revision(1)
            .filter("//depot/...");

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            [
                "cachepurge",
                "-a",
                "-i",
                "86400",
                "-n",
                "-R",
                "-S",
                "1",
                "-O",
                "-D",
                "//depot/..."
            ]
        );
    }

    #[test]
    fn switching_goals_preserves_common_options() {
        let purge = CachePurge::new("p4", GlobalOpts::new())
            .interval(60)
            .purge_all()
            .purge_oldest_first(true)
            .size(4_096);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-s", "4096", "-i", "60", "-O"]
        );
        assert_eq!(purge.get_size(), 4_096);
        assert!(purge.get_purge_oldest_first());

        let purge = purge.free_space(1_024);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["cachepurge", "-f", "1024", "-i", "60", "-O"]
        );
        assert_eq!(purge.get_free_space(), 1_024);
    }

    #[test]
    fn global_opts_are_injected_once() {
        let purge =
            CachePurge::new("p4", GlobalOpts::new().port("localhost:1666")).revision_count(10);

        assert_eq!(
            args_of(&purge.setup_command("p4")),
            ["-p", "localhost:1666", "cachepurge", "-m", "10"]
        );
    }

    #[test]
    fn setters_update_the_same_command() {
        let mut purge = CachePurge::new("p4", GlobalOpts::new()).free_space(2_048);
        purge
            .set_preview(true)
            .set_purge_in_order(true)
            .set_purge_oldest_first(true)
            .set_retain_recent_revision(3)
            .set_filter("//depot/src/...")
            .set_interval(300);

        assert!(purge.get_preview());
        assert!(purge.get_purge_in_order());
        assert!(purge.get_purge_oldest_first());
        assert_eq!(purge.get_retain_recent_revision(), Some(3));
        assert_eq!(purge.get_filter(), Some("//depot/src/..."));
        assert_eq!(purge.get_interval(), Some(300));
        assert_eq!(
            args_of(&purge.setup_command("p4")),
            [
                "cachepurge",
                "-f",
                "2048",
                "-i",
                "300",
                "-n",
                "-R",
                "-S",
                "3",
                "-O",
                "-D",
                "//depot/src/..."
            ]
        );
    }
}
