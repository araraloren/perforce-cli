#[cfg(not(feature = "lt2022_1"))]
use super::IdentityChangeListMode;
use super::{
    ArbitraryJobStatus, Change, ForceOperationMode, OriginalChangeListMode, StdinMode, StdoutMode,
    Unselected, UpdateOperationMode,
};

// ---------------------------------------------------------------------------
// StdoutMode: `-f`, `-s`, `-O`, `-I` may still be selected
// ---------------------------------------------------------------------------

impl<O, C> Change<StdoutMode, O, C, Unselected> {
    /// Selects `-s` in the stdout form.
    pub fn arbitrary_job_status(self) -> Change<StdoutMode, O, C, ArbitraryJobStatus> {
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

impl<C, S> Change<StdoutMode, Unselected, C, S> {
    /// Selects `-f` in the stdout form.
    pub fn force(self) -> Change<StdoutMode, ForceOperationMode, C, S> {
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

impl<O, S> Change<StdoutMode, O, Unselected, S> {
    /// Selects `-O` in the stdout form (reference by original changelist
    /// number).
    pub fn original(self) -> Change<StdoutMode, O, OriginalChangeListMode, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: self.permission_mode,
            changelist_mode: OriginalChangeListMode,
            job_status_mode: self.job_status_mode,
        }
    }

    /// Selects `-I` in the stdout form (reference by Identity field).
    #[cfg(not(feature = "lt2022_1"))]
    pub fn identity(self) -> Change<StdoutMode, O, IdentityChangeListMode, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: self.permission_mode,
            changelist_mode: IdentityChangeListMode,
            job_status_mode: self.job_status_mode,
        }
    }
}

// ---------------------------------------------------------------------------
// StdinMode: `-f`, `-u`, `-s` may still be selected
// ---------------------------------------------------------------------------

impl<O> Change<StdinMode, O, Unselected, Unselected> {
    /// Selects `-s` in the stdin form.
    pub fn arbitrary_job_status(self) -> Change<StdinMode, O, Unselected, ArbitraryJobStatus> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: self.permission_mode,
            changelist_mode: Unselected,
            job_status_mode: ArbitraryJobStatus,
        }
    }
}

impl<S> Change<StdinMode, Unselected, Unselected, S> {
    /// Selects `-f` in the stdin form.
    pub fn force(self) -> Change<StdinMode, ForceOperationMode, Unselected, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: ForceOperationMode,
            changelist_mode: Unselected,
            job_status_mode: self.job_status_mode,
        }
    }

    /// Selects `-u` in the stdin form.
    pub fn update(self) -> Change<StdinMode, UpdateOperationMode, Unselected, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: UpdateOperationMode,
            changelist_mode: Unselected,
            job_status_mode: self.job_status_mode,
        }
    }
}
