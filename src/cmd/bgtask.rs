use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Upper bound (in seconds) of the `-i` interval: the number of seconds in
/// 31 days.
const MAX_INTERVAL_SECONDS: u64 = 2_678_400;

// ---------------------------------------------------------------------------
// Task-target dimension: `-e command` vs `-t triggerName`
// ---------------------------------------------------------------------------

/// The task is given as a command string on the command line (`-e command`).
///
/// Entered with [`BackGroundTask::execute`]; mutually exclusive with
/// [`TriggerMode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandMode {
    execute: String,
}

impl ExclusiveOption for CommandMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-e").arg(&self.execute);
    }
}

/// The task is a trigger already defined in the triggers table
/// (`-t triggerName`).
///
/// Entered with [`BackGroundTask::trigger`]; mutually exclusive with
/// [`CommandMode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerMode {
    trigger: String,
}

impl ExclusiveOption for TriggerMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-t").arg(&self.trigger);
    }
}

/// Marker for a selected task target: one of [`CommandMode`] or
/// [`TriggerMode`].
///
/// Exactly one of `-e` / `-t` is required, so spawning is only available once
/// [`BackGroundTask`] has transitioned into one of these states.
pub trait TaskTarget: ExclusiveOption {}

impl TaskTarget for CommandMode {}
impl TaskTarget for TriggerMode {}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 bgtask`.
///
/// Run background commands or triggers on the server. The server saves the
/// task output to its log file; this command requires `super` access.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] bgtask [-b retries] [-d] [-i interval] [-m runCount]
///                    [-w seconds] {-e command | -t triggerName}
/// ```
///
/// The `M` type parameter tracks the required task target at compile time:
/// it starts as [`Unselected`], and one of [`Self::execute`] /
/// [`Self::trigger`] must be used to move into the [`CommandMode`] /
/// [`TriggerMode`] state before the command can be spawned. The two states
/// can be switched freely with the same two methods.
#[derive(Debug, Clone, Default)]
pub struct BackGroundTask<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    /// `-b retries`
    retries: Option<u64>,

    /// `-d`
    detach: bool,

    /// `-i interval`
    interval: Option<u64>,

    /// `-m runCount`
    runs: Option<u64>,

    /// `-w seconds`
    wait: Option<u64>,

    mode: M,
}

impl BackGroundTask<Unselected> {
    /// Creates a new `p4 bgtask` command with no task target selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. Use
    /// [`Self::execute`] or [`Self::trigger`] to select the required
    /// `-e` / `-t` target.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }
}

impl<M> BackGroundTask<M> {
    /// Selects `-e command`: run the given command string on the server.
    ///
    /// This transitions into the [`CommandMode`] state, discarding any
    /// previously selected `-e` / `-t` target; the scheduling options are
    /// preserved.
    pub fn execute(self, command: impl Into<String>) -> BackGroundTask<CommandMode> {
        BackGroundTask {
            bin: self.bin,
            global_opts: self.global_opts,
            retries: self.retries,
            detach: self.detach,
            interval: self.interval,
            runs: self.runs,
            wait: self.wait,
            mode: CommandMode {
                execute: command.into(),
            },
        }
    }

    /// Selects `-t triggerName`: run the named bgtask trigger, already defined
    /// in the triggers table, on the server.
    ///
    /// This transitions into the [`TriggerMode`] state, discarding any
    /// previously selected `-e` / `-t` target; the scheduling options are
    /// preserved.
    pub fn trigger(self, name: impl Into<String>) -> BackGroundTask<TriggerMode> {
        BackGroundTask {
            bin: self.bin,
            global_opts: self.global_opts,
            retries: self.retries,
            detach: self.detach,
            interval: self.interval,
            runs: self.runs,
            wait: self.wait,
            mode: TriggerMode {
                trigger: name.into(),
            },
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
    /// -b retries
    ///
    /// Maximum number of execution errors before ceasing to attempt
    /// execution. The default is `1`.
    pub fn get_retries(&self) -> Option<u64> {
        self.retries
    }

    /// # Description
    ///
    /// -b retries
    ///
    /// Maximum number of execution errors before ceasing to attempt
    /// execution. The default is `1`.
    pub fn set_retries(&mut self, v: u64) -> &mut Self {
        self.retries = Some(v);
        self
    }

    /// # Description
    ///
    /// -b retries
    ///
    /// Maximum number of execution errors before ceasing to attempt
    /// execution. The default is `1`.
    pub fn retries(mut self, v: u64) -> Self {
        self.retries = Some(v);
        self
    }

    /// # Description
    ///
    /// -d
    ///
    /// Detach the client so it does not see the output of the server-side
    /// task execution. Not allowed when the task is specified in a
    /// `startup.N` configurable.
    pub fn get_detach(&self) -> bool {
        self.detach
    }

    /// # Description
    ///
    /// -d
    ///
    /// Detach the client so it does not see the output of the server-side
    /// task execution. Not allowed when the task is specified in a
    /// `startup.N` configurable.
    pub fn set_detach(&mut self, v: bool) -> &mut Self {
        self.detach = v;
        self
    }

    /// # Description
    ///
    /// -d
    ///
    /// Detach the client so it does not see the output of the server-side
    /// task execution. Not allowed when the task is specified in a
    /// `startup.N` configurable.
    pub fn detach(mut self, v: bool) -> Self {
        self.detach = v;
        self
    }

    /// # Description
    ///
    /// -i interval
    ///
    /// Seconds between command invocations. The default is `1` second; the
    /// maximum is 2,678,400 (the number of seconds in 31 days).
    pub fn get_interval(&self) -> Option<u64> {
        self.interval
    }

    /// # Description
    ///
    /// -i interval
    ///
    /// Seconds between command invocations. The default is `1` second; the
    /// maximum is 2,678,400 (the number of seconds in 31 days).
    pub fn set_interval(&mut self, v: u64) -> &mut Self {
        debug_assert!(
            v <= MAX_INTERVAL_SECONDS,
            "bgtask -i interval must not exceed {MAX_INTERVAL_SECONDS} seconds (31 days)"
        );
        self.interval = Some(v);
        self
    }

    /// # Description
    ///
    /// -i interval
    ///
    /// Seconds between command invocations. The default is `1` second; the
    /// maximum is 2,678,400 (the number of seconds in 31 days).
    pub fn interval(mut self, v: u64) -> Self {
        debug_assert!(
            v <= MAX_INTERVAL_SECONDS,
            "bgtask -i interval must not exceed {MAX_INTERVAL_SECONDS} seconds (31 days)"
        );
        self.interval = Some(v);
        self
    }

    /// # Description
    ///
    /// -m runCount
    ///
    /// Maximum number of times the command is run. The default is `1`.
    pub fn get_runs(&self) -> Option<u64> {
        self.runs
    }

    /// # Description
    ///
    /// -m runCount
    ///
    /// Maximum number of times the command is run. The default is `1`.
    pub fn set_runs(&mut self, v: u64) -> &mut Self {
        self.runs = Some(v);
        self
    }

    /// # Description
    ///
    /// -m runCount
    ///
    /// Maximum number of times the command is run. The default is `1`.
    pub fn runs(mut self, v: u64) -> Self {
        self.runs = Some(v);
        self
    }

    /// # Description
    ///
    /// -w seconds
    ///
    /// Seconds to wait after an execution error before attempting the next
    /// execution. The default is `5`.
    pub fn get_wait(&self) -> Option<u64> {
        self.wait
    }

    /// # Description
    ///
    /// -w seconds
    ///
    /// Seconds to wait after an execution error before attempting the next
    /// execution. The default is `5`.
    pub fn set_wait(&mut self, v: u64) -> &mut Self {
        self.wait = Some(v);
        self
    }

    /// # Description
    ///
    /// -w seconds
    ///
    /// Seconds to wait after an execution error before attempting the next
    /// execution. The default is `5`.
    pub fn wait(mut self, v: u64) -> Self {
        self.wait = Some(v);
        self
    }
}

impl BackGroundTask<CommandMode> {
    /// The command string selected with `-e`.
    pub fn get_command(&self) -> &str {
        &self.mode.execute
    }
}

impl BackGroundTask<TriggerMode> {
    /// The trigger name selected with `-t`.
    pub fn get_trigger(&self) -> &str {
        &self.mode.trigger
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> SubCommand for BackGroundTask<M> {
    fn name(&self) -> &str {
        "bgtask"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(retries) = self.retries {
            command.arg("-b").arg(retries.to_string());
        }

        if self.detach {
            command.arg("-d");
        }

        if let Some(interval) = self.interval {
            command.arg("-i").arg(interval.to_string());
        }

        if let Some(runs) = self.runs {
            command.arg("-m").arg(runs.to_string());
        }

        if let Some(wait) = self.wait {
            command.arg("-w").arg(wait.to_string());
        }

        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning: only after a task target (`-e` / `-t`) has been selected
// ---------------------------------------------------------------------------

impl<M: TaskTarget> ParameterizedSpawn<()> for BackGroundTask<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 bgtask` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
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

    /// Dry-run checks of the assembled `p4 bgtask` command line; no process
    /// is spawned.
    #[test]
    fn execute_target_only() {
        let task = BackGroundTask::new("p4", GlobalOpts::new()).execute("top -b -n 1");

        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["bgtask", "-e", "top -b -n 1"]
        );
        assert_eq!(task.get_command(), "top -b -n 1");
    }

    #[test]
    fn trigger_target_only() {
        let task = BackGroundTask::new("p4", GlobalOpts::new()).trigger("log_checker");

        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["bgtask", "-t", "log_checker"]
        );
        assert_eq!(task.get_trigger(), "log_checker");
    }

    #[test]
    fn all_options_injected_in_syntax_order() {
        let task = BackGroundTask::new("p4", GlobalOpts::new())
            .retries(3)
            .detach(true)
            .interval(86_400)
            .runs(5)
            .wait(10)
            .trigger("p4dstate");

        assert_eq!(
            args_of(&task.setup_command("p4")),
            [
                "bgtask", "-b", "3", "-d", "-i", "86400", "-m", "5", "-w", "10", "-t", "p4dstate"
            ]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let task =
            BackGroundTask::new("p4", GlobalOpts::new().port("localhost:1666")).execute("verify");

        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["-p", "localhost:1666", "bgtask", "-e", "verify"]
        );
    }

    #[test]
    fn switching_targets_replaces_selection() {
        let task = BackGroundTask::new("p4", GlobalOpts::new())
            .runs(2)
            .execute("old command")
            .trigger("verify");

        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["bgtask", "-m", "2", "-t", "verify"]
        );
        assert_eq!(task.get_trigger(), "verify");

        let task = task.execute("new command");
        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["bgtask", "-m", "2", "-e", "new command"]
        );
        assert_eq!(task.get_command(), "new command");
    }

    #[test]
    fn interval_accepts_the_document_maximum() {
        let task = BackGroundTask::new("p4", GlobalOpts::new())
            .interval(MAX_INTERVAL_SECONDS)
            .trigger("verify");

        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["bgtask", "-i", "2678400", "-t", "verify"]
        );
    }

    #[test]
    #[should_panic(expected = "31 days")]
    fn interval_above_maximum_panics_in_debug() {
        let _ = BackGroundTask::new("p4", GlobalOpts::new()).interval(MAX_INTERVAL_SECONDS + 1);
    }

    #[test]
    fn setters_update_the_same_command() {
        let mut task = BackGroundTask::new("p4", GlobalOpts::new()).trigger("verify");
        task.set_retries(4).set_detach(true).set_wait(30);

        assert_eq!(task.get_retries(), Some(4));
        assert!(task.get_detach());
        assert_eq!(task.get_wait(), Some(30));
        assert_eq!(
            args_of(&task.setup_command("p4")),
            ["bgtask", "-b", "4", "-d", "-w", "30", "-t", "verify"]
        );
    }
}
