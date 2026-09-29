use std::process::Command;

use super::{
    Change, ExclusiveOption, ForceOperationMode, IdentityChangeListMode, OnlyForceOperationMode,
    OnlyIdentityChangeListMode, OnlyOriginalChangeListMode, OnlyUpdateOperationMode,
    OriginalChangeListMode, TakesChangeList, Unselected, UpdateOperationMode, Visibility,
};

// ---------------------------------------------------------------------------
// Form dimension: the merged `-t` / `-U` form
// ---------------------------------------------------------------------------

/// The change-type / owner-transfer form:
/// `p4 change -t restricted | public [-U user] [-f|-u|-O|-I] changelist`.
///
/// The `-t` and `-U` syntax forms of `p4 change` describe the same
/// operation — changing the type and/or the owner of a changelist — so both
/// are encoded by this single type. At least one of the two values is always
/// set: entry points and builder methods only ever assign a value, never
/// clear one.
///
/// The form accepts at most one of `-f`, `-u`, `-O`, `-I` (carried by the
/// `Only*` modes) and never `-s`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOrVisibilityMode {
    pub(super) visibility: Option<Visibility>,

    pub(super) user: Option<String>,
}

impl ExclusiveOption for UserOrVisibilityMode {
    fn inject_args(&self, command: &mut Command) {
        if let Some(visibility) = &self.visibility {
            command.arg("-t").arg(visibility.as_str());
        }

        if let Some(user) = &self.user {
            command.arg("-U").arg(user);
        }
    }
}

impl TakesChangeList for UserOrVisibilityMode {}

// ---------------------------------------------------------------------------
// Mapping trait for entering the form
// ---------------------------------------------------------------------------

/// Maps the combined `(operation, changelist-reference)` selection when
/// entering the `-t` / `-U` form.
///
/// Only the five legal combinations of that form (`nothing`, `-f`, `-u`,
/// `-O`, `-I`) implement this, so `-f` together with `-O` (and similar)
/// fails to compile rather than silently dropping one option.
///
/// The output states are associated types so callers never have to specify
/// them: they are uniquely determined by the input combination.
#[doc(hidden)]
pub trait TransferUVMode {
    type P;
    type C;
    fn transfer(self) -> (Self::P, Self::C);
}

impl TransferUVMode for (Unselected, Unselected) {
    type P = Unselected;
    type C = Unselected;
    fn transfer(self) -> (Unselected, Unselected) {
        (Unselected, Unselected)
    }
}

impl TransferUVMode for (ForceOperationMode, Unselected) {
    type P = OnlyForceOperationMode;
    type C = Unselected;
    fn transfer(self) -> (OnlyForceOperationMode, Unselected) {
        (OnlyForceOperationMode, Unselected)
    }
}

impl TransferUVMode for (UpdateOperationMode, Unselected) {
    type P = OnlyUpdateOperationMode;
    type C = Unselected;
    fn transfer(self) -> (OnlyUpdateOperationMode, Unselected) {
        (OnlyUpdateOperationMode, Unselected)
    }
}

impl TransferUVMode for (Unselected, OriginalChangeListMode) {
    type P = Unselected;
    type C = OnlyOriginalChangeListMode;
    fn transfer(self) -> (Unselected, OnlyOriginalChangeListMode) {
        (Unselected, OnlyOriginalChangeListMode)
    }
}

impl TransferUVMode for (Unselected, IdentityChangeListMode) {
    type P = Unselected;
    type C = OnlyIdentityChangeListMode;
    fn transfer(self) -> (Unselected, OnlyIdentityChangeListMode) {
        (Unselected, OnlyIdentityChangeListMode)
    }
}

// ---------------------------------------------------------------------------
// UserOrVisibilityMode: setting the form values
// ---------------------------------------------------------------------------

impl<O, C> Change<UserOrVisibilityMode, O, C, Unselected> {
    /// Sets the new owner (`-U user`) of the changelist.
    ///
    /// `user` is the user the changelist is transferred to.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.mode.user = Some(user.into());
        self
    }

    /// Sets the new owner (`-U user`) of the changelist.
    ///
    /// `user` is the user the changelist is transferred to.
    pub fn set_user(&mut self, user: impl Into<String>) -> &mut Self {
        self.mode.user = Some(user.into());
        self
    }

    /// Returns the new owner (`-U user`) of the changelist, if set.
    pub fn get_user(&self) -> Option<&str> {
        self.mode.user.as_deref()
    }

    /// Sets the changelist visibility (`-t restricted | public`).
    ///
    /// `visibility` selects the new changelist type.
    pub fn visibility(mut self, visibility: Visibility) -> Self {
        self.mode.visibility = Some(visibility);
        self
    }

    /// Sets the changelist visibility (`-t restricted | public`).
    ///
    /// `visibility` selects the new changelist type.
    pub fn set_visibility(&mut self, visibility: Visibility) -> &mut Self {
        self.mode.visibility = Some(visibility);
        self
    }

    /// Returns the changelist visibility (`-t restricted | public`), if set.
    pub fn get_visibility(&self) -> Option<Visibility> {
        self.mode.visibility
    }
}

// ---------------------------------------------------------------------------
// UserOrVisibilityMode: selecting the single extra option
// ---------------------------------------------------------------------------

// The form accepts at most one of `-f`, `-u`, `-O`, `-I`, so each selection
// method is only available while neither dimension has been selected yet.
impl Change<UserOrVisibilityMode, Unselected, Unselected, Unselected> {
    /// Selects `-f` (force the type/owner change).
    ///
    /// Mutually exclusive with `-u`, `-O`, and `-I`.
    pub fn force(
        self,
    ) -> Change<UserOrVisibilityMode, OnlyForceOperationMode, Unselected, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: OnlyForceOperationMode,
            changelist_mode: Unselected,
            job_status_mode: Unselected,
        }
    }

    /// Selects `-u` (update a submitted changelist).
    ///
    /// Mutually exclusive with `-f`, `-O`, and `-I`.
    pub fn update(
        self,
    ) -> Change<UserOrVisibilityMode, OnlyUpdateOperationMode, Unselected, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: OnlyUpdateOperationMode,
            changelist_mode: Unselected,
            job_status_mode: Unselected,
        }
    }

    /// Selects `-O`: interpret the changelist number as the original,
    /// pre-renumber number.
    ///
    /// Mutually exclusive with `-f`, `-u`, and `-I`.
    pub fn original(
        self,
    ) -> Change<UserOrVisibilityMode, Unselected, OnlyOriginalChangeListMode, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: Unselected,
            changelist_mode: OnlyOriginalChangeListMode,
            job_status_mode: Unselected,
        }
    }

    /// Selects `-I`: interpret the changelist number as the Identity field.
    ///
    /// Mutually exclusive with `-f`, `-u`, and `-O`.
    pub fn identity(
        self,
    ) -> Change<UserOrVisibilityMode, Unselected, OnlyIdentityChangeListMode, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: Unselected,
            changelist_mode: OnlyIdentityChangeListMode,
            job_status_mode: Unselected,
        }
    }
}
