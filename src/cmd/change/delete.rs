use std::process::Command;

use super::{
    ArbitraryJobStatus, Change, ExclusiveOption, ForceOperationMode, OriginalChangeListMode,
    TakesChangeList, Unselected,
};

// ---------------------------------------------------------------------------
// Form dimension: the delete variants
// ---------------------------------------------------------------------------

/// Local variant of the delete form: `p4 change -d [-f -s -O] changelist`.
///
/// The remote variant (`-d -f --serverid=X`) is [`RemoteDeleteMode`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocalDeleteMode;

/// Remote variant of the delete form: `p4 change -d -f --serverid=X changelist`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteDeleteMode {
    pub(super) server_id: String,
}

/// The delete form: `p4 change -d [-f -s -O] changelist`.
///
/// The `L` parameter distinguishes the [`LocalDeleteMode`] form from the
/// [`RemoteDeleteMode`] form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteMode<L = LocalDeleteMode> {
    pub(super) loc: L,
}

impl ExclusiveOption for LocalDeleteMode {}

impl ExclusiveOption for RemoteDeleteMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-f");
        command.arg(format!("--serverid={}", self.server_id));
    }
}

impl<L: ExclusiveOption> ExclusiveOption for DeleteMode<L> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-d");
        self.loc.inject_args(command);
    }
}

impl<L> TakesChangeList for DeleteMode<L> {}

// ---------------------------------------------------------------------------
// Mapping traits for entering the delete forms
// ---------------------------------------------------------------------------

/// Marker trait for the option states allowed when entering the local
/// `-d` form from the edit form.
///
/// The same trait bounds both orthogonal dimensions of the transition:
///
/// - the operation dimension: `Unselected` and [`ForceOperationMode`],
///   since `-d` accepts `-f` but not `-u`;
/// - the changelist-reference dimension: `Unselected` and
///   [`OriginalChangeListMode`], since `-d` accepts `-O` but not `-I`.
///
/// `Unselected` is shared by both dimensions and is therefore implemented
/// only once.
#[doc(hidden)]
pub trait TransferDeleteMode {}

// Operation dimension: `-d` accepts `-f` but not `-u`.
impl TransferDeleteMode for Unselected {}
impl TransferDeleteMode for ForceOperationMode {}

// Changelist-reference dimension: `-d` accepts `-O` but not `-I`.
impl TransferDeleteMode for OriginalChangeListMode {}

/// Marks the operation modes allowed when entering the remote-delete form.
///
/// Only `Unselected` and `ForceOperationMode` are allowed: the remote
/// delete form accepts neither `-u` nor any explicit `-f` (its `-f` is
/// built in).
#[doc(hidden)]
pub trait TransferRemoteDeleteMode {}

impl TransferRemoteDeleteMode for Unselected {}
impl TransferRemoteDeleteMode for ForceOperationMode {}

// ---------------------------------------------------------------------------
// DeleteMode<LocalDeleteMode>: `-f`, `-s`, `-O` may still be selected
// ---------------------------------------------------------------------------

impl<O, C> Change<DeleteMode<LocalDeleteMode>, O, C, Unselected> {
    /// Selects `-s` in the delete form.
    pub fn arbitrary_job_status(
        self,
    ) -> Change<DeleteMode<LocalDeleteMode>, O, C, ArbitraryJobStatus> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: self.permission_mode,
            changelist_mode: self.changelist_mode,
            job_status_mode: ArbitraryJobStatus,
        }
    }
}

impl<C, S> Change<DeleteMode<LocalDeleteMode>, Unselected, C, S> {
    /// Selects `-f` in the delete form (forcibly delete a submitted
    /// changelist).
    pub fn force(self) -> Change<DeleteMode<LocalDeleteMode>, ForceOperationMode, C, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: ForceOperationMode,
            changelist_mode: self.changelist_mode,
            job_status_mode: self.job_status_mode,
        }
    }
}

impl<O, S> Change<DeleteMode<LocalDeleteMode>, O, Unselected, S> {
    /// Selects `-O` in the delete form (reference by original changelist
    /// number).
    pub fn original(self) -> Change<DeleteMode<LocalDeleteMode>, O, OriginalChangeListMode, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: self.permission_mode,
            changelist_mode: OriginalChangeListMode,
            job_status_mode: self.job_status_mode,
        }
    }
}

// Local delete -> remote delete: no `-s`/`-O`; `-u` is never allowed.
impl<O: TransferRemoteDeleteMode> Change<DeleteMode<LocalDeleteMode>, O, Unselected, Unselected> {
    /// Transitions to the remote delete form (`-d -f --serverid=X`).
    ///
    /// Available from the local delete form as long as neither `-s`,
    /// `-O`, nor `-u` is selected (the remote delete form accepts no
    /// extra options; its `-f` is built in). A previously selected `-f`
    /// is simply dropped in favor of the built-in one.
    ///
    /// `server_id` is the server id of the commit server the changelist's
    /// client is bound to.
    pub fn server_id(
        self,
        server_id: impl Into<String>,
    ) -> Change<DeleteMode<RemoteDeleteMode>, Unselected, Unselected, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                loc: RemoteDeleteMode {
                    server_id: server_id.into(),
                },
            },
            permission_mode: Unselected,
            changelist_mode: Unselected,
            job_status_mode: Unselected,
        }
    }
}
