use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Maximum value of the `-w` quiesce wait: one hour, in seconds.
const MAX_QUIESCE_WAIT_SECONDS: u64 = 3600;

/// Maximum value of the `-v` verification time: twelve hours, in seconds.
const MAX_VERIFICATION_TIME_SECONDS: u64 = 43_200;

/// `p4 [g-opts] failback [-y] [-w quiesce_wait] [-v verification_time] [failback_message]`
///
#[cfg_attr(
    feature = "lt2025_1",
    doc = "After a failover with master server participation, fail back to restore the previous role of that master server."
)]
#[cfg_attr(
    not(feature = "lt2025_1"),
    doc = "After a failover with target server participation, fail back to restore the previous role of that target server."
)]
///
#[cfg_attr(
    feature = "lt2024_1",
    doc = "After a failover (see Failover in Helix Core Server Administrator Guide) in which the master participated, `p4 failback` can reestablish the relationship between the master server and the standby server to what it was prior to failover. See the prerequisites and procedure for Failback after failover in Helix Core Server Administrator Guide."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2024_1")),
    doc = "After a failover (see Failover in the Helix Core Server Administrator Guide) in which the master participated, `p4 failback` can reestablish the relationship between the master server and the standby server to what it was prior to failover. See the prerequisites and procedure for Failback after failover in the Helix Core Server Administrator Guide."
)]
#[cfg_attr(
    all(feature = "lt2025_1", not(feature = "lt2024_2")),
    doc = "After a failover (see Failover in the P4 Server Administration Documentation) in which the master participated, `p4 failback` can reestablish the relationship between the master server and the standby server to what it was prior to failover. See the prerequisites and procedure for Failback after failover in the P4 Server Administration Documentation."
)]
#[cfg_attr(
    not(feature = "lt2025_1"),
    doc = "After a failover (see Failover in the P4 Server Administration Documentation) in which the target server participated, `p4 failback` can reestablish the relationship between the target server and the standby server to what it was prior to failover. See the prerequisites and procedure for Failback after failover in the P4 Server Administration Documentation."
)]
///
/// Without [`Self::yes`], the command merely reports what it would do.
#[derive(Debug, Clone, Default)]
pub struct Failback {
    bin: PathBuf,

    global_opts: GlobalOpts,

    yes: bool,

    quiesce_wait: Option<u64>,

    verification_time: Option<u64>,
}

impl Failback {
    /// Creates a new `p4 failback` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            yes: false,
            quiesce_wait: None,
            verification_time: None,
        }
    }

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

    /// # Description
    ///
    /// -y
    ///
    /// Perform the failback operation. Without this option, `p4 failback`
    /// merely reports what it would do.
    ///
    /// > Carefully consider the report of what failback would do BEFORE you
    /// > include the `-y` option and launch the operation. During the failback
    /// > process, end-user clients cannot issue commands against the server.
    pub fn get_yes(&self) -> bool {
        self.yes
    }

    /// # Description
    ///
    /// -y
    ///
    /// Perform the failback operation. Without this option, `p4 failback`
    /// merely reports what it would do.
    ///
    /// > Carefully consider the report of what failback would do BEFORE you
    /// > include the `-y` option and launch the operation. During the failback
    /// > process, end-user clients cannot issue commands against the server.
    pub fn set_yes(&mut self, v: bool) -> &mut Self {
        self.yes = v;
        self
    }

    /// # Description
    ///
    /// -y
    ///
    /// Perform the failback operation. Without this option, `p4 failback`
    /// merely reports what it would do.
    ///
    /// > Carefully consider the report of what failback would do BEFORE you
    /// > include the `-y` option and launch the operation. During the failback
    /// > process, end-user clients cannot issue commands against the server.
    pub fn yes(mut self, v: bool) -> Self {
        self.yes = v;
        self
    }

    /// # Description
    ///
    /// -w quiesce_wait
    ///
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "The waiting period in seconds for commands in the master server to complete. At the end of this `quiesceWait` period, all user commands, regardless of their activity, will be stalled."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The waiting period in seconds for commands in the target server to complete. At the end of this `quiesceWait` period, all user commands, regardless of their activity, will be stalled."
    )]
    ///
    /// If `-w` is not specified, the default `quiesceWait` value is `60`
    ///
    /// You can set the waiting period to any integer value between `0`
    /// (immediate) and `3600` seconds (1 hour).
    ///
    /// To cancel the failback during the `quiesceWait` period, press Ctrl-C
    pub fn get_quiesce_wait(&self) -> Option<u64> {
        self.quiesce_wait
    }

    /// # Description
    ///
    /// -w quiesce_wait
    ///
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "The waiting period in seconds for commands in the master server to complete. At the end of this `quiesceWait` period, all user commands, regardless of their activity, will be stalled."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The waiting period in seconds for commands in the target server to complete. At the end of this `quiesceWait` period, all user commands, regardless of their activity, will be stalled."
    )]
    ///
    /// If `-w` is not specified, the default `quiesceWait` value is `60`
    ///
    /// You can set the waiting period to any integer value between `0`
    /// (immediate) and `3600` seconds (1 hour).
    ///
    /// To cancel the failback during the `quiesceWait` period, press Ctrl-C
    pub fn set_quiesce_wait(&mut self, v: u64) -> &mut Self {
        debug_assert!(
            v <= MAX_QUIESCE_WAIT_SECONDS,
            "failback -w quiesce_wait must not exceed {MAX_QUIESCE_WAIT_SECONDS} seconds (1 hour)"
        );
        self.quiesce_wait = Some(v);
        self
    }

    /// # Description
    ///
    /// -w quiesce_wait
    ///
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "The waiting period in seconds for commands in the master server to complete. At the end of this `quiesceWait` period, all user commands, regardless of their activity, will be stalled."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The waiting period in seconds for commands in the target server to complete. At the end of this `quiesceWait` period, all user commands, regardless of their activity, will be stalled."
    )]
    ///
    /// If `-w` is not specified, the default `quiesceWait` value is `60`
    ///
    /// You can set the waiting period to any integer value between `0`
    /// (immediate) and `3600` seconds (1 hour).
    ///
    /// To cancel the failback during the `quiesceWait` period, press Ctrl-C
    pub fn quiesce_wait(mut self, v: u64) -> Self {
        self.set_quiesce_wait(v);
        self
    }

    /// # Description
    ///
    /// -v verification_time
    ///
    /// Specifies the `verificationTime` as a number of seconds prior to the
    /// launch of the failback command. The default value of `300` seconds
    /// means that any file content that was updated from 1 - 300 seconds prior
    /// to launch of failback will be verified. This is to ensure that this
    /// file content will be correct when the failback process completes on the
    #[cfg_attr(feature = "lt2025_1", doc = "new master server.")]
    #[cfg_attr(not(feature = "lt2025_1"), doc = "new target server.")]
    ///
    /// The failback command proceeds as soon as it has determined that the new
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "master server has received correct file content."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "target server has received correct file content."
    )]
    ///
    /// The legal range is between `0` (no verification) and `43200` seconds
    /// (12 hours).
    pub fn get_verification_time(&self) -> Option<u64> {
        self.verification_time
    }

    /// # Description
    ///
    /// -v verification_time
    ///
    /// Specifies the `verificationTime` as a number of seconds prior to the
    /// launch of the failback command. The default value of `300` seconds
    /// means that any file content that was updated from 1 - 300 seconds prior
    /// to launch of failback will be verified. This is to ensure that this
    /// file content will be correct when the failback process completes on the
    #[cfg_attr(feature = "lt2025_1", doc = "new master server.")]
    #[cfg_attr(not(feature = "lt2025_1"), doc = "new target server.")]
    ///
    /// The failback command proceeds as soon as it has determined that the new
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "master server has received correct file content."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "target server has received correct file content."
    )]
    ///
    /// The legal range is between `0` (no verification) and `43200` seconds
    /// (12 hours).
    pub fn set_verification_time(&mut self, v: u64) -> &mut Self {
        debug_assert!(
            v <= MAX_VERIFICATION_TIME_SECONDS,
            "failback -v verification_time must not exceed {MAX_VERIFICATION_TIME_SECONDS} seconds (12 hours)"
        );
        self.verification_time = Some(v);
        self
    }

    /// # Description
    ///
    /// -v verification_time
    ///
    /// Specifies the `verificationTime` as a number of seconds prior to the
    /// launch of the failback command. The default value of `300` seconds
    /// means that any file content that was updated from 1 - 300 seconds prior
    /// to launch of failback will be verified. This is to ensure that this
    /// file content will be correct when the failback process completes on the
    #[cfg_attr(feature = "lt2025_1", doc = "new master server.")]
    #[cfg_attr(not(feature = "lt2025_1"), doc = "new target server.")]
    ///
    /// The failback command proceeds as soon as it has determined that the new
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "master server has received correct file content."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "target server has received correct file content."
    )]
    ///
    /// The legal range is between `0` (no verification) and `43200` seconds
    /// (12 hours).
    pub fn verification_time(mut self, v: u64) -> Self {
        self.set_verification_time(v);
        self
    }
}

impl SubCommand for Failback {
    fn name(&self) -> &str {
        "failback"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.yes {
            command.arg("-y");
        }

        if let Some(quiesce_wait) = self.quiesce_wait {
            command.arg("-w").arg(quiesce_wait.to_string());
        }

        if let Some(verification_time) = self.verification_time {
            command.arg("-v").arg(verification_time.to_string());
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Failback {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 failback` with the given `failback_message` as the final
    /// argument; the process is a child process with piped standard output
    /// and error streams, use the returned [`Child`] handle to wait for it or
    /// interact with it.
    ///
    /// # Description
    ///
    /// failback_message
    ///
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "The text to display to end-users during the failback operation when end-users attempt to start new commands on the master server. The default message is:"
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The text to display to end-users during the failback operation when end-users attempt to start new commands on the target server. The default message is:"
    )]
    ///
    /// Server currently in failback mode, try again after failback has
    /// completed
    ///
    /// The `failbackMessage` must be the final argument specified in the
    /// `p4 failback` command. For example, `p4 failback -w 90 -v 500 -y The
    /// Perforce server is undergoing scheduled maintenance` and quotation
    /// marks are not required.
    fn spawn_with(&mut self, (message,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(message)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<()> for Failback {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 failback` without a `failback_message` as a child process
    /// with piped standard output and error streams; use the returned
    /// [`Child`] handle to wait for it or interact with it.
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
        let mut command = Failback::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 failback` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let failback = Failback::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&failback.setup_command("p4")), ["failback"]);
    }

    #[test]
    fn yes_flag() {
        let failback = Failback::new("p4", GlobalOpts::new()).yes(true);

        assert!(failback.get_yes());
        assert_eq!(args_of(&failback.setup_command("p4")), ["failback", "-y"]);
    }

    #[test]
    fn quiesce_wait_and_verification_time() {
        let failback = Failback::new("p4", GlobalOpts::new())
            .quiesce_wait(90)
            .verification_time(500);

        assert_eq!(failback.get_quiesce_wait(), Some(90));
        assert_eq!(failback.get_verification_time(), Some(500));
        assert_eq!(
            args_of(&failback.setup_command("p4")),
            ["failback", "-w", "90", "-v", "500"]
        );
    }

    #[test]
    fn all_options_in_syntax_order() {
        let mut failback = Failback::new("p4", GlobalOpts::new()).yes(true);
        failback
            .set_quiesce_wait(MAX_QUIESCE_WAIT_SECONDS)
            .set_verification_time(MAX_VERIFICATION_TIME_SECONDS);

        assert_eq!(
            args_of(&failback.setup_command("p4")),
            ["failback", "-y", "-w", "3600", "-v", "43200"]
        );
    }

    #[test]
    #[should_panic(expected = "1 hour")]
    fn quiesce_wait_above_maximum_panics_in_debug() {
        let _ = Failback::new("p4", GlobalOpts::new()).quiesce_wait(MAX_QUIESCE_WAIT_SECONDS + 1);
    }

    #[test]
    #[should_panic(expected = "12 hours")]
    fn verification_time_above_maximum_panics_in_debug() {
        let _ = Failback::new("p4", GlobalOpts::new())
            .verification_time(MAX_VERIFICATION_TIME_SECONDS + 1);
    }

    #[test]
    fn global_opts_are_injected_once() {
        let failback = Failback::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .yes(true);

        assert_eq!(
            args_of(&failback.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "failback", "-y"]
        );
    }
}
