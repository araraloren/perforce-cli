use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::Unselected;
use super::{ExclusiveOption, SubCommand};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Maximum value of the `-w` quiesce wait: one hour, in seconds.
const MAX_QUIESCE_WAIT_TIME_SECONDS: u64 = 3600;

/// Maximum value of the `-v` verification time: twelve hours, in seconds.
const MAX_VERIFICATION_TIME_SECONDS: u64 = 43_200;

/// `-m` mode of `p4 failover`: the target server is required to participate
/// in failover. If the target server cannot be accessed by the standby
/// server, the failover will not occur.
#[derive(Debug, Clone, Default)]
pub struct RequireTarget;

/// `-i` mode of `p4 failover`: ignore the original target server even if it
/// is accessible. The optional `-s serverID` value naming the new target can
/// be supplied via [`Failover::ignore_target_with`] or the
/// [`Failover::server_id`] setter.
#[derive(Debug, Clone, Default)]
pub struct IgnoreTarget {
    server_id: Option<String>,
}

impl ExclusiveOption for RequireTarget {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-m");
    }
}

impl ExclusiveOption for IgnoreTarget {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-i");

        if let Some(server_id) = &self.server_id {
            command.arg("-s").arg(server_id);
        }
    }
}

#[cfg_attr(
    feature = "lt2023_1",
    doc = "`p4 failover [-y] [ -m | [-i] -s serverID ] [-w waitForQuiesce][-v verificationTime][failoverMessage]`"
)]
#[cfg_attr(
    not(feature = "lt2023_1"),
    doc = "`p4 failover [-y] [ -m | [-i] -s serverID ] [-w waitForQuiesce] [-v verificationTime] [failoverMessage]`"
)]
///
/// Fail over to a standby server.
///
/// Allows the administrator of a standby server to initiate the process of failing over from the current
#[cfg_attr(
    feature = "lt2019_1",
    doc = "master server to that standby server. In this context, the master server is the server from which the standby server makes its journalcopy (see `p4 journalcopy`). The \"master\" server can also be an edge server. The \"master\" server can also be an edge server. The \"standby\" server can be either a \"standby\" or \"forwarding-standby\" server.",
    doc = "",
    doc = "Failing over to a \"mandatory\" (see `p4 server` `Options:`) standby when the master server is not part of the failover process ensures that none of the downstream replicas will be ahead of the new master server.",
    doc = "",
    doc = "For the local High Availability standby server, we recommend `mandatory`",
    doc = "",
    doc = "For the remote Disaster Recovery standby server, we recommend `nomandatory`",
    doc = "",
    doc = "For details, see the \"Failover\" topic in the \"Backup and recovery\" chapter of Helix Core Server Administrator Guide: Fundamentals."
)]
#[cfg_attr(
    all(feature = "lt2019_2", not(feature = "lt2019_1")),
    doc = "master server to that standby server. In this context, the master server is the server from which the standby server makes its journalcopy (see `p4 journalcopy`). The \"master\" server can also be an edge server. The \"standby\" server can be either a \"standby\" or \"forwarding-standby\" server.",
    doc = "",
    doc = "Failing over to a \"mandatory\" (see `p4 server` `Options:`) standby when the master server is not part of the failover process ensures that none of the downstream replicas will be ahead of the new master server.",
    doc = "",
    doc = "For the local High Availability standby server, we recommend `mandatory`, but only after journalcopying is complete.",
    doc = "",
    doc = "Follow the best practice guidelines at \"High availability with the mandatory server specification option\" in Helix Core Server Administrator Guide: Fundamentals.",
    doc = "",
    doc = "For the remote Disaster Recovery standby server, we recommend `nomandatory`",
    doc = "",
    doc = "For details, see the \"Failover\" topic in the \"Backup and recovery\" chapter of Helix Core Server Administrator Guide: Fundamentals."
)]
#[cfg_attr(
    all(feature = "lt2020_1", not(feature = "lt2019_2")),
    doc = "master server to that standby server. In this context, the master server is the server from which the standby server makes its journalcopy (see `p4 journalcopy`). The \"master\" server can also be an edge server. The \"standby\" server can be either a \"standby\" or \"forwarding-standby\" server.",
    doc = "",
    doc = "Failing over to a \"mandatory\" (see `p4 server` `Options:`) standby when the master server is not part of the failover process ensures that none of the downstream replicas will be ahead of the new master server.",
    doc = "",
    doc = "For the local High Availability standby server, we recommend `mandatory`, but only after journalcopying is complete.",
    doc = "",
    doc = "Follow the best practice guidelines at \"High availability with the mandatory server specification option\" in Helix Core Server Administrator Guide.",
    doc = "",
    doc = "For the remote Disaster Recovery standby server, we recommend `nomandatory`",
    doc = "",
    doc = "For details, see the \"Failover\" topic in the \"Backup and recovery\" chapter of Helix Core Server Administrator Guide."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2020_1")),
    doc = "master server to that standby server. In this context, the master server is the server from which the standby server makes its journalcopy (see `p4 journalcopy`). The \"master\" server can also be an edge server. The \"standby\" server can be either a \"standby\" or \"forwarding-standby\" server.",
    doc = "",
    doc = "Failing over to a \"mandatory\" (see `p4 server` `Options:`) standby when the master server is not part of the failover process ensures that none of the downstream replicas will be ahead of the new master server.",
    doc = "",
    doc = "For the local High Availability standby server, we recommend `mandatory`, but only after journalcopying is complete.",
    doc = "",
    doc = "For the remote Disaster Recovery standby server, we recommend `nomandatory`",
    doc = "",
    doc = "For details, see the Failover topic in the Backup and recovery chapter of Helix Core Server Administrator Guide."
)]
#[cfg_attr(
    all(feature = "lt2025_1", not(feature = "lt2024_2")),
    doc = "master server to its standby server. In this context, the master server is the server from which the standby server makes its journalcopy (see `p4 journalcopy`). The \"master\" server can also be an edge server. The \"standby\" server can be either a \"standby\" or \"forwarding-standby\" server.",
    doc = "",
    doc = "Failing over to a \"mandatory\" (see `p4 server` `Options:`) standby when the master server is not part of the failover process ensures that none of the downstream replicas will be ahead of the new master server.",
    doc = "",
    doc = "For the local High Availability standby server, we recommend `mandatory`, but only after journalcopying is complete.",
    doc = "",
    doc = "For the remote Disaster Recovery standby server, we recommend `nomandatory`",
    doc = "",
    doc = "To learn more, see the Failover topic in the Backup and recovery chapter of P4 Server Administration Documentation."
)]
#[cfg_attr(
    not(feature = "lt2025_1"),
    doc = "target server to its standby server. In this context, the target server is the server from which the standby server makes its journalcopy (see `p4 journalcopy`). The target server can also be an edge server. The standby server can be either a standby server or forwarding-standby server.",
    doc = "",
    doc = "Failing over to a \"mandatory\" (see `p4 server` `Options:`) standby when the target server is not part of the failover process ensures that none of the downstream replicas will be ahead of the new target server.",
    doc = "",
    doc = "For the local High Availability standby server, we recommend `mandatory`, but only after journalcopying is complete.",
    doc = "",
    doc = "For the remote Disaster Recovery standby server, we recommend `nomandatory`",
    doc = "",
    doc = "To learn more, see the Failover topic in the Backup and recovery chapter of P4 Server Administration Documentation."
)]
///
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2020_2")),
    doc = "2020.2 and later: Failover from a mandatory standby server when the master is not participating includes checking the `ReplicatingFrom` field of the standby server spec for the master's serverID when `-s` is not specified on the command line.",
    doc = "",
    doc = "Part of the failover process involves stopping the journalcopy and pull threads. If the failover process fails, any `pull -L`, `pull -u`, or journalcopy threads that were configured using `startup.N` configurables will automatically be restarted.",
    doc = "",
    doc = "Prior to 2020.2: Failover from a mandatory standby server when the master is not participating used to require specifying `-s <serverID>`.",
    doc = "",
    doc = "If the failover process fails, the journalcopy and pull threads needed to be restarted manually."
)]
#[cfg_attr(
    all(feature = "lt2025_1", not(feature = "lt2024_2")),
    doc = "2020.2 and later: Failover from a mandatory standby server when the master is not participating includes checking the `ReplicatingFrom` field of the standby server spec for the master's serverID when `-s` is not specified on the command line.",
    doc = "",
    doc = "Part of the failover process involves stopping the journalcopy and pull threads. If the failover process fails, any `pull -L`, `pull -u`, or journalcopy threads that were configured using `startup.N` configurables will automatically be restarted.",
    doc = "",
    doc = "Prior to 2020.2: Failover from a mandatory standby server when the master is not participating used to require specifying `-s <serverID>`.",
    doc = "",
    doc = "If the failover process fails, the journalcopy and pull threads needed to be restarted manually."
)]
#[cfg_attr(
    not(feature = "lt2025_1"),
    doc = "2020.2 and later: Failover from a mandatory standby server when the target server is not participating includes checking the `ReplicatingFrom` field of the standby server spec for the target server's serverID when `-s` is not specified on the command line.",
    doc = "",
    doc = "Part of the failover process involves stopping the journalcopy and pull threads. If the failover process fails, any `pull -L`, `pull -u`, or journalcopy threads that were configured using `startup.N` configurables will automatically be restarted.",
    doc = "",
    doc = "Prior to 2020.2: Failover from a mandatory standby server when the target server is not participating used to require specifying `-s <serverID>`.",
    doc = "",
    doc = "If the failover process fails, the journalcopy and pull threads needed to be restarted manually."
)]
#[derive(Debug, Clone, Default)]
pub struct Failover<M> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    yes: bool,

    verification_time: Option<u64>,

    quiesce_wait_time: Option<u64>,

    mode: M,
}

impl Failover<Unselected> {
    /// Creates a new `p4 failover` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            yes: false,
            verification_time: None,
            quiesce_wait_time: None,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    /// -m
    ///
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "The master server is required to participate in failover, which excludes the `-i` option. If the master server cannot be accessed by the standby server, the failover will not occur."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The target server is required to participate in failover, which excludes the `-i` option. If the target server cannot be accessed by the standby server, the failover will not occur."
    )]
    pub fn required_target(self) -> Failover<RequireTarget> {
        Failover {
            bin: self.bin,
            global_opts: self.global_opts,
            yes: self.yes,
            verification_time: self.verification_time,
            quiesce_wait_time: self.quiesce_wait_time,
            mode: RequireTarget,
        }
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the original
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "master server, even if it is accessible.",
        doc = "",
        doc = "This option and the `-m` option are mutually exclusive.",
        doc = "",
        doc = "Using the `-i` flag when the master server is accessible can result in an undesirable \"split-brain\" scenario in which two master servers process divergent datasets."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "target server, even if it is accessible.",
        doc = "",
        doc = "This option and the `-m` option are mutually exclusive.",
        doc = "",
        doc = "Using the `-i` flag when the target server is accessible can result in an undesirable \"split-brain\" scenario in which two target servers process divergent datasets."
    )]
    pub fn ignore_target(self) -> Failover<IgnoreTarget> {
        Failover {
            bin: self.bin,
            global_opts: self.global_opts,
            yes: self.yes,
            verification_time: self.verification_time,
            quiesce_wait_time: self.quiesce_wait_time,
            mode: IgnoreTarget { server_id: None },
        }
    }

    /// Same as [`Failover::ignore_target`], and the given `server_id` is
    /// passed to the command as the `-s serverID` option.
    ///
    /// # Description
    ///
    /// -i
    ///
    /// Ignore the original
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "master server, even if it is accessible.",
        doc = "",
        doc = "This option and the `-m` option are mutually exclusive.",
        doc = "",
        doc = "Using the `-i` flag when the master server is accessible can result in an undesirable \"split-brain\" scenario in which two master servers process divergent datasets."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "target server, even if it is accessible.",
        doc = "",
        doc = "This option and the `-m` option are mutually exclusive.",
        doc = "",
        doc = "Using the `-i` flag when the target server is accessible can result in an undesirable \"split-brain\" scenario in which two target servers process divergent datasets."
    )]
    pub fn ignore_target_with(self, server_id: impl Into<String>) -> Failover<IgnoreTarget> {
        Failover {
            bin: self.bin,
            global_opts: self.global_opts,
            yes: self.yes,
            verification_time: self.verification_time,
            quiesce_wait_time: self.quiesce_wait_time,
            mode: IgnoreTarget {
                server_id: Some(server_id.into()),
            },
        }
    }
}

impl Failover<IgnoreTarget> {
    /// # Description
    ///
    /// -s serverID
    ///
    #[cfg_attr(
        feature = "lt2020_2",
        doc = "If the existing master server participates in failover, the new master automatically gets the `serverID` of the old master and this option is not required. However, if the standby server cannot access the master server, use this option to specify the `serverID` of the master server."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2020_2")),
        doc = "If the existing master server participates in failover, the new master automatically gets the `serverID` of the old master and this option is not required.",
        doc = "",
        doc = "Prior to 2020.2, if the standby server could not access the master server, this option was necessary to specify the `serverID` of the master server.",
        doc = "",
        doc = "For 2020.2 and later, if the standby server cannot access the master server, there is automatic checking the `ReplicatingFrom` field of the standby server spec for the master's serverID when `-s` is not specified on the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "If the existing target server participates in failover, the new target server automatically gets the `serverID` of the old target server and this option is not required.",
        doc = "",
        doc = "Prior to 2020.2, if the standby server could not access the target server, this option was necessary to specify the `serverID` of the target server.",
        doc = "",
        doc = "For 2020.2 and later, if the standby server cannot access the target server, there is automatic checking the `ReplicatingFrom` field of the standby server spec for the target server's serverID when `-s` is not specified on the command line."
    )]
    pub fn get_server_id(&self) -> Option<&str> {
        self.mode.server_id.as_deref()
    }

    /// # Description
    ///
    /// -s serverID
    ///
    #[cfg_attr(
        feature = "lt2020_2",
        doc = "If the existing master server participates in failover, the new master automatically gets the `serverID` of the old master and this option is not required. However, if the standby server cannot access the master server, use this option to specify the `serverID` of the master server."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2020_2")),
        doc = "If the existing master server participates in failover, the new master automatically gets the `serverID` of the old master and this option is not required.",
        doc = "",
        doc = "Prior to 2020.2, if the standby server could not access the master server, this option was necessary to specify the `serverID` of the master server.",
        doc = "",
        doc = "For 2020.2 and later, if the standby server cannot access the master server, there is automatic checking the `ReplicatingFrom` field of the standby server spec for the master's serverID when `-s` is not specified on the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "If the existing target server participates in failover, the new target server automatically gets the `serverID` of the old target server and this option is not required.",
        doc = "",
        doc = "Prior to 2020.2, if the standby server could not access the target server, this option was necessary to specify the `serverID` of the target server.",
        doc = "",
        doc = "For 2020.2 and later, if the standby server cannot access the target server, there is automatic checking the `ReplicatingFrom` field of the standby server spec for the target server's serverID when `-s` is not specified on the command line."
    )]
    pub fn set_server_id(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.server_id = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -s serverID
    ///
    #[cfg_attr(
        feature = "lt2020_2",
        doc = "If the existing master server participates in failover, the new master automatically gets the `serverID` of the old master and this option is not required. However, if the standby server cannot access the master server, use this option to specify the `serverID` of the master server."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2020_2")),
        doc = "If the existing master server participates in failover, the new master automatically gets the `serverID` of the old master and this option is not required.",
        doc = "",
        doc = "Prior to 2020.2, if the standby server could not access the master server, this option was necessary to specify the `serverID` of the master server.",
        doc = "",
        doc = "For 2020.2 and later, if the standby server cannot access the master server, there is automatic checking the `ReplicatingFrom` field of the standby server spec for the master's serverID when `-s` is not specified on the command line."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "If the existing target server participates in failover, the new target server automatically gets the `serverID` of the old target server and this option is not required.",
        doc = "",
        doc = "Prior to 2020.2, if the standby server could not access the target server, this option was necessary to specify the `serverID` of the target server.",
        doc = "",
        doc = "For 2020.2 and later, if the standby server cannot access the target server, there is automatic checking the `ReplicatingFrom` field of the standby server spec for the target server's serverID when `-s` is not specified on the command line."
    )]
    pub fn server_id(mut self, v: impl Into<String>) -> Self {
        self.set_server_id(v);
        self
    }
}

impl<M: ExclusiveOption> Failover<M> {
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
    /// Perform the failover operation. Without this option, `p4 failover`
    /// merely reports what it would do.
    ///
    /// Carefully consider the report of what failover would do BEFORE you
    /// include the `-y` option and launch the operation. During the failover
    /// process, end-user clients cannot issue commands against the server.
    pub fn get_yes(&self) -> bool {
        self.yes
    }

    /// # Description
    ///
    /// -y
    ///
    /// Perform the failover operation. Without this option, `p4 failover`
    /// merely reports what it would do.
    ///
    /// Carefully consider the report of what failover would do BEFORE you
    /// include the `-y` option and launch the operation. During the failover
    /// process, end-user clients cannot issue commands against the server.
    pub fn set_yes(&mut self, v: bool) -> &mut Self {
        self.yes = v;
        self
    }

    /// # Description
    ///
    /// -y
    ///
    /// Perform the failover operation. Without this option, `p4 failover`
    /// merely reports what it would do.
    ///
    /// Carefully consider the report of what failover would do BEFORE you
    /// include the `-y` option and launch the operation. During the failover
    /// process, end-user clients cannot issue commands against the server.
    pub fn yes(mut self, v: bool) -> Self {
        self.yes = v;
        self
    }

    /// # Description
    ///
    /// -v verificationTime
    ///
    /// Specifies the `verificationTime` as a number of seconds prior to the launch of the failover command. The default value
    #[cfg_attr(
        feature = "lt2022_1",
        doc = "is `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new master server."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_1")),
        doc = "of `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new master server."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "of `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new target server."
    )]
    ///
    #[cfg_attr(
        feature = "lt2022_1",
        doc = "The failover command proceeds as soon as it has determined that the new master server has received correct file content."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_1")),
        doc = "The failover command proceeds as soon as it has determined that the new master server has received correct file content."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The failover command proceeds as soon as it has determined that the new target server has received correct file content."
    )]
    ///
    /// The legal range is between `0` (no verification) and `43200` seconds
    /// (12 hours).
    pub fn get_verification_time(&self) -> Option<u64> {
        self.verification_time
    }

    /// # Description
    ///
    /// -v verificationTime
    ///
    /// Specifies the `verificationTime` as a number of seconds prior to the launch of the failover command. The default value
    #[cfg_attr(
        feature = "lt2022_1",
        doc = "is `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new master server."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_1")),
        doc = "of `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new master server."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "of `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new target server."
    )]
    ///
    #[cfg_attr(
        feature = "lt2022_1",
        doc = "The failover command proceeds as soon as it has determined that the new master server has received correct file content."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_1")),
        doc = "The failover command proceeds as soon as it has determined that the new master server has received correct file content."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The failover command proceeds as soon as it has determined that the new target server has received correct file content."
    )]
    ///
    /// The legal range is between `0` (no verification) and `43200` seconds
    /// (12 hours).
    pub fn set_verification_time(&mut self, v: u64) -> &mut Self {
        debug_assert!(
            v <= MAX_VERIFICATION_TIME_SECONDS,
            "failover -v verification_time must not exceed {MAX_VERIFICATION_TIME_SECONDS} seconds (12 hours)"
        );
        self.verification_time = Some(v);
        self
    }

    /// # Description
    ///
    /// -v verificationTime
    ///
    /// Specifies the `verificationTime` as a number of seconds prior to the launch of the failover command. The default value
    #[cfg_attr(
        feature = "lt2022_1",
        doc = "is `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new master server."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_1")),
        doc = "of `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new master server."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "of `300` seconds means that any file content that was updated from 1 - 300 seconds prior to launch of failover will be verified. This is to ensure that this file content will be correct when the failover process completes on the new target server."
    )]
    ///
    #[cfg_attr(
        feature = "lt2022_1",
        doc = "The failover command proceeds as soon as it has determined that the new master server has received correct file content."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_1")),
        doc = "The failover command proceeds as soon as it has determined that the new master server has received correct file content."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "The failover command proceeds as soon as it has determined that the new target server has received correct file content."
    )]
    ///
    /// The legal range is between `0` (no verification) and `43200` seconds
    /// (12 hours).
    pub fn verification_time(mut self, v: u64) -> Self {
        self.set_verification_time(v);
        self
    }

    /// # Description
    ///
    /// -w waitForQuiesce
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
    /// To cancel the failover during the `quiesceWait` period, press Ctrl-C
    pub fn get_quiesce_wait_time(&self) -> Option<u64> {
        self.quiesce_wait_time
    }

    /// # Description
    ///
    /// -w waitForQuiesce
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
    /// To cancel the failover during the `quiesceWait` period, press Ctrl-C
    pub fn set_quiesce_wait_time(&mut self, v: u64) -> &mut Self {
        debug_assert!(
            v <= MAX_QUIESCE_WAIT_TIME_SECONDS,
            "failover -w quiesce_wait must not exceed {MAX_QUIESCE_WAIT_TIME_SECONDS} seconds (1 hour)"
        );
        self.quiesce_wait_time = Some(v);
        self
    }

    /// # Description
    ///
    /// -w waitForQuiesce
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
    /// To cancel the failover during the `quiesceWait` period, press Ctrl-C
    pub fn quiesce_wait_time(mut self, v: u64) -> Self {
        self.set_quiesce_wait_time(v);
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Failover<M> {
    fn name(&self) -> &str {
        "failover"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.yes {
            command.arg("-y");
        }

        self.mode.inject_args(command);

        if let Some(quiesce_wait_time) = self.quiesce_wait_time {
            command.arg("-w").arg(quiesce_wait_time.to_string());
        }

        if let Some(verification_time) = self.verification_time {
            command.arg("-v").arg(verification_time.to_string());
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption> ParameterizedSpawn<()> for Failover<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 failover` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I: AsRef<OsStr>, M: ExclusiveOption> ParameterizedSpawn<(I,)> for Failover<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 failover` with the given `failoverMessage` as the final
    /// argument; the process is a child process with piped standard output
    /// and error streams, use the returned [`Child`] handle to wait for it or
    /// interact with it.
    ///
    /// # Description
    ///
    /// failoverMessage
    ///
    #[cfg_attr(
        feature = "lt2025_1",
        doc = "If the master server participates in the failover operation, `failoverMessage` is the text to display to end-users during the failover operation when end-users attempt to start new commands on the master server. The default message is:"
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "If the target server participates in the failover operation, `failoverMessage` is the text to display to end-users during the failover operation when end-users attempt to start new commands on the target server. The default message is:"
    )]
    ///
    /// Server currently in failover mode, try again after failover has
    /// completed
    ///
    /// The `failoverMessage` must be the final argument specified in the
    /// `p4 failover` command. For example, `p4 failover -w 90 -v 500 -y The
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut command = Failover::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 failover` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let failover = Failover::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&failover.setup_command("p4")), ["failover"]);
    }

    #[test]
    fn yes_flag() {
        let failover = Failover::new("p4", GlobalOpts::new()).yes(true);

        assert!(failover.get_yes());
        assert_eq!(args_of(&failover.setup_command("p4")), ["failover", "-y"]);
    }

    #[test]
    fn required_target_mode() {
        let failover = Failover::new("p4", GlobalOpts::new()).required_target();

        assert_eq!(args_of(&failover.setup_command("p4")), ["failover", "-m"]);
    }

    #[test]
    fn ignore_target_mode_without_server_id() {
        let failover = Failover::new("p4", GlobalOpts::new()).ignore_target();

        assert_eq!(failover.get_server_id(), None);
        assert_eq!(args_of(&failover.setup_command("p4")), ["failover", "-i"]);
    }

    #[test]
    fn ignore_target_with_server_id() {
        let failover = Failover::new("p4", GlobalOpts::new()).ignore_target_with("backup_server");

        assert_eq!(
            args_of(&failover.setup_command("p4")),
            ["failover", "-i", "-s", "backup_server"]
        );
    }

    #[test]
    fn server_id_accessors() {
        let mut failover = Failover::new("p4", GlobalOpts::new()).ignore_target();

        failover.set_server_id("backup_server");
        assert_eq!(failover.get_server_id(), Some("backup_server"));

        let failover = failover.server_id("new_target");
        assert_eq!(failover.get_server_id(), Some("new_target"));
        assert_eq!(
            args_of(&failover.setup_command("p4")),
            ["failover", "-i", "-s", "new_target"]
        );
    }

    #[test]
    fn quiesce_wait_time_and_verification_time() {
        let failover = Failover::new("p4", GlobalOpts::new())
            .quiesce_wait_time(90)
            .verification_time(500);

        assert_eq!(failover.get_quiesce_wait_time(), Some(90));
        assert_eq!(failover.get_verification_time(), Some(500));
        assert_eq!(
            args_of(&failover.setup_command("p4")),
            ["failover", "-w", "90", "-v", "500"]
        );
    }

    #[test]
    fn transitions_preserve_options() {
        let failover = Failover::new("p4", GlobalOpts::new())
            .yes(true)
            .quiesce_wait_time(90)
            .required_target();

        assert_eq!(
            args_of(&failover.setup_command("p4")),
            ["failover", "-y", "-m", "-w", "90"]
        );
    }

    #[test]
    fn all_options_in_syntax_order() {
        let failover = Failover::new("p4", GlobalOpts::new())
            .yes(true)
            .ignore_target_with("new_target")
            .quiesce_wait_time(MAX_QUIESCE_WAIT_TIME_SECONDS)
            .verification_time(MAX_VERIFICATION_TIME_SECONDS);

        assert_eq!(
            args_of(&failover.setup_command("p4")),
            [
                "failover",
                "-y",
                "-i",
                "-s",
                "new_target",
                "-w",
                "3600",
                "-v",
                "43200"
            ]
        );
    }

    #[test]
    #[should_panic(expected = "1 hour")]
    fn quiesce_wait_time_above_maximum_panics_in_debug() {
        let _ = Failover::new("p4", GlobalOpts::new())
            .quiesce_wait_time(MAX_QUIESCE_WAIT_TIME_SECONDS + 1);
    }

    #[test]
    #[should_panic(expected = "12 hours")]
    fn verification_time_above_maximum_panics_in_debug() {
        let _ = Failover::new("p4", GlobalOpts::new())
            .verification_time(MAX_VERIFICATION_TIME_SECONDS + 1);
    }

    #[test]
    fn global_opts_are_injected_once() {
        let failover = Failover::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .yes(true);

        assert_eq!(
            args_of(&failover.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "failover", "-y"]
        );
    }
}
