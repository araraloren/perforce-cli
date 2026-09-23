use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Configuration for the `--parallel` option of `p4 sync`.
///
/// Controls how files are transferred in parallel. `threads` is required;
/// all other sub-options are optional and fall back to server defaults when
/// omitted.
#[derive(Debug, Clone, Default)]
pub struct ParallelConfig {
    /// Number of concurrent network connections (`threads=N`).
    pub threads: u64,

    /// Number of files in a batch (`batch=N`).
    pub batch_files: Option<u64>,

    /// Number of bytes in a batch (`batchsize=N`).
    pub batch_size_bytes: Option<u64>,

    /// Minimum number of files for a parallel sync (`min=N`).
    pub min_files: Option<u64>,

    /// Minimum number of bytes for a parallel sync (`minsize=N`).
    pub min_size_bytes: Option<u64>,
}

impl ParallelConfig {
    /// Builds the value passed to `--parallel=...`, e.g.
    /// `threads=4,batch=8,batchsize=512K,min=9,minsize=576K`.
    pub fn as_arg(&self) -> String {
        let mut parts = vec![format!("threads={}", self.threads)];

        if let Some(v) = self.batch_files {
            parts.push(format!("batch={}", v));
        }
        if let Some(v) = self.batch_size_bytes {
            parts.push(format!("batchsize={}", v));
        }
        if let Some(v) = self.min_files {
            parts.push(format!("min={}", v));
        }
        if let Some(v) = self.min_size_bytes {
            parts.push(format!("minsize={}", v));
        }

        parts.join(",")
    }
}

/// The `--use-stream-change` value controlling which stream specification
/// version is used to generate the client view.
#[cfg(not(feature = "lt2022_2"))]
#[derive(Debug, Clone, Copy)]
pub enum StreamSpecVersion {
    /// `--use-stream-change` with no value: the maximum change number in the
    /// file list determines the stream spec version.
    MaxInFilelists,
    /// `--use-stream-change=0`: use the current stream spec version.
    Current,
    /// `--use-stream-change=N`: use the stream spec version at or before
    /// change `N`.
    ChangeNumber(u32),
}

#[cfg(not(feature = "lt2022_2"))]
impl StreamSpecVersion {
    /// `--use-stream-change` with no value: the maximum change number in the
    /// file list determines the stream spec version.
    pub fn max_in_filelists() -> Self {
        StreamSpecVersion::MaxInFilelists
    }

    /// `--use-stream-change=0`: use the current stream spec version.
    pub fn current() -> Self {
        StreamSpecVersion::Current
    }

    /// `--use-stream-change=N`: use the stream spec version at or before
    /// change `n`.
    pub fn at_change(n: u32) -> Self {
        StreamSpecVersion::ChangeNumber(n)
    }

    /// Injects the `--use-stream-change` argument(s) into `command`.
    pub fn inject_arg(&self, command: &mut Command) {
        match self {
            StreamSpecVersion::MaxInFilelists => {
                command.arg("--use-stream-change");
            }
            StreamSpecVersion::Current => {
                command.arg("--use-stream-change=0");
            }
            StreamSpecVersion::ChangeNumber(n) => {
                command.arg(format!("--use-stream-change={}", n));
            }
        }
    }
}

/// Preview mode of `p4 sync` (`-n`): display the results of the sync without
/// actually performing the sync.
///
/// Entered with [`Sync::preview_result`]. Mutually exclusive with
/// [`PreviewNetworkTraffic`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewResult;

impl ExclusiveOption for PreviewResult {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-n");
    }
}

/// Preview mode of `p4 sync` (`-N`): display a summary of the expected
/// network traffic associated with a sync, without performing the sync.
///
/// Entered with [`Sync::preview_network_traffic`]. Mutually exclusive with
/// [`PreviewResult`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewNetworkTraffic;

impl ExclusiveOption for PreviewNetworkTraffic {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-N");
    }
}

/// Force execution sub-mode of [`RegularMode`]: entered when any of `-f`,
/// `-k`, or `-r` is set.
///
/// In this state the command is locked into the regular sync form and can no
/// longer transition to [`SafeCheckMode`] or [`PopulateMode`].
#[derive(Debug, Clone, Default)]
pub struct ForceRegularMode {
    force: bool,

    metadata_only: bool,

    #[cfg(not(feature = "lt2015_1"))]
    reopen_moved_files: bool,
}

impl ExclusiveOption for ForceRegularMode {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }

        if self.metadata_only {
            command.arg("-k");
        }

        #[cfg(not(feature = "lt2015_1"))]
        if self.reopen_moved_files {
            command.arg("-r");
        }
    }
}

/// Safe sync sub-mode of [`RegularMode`] (`-s`): compare the content in the
/// client workspace against what was last synced and do not overwrite files
/// that were modified outside of P4 Server's control.
///
/// Entered with [`Sync::safe_check`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SafeCheckMode;

impl ExclusiveOption for SafeCheckMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s");
    }
}

/// Populate sub-mode of [`RegularMode`] (`-p`): populate a client workspace
/// but do not update the have list.
///
/// Entered with [`Sync::populate`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PopulateMode;

impl ExclusiveOption for PopulateMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-p");
    }
}

/// Regular mode of `p4 sync`: the command forms that operate on the client
/// workspace.
///
/// All common options are stored directly on this struct. The `Mode` type
/// parameter selects the mutually exclusive execution sub-mode:
///
/// - [`Unselected`] (default): the first command form, plain regular sync.
/// - [`ForceRegularMode`]: entered by setting `-f`, `-k`, or `-r`; locks out
///   further transitions to [`SafeCheckMode`] or [`PopulateMode`].
/// - [`SafeCheckMode`]: entered by [`Sync::safe_check`] (`-s`).
/// - [`PopulateMode`]: entered by [`Sync::populate`] (`-p`).
///
/// The `P` type parameter tracks the preview mode ([`Unselected`] by default,
/// [`PreviewResult`] or [`PreviewNetworkTraffic`] otherwise).
#[derive(Debug, Clone, Default)]
pub struct RegularMode<Mode = Unselected, P = Unselected> {
    #[cfg(not(feature = "lt2022_2"))]
    verify_edge_replication: bool,

    script_list_mode: bool,

    #[cfg(not(feature = "lt2022_1"))]
    suppress_keyword_expansion: bool,

    quiet_mode: bool,

    limit: Option<u64>,

    parallel: Option<ParallelConfig>,

    #[cfg(not(feature = "lt2022_2"))]
    stream_spec_version: Option<StreamSpecVersion>,

    mode: Mode,

    preview: P,
}

impl<Mode: ExclusiveOption, P: ExclusiveOption> ExclusiveOption for RegularMode<Mode, P> {
    fn inject_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2022_2"))]
        if self.verify_edge_replication {
            command.arg("-E");
        }

        if self.script_list_mode {
            command.arg("-L");
        }

        #[cfg(not(feature = "lt2022_1"))]
        if self.suppress_keyword_expansion {
            command.arg("-K");
        }

        if self.quiet_mode {
            command.arg("-q");
        }

        self.mode.inject_args(command);

        self.preview.inject_args(command);

        if let Some(max) = self.limit {
            command.arg("-m").arg(max.to_string());
        }

        if let Some(parallel) = &self.parallel {
            command.arg(format!("--parallel={}", parallel.as_arg()));
        }

        #[cfg(not(feature = "lt2022_2"))]
        if let Some(version) = &self.stream_spec_version {
            version.inject_arg(command);
        }
    }
}

/// Sync-time mode of `p4 sync` (`-k --sync-time=N`): update the have list to
/// reflect the state of the depot at the given time without transferring
/// files.
///
/// Entered with [`Sync::sync_time`]. This mode always implies `-k` (metadata
/// only), so no separate interface is provided for it.
#[cfg(not(feature = "lt2025_1"))]
#[derive(Debug, Clone)]
pub struct SyncTimeMode {
    sync_time: String,
}

#[cfg(not(feature = "lt2025_1"))]
impl ExclusiveOption for SyncTimeMode {
    fn inject_args(&self, command: &mut Command) {
        command
            .arg("-k")
            .arg(format!("--sync-time={}", self.sync_time));
    }
}

///
/// Update the client workspace to reflect the contents of the depot.
///
/// The `M` type parameter tracks the top-level mode at compile time. The
/// default [`Unselected`] state syncs files without local options; setting
/// any regular option or calling a mode-transition method moves into
/// [`RegularMode`]; [`Self::sync_time`] moves into [`SyncTimeMode`].
#[derive(Debug, Clone, Default)]
pub struct Sync<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

impl Sync<Unselected> {
    /// Creates a new `p4 sync` command.
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
    /// `-k --sync-time=N`
    ///
    /// Update the have list to reflect the state of the depot at the given
    /// time without transferring files.
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2025_1")),
        doc = "The value of `N` can be Unix epoch time or the perforce date",
        doc = "time format."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "The value of `N` can be Unix epoch time or the Perforce date",
        doc = "time format."
    )]
    ///
    /// This mode always implies `-k`, so no separate interface is provided
    /// for it. Transitions this command to the [`SyncTimeMode`] state.
    #[cfg(not(feature = "lt2025_1"))]
    pub fn sync_time(self, time: impl Into<String>) -> Sync<SyncTimeMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SyncTimeMode {
                sync_time: time.into(),
            },
        }
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Safe sync: compare the content in the client workspace against what
    /// was last synced.
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "If the file was modified outside of Perforce control, an error",
        doc = "message is displayed and the file is not overwritten."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2017_2")),
        doc = "If the file was modified outside of the control of Helix",
        doc = "Server, an error message is displayed and the file is not",
        doc = "overwritten."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "If the file was modified outside of the control of Helix Core",
        doc = "Server, an error message is displayed and the file is not",
        doc = "overwritten."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "If the file was modified outside of the control of P4 Server,",
        doc = "an error message is displayed and the file is not overwritten."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`SafeCheckMode`] sub-mode.
    pub fn enable_safe_check(self) -> Sync<RegularMode<SafeCheckMode>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                mode: SafeCheckMode,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-p`
    ///
    /// Populate a client workspace, but do not update the have list. Any
    /// file that is already synced or opened is bypassed with a warning
    /// message.
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`PopulateMode`] sub-mode.
    pub fn populate_client_workspace(self) -> Sync<RegularMode<PopulateMode>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                mode: PopulateMode,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-E`
    ///
    /// For edge servers replicating from a commit or an upstream edge, verify
    /// that any changelists specified in the revSpec are submitted before
    /// continuing with the sync.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn verify_edge_replication(self, v: bool) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                verify_edge_replication: v,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-L`
    ///
    /// For scripting purposes, perform the sync on a list of valid file
    /// arguments in full depot syntax with a valid revision number.
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "",
        doc = "When this flag is used, the arguments are processed together by",
        doc = "building an internal table similar to a label. This file list",
        doc = "processing is significantly faster than having to call the",
        doc = "internal query engine for each individual file argument. However,",
        doc = "the file argument syntax is strict and the command will not run",
        doc = "if an error is encountered."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn script_list_mode(self, v: bool) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                script_list_mode: v,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-K`
    ///
    /// Suppress keyword expansion when updating `+k` type files on the
    /// client.
    #[cfg_attr(feature = "lt2024_2", doc = "See File type modifiers.")]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "To learn more, see File type modifiers."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn suppress_keyword_expansion(self, v: bool) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                suppress_keyword_expansion: v,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-q`
    ///
    /// Quiet operation: suppress normal output messages. Messages describing
    /// errors or exceptional conditions are not suppressed.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn quiet_mode(self, v: bool) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                quiet_mode: v,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-m max`
    ///
    /// Sync only the first `max` files specified.
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "",
        doc = "This option is useful in conjunction with tagged output and the",
        doc = "`-n` flag, to preview how many files will be synced without",
        doc = "transferring all the file data."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn limit(self, v: u64) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                limit: Some(v),
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `--parallel=threads=N[,batch=N][,batchsize=N][,min=N][,minsize=N]`
    ///
    /// Specify options for parallel file transfer.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    pub fn parallel(self, v: ParallelConfig) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                parallel: Some(v),
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `--use-stream-change[N]`
    ///
    /// Specify the stream specification version to use for generating the
    /// client view for sync.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn stream_spec_version(self, v: StreamSpecVersion) -> Sync<RegularMode> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                stream_spec_version: Some(v),
                ..RegularMode::default()
            },
        }
    }

    /// `--use-stream-change` (no value): the maximum change number in the
    /// file list determines the stream spec version.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sc_max_change_number(self) -> Sync<RegularMode> {
        self.stream_spec_version(StreamSpecVersion::MaxInFilelists)
    }

    /// `--use-stream-change=0`: use the current stream spec version.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sc_current_stream_spec(self) -> Sync<RegularMode> {
        self.stream_spec_version(StreamSpecVersion::Current)
    }

    /// `--use-stream-change=N`: use the stream spec version at or before
    /// change `n`.
    ///
    /// Transitions this command to the [`RegularMode`] state.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sc_change_number(self, n: u32) -> Sync<RegularMode> {
        self.stream_spec_version(StreamSpecVersion::ChangeNumber(n))
    }

    /// # Description
    ///
    /// `-n`
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Display the results of the sync without actually performing the",
        doc = "sync.",
        doc = "",
        doc = "This lets you make sure that the sync does what you think it",
        doc = "does before you do it."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "Preview mode: display the results of the sync without actually",
        doc = "performing the sync."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`PreviewResult`] preview mode.
    pub fn preview_result(self) -> Sync<RegularMode<Unselected, PreviewResult>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                preview: PreviewResult,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-N`
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Display a summary of the expected network traffic associated",
        doc = "with a sync, without performing the sync."
    )]
    #[cfg_attr(
        all(feature = "lt2021_2", not(feature = "lt2016_1")),
        doc = "Preview mode: display a summary of the expected network traffic",
        doc = "associated with a sync, without performing the sync."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Preview mode: display a summary of the expected network traffic",
        doc = "associated with a sync, without performing the sync.",
        doc = "",
        doc = "This tells you how many files are to be added or updated, which",
        doc = "is useful if there are many large files, limits on bandwidth, or",
        doc = "limits on disk space."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`PreviewNetworkTraffic`] preview mode.
    pub fn preview_network_traffic(self) -> Sync<RegularMode<Unselected, PreviewNetworkTraffic>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                preview: PreviewNetworkTraffic,
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-f`
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Force the sync. Perforce performs the sync even if the client",
        doc = "workspace already has the file at the specified revision. If the",
        doc = "file is writable, it is overwritten.",
        doc = "",
        doc = "This flag does not affect open files, but it does override the",
        doc = "noclobber client option."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2014_2")),
        doc = "Force the sync. Perforce performs the sync even if the client",
        doc = "workspace already has the file at the specified revision. If the",
        doc = "file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2017_2")),
        doc = "Force the sync. Helix Server performs the sync even if the",
        doc = "client workspace already has the file at the specified",
        doc = "revision. If the file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option (see p4 client)."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Force the sync. Helix Core Server performs the sync even if the",
        doc = "client workspace already has the file at the specified",
        doc = "revision. If the file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option (see p4 client)."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Force the sync. P4 Server performs the sync even if the client",
        doc = "workspace already has the file at the specified revision. If the",
        doc = "file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option (see p4 client)."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`ForceRegularMode`] sub-mode, which prevents further transitions to
    /// [`SafeCheckMode`] or [`PopulateMode`].
    pub fn force(self, v: bool) -> Sync<RegularMode<ForceRegularMode>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                mode: ForceRegularMode {
                    force: v,
                    ..ForceRegularMode::default()
                },
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-k`
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "Keep existing workspace files; update the have list without",
        doc = "updating the client workspace."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Update server metadata without syncing files. Keep existing",
        doc = "workspace files and update the have list without updating the",
        doc = "client workspace."
    )]
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`ForceRegularMode`] sub-mode, which prevents further transitions to
    /// [`SafeCheckMode`] or [`PopulateMode`].
    pub fn metadata_only(self, v: bool) -> Sync<RegularMode<ForceRegularMode>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                mode: ForceRegularMode {
                    metadata_only: v,
                    ..ForceRegularMode::default()
                },
                ..RegularMode::default()
            },
        }
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reopen files that are mapped to new locations in the depot, in the new
    /// location.
    ///
    /// Transitions this command to the [`RegularMode`] state with the
    /// [`ForceRegularMode`] sub-mode, which prevents further transitions to
    /// [`SafeCheckMode`] or [`PopulateMode`].
    #[cfg(not(feature = "lt2015_1"))]
    pub fn reopen_moved_files(self, v: bool) -> Sync<RegularMode<ForceRegularMode>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                mode: ForceRegularMode {
                    reopen_moved_files: v,
                    ..ForceRegularMode::default()
                },
                ..RegularMode::default()
            },
        }
    }
}

// ---- Common option accessors (available in every RegularMode sub-mode) ----

impl<Mode: ExclusiveOption, P: ExclusiveOption> Sync<RegularMode<Mode, P>> {
    /// Returns whether edge replication is verified (`-E`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_verify_edge_replication(&self) -> bool {
        self.mode.verify_edge_replication
    }

    /// Sets whether edge replication is verified (`-E`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_verify_edge_replication(&mut self, v: bool) -> &mut Self {
        self.mode.verify_edge_replication = v;
        self
    }

    /// Sets whether edge replication is verified (`-E`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn verify_edge_replication(mut self, v: bool) -> Self {
        self.mode.verify_edge_replication = v;
        self
    }

    /// Returns whether script list mode is enabled (`-L`).
    pub fn get_script_list_mode(&self) -> bool {
        self.mode.script_list_mode
    }

    /// Sets whether script list mode is enabled (`-L`).
    pub fn set_script_list_mode(&mut self, v: bool) -> &mut Self {
        self.mode.script_list_mode = v;
        self
    }

    /// Sets whether script list mode is enabled (`-L`).
    pub fn script_list_mode(mut self, v: bool) -> Self {
        self.mode.script_list_mode = v;
        self
    }

    /// Returns whether keyword expansion is suppressed (`-K`).
    #[cfg(not(feature = "lt2022_1"))]
    pub fn get_suppress_keyword_expansion(&self) -> bool {
        self.mode.suppress_keyword_expansion
    }

    /// Sets whether keyword expansion is suppressed (`-K`).
    #[cfg(not(feature = "lt2022_1"))]
    pub fn set_suppress_keyword_expansion(&mut self, v: bool) -> &mut Self {
        self.mode.suppress_keyword_expansion = v;
        self
    }

    /// Sets whether keyword expansion is suppressed (`-K`).
    #[cfg(not(feature = "lt2022_1"))]
    pub fn suppress_keyword_expansion(mut self, v: bool) -> Self {
        self.mode.suppress_keyword_expansion = v;
        self
    }

    /// Returns whether quiet mode is enabled (`-q`).
    pub fn get_quiet_mode(&self) -> bool {
        self.mode.quiet_mode
    }

    /// Sets whether quiet mode is enabled (`-q`).
    pub fn set_quiet_mode(&mut self, v: bool) -> &mut Self {
        self.mode.quiet_mode = v;
        self
    }

    /// Sets whether quiet mode is enabled (`-q`).
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.mode.quiet_mode = v;
        self
    }

    /// Returns the maximum number of files to sync (`-m max`).
    pub fn get_limit(&self) -> Option<u64> {
        self.mode.limit
    }

    /// Sets the maximum number of files to sync (`-m max`).
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.mode.limit = Some(v);
        self
    }

    /// Sets the maximum number of files to sync (`-m max`).
    pub fn limit(mut self, v: u64) -> Self {
        self.mode.limit = Some(v);
        self
    }

    /// Returns the parallel sync configuration (`--parallel`).
    pub fn get_parallel(&self) -> Option<&ParallelConfig> {
        self.mode.parallel.as_ref()
    }

    /// Sets the parallel sync configuration (`--parallel`).
    pub fn set_parallel(&mut self, v: ParallelConfig) -> &mut Self {
        self.mode.parallel = Some(v);
        self
    }

    /// Sets the parallel sync configuration (`--parallel`).
    pub fn parallel(mut self, v: ParallelConfig) -> Self {
        self.mode.parallel = Some(v);
        self
    }

    /// Returns the stream spec version (`--use-stream-change`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_stream_spec_version(&self) -> Option<StreamSpecVersion> {
        self.mode.stream_spec_version
    }

    /// Sets the stream spec version (`--use-stream-change`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_stream_spec_version(&mut self, v: StreamSpecVersion) -> &mut Self {
        self.mode.stream_spec_version = Some(v);
        self
    }

    /// Sets the stream spec version (`--use-stream-change`).
    #[cfg(not(feature = "lt2022_2"))]
    pub fn stream_spec_version(mut self, v: StreamSpecVersion) -> Self {
        self.mode.stream_spec_version = Some(v);
        self
    }

    /// `--use-stream-change` (no value): the maximum change number in the
    /// file list determines the stream spec version.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_sc_max_change_number(&mut self) -> &mut Self {
        self.mode.stream_spec_version = Some(StreamSpecVersion::MaxInFilelists);
        self
    }

    /// `--use-stream-change` (no value): the maximum change number in the
    /// file list determines the stream spec version.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sc_max_change_number(mut self) -> Self {
        self.mode.stream_spec_version = Some(StreamSpecVersion::MaxInFilelists);
        self
    }

    /// `--use-stream-change=0`: use the current stream spec version.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_sc_current_stream_spec(&mut self) -> &mut Self {
        self.mode.stream_spec_version = Some(StreamSpecVersion::Current);
        self
    }

    /// `--use-stream-change=0`: use the current stream spec version.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sc_current_stream_spec(mut self) -> Self {
        self.mode.stream_spec_version = Some(StreamSpecVersion::Current);
        self
    }

    /// `--use-stream-change=N`: use the stream spec version at or before
    /// change `n`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_sc_change_number(&mut self, n: u32) -> &mut Self {
        self.mode.stream_spec_version = Some(StreamSpecVersion::ChangeNumber(n));
        self
    }

    /// `--use-stream-change=N`: use the stream spec version at or before
    /// change `n`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sc_change_number(mut self, n: u32) -> Self {
        self.mode.stream_spec_version = Some(StreamSpecVersion::ChangeNumber(n));
        self
    }
}

// ---- Sub-mode transitions (only from the Unselected sub-mode) ----

impl<P: ExclusiveOption> Sync<RegularMode<Unselected, P>> {
    /// # Description
    ///
    /// `-f`
    ///
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Force the sync. Perforce performs the sync even if the client",
        doc = "workspace already has the file at the specified revision. If the",
        doc = "file is writable, it is overwritten.",
        doc = "",
        doc = "This flag does not affect open files, but it does override the",
        doc = "noclobber client option."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2014_2")),
        doc = "Force the sync. Perforce performs the sync even if the client",
        doc = "workspace already has the file at the specified revision. If the",
        doc = "file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2017_2")),
        doc = "Force the sync. Helix Server performs the sync even if the",
        doc = "client workspace already has the file at the specified",
        doc = "revision. If the file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option (see p4 client)."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Force the sync. Helix Core Server performs the sync even if the",
        doc = "client workspace already has the file at the specified",
        doc = "revision. If the file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option (see p4 client)."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Force the sync. P4 Server performs the sync even if the client",
        doc = "workspace already has the file at the specified revision. If the",
        doc = "file is writable, it is overwritten.",
        doc = "",
        doc = "This option does not affect open files, but it does override the",
        doc = "noclobber client option (see p4 client)."
    )]
    ///
    /// Transitions the sub-mode to [`ForceRegularMode`], which prevents
    /// further transitions to [`SafeCheckMode`] or [`PopulateMode`].
    pub fn force(self, v: bool) -> Sync<RegularMode<ForceRegularMode, P>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: ForceRegularMode {
                    force: v,
                    ..ForceRegularMode::default()
                },
                preview: self.mode.preview,
            },
        }
    }

    /// # Description
    ///
    /// `-k`
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "Keep existing workspace files; update the have list without",
        doc = "updating the client workspace."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Update server metadata without syncing files. Keep existing",
        doc = "workspace files and update the have list without updating the",
        doc = "client workspace."
    )]
    ///
    /// Transitions the sub-mode to [`ForceRegularMode`], which prevents
    /// further transitions to [`SafeCheckMode`] or [`PopulateMode`].
    pub fn metadata_only(self, v: bool) -> Sync<RegularMode<ForceRegularMode, P>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: ForceRegularMode {
                    metadata_only: v,
                    ..ForceRegularMode::default()
                },
                preview: self.mode.preview,
            },
        }
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Reopen files that are mapped to new locations in the depot, in the new
    /// location.
    ///
    /// Transitions the sub-mode to [`ForceRegularMode`], which prevents
    /// further transitions to [`SafeCheckMode`] or [`PopulateMode`].
    #[cfg(not(feature = "lt2015_1"))]
    pub fn reopen_moved_files(self, v: bool) -> Sync<RegularMode<ForceRegularMode, P>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: ForceRegularMode {
                    reopen_moved_files: v,
                    ..ForceRegularMode::default()
                },
                preview: self.mode.preview,
            },
        }
    }

    /// # Description
    ///
    /// `-s`
    ///
    /// Safe sync: compare the content in the client workspace against what
    /// was last synced.
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "If the file was modified outside of Perforce control, an error",
        doc = "message is displayed and the file is not overwritten."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2017_2")),
        doc = "If the file was modified outside of the control of Helix",
        doc = "Server, an error message is displayed and the file is not",
        doc = "overwritten."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "If the file was modified outside of the control of Helix Core",
        doc = "Server, an error message is displayed and the file is not",
        doc = "overwritten."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "If the file was modified outside of the control of P4 Server,",
        doc = "an error message is displayed and the file is not overwritten."
    )]
    ///
    /// Transitions the sub-mode to [`SafeCheckMode`].
    pub fn safe_check(self) -> Sync<RegularMode<SafeCheckMode, P>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: SafeCheckMode,
                preview: self.mode.preview,
            },
        }
    }

    /// # Description
    ///
    /// `-p`
    ///
    /// Populate a client workspace, but do not update the have list. Any
    /// file that is already synced or opened is bypassed with a warning
    /// message.
    ///
    /// Transitions the sub-mode to [`PopulateMode`].
    pub fn populate(self) -> Sync<RegularMode<PopulateMode, P>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: PopulateMode,
                preview: self.mode.preview,
            },
        }
    }
}

// ---- Preview transitions (only when preview is Unselected) ----

impl<Mode: ExclusiveOption> Sync<RegularMode<Mode, Unselected>> {
    /// # Description
    ///
    /// `-n`
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Display the results of the sync without actually performing the",
        doc = "sync.",
        doc = "",
        doc = "This lets you make sure that the sync does what you think it",
        doc = "does before you do it."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "Preview mode: display the results of the sync without actually",
        doc = "performing the sync."
    )]
    ///
    /// Transitions the preview mode to [`PreviewResult`].
    pub fn preview_result(self) -> Sync<RegularMode<Mode, PreviewResult>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: self.mode.mode,
                preview: PreviewResult,
            },
        }
    }

    /// # Description
    ///
    /// `-N`
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Display a summary of the expected network traffic associated",
        doc = "with a sync, without performing the sync."
    )]
    #[cfg_attr(
        all(feature = "lt2021_2", not(feature = "lt2016_1")),
        doc = "Preview mode: display a summary of the expected network traffic",
        doc = "associated with a sync, without performing the sync."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Preview mode: display a summary of the expected network traffic",
        doc = "associated with a sync, without performing the sync.",
        doc = "",
        doc = "This tells you how many files are to be added or updated, which",
        doc = "is useful if there are many large files, limits on bandwidth, or",
        doc = "limits on disk space."
    )]
    ///
    /// Transitions the preview mode to [`PreviewNetworkTraffic`].
    pub fn preview_network_traffic(self) -> Sync<RegularMode<Mode, PreviewNetworkTraffic>> {
        Sync {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularMode {
                #[cfg(not(feature = "lt2022_2"))]
                verify_edge_replication: self.mode.verify_edge_replication,
                script_list_mode: self.mode.script_list_mode,
                #[cfg(not(feature = "lt2022_1"))]
                suppress_keyword_expansion: self.mode.suppress_keyword_expansion,
                quiet_mode: self.mode.quiet_mode,
                limit: self.mode.limit,
                parallel: self.mode.parallel,
                #[cfg(not(feature = "lt2022_2"))]
                stream_spec_version: self.mode.stream_spec_version,
                mode: self.mode.mode,
                preview: PreviewNetworkTraffic,
            },
        }
    }
}

// ---- Force-only option accessors (only in ForceRegularMode sub-mode) ----

impl<P: ExclusiveOption> Sync<RegularMode<ForceRegularMode, P>> {
    /// Returns whether the sync is forced (`-f`).
    pub fn get_force(&self) -> bool {
        self.mode.mode.force
    }

    /// Sets whether the sync is forced (`-f`).
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.mode.force = v;
        self
    }

    /// Sets whether the sync is forced (`-f`).
    pub fn force(mut self, v: bool) -> Self {
        self.mode.mode.force = v;
        self
    }

    /// Returns whether only metadata is updated (`-k`).
    pub fn get_metadata_only(&self) -> bool {
        self.mode.mode.metadata_only
    }

    /// Sets whether only metadata is updated (`-k`).
    pub fn set_metadata_only(&mut self, v: bool) -> &mut Self {
        self.mode.mode.metadata_only = v;
        self
    }

    /// Sets whether only metadata is updated (`-k`).
    pub fn metadata_only(mut self, v: bool) -> Self {
        self.mode.mode.metadata_only = v;
        self
    }

    /// Returns whether moved files are reopened (`-r`).
    #[cfg(not(feature = "lt2015_1"))]
    pub fn get_reopen_moved_files(&self) -> bool {
        self.mode.mode.reopen_moved_files
    }

    /// Sets whether moved files are reopened (`-r`).
    #[cfg(not(feature = "lt2015_1"))]
    pub fn set_reopen_moved_files(&mut self, v: bool) -> &mut Self {
        self.mode.mode.reopen_moved_files = v;
        self
    }

    /// Sets whether moved files are reopened (`-r`).
    #[cfg(not(feature = "lt2015_1"))]
    pub fn reopen_moved_files(mut self, v: bool) -> Self {
        self.mode.mode.reopen_moved_files = v;
        self
    }
}

// ---- SyncTimeMode accessors ----

#[cfg(not(feature = "lt2025_1"))]
impl Sync<SyncTimeMode> {
    /// Returns the sync time value (`--sync-time=N`).
    pub fn get_sync_time(&self) -> &str {
        &self.mode.sync_time
    }

    /// Sets the sync time value (`--sync-time=N`). The value can be Unix
    /// epoch time or the Perforce date time format.
    pub fn set_sync_time(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.sync_time = v.into();
        self
    }

    /// Sets the sync time value (`--sync-time=N`). The value can be Unix
    /// epoch time or the Perforce date time format.
    pub fn sync_time(mut self, v: impl Into<String>) -> Self {
        self.mode.sync_time = v.into();
        self
    }
}

// ---- Shared: executors + global opts ----

impl<M: ExclusiveOption, S, I> ParameterizedSpawn<(S,)> for Sync<M>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 sync` for the given files as a child process with piped
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

impl<M: ExclusiveOption> Sync<M> {
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

impl<M: ExclusiveOption> SubCommand for Sync<M> {
    fn name(&self) -> &str {
        "sync"
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

    #[test]
    fn without_options() {
        let sync = Sync::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&sync.setup_command("p4")), ["sync"]);
    }

    #[test]
    #[cfg(not(feature = "lt2025_1"))]
    fn sync_time_mode() {
        let sync = Sync::new("p4", GlobalOpts::new()).sync_time("2024/01/01");

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-k", "--sync-time=2024/01/01"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2025_1"))]
    fn sync_time_mode_epoch() {
        let sync = Sync::new("p4", GlobalOpts::new()).sync_time("1700000000");

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-k", "--sync-time=1700000000"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2025_1"))]
    fn sync_time_set_style() {
        let mut sync = Sync::new("p4", GlobalOpts::new()).sync_time("2024/01/01");
        sync.set_sync_time("2024/06/01");

        assert_eq!(sync.get_sync_time(), "2024/06/01");
        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-k", "--sync-time=2024/06/01"]
        );
    }

    #[test]
    fn regular_mode_common_options() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .script_list_mode(true)
            .quiet_mode(true)
            .limit(5);

        #[cfg(not(feature = "lt2022_2"))]
        {
            let sync = sync.verify_edge_replication(true);
            assert_eq!(
                args_of(&sync.setup_command("p4")),
                ["sync", "-E", "-L", "-q", "-m", "5"]
            );
        }

        #[cfg(feature = "lt2022_2")]
        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-L", "-q", "-m", "5"]
        );
    }

    #[test]
    fn regular_mode_force_options() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .force(true)
            .metadata_only(true);

        #[cfg(not(feature = "lt2015_1"))]
        let sync = sync.reopen_moved_files(true);

        #[cfg(not(feature = "lt2015_1"))]
        let expected = vec!["sync", "-f", "-k", "-r"];
        #[cfg(feature = "lt2015_1")]
        let expected = vec!["sync", "-f", "-k"];

        assert_eq!(args_of(&sync.setup_command("p4")), expected);
    }

    #[test]
    fn regular_mode_combined() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .quiet_mode(true)
            .force(true)
            .limit(10);

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-f", "-m", "10"]
        );
    }

    #[test]
    fn regular_mode_preview_result() {
        let sync = Sync::new("p4", GlobalOpts::new()).preview_result();

        assert_eq!(args_of(&sync.setup_command("p4")), ["sync", "-n"]);
    }

    #[test]
    fn regular_mode_preview_network_traffic() {
        let sync = Sync::new("p4", GlobalOpts::new()).preview_network_traffic();

        assert_eq!(args_of(&sync.setup_command("p4")), ["sync", "-N"]);
    }

    #[test]
    fn regular_mode_preview_with_options() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .quiet_mode(true)
            .preview_result()
            .limit(3);

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-n", "-m", "3"]
        );
    }

    #[test]
    fn regular_mode_parallel() {
        let sync = Sync::new("p4", GlobalOpts::new()).parallel(ParallelConfig {
            threads: 4,
            batch_files: Some(8),
            batch_size_bytes: None,
            min_files: Some(9),
            min_size_bytes: None,
        });

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "--parallel=threads=4,batch=8,min=9"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2022_2"))]
    fn regular_mode_stream_spec_auto() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .stream_spec_version(StreamSpecVersion::MaxInFilelists);

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "--use-stream-change"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2022_2"))]
    fn regular_mode_stream_spec_current() {
        let sync =
            Sync::new("p4", GlobalOpts::new()).stream_spec_version(StreamSpecVersion::Current);

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "--use-stream-change=0"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2022_2"))]
    fn regular_mode_stream_spec_specific() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .stream_spec_version(StreamSpecVersion::ChangeNumber(123));

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "--use-stream-change=123"]
        );
    }

    #[test]
    fn safe_check_mode() {
        let sync = Sync::new("p4", GlobalOpts::new()).enable_safe_check();

        assert_eq!(args_of(&sync.setup_command("p4")), ["sync", "-s"]);
    }

    #[test]
    fn safe_check_mode_with_options() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .enable_safe_check()
            .quiet_mode(true)
            .limit(5);

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-s", "-m", "5"]
        );
    }

    #[test]
    fn populate_mode() {
        let sync = Sync::new("p4", GlobalOpts::new()).populate_client_workspace();

        assert_eq!(args_of(&sync.setup_command("p4")), ["sync", "-p"]);
    }

    #[test]
    fn populate_mode_with_options() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .populate_client_workspace()
            .quiet_mode(true)
            .limit(5);

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-p", "-m", "5"]
        );
    }

    #[test]
    fn transition_to_safe_check_from_regular() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .quiet_mode(true)
            .limit(5)
            .safe_check();

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-s", "-m", "5"]
        );
    }

    #[test]
    fn transition_to_populate_from_regular() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .quiet_mode(true)
            .limit(5)
            .populate();

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-p", "-m", "5"]
        );
    }

    #[test]
    fn transition_preserves_preview() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .preview_result()
            .quiet_mode(true)
            .safe_check();

        assert_eq!(
            args_of(&sync.setup_command("p4")),
            ["sync", "-q", "-s", "-n"]
        );
    }

    #[test]
    fn force_mode_blocks_safe_check() {
        // Once in ForceRegularMode, safe_check/populate are unavailable at
        // compile time. This test just exercises force mode injection.
        let sync = Sync::new("p4", GlobalOpts::new())
            .force(true)
            .quiet_mode(true);

        assert_eq!(args_of(&sync.setup_command("p4")), ["sync", "-q", "-f"]);
    }

    #[test]
    fn all_regular_options_order() {
        let sync = Sync::new("p4", GlobalOpts::new())
            .script_list_mode(true)
            .quiet_mode(true)
            .force(true)
            .metadata_only(true)
            .limit(5)
            .parallel(ParallelConfig {
                threads: 2,
                batch_files: None,
                batch_size_bytes: None,
                min_files: None,
                min_size_bytes: None,
            });

        #[cfg(not(feature = "lt2022_2"))]
        let sync = sync.verify_edge_replication(true);
        #[cfg(not(feature = "lt2022_1"))]
        let sync = sync.suppress_keyword_expansion(true);
        #[cfg(not(feature = "lt2015_1"))]
        let sync = sync.reopen_moved_files(true);
        #[cfg(not(feature = "lt2022_2"))]
        let sync = sync.stream_spec_version(StreamSpecVersion::Current);

        let mut expected: Vec<&str> = vec!["sync", "-L", "-q", "-f", "-k"];
        #[cfg(not(feature = "lt2022_2"))]
        expected.insert(1, "-E");
        #[cfg(not(feature = "lt2022_1"))]
        {
            let k = expected.iter().position(|&a| a == "-L").unwrap() + 1;
            expected.insert(k, "-K");
        }
        #[cfg(not(feature = "lt2015_1"))]
        expected.push("-r");
        expected.extend(["-m", "5", "--parallel=threads=2"]);
        #[cfg(not(feature = "lt2022_2"))]
        expected.push("--use-stream-change=0");

        assert_eq!(args_of(&sync.setup_command("p4")), expected);
    }
}
