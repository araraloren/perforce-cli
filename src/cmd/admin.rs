use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::{ParameterizedSpawn, SpawnExt};

/// Entry point for the `p4 admin` subcommands.
///
/// Allows Perforce superusers to perform administrative tasks even when
/// working from a different machine than the one running the shared Perforce
/// service. Use one of the builder methods to select the operation:
/// [`checkpoint`](Self::checkpoint), [`journal`](Self::journal),
/// [`stop`](Self::stop), [`restart`](Self::restart),
/// [`updatespecdepot`](Self::updatespecdepot),
#[cfg_attr(
    not(feature = "lt2015_1"),
    doc = " [`setldapusers`](Self::setldapusers),"
)]
#[cfg_attr(
    not(feature = "lt2018_1"),
    doc = " [`end_journal`](Self::end_journal),"
)]
#[cfg_attr(
    not(feature = "lt2023_1"),
    doc = " [`sysinfo`](Self::sysinfo), [`resource_monitor`](Self::resource_monitor),"
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = " [`replica_filter_reconcile`](Self::replica_filter_reconcile),"
)]
/// or [`resetpassword`](Self::resetpassword).
#[derive(Debug, Clone, Default)]
pub struct AdminEntry {
    bin: PathBuf,

    global_opts: GlobalOpts,
}

impl AdminEntry {
    /// Creates the entry point for `p4 admin` subcommands.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
        }
    }

    /// Take a checkpoint.
    ///
    /// Equivalent to logging in to the server machine and running
    /// `p4d -jc [prefix]`: a checkpoint is taken and the journal is copied to
    /// a numbered file.
    pub fn checkpoint(self) -> Admin<CheckPoint<Unselected>> {
        Admin::new(self.bin, self.global_opts, CheckPoint::default())
    }

    /// Rotate the journal.
    ///
    /// Equivalent to `p4d -jj`. The files are created in the server root
    /// specified when the Perforce service was started.
    pub fn journal(self) -> Admin<Journal> {
        Admin::new(self.bin, self.global_opts, Journal::default())
    }

    /// Stop the Perforce service.
    ///
    /// Locks the database to ensure that it is in a consistent state upon
    /// restart, and then shuts down the Perforce background process.
    pub fn stop(self) -> Admin<Stop> {
        Admin::new(self.bin, self.global_opts, Stop)
    }

    /// Restart the Perforce service.
    ///
    /// Locks the database, restarts the service, and applies any
    /// `p4 configure` settings that require a restart.
    pub fn restart(self) -> Admin<Restart> {
        Admin::new(self.bin, self.global_opts, Restart)
    }

    /// Archive stored forms into the spec depot.
    ///
    /// Causes the Perforce service to archive stored forms (specifically the
    /// `client`, `depot`, `branch`, `label`, `typemap`, `group`, `user`, and
    /// `job` forms) into the spec depot. Only those forms that have not yet
    /// been archived are created. The spec depot must exist first.
    pub fn updatespecdepot(self) -> Admin<UpdateSpecDepot<Unselected>> {
        Admin::new(self.bin, self.global_opts, UpdateSpecDepot::default())
    }

    /// Force users to change their passwords.
    ///
    /// Forces specified users with existing passwords to change their
    /// passwords before they can run another command.
    pub fn resetpassword(self) -> Admin<ResetPassword<Unselected>> {
        Admin::new(self.bin, self.global_opts, ResetPassword::default())
    }

    /// Set the LDAP users.
    ///
    /// Converts all existing non-super users to use LDAP authentication. The
    /// command changes the `AuthMethod` field in the user specification for
    /// each user from `perforce` to `ldap`. If super users want to use LDAP
    /// authentication, they must set their `AuthMethod` manually.
    #[cfg(not(feature = "lt2015_1"))]
    pub fn setldapusers(self) -> Admin<SetLdapUsers> {
        Admin::new(self.bin, self.global_opts, SetLdapUsers)
    }

    /// End journal replication at a failover consistency point.
    ///
    /// In a failover scenario, this command ends journal replication at the
    /// most recent successfully replicated consistency point, returns the
    /// journal number and the offset of that consistency point, and stops the
    /// standby server's journalcopy thread.
    #[cfg(not(feature = "lt2018_1"))]
    pub fn end_journal(self) -> Admin<EndJournal> {
        Admin::new(self.bin, self.global_opts, EndJournal)
    }

    /// Dump system information for Perforce Support.
    ///
    /// Dumps the output of reporting commands as run on the server host
    /// operating system. This is intended for use under guidance of Perforce
    /// Support to gather information about the environment of
    #[cfg_attr(
        all(not(feature = "lt2023_1"), feature = "lt2024_2"),
        doc = "Helix Core Server."
    )]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "P4 Server.")]
    #[cfg(not(feature = "lt2023_1"))]
    pub fn sysinfo(self) -> Admin<SysInfo> {
        Admin::new(self.bin, self.global_opts, SysInfo)
    }

    /// Report server resource usage.
    ///
    /// Explained in the output of `p4 help admin-resource-monitor`. See also
    /// System resources in the Performance tuning chapter of
    #[cfg_attr(
        all(not(feature = "lt2023_1"), feature = "lt2024_1"),
        doc = "Helix Core Server",
        doc = "Administrator Guide."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_1"), feature = "lt2024_2"),
        doc = "the Helix Core Server Administrator Guide."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server",
        doc = "Administration Documentation."
    )]
    #[cfg(not(feature = "lt2023_1"))]
    pub fn resource_monitor(self) -> Admin<ResourceMonitor> {
        Admin::new(self.bin, self.global_opts, ResourceMonitor)
    }

    /// Reconcile a replica after its filter rules change.
    ///
    /// By default, if the filtering rules change in a replica or edge server
    /// spec, replication adjusts automatically; a set of `rpl.filter.*`
    /// configurables controls that behavior. This command performs the
    /// reconciliation manually.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn replica_filter_reconcile(self) -> Admin<ReplicaFilterReconcile<Unselected>> {
        Admin::new(
            self.bin,
            self.global_opts,
            ReplicaFilterReconcile::default(),
        )
    }
}

/// A `p4 admin` operation wrapping a selected [`SubCommand`].
#[derive(Debug, Clone, Default)]
pub struct Admin<T: SubCommand> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    sub_command: T,
}

impl<T: SubCommand> SubCommand for Admin<T> {
    fn name(&self) -> &str {
        "admin"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.sub_command.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---- Executors ----
//
// Each `p4 admin` subcommand type-state implements `ParameterizedSpawn`; for
// the no-input states the blanket `SpawnExt`/`ParameterizedOutput`/`OutputExt`
// impls in [crate::cmd] provide `spawn`, `output_with`, and `output`.

impl<T: SubCommand> Admin<T> {
    /// Spawns the assembled `p4 admin` command as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_piped(&mut self) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn for Admin<Stop> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin stop` as a child process with piped standard output
    /// and error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

impl ParameterizedSpawn for Admin<Restart> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin restart` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

impl<S: ExclusiveOption> ParameterizedSpawn for Admin<UpdateSpecDepot<S>> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin updatespecdepot` as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

impl<T: ExclusiveOption> ParameterizedSpawn for Admin<ResetPassword<T>> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin resetpassword` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

#[cfg(not(feature = "lt2015_1"))]
impl ParameterizedSpawn for Admin<SetLdapUsers> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin setldapusers` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

#[cfg(not(feature = "lt2018_1"))]
impl ParameterizedSpawn for Admin<EndJournal> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin endjournal` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl ParameterizedSpawn for Admin<SysInfo> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin sysinfo` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

#[cfg(not(feature = "lt2023_1"))]
impl ParameterizedSpawn for Admin<ResourceMonitor> {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin resource-monitor` as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_piped()
    }
}

impl<T: SubCommand> Admin<T> {
    /// Creates a `p4 admin` command wrapping the given subcommand.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts, sub_command: T) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            sub_command,
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

pub mod compression {
    /// Compress both the checkpoint and the journal (`-z`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Both;

    /// Compress the checkpoint only, leaving the journal uncompressed
    /// (`-Z`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct CheckPointOnly;
}

impl ExclusiveOption for compression::Both {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-z");
    }
}

impl ExclusiveOption for compression::CheckPointOnly {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-Z");
    }
}

#[cfg_attr(
    feature = "lt2022_2",
    doc = "`p4 admin checkpoint [-z | -Z] [prefix]`: take a checkpoint."
)]
#[cfg_attr(
    all(feature = "lt2023_1", not(feature = "lt2022_2")),
    doc = "`p4 admin checkpoint [[-z | -Z]] [prefix]`: take a checkpoint."
)]
#[cfg_attr(
    not(feature = "lt2023_1"),
    doc = "`p4 admin checkpoint [-z | -Z] [-p [-N threads] [-m]] [prefix]`: take a checkpoint."
)]
/// The `C` type parameter encodes the compression mode (none, `-z`, or
/// `-Z`) at compile time; see [`ExclusiveOption`].
#[derive(Debug, Clone, Default)]
pub struct CheckPoint<C = Unselected> {
    compression: C,

    /// Added in p4 2023.1.
    #[cfg(not(feature = "lt2023_1"))]
    parallel: bool,

    /// Added in p4 2023.1.
    #[cfg(not(feature = "lt2023_1"))]
    threads: Option<u32>,

    /// Added in p4 2023.1.
    #[cfg(not(feature = "lt2023_1"))]
    multiple_files: bool,
}

impl<C: ExclusiveOption> SubCommand for CheckPoint<C> {
    fn name(&self) -> &str {
        "checkpoint"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.compression.inject_args(command);

        #[cfg(not(feature = "lt2023_1"))]
        {
            if self.parallel {
                command.arg("-p");
            }
            if let Some(threads) = self.threads {
                command.arg("-N").arg(threads.to_string());
            }
            if self.multiple_files {
                command.arg("-m");
            }
        }
    }
}

impl<C: ExclusiveOption> ParameterizedSpawn for Admin<CheckPoint<C>> {
    type Input<'a> = Option<&'a str>;
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin checkpoint` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    ///
    /// Pass `Some(prefix)` to name the checkpoint with the given prefix, or
    /// `None` to use the default checkpoint name.
    fn spawn_with<'a>(&mut self, prefix: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut command = self.setup_command(&self.bin);

        if let Some(prefix) = prefix {
            command.arg(prefix);
        }
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<C: ExclusiveOption> SpawnExt for Admin<CheckPoint<C>> {
    fn spawn<'a>(&mut self) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_with(None)
    }
}

impl<C: ExclusiveOption> Admin<CheckPoint<C>> {
    /// # Description
    ///
    /// -p
    ///
    /// Requests a parallel checkpoint.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_parallel(&self) -> bool {
        self.sub_command.parallel
    }

    /// # Description
    ///
    /// -p
    ///
    /// Requests a parallel checkpoint.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_parallel(&mut self, parallel: bool) -> &mut Self {
        self.sub_command.parallel = parallel;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    /// Requests a parallel checkpoint.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn parallel(mut self, parallel: bool) -> Self {
        self.sub_command.parallel = parallel;
        self
    }

    /// # Description
    ///
    /// -N threads
    ///
    /// Specifies the number of threads to use during the parallel request.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_threads(&self) -> Option<&u32> {
        self.sub_command.threads.as_ref()
    }

    /// # Description
    ///
    /// -N threads
    ///
    /// Specifies the number of threads to use during the parallel request.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_threads(&mut self, threads: u32) -> &mut Self {
        self.sub_command.threads = Some(threads);
        self
    }

    /// # Description
    ///
    /// -N threads
    ///
    /// Specifies the number of threads to use during the parallel request.
    #[cfg(not(feature = "lt2023_1"))]
    pub fn threads(mut self, threads: u32) -> Self {
        self.sub_command.threads = Some(threads);
        self
    }

    /// # Description
    ///
    /// -m
    ///
    /// Uses multiple files if there are multiple parallel threads because
    /// `db.checkpoint.threads` is greater than 1 or the `-N` option is greater
    /// than 1. See Parallel checkpointing, dumping and recovery
    #[cfg_attr(
        all(not(feature = "lt2023_1"), feature = "lt2024_1"),
        doc = "in Helix Core",
        doc = "Server Administrator Guide. See also checkpoint examples."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_1"), feature = "lt2024_2"),
        doc = "in the Helix",
        doc = "Core Server Administrator Guide. See also checkpoint examples."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_2"), feature = "lt2025_1"),
        doc = "in the P4",
        doc = "Server Administration Documentation. See also checkpoint examples."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "in the P4",
        doc = "Server Administration Documentation. See also Checkpoint examples."
    )]
    #[cfg(not(feature = "lt2023_1"))]
    pub fn get_multiple_files(&self) -> bool {
        self.sub_command.multiple_files
    }

    /// # Description
    ///
    /// -m
    ///
    /// Uses multiple files if there are multiple parallel threads because
    /// `db.checkpoint.threads` is greater than 1 or the `-N` option is greater
    /// than 1. See Parallel checkpointing, dumping and recovery
    #[cfg_attr(
        all(not(feature = "lt2023_1"), feature = "lt2024_1"),
        doc = "in Helix Core",
        doc = "Server Administrator Guide. See also checkpoint examples."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_1"), feature = "lt2024_2"),
        doc = "in the Helix",
        doc = "Core Server Administrator Guide. See also checkpoint examples."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_2"), feature = "lt2025_1"),
        doc = "in the P4",
        doc = "Server Administration Documentation. See also checkpoint examples."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "in the P4",
        doc = "Server Administration Documentation. See also Checkpoint examples."
    )]
    #[cfg(not(feature = "lt2023_1"))]
    pub fn set_multiple_files(&mut self, multiple_files: bool) -> &mut Self {
        self.sub_command.multiple_files = multiple_files;
        self
    }

    /// # Description
    ///
    /// -m
    ///
    /// Uses multiple files if there are multiple parallel threads because
    /// `db.checkpoint.threads` is greater than 1 or the `-N` option is greater
    /// than 1. See Parallel checkpointing, dumping and recovery
    #[cfg_attr(
        all(not(feature = "lt2023_1"), feature = "lt2024_1"),
        doc = "in Helix Core",
        doc = "Server Administrator Guide. See also checkpoint examples."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_1"), feature = "lt2024_2"),
        doc = "in the Helix",
        doc = "Core Server Administrator Guide. See also checkpoint examples."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_2"), feature = "lt2025_1"),
        doc = "in the P4",
        doc = "Server Administration Documentation. See also checkpoint examples."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "in the P4",
        doc = "Server Administration Documentation. See also Checkpoint examples."
    )]
    #[cfg(not(feature = "lt2023_1"))]
    pub fn multiple_files(mut self, multiple_files: bool) -> Self {
        self.sub_command.multiple_files = multiple_files;
        self
    }
}

impl Admin<CheckPoint<Unselected>> {
    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "For `p4 admin checkpoint` and `p4 admin journal`, save the checkpoint",
        doc = "and saved journal file in compressed (gzip) format, appending the `.gz`",
        doc = "suffix to the files."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_2")),
        doc = "For `p4 admin checkpoint -z` and `p4 admin journal -z`, save the",
        doc = "checkpoint and journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z` or `-Z`, no compression occurs."
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Save the checkpoint and journal file in compressed format. The `.gz`",
        doc = "suffix is appended to compressed journals and checkpoint files, which",
        doc = "are in gzip format. If you do not specify `-z` or `-Z`, no compression",
        doc = "occurs."
    )]
    pub fn compress_both(self) -> Admin<CheckPoint<compression::Both>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: CheckPoint::<compression::Both> {
                compression: compression::Both,
                #[cfg(not(feature = "lt2023_1"))]
                parallel: self.sub_command.parallel,
                #[cfg(not(feature = "lt2023_1"))]
                threads: self.sub_command.threads,
                #[cfg(not(feature = "lt2023_1"))]
                multiple_files: self.sub_command.multiple_files,
            },
        }
    }

    /// # Description
    ///
    /// -Z
    ///
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "For `p4 admin checkpoint` and `p4 admin journal`, save the checkpoint",
        doc = "in compressed (gzip) format, appending the `.gz` suffix to the file, but",
        doc = "leave the journal uncompressed for use by replica servers."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2017_2")),
        doc = "For `p4 admin checkpoint`, save the checkpoint in compressed (gzip)",
        doc = "format, appending the `.gz` suffix to the file, but leave the journal",
        doc = "uncompressed for use by replica servers."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_2")),
        doc = "For `p4 admin checkpoint -Z`, save the checkpoint in compressed format,",
        doc = "but leave the journal uncompressed for use by replica servers."
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "For `p4 admin checkpoint -Z`, save the checkpoint in compressed format,",
        doc = "but leave the journal uncompressed for use by replica servers. The",
        doc = "`.gz` suffix is appended to compressed journals and checkpoint files,",
        doc = "which are in gzip format. If you do not specify `-z` or `-Z`, no",
        doc = "compression occurs."
    )]
    pub fn compress_checkpoint_only(self) -> Admin<CheckPoint<compression::CheckPointOnly>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: CheckPoint::<compression::CheckPointOnly> {
                compression: compression::CheckPointOnly,
                #[cfg(not(feature = "lt2023_1"))]
                parallel: self.sub_command.parallel,
                #[cfg(not(feature = "lt2023_1"))]
                threads: self.sub_command.threads,
                #[cfg(not(feature = "lt2023_1"))]
                multiple_files: self.sub_command.multiple_files,
            },
        }
    }
}

/// `p4 admin journal [-z] [prefix]`: rotate the journal.
#[derive(Debug, Clone, Default)]
pub struct Journal {
    gzip: bool,
}

impl SubCommand for Journal {
    fn name(&self) -> &str {
        "journal"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.gzip {
            command.arg("-z");
        }
    }
}

impl ParameterizedSpawn for Admin<Journal> {
    type Input<'a> = Option<&'a str>;
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin journal` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    ///
    /// Pass `Some(prefix)` to name the journal with the given prefix, or
    /// `None` to use the default journal name.
    fn spawn_with<'a>(&mut self, prefix: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut command = self.setup_command(&self.bin);

        if let Some(prefix) = prefix {
            command.arg(prefix);
        }
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl SpawnExt for Admin<Journal> {
    fn spawn<'a>(&mut self) -> Result<Self::Output<'a>, Self::Error> {
        self.spawn_with(None)
    }
}

impl Admin<Journal> {
    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "For `p4 admin checkpoint` and `p4 admin journal`, save the checkpoint",
        doc = "and saved journal file in compressed (gzip) format, appending the `.gz`",
        doc = "suffix to the files."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_2")),
        doc = "For `p4 admin checkpoint -z` and `p4 admin journal -z`, save the",
        doc = "checkpoint and journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z` or `-Z`, no compression occurs."
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Save the journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z`, no compression occurs."
    )]
    pub fn get_gzip(&self) -> bool {
        self.sub_command.gzip
    }

    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "For `p4 admin checkpoint` and `p4 admin journal`, save the checkpoint",
        doc = "and saved journal file in compressed (gzip) format, appending the `.gz`",
        doc = "suffix to the files."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_2")),
        doc = "For `p4 admin checkpoint -z` and `p4 admin journal -z`, save the",
        doc = "checkpoint and journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z` or `-Z`, no compression occurs."
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Save the journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z`, no compression occurs."
    )]
    pub fn set_gzip(&mut self, gzip: bool) -> &mut Self {
        self.sub_command.gzip = gzip;
        self
    }

    /// # Description
    ///
    /// -z
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "For `p4 admin checkpoint` and `p4 admin journal`, save the checkpoint",
        doc = "and saved journal file in compressed (gzip) format, appending the `.gz`",
        doc = "suffix to the files."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_2")),
        doc = "For `p4 admin checkpoint -z` and `p4 admin journal -z`, save the",
        doc = "checkpoint and journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z` or `-Z`, no compression occurs."
    )]
    #[cfg_attr(
        not(feature = "lt2023_1"),
        doc = "Save the journal file in compressed format. The `.gz` suffix is",
        doc = "appended to compressed journals and checkpoint files, which are in",
        doc = "gzip format. If you do not specify `-z`, no compression occurs."
    )]
    pub fn gzip(mut self, gzip: bool) -> Self {
        self.sub_command.gzip = gzip;
        self
    }
}

/// `p4 admin stop`: stop the Perforce service.
#[derive(Debug, Clone, Default)]
pub struct Stop;

impl SubCommand for Stop {
    fn name(&self) -> &str {
        "stop"
    }

    fn inject_local_args(&self, _: &mut Command) {}
}

/// `p4 admin restart`: restart the Perforce service.
#[derive(Debug, Clone, Default)]
pub struct Restart;

impl SubCommand for Restart {
    fn name(&self) -> &str {
        "restart"
    }

    fn inject_local_args(&self, _: &mut Command) {}
}

/// The form specification type archived by `p4 admin updatespecdepot -s`.
#[derive(Debug, Clone)]
pub enum SpecifiedType {
    Client,
    Depot,
    /// Added in p4 2018.1.
    #[cfg(not(feature = "lt2018_1"))]
    Repo,
    Branch,
    Label,
    TypeMap,
    Group,
    User,
    Job,
    /// Added in p4 2016.1.
    #[cfg(not(feature = "lt2016_1"))]
    Stream,
    /// Added in p4 2016.1.
    #[cfg(not(feature = "lt2016_1"))]
    Triggers,
    /// Added in p4 2016.1.
    #[cfg(not(feature = "lt2016_1"))]
    Protect,
    /// Added in p4 2016.1.
    #[cfg(not(feature = "lt2016_1"))]
    Server,
    /// Added in p4 2016.1.
    #[cfg(not(feature = "lt2016_1"))]
    License,
    /// Added in p4 2016.1.
    #[cfg(not(feature = "lt2016_1"))]
    JobSpec,
}

impl SpecifiedType {
    /// CLI value used with `-s`, used when rendering the command arguments.
    pub(crate) fn to_str(&self) -> &str {
        match self {
            Self::Client => "client",
            Self::Depot => "depot",
            #[cfg(not(feature = "lt2018_1"))]
            Self::Repo => "repo",
            Self::Branch => "branch",
            Self::Label => "label",
            Self::TypeMap => "typemap",
            Self::Group => "group",
            Self::User => "user",
            Self::Job => "job",
            #[cfg(not(feature = "lt2016_1"))]
            Self::Stream => "stream",
            #[cfg(not(feature = "lt2016_1"))]
            Self::Triggers => "triggers",
            #[cfg(not(feature = "lt2016_1"))]
            Self::Protect => "protect",
            #[cfg(not(feature = "lt2016_1"))]
            Self::Server => "server",
            #[cfg(not(feature = "lt2016_1"))]
            Self::License => "license",
            #[cfg(not(feature = "lt2016_1"))]
            Self::JobSpec => "jobspec",
        }
    }
}

/// Variants of the `[-a | -s type]` mutually exclusive option group of
/// `p4 admin updatespecdepot`.
pub mod spec {
    use super::SpecifiedType;

    /// Archive all current forms (`-a`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct All;

    /// Archive forms of the specified type (`-s type`).
    #[derive(Debug, Clone)]
    pub struct Selected(pub SpecifiedType);
}

impl ExclusiveOption for spec::All {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");
    }
}

impl ExclusiveOption for spec::Selected {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s").arg(self.0.to_str());
    }
}

/// `p4 admin updatespecdepot [-a | -s type]`: archive forms into the spec
/// depot.
///
/// The `S` type parameter encodes the selected variant of the `[-a | -s type]`
/// group at compile time; see [`ExclusiveOption`] and [`spec`].
#[derive(Debug, Clone, Default)]
pub struct UpdateSpecDepot<S = Unselected> {
    spec: S,
}

impl<S: ExclusiveOption> SubCommand for UpdateSpecDepot<S> {
    fn name(&self) -> &str {
        "updatespecdepot"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.spec.inject_args(command);
    }
}

impl Admin<UpdateSpecDepot<Unselected>> {
    /// # Description
    ///
    /// -a
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "For `p4 admin updatespecdepot`, update the spec depot with all current",
        doc = "forms."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "For `p4 admin updatespecdepot -a`, update the spec depot with all",
        doc = "current forms."
    )]
    pub fn all(self) -> Admin<UpdateSpecDepot<spec::All>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: UpdateSpecDepot { spec: spec::All },
        }
    }

    /// # Description
    ///
    /// -s type
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `branch`,",
        doc = "`label`, `typemap`, `group`, `user`, or `job`."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2016_1")),
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `branch`,",
        doc = "`label`, `typemap`, `group`, `user`, `job`, `stream`, `triggers`,",
        doc = "`protect`, `server`, `license`, or `jobspec`."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2018_1")),
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `repo`,",
        doc = "`branch`, `label`, `typemap`, `group`, `user`, `job`, `stream`,",
        doc = "`triggers`, `protect`, `server`, `license`, or `jobspec`."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "For `p4 admin updatespecdepot -s`, update the spec depot with forms of",
        doc = "the specified type, where type is one of `client`, `depot`, `repo`,",
        doc = "`branch`, `label`, `typemap`, `group`, `user`, `job`, `stream`,",
        doc = "`triggers`, `protect`, `server`, `license`, or `jobspec`."
    )]
    pub fn specified_type(
        self,
        specified_type: SpecifiedType,
    ) -> Admin<UpdateSpecDepot<spec::Selected>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: UpdateSpecDepot {
                spec: spec::Selected(specified_type),
            },
        }
    }
}

impl Admin<UpdateSpecDepot<spec::Selected>> {
    /// # Description
    ///
    /// -s type
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `branch`,",
        doc = "`label`, `typemap`, `group`, `user`, or `job`."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2016_1")),
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `branch`,",
        doc = "`label`, `typemap`, `group`, `user`, `job`, `stream`, `triggers`,",
        doc = "`protect`, `server`, `license`, or `jobspec`."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2018_1")),
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `repo`,",
        doc = "`branch`, `label`, `typemap`, `group`, `user`, `job`, `stream`,",
        doc = "`triggers`, `protect`, `server`, `license`, or `jobspec`."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "For `p4 admin updatespecdepot -s`, update the spec depot with forms of",
        doc = "the specified type, where type is one of `client`, `depot`, `repo`,",
        doc = "`branch`, `label`, `typemap`, `group`, `user`, `job`, `stream`,",
        doc = "`triggers`, `protect`, `server`, `license`, or `jobspec`."
    )]
    pub fn get_specified_type(&self) -> &SpecifiedType {
        &self.sub_command.spec.0
    }

    /// # Description
    ///
    /// -s type
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `branch`,",
        doc = "`label`, `typemap`, `group`, `user`, or `job`."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2016_1")),
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `branch`,",
        doc = "`label`, `typemap`, `group`, `user`, `job`, `stream`, `triggers`,",
        doc = "`protect`, `server`, `license`, or `jobspec`."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2018_1")),
        doc = "For `p4 admin updatespecdepot`, update the spec depot with forms of the",
        doc = "specified type, where type is one of `client`, `depot`, `repo`,",
        doc = "`branch`, `label`, `typemap`, `group`, `user`, `job`, `stream`,",
        doc = "`triggers`, `protect`, `server`, `license`, or `jobspec`."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "For `p4 admin updatespecdepot -s`, update the spec depot with forms of",
        doc = "the specified type, where type is one of `client`, `depot`, `repo`,",
        doc = "`branch`, `label`, `typemap`, `group`, `user`, `job`, `stream`,",
        doc = "`triggers`, `protect`, `server`, `license`, or `jobspec`."
    )]
    pub fn set_specified_type(&mut self, specified_type: SpecifiedType) -> &mut Self {
        self.sub_command.spec = spec::Selected(specified_type);
        self
    }
}

/// Variants of the `{-a | -u user}` mutually exclusive option group of
/// `p4 admin resetpassword`.
pub mod set_password {
    /// Reset all users' passwords (`-a`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct All;

    /// Reset a single user's password (`-u user`).
    #[derive(Debug, Clone)]
    pub struct User(pub String);
}

impl ExclusiveOption for set_password::All {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-a");
    }
}

impl ExclusiveOption for set_password::User {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-u").arg(&self.0);
    }
}

/// `p4 admin resetpassword {-a | -u user} [-l]`: force users to reset their
/// passwords.
///
/// The `T` type parameter encodes the selected variant of the `{-a | -u user}`
/// group at compile time; see [`ExclusiveOption`] and [`set_password`].
#[cfg_attr(
    feature = "lt2025_2",
    doc = "`p4 admin resetpassword -a | -u user`: force users to reset their passwords."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "`p4 admin resetpassword {-a | -u user} [-l]`: force users to reset their passwords."
)]
#[derive(Debug, Clone, Default)]
pub struct ResetPassword<T = Unselected> {
    set_password: T,

    /// Added in p4 2025.2.
    #[cfg(not(feature = "lt2025_2"))]
    super_user: bool,
}

impl<T: ExclusiveOption> SubCommand for ResetPassword<T> {
    fn name(&self) -> &str {
        "resetpassword"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.set_password.inject_args(command);
        #[cfg(not(feature = "lt2025_2"))]
        if self.super_user {
            command.arg("-l");
        }
    }
}

impl Admin<ResetPassword<Unselected>> {
    /// # Description
    ///
    /// -a
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Force password reset of all users with passwords, including the",
        doc = "superuser who issued the command. Only the passwords of users who",
        doc = "presently exist (and who have passwords) are reset."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "All users.")]
    pub fn all(self) -> Admin<ResetPassword<set_password::All>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: ResetPassword {
                set_password: set_password::All,
                #[cfg(not(feature = "lt2025_2"))]
                super_user: self.sub_command.super_user,
            },
        }
    }

    /// # Description
    ///
    /// -u user
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Force a single user with an existing password to reset their password",
        doc = "before they can run another command."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "The specified user.")]
    pub fn user(self, user: impl Into<String>) -> Admin<ResetPassword<set_password::User>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: ResetPassword {
                set_password: set_password::User(user.into()),
                #[cfg(not(feature = "lt2025_2"))]
                super_user: self.sub_command.super_user,
            },
        }
    }
}

impl Admin<ResetPassword<set_password::User>> {
    /// # Description
    ///
    /// -u user
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Force a single user with an existing password to reset their password",
        doc = "before they can run another command."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "The specified user.")]
    pub fn get_user(&self) -> &str {
        &self.sub_command.set_password.0
    }

    /// # Description
    ///
    /// -u user
    ///
    #[cfg_attr(
        feature = "lt2023_1",
        doc = "Force a single user with an existing password to reset their password",
        doc = "before they can run another command."
    )]
    #[cfg_attr(not(feature = "lt2023_1"), doc = "The specified user.")]
    pub fn set_user(&mut self, user: impl Into<String>) -> &mut Self {
        self.sub_command.set_password.0 = user.into();
        self
    }
}

impl<T: ExclusiveOption> Admin<ResetPassword<T>> {
    /// # Description
    ///
    /// -l
    ///
    /// Super user.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_super_user(&self) -> bool {
        self.sub_command.super_user
    }

    /// # Description
    ///
    /// -l
    ///
    /// Super user.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_super_user(&mut self, super_user: bool) -> &mut Self {
        self.sub_command.super_user = super_user;
        self
    }

    /// # Description
    ///
    /// -l
    ///
    /// Super user.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn super_user(mut self, super_user: bool) -> Self {
        self.sub_command.super_user = super_user;
        self
    }
}

/// `p4 admin setldapusers`: convert existing non-super users to LDAP
/// authentication. Added in p4 2015.1.
#[cfg(not(feature = "lt2015_1"))]
#[derive(Debug, Clone, Default)]
pub struct SetLdapUsers;

#[cfg(not(feature = "lt2015_1"))]
impl SubCommand for SetLdapUsers {
    fn name(&self) -> &str {
        "setldapusers"
    }

    fn inject_local_args(&self, _: &mut Command) {}
}

/// `p4 admin end-journal`: end journal replication at a failover consistency
/// point. Added in p4 2018.1.
#[cfg(not(feature = "lt2018_1"))]
#[derive(Debug, Clone, Default)]
pub struct EndJournal;

#[cfg(not(feature = "lt2018_1"))]
impl SubCommand for EndJournal {
    fn name(&self) -> &str {
        "end-journal"
    }

    fn inject_local_args(&self, _: &mut Command) {}
}

/// `p4 admin sysinfo`: dump system information for Perforce Support. Added in
/// p4 2023.1.
#[cfg(not(feature = "lt2023_1"))]
#[derive(Debug, Clone, Default)]
pub struct SysInfo;

#[cfg(not(feature = "lt2023_1"))]
impl SubCommand for SysInfo {
    fn name(&self) -> &str {
        "sysinfo"
    }

    fn inject_local_args(&self, _: &mut Command) {}
}

/// `p4 admin resource-monitor`: report server resource usage. Added in p4
/// 2023.1.
#[cfg(not(feature = "lt2023_1"))]
#[derive(Debug, Clone, Default)]
pub struct ResourceMonitor;

#[cfg(not(feature = "lt2023_1"))]
impl SubCommand for ResourceMonitor {
    fn name(&self) -> &str {
        "resource-monitor"
    }

    fn inject_local_args(&self, _: &mut Command) {}
}

/// Variants of the `[--restrict-only | --expand-only]` mutually exclusive
/// option group of `p4 admin replica-filter-reconcile`.
#[cfg(not(feature = "lt2025_2"))]
pub mod reconcile {
    /// Only remove applicable database records (`--restrict-only`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct RestrictOnly;

    /// Only add applicable database records (`--expand-only`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ExpandOnly;
}

#[cfg(not(feature = "lt2025_2"))]
impl ExclusiveOption for reconcile::RestrictOnly {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--restrict-only");
    }
}

#[cfg(not(feature = "lt2025_2"))]
impl ExclusiveOption for reconcile::ExpandOnly {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--expand-only");
    }
}

/// `p4 admin replica-filter-reconcile [--restrict-only | --expand-only]
/// [table ...]`: reconcile the replica database after filter changes. Added in
/// p4 2025.2.
///
/// The `M` type parameter encodes the selected variant of the
/// `[--restrict-only | --expand-only]` group at compile time; see
/// [`ExclusiveOption`] and [`reconcile`].
#[cfg(not(feature = "lt2025_2"))]
#[derive(Debug, Clone, Default)]
pub struct ReplicaFilterReconcile<M = Unselected> {
    reconcile: M,
}

#[cfg(not(feature = "lt2025_2"))]
impl<M: ExclusiveOption> SubCommand for ReplicaFilterReconcile<M> {
    fn name(&self) -> &str {
        "replica-filter-reconcile"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.reconcile.inject_args(command);
    }
}

#[cfg(not(feature = "lt2025_2"))]
impl Admin<ReplicaFilterReconcile<Unselected>> {
    /// # Description
    ///
    /// --restrict-only
    ///
    /// Reconcile the replica database by only removing applicable database
    /// records.
    pub fn restrict_only(self) -> Admin<ReplicaFilterReconcile<reconcile::RestrictOnly>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: ReplicaFilterReconcile {
                reconcile: reconcile::RestrictOnly,
            },
        }
    }

    /// # Description
    ///
    /// --expand-only
    ///
    /// Reconcile the replica database by only adding applicable database
    /// records.
    pub fn expand_only(self) -> Admin<ReplicaFilterReconcile<reconcile::ExpandOnly>> {
        Admin {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: ReplicaFilterReconcile {
                reconcile: reconcile::ExpandOnly,
            },
        }
    }
}

#[cfg(not(feature = "lt2025_2"))]
impl<M: ExclusiveOption> ParameterizedSpawn for Admin<ReplicaFilterReconcile<M>> {
    type Input<'a> = &'a [&'a OsStr];
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 admin replica-filter-reconcile` for the given tables as a
    /// child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    ///
    /// Pass an empty slice to reconcile all applicable database tables.
    fn spawn_with<'a>(&mut self, tables: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .args(tables)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 admin ...` command lines; no
    /// process is spawned.
    #[test]
    fn stop() {
        let admin = AdminEntry::new("p4", GlobalOpts::new()).stop();

        assert_eq!(args_of(&admin.setup_command("p4")), ["admin", "stop"]);
    }

    #[test]
    fn restart() {
        let admin = AdminEntry::new("p4", GlobalOpts::new()).restart();

        assert_eq!(args_of(&admin.setup_command("p4")), ["admin", "restart"]);
    }

    #[cfg(not(feature = "lt2015_1"))]
    #[test]
    fn setldapusers() {
        let admin = AdminEntry::new("p4", GlobalOpts::new()).setldapusers();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "setldapusers"]
        );
    }

    #[cfg(not(feature = "lt2018_1"))]
    #[test]
    fn end_journal() {
        let admin = AdminEntry::new("p4", GlobalOpts::new()).end_journal();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "end-journal"]
        );
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn sysinfo() {
        let admin = AdminEntry::new("p4", GlobalOpts::new()).sysinfo();

        assert_eq!(args_of(&admin.setup_command("p4")), ["admin", "sysinfo"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn resource_monitor() {
        let admin = AdminEntry::new("p4", GlobalOpts::new()).resource_monitor();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "resource-monitor"]
        );
    }

    #[test]
    fn checkpoint_compress_both() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .checkpoint()
            .compress_both();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "checkpoint", "-z"]
        );
    }

    #[test]
    fn checkpoint_compress_checkpoint_only() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .checkpoint()
            .compress_checkpoint_only();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "checkpoint", "-Z"]
        );
    }

    #[test]
    fn checkpoint_with_prefix() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .checkpoint()
            .compress_both();

        // Mirrors `spawn_with`/`output_with`, which append the prefix after
        // the assembled command.
        let mut command = admin.setup_command("p4");
        command.arg("ckp");

        assert_eq!(args_of(&command), ["admin", "checkpoint", "-z", "ckp"]);
    }

    #[cfg(not(feature = "lt2023_1"))]
    #[test]
    fn checkpoint_parallel_options() {
        let mut admin = AdminEntry::new("p4", GlobalOpts::new()).checkpoint();
        admin
            .set_parallel(true)
            .set_threads(4)
            .set_multiple_files(true);

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "checkpoint", "-p", "-N", "4", "-m"]
        );
    }

    #[test]
    fn journal_gzip() {
        let mut admin = AdminEntry::new("p4", GlobalOpts::new()).journal();
        admin.set_gzip(true);

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "journal", "-z"]
        );
    }

    #[test]
    fn updatespecdepot_all() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .updatespecdepot()
            .all();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "updatespecdepot", "-a"]
        );
    }

    /// The `-s type` variant is covered here because `SpecifiedType` is not
    /// exported outside the crate.
    #[test]
    fn updatespecdepot_specified_type() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .updatespecdepot()
            .specified_type(SpecifiedType::Client);

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "updatespecdepot", "-s", "client"]
        );
    }

    #[test]
    fn resetpassword_all() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .resetpassword()
            .all();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "resetpassword", "-a"]
        );
    }

    #[test]
    fn resetpassword_single_user() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .resetpassword()
            .user("bruno");

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "resetpassword", "-u", "bruno"]
        );
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn resetpassword_super_user() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .resetpassword()
            .super_user(true);

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "resetpassword", "-l"]
        );
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn replica_filter_reconcile_restrict_only() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .replica_filter_reconcile()
            .restrict_only();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "replica-filter-reconcile", "--restrict-only"]
        );
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn replica_filter_reconcile_expand_only() {
        let admin = AdminEntry::new("p4", GlobalOpts::new())
            .replica_filter_reconcile()
            .expand_only();

        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["admin", "replica-filter-reconcile", "--expand-only"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let global_opts = GlobalOpts::new().port("localhost:1666");
        let admin = AdminEntry::new("p4", global_opts).checkpoint();

        // The inner subcommand must not inject the global options a second
        // time.
        assert_eq!(
            args_of(&admin.setup_command("p4")),
            ["-p", "localhost:1666", "admin", "checkpoint"]
        );
    }
}
