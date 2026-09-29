use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

pub mod delete;
pub mod inout;
pub mod userorvis;

pub use delete::{DeleteMode, LocalDeleteMode, TransferDeleteMode};
#[cfg(not(feature = "lt2015_2"))]
pub use delete::{RemoteDeleteMode, TransferRemoteDeleteMode};
pub use userorvis::{TransferUVMode, UserOrVisibilityMode};

// The seven syntax forms of `p4 change`:
//
// `p4 [g-opts] change [-s] [-f | -u] [[-O|-I] changelist]`
// `p4 [g-opts] change -d [-f -s -O] changelist`
// `p4 [g-opts] change -o [-s] [-f] [[-O|-I] changelist]`
// `p4 [g-opts] change -i [-s] [-f | -u]`
// `p4 [g-opts] change -t restricted | public [-U user] [-f|-u|-O|-I] changelist`
// `p4 [g-opts] change -U user [-t restricted | public] [-f] changelist`
// `p4 [g-opts] change -d -f --serverid=X changelist`
//
// The `-t` and `-U` forms describe the same operation — changing the type
// and/or the owner of a changelist — so they share a single form type,
// `UserOrVisibilityMode`, which accepts at most one of `-f`, `-u`, `-O`,
// and `-I` (and never `-s`).
//
// Compatibility matrix of the options across the forms:
//
// | option | edit | -d | -o | -i | -t/-U  | -d --serverid |
// |--------|------|----|----|----|--------|---------------|
// | -s     |  yes | yes| yes| yes| no     | no            |
// | -f     |  yes | yes| yes| yes| one of | forced        |
// | -u     |  yes | no | no | yes| one of | no            |
// | -O     |  yes | yes| yes| no | one of | no            |
// | -I     |  yes | no | yes| no | one of | no            |
//
// The matrix is encoded in four orthogonal type parameters:
//
// - `F`: the command form (edit/delete/stdout/stdin/change-type-or-owner)
// - `O`: the `-f` / `-u` operation
// - `C`: the `-O` / `-I` changelist reference
// - `S`: the `-s` arbitrary job status flag
//
// Transitions between forms are only `impl`ed when every selected option is
// accepted by the target form, so an illegal transition (for example moving
// `-I` into `change -d`, or `-s` into `change -t`) does not compile.

/// Visibility of a changelist as set with `-t restricted | public`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// The changelist is visible without restrictions (`public`).
    Public,

    /// The changelist is hidden from users who do not own it or lack `list`
    /// permission on its files (`restricted`).
    Restricted,
}

impl Visibility {
    /// Returns the command-line representation of this visibility.
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Restricted => "restricted",
        }
    }
}

// ---------------------------------------------------------------------------
// Form dimension (`F`)
// ---------------------------------------------------------------------------

/// The plain edit/create form: `p4 change [-s] [-f|-u] [[-O|-I] changelist]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegularOperationMode;

/// The stdout form: `p4 change -o [-s] [-f] [[-O|-I] changelist]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StdoutMode;

/// The stdin form: `p4 change -i [-s] [-f|-u]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StdinMode;

// ---------------------------------------------------------------------------
// Operation dimension (`O`): `-f` / `-u`
// ---------------------------------------------------------------------------

/// `-f` selected in a form where it may be combined with other options
/// (`-s`, `-O`, `-I`): the edit, `-d`, `-o`, and `-i` forms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ForceOperationMode;

/// `-u` selected in a form where it may be combined with `-s`: the edit and
/// `-i` forms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpdateOperationMode;

/// `-f` selected as the single operation of the `-t` / `-U` forms, where it
/// is mutually exclusive with `-u`, `-O`, and `-I`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnlyForceOperationMode;

/// `-u` selected as the single operation of the `-t` / `-U` forms, where
/// it is mutually exclusive with `-f`, `-O`, and `-I`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnlyUpdateOperationMode;

// ---------------------------------------------------------------------------
// Changelist-reference dimension (`C`): `-O` / `-I`
// ---------------------------------------------------------------------------

/// `-O` selected in a form where it may be combined with other options
/// (`-f`, `-s`): the edit, `-d`, and `-o` forms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OriginalChangeListMode;

/// `-I` selected in a form where it may be combined with other options
/// (`-f`, `-s`): the edit and `-o` forms.
#[cfg(not(feature = "lt2022_1"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IdentityChangeListMode;

/// `-O` selected as the single reference of the `-t` / `-U` forms, where
/// it is mutually exclusive with `-f`, `-u`, and `-I`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnlyOriginalChangeListMode;

/// `-I` selected as the single reference of the `-t` / `-U` forms, where
/// it is mutually exclusive with `-f`, `-u`, and `-O`.
#[cfg(not(feature = "lt2015_2"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnlyIdentityChangeListMode;

// ---------------------------------------------------------------------------
// Job-status dimension (`S`): `-s`
// ---------------------------------------------------------------------------

/// `-s` selected: jobs may be assigned arbitrary status values on submission.
///
/// Unlike a plain boolean field, selecting `-s` is its own type state so that
/// forms that do not accept `-s` (the `-t`, `-U`, and remote `-d` forms)
/// cannot be entered once it is set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArbitraryJobStatus;

// ---------------------------------------------------------------------------
// ExclusiveOption implementations
// ---------------------------------------------------------------------------

impl ExclusiveOption for ForceOperationMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-f");
    }
}

impl ExclusiveOption for UpdateOperationMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-u");
    }
}

impl ExclusiveOption for OnlyForceOperationMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-f");
    }
}

impl ExclusiveOption for OnlyUpdateOperationMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-u");
    }
}

impl ExclusiveOption for OriginalChangeListMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-O");
    }
}

#[cfg(not(feature = "lt2022_1"))]
impl ExclusiveOption for IdentityChangeListMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-I");
    }
}

impl ExclusiveOption for OnlyOriginalChangeListMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-O");
    }
}

#[cfg(not(feature = "lt2015_2"))]
impl ExclusiveOption for OnlyIdentityChangeListMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-I");
    }
}

impl ExclusiveOption for ArbitraryJobStatus {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-s");
    }
}

impl ExclusiveOption for RegularOperationMode {}

impl ExclusiveOption for StdoutMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-o");
    }
}

impl ExclusiveOption for StdinMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-i");
    }
}

// ---------------------------------------------------------------------------
// Marker traits bounding the cross-form transitions
// ---------------------------------------------------------------------------

/// Forms that take a `changelist` positional argument at spawn time.
///
/// [`StdinMode`] (`change -i`) deliberately does not implement this, so no
/// changelist argument can be passed to it.
#[doc(hidden)]
pub trait TakesChangeList {}

impl TakesChangeList for RegularOperationMode {}
impl TakesChangeList for StdoutMode {}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 change`.
///
/// Create or edit a changelist specification.
///
/// The command `p4 changelist` is an alias for `p4 change`.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] change [-s] [-f | -u] [[-O|-I] changelist]
/// p4 [g-opts] change -d [-f -s -O] changelist
/// p4 [g-opts] change -o [-s] [-f] [[-O|-I] changelist]
/// p4 [g-opts] change -i [-s] [-f | -u]
/// p4 [g-opts] change -t restricted | public [-U user] [-f|-u|-O|-I] changelist
/// p4 [g-opts] change -U user [-t restricted | public] [-f] changelist
/// p4 [g-opts] change -d -f --serverid=X changelist
/// ```
///
/// The four type parameters track, independently and at compile time:
///
/// - `F`: the command form — [`RegularOperationMode`], [`DeleteMode`],
///   [`StdoutMode`], [`StdinMode`], or [`UserOrVisibilityMode`];
/// - `O`: the `-f` / `-u` operation — [`ForceOperationMode`],
///   [`UpdateOperationMode`], or the `Only*` variants used inside the
///   `-t` / `-U` form;
/// - `C`: the `-O` / `-I` changelist reference —
///   [`OriginalChangeListMode`]
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "   / [`IdentityChangeListMode`], or their"
)]
#[cfg_attr(feature = "lt2022_1", doc = "   , or its")]
///   `Only*` variants;
/// - `S`: the `-s` arbitrary job status flag ([`ArbitraryJobStatus`]).
///
/// Start from [`Change::new`] (or [`crate::P4Cli::change`]), enter a form
/// with one of [`Self::delete`], [`Self::stdout`], [`Self::stdin`],
/// [`Self::visibility`], or [`Self::user`] (or one of the
/// [`Self::force`]/[`Self::update`]/[`Self::original`]/
#[cfg_attr(not(feature = "lt2022_1"), doc = "[`Self::identity`]/")]
/// [`Self::arbitrary_job_status`] entry points that select an option of the
/// plain edit form), and move between compatible forms with the
/// `delete`, `stdout`, `stdin`, `visibility`, `user`, and `server_id`
/// transition methods.
#[derive(Debug, Clone, Default)]
pub struct Change<M = Unselected, P = Unselected, C = Unselected, S = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,

    permission_mode: P,

    changelist_mode: C,

    job_status_mode: S,
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

impl Change<Unselected, Unselected, Unselected, Unselected> {
    /// Creates a new `p4 change` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: Unselected,
            permission_mode: Unselected,
            changelist_mode: Unselected,
            job_status_mode: Unselected,
        }
    }

    /// Enters the local delete form: `p4 change -d changelist`.
    pub fn delete(self) -> Change<DeleteMode<LocalDeleteMode>, Unselected, Unselected, Unselected> {
        self.with_mode(DeleteMode {
            loc: LocalDeleteMode,
        })
    }

    /// Enters the remote delete form.
    ///
    /// `-f` is fixed in this form and is injected together with the form;
    /// none of the other options are accepted.
    ///
    /// `server_id` is the server id of the commit server the changelist's
    /// client is bound to.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn server_id(
        self,
        server_id: impl Into<String>,
    ) -> Change<DeleteMode<RemoteDeleteMode>, Unselected, Unselected, Unselected> {
        self.with_mode(DeleteMode {
            loc: RemoteDeleteMode {
                server_id: server_id.into(),
            },
        })
    }

    /// Enters the stdout form: `p4 change -o`.
    pub fn stdout(self) -> Change<StdoutMode, Unselected, Unselected, Unselected> {
        self.with_mode(StdoutMode)
    }

    /// Enters the stdin form: `p4 change -i`.
    pub fn stdin(self) -> Change<StdinMode, Unselected, Unselected, Unselected> {
        self.with_mode(StdinMode)
    }

    /// Enters the change-type form: `p4 change -t restricted | public
    /// changelist`.
    ///
    /// `visibility` selects the new changelist type; the optional new owner
    /// can be added later with the `user` builder of the entered form.
    pub fn visibility(
        self,
        visibility: Visibility,
    ) -> Change<UserOrVisibilityMode, Unselected, Unselected, Unselected> {
        self.with_mode(UserOrVisibilityMode {
            visibility: Some(visibility),
            user: None,
        })
    }

    /// Enters the owner-transfer form: `p4 change -U user changelist`.
    ///
    /// `user` is the new owner of the empty pending changelist; the optional
    /// visibility can be added later with the `visibility` builder of the
    /// entered form.
    pub fn user(
        self,
        user: impl Into<String>,
    ) -> Change<UserOrVisibilityMode, Unselected, Unselected, Unselected> {
        self.with_mode(UserOrVisibilityMode {
            user: Some(user.into()),
            visibility: None,
        })
    }

    /// Selects `-s` and enters the plain edit form.
    pub fn arbitrary_job_status(
        self,
    ) -> Change<RegularOperationMode, Unselected, Unselected, ArbitraryJobStatus> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularOperationMode,
            permission_mode: Unselected,
            changelist_mode: Unselected,
            job_status_mode: ArbitraryJobStatus,
        }
    }

    /// Selects `-f` and enters the plain edit form.
    pub fn force(self) -> Change<RegularOperationMode, ForceOperationMode, Unselected, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularOperationMode,
            permission_mode: ForceOperationMode,
            changelist_mode: Unselected,
            job_status_mode: Unselected,
        }
    }

    /// Selects `-u` and enters the plain edit form.
    pub fn update(
        self,
    ) -> Change<RegularOperationMode, UpdateOperationMode, Unselected, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularOperationMode,
            permission_mode: UpdateOperationMode,
            changelist_mode: Unselected,
            job_status_mode: Unselected,
        }
    }

    /// Selects `-O` and enters the plain edit form.
    pub fn original(
        self,
    ) -> Change<RegularOperationMode, Unselected, OriginalChangeListMode, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularOperationMode,
            permission_mode: Unselected,
            changelist_mode: OriginalChangeListMode,
            job_status_mode: Unselected,
        }
    }

    /// Selects `-I` and enters the plain edit form.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn identity(
        self,
    ) -> Change<RegularOperationMode, Unselected, IdentityChangeListMode, Unselected> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: RegularOperationMode,
            permission_mode: Unselected,
            changelist_mode: IdentityChangeListMode,
            job_status_mode: Unselected,
        }
    }
}

impl<M, P, C, S> Change<M, P, C, S> {
    /// Rebuilds the command with another form while preserving the selected
    /// options and the `bin` / global options.
    fn with_mode<OTHERMODE>(self, mode: OTHERMODE) -> Change<OTHERMODE, P, C, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode,
            permission_mode: self.permission_mode,
            changelist_mode: self.changelist_mode,
            job_status_mode: self.job_status_mode,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared accessors
// ---------------------------------------------------------------------------

impl<M, P, C, S> Change<M, P, C, S> {
    /// Returns the global options of this command.
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// Sets the global options of this command.
    pub fn set_global_opts(&mut self, v: GlobalOpts) -> &mut Self {
        self.global_opts = v;
        self
    }

    /// Builder for the global options of this command.
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.global_opts = v;
        self
    }
}

/// Operation modes allowed to enter the `-o` form.
///
/// Only `Unselected` and [`ForceOperationMode`] are allowed: `-o` does not
/// accept `-u`.
#[doc(hidden)]
pub trait TransferStdoutMode {}

impl TransferStdoutMode for Unselected {}
impl TransferStdoutMode for ForceOperationMode {}

// ---------------------------------------------------------------------------
// RegularOperationMode: selecting options and transitioning to other forms
// ---------------------------------------------------------------------------

impl<P, C, S> Change<RegularOperationMode, P, C, S> {
    /// Transitions to the local delete form (`-d`), preserving every
    /// selected option accepted by that form.
    ///
    /// Available when no `-u` or `-I` is selected (`-d` accepts only `-f`,
    /// `-s`, and `-O`).
    pub fn delete(self) -> Change<DeleteMode<LocalDeleteMode>, P, C, S>
    where
        P: TransferDeleteMode,
        C: TransferDeleteMode,
    {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                loc: LocalDeleteMode,
            },
            permission_mode: self.permission_mode,
            changelist_mode: self.changelist_mode,
            job_status_mode: self.job_status_mode,
        }
    }

    /// Transitions to the stdout form (`-o`), preserving every selected
    /// option accepted by that form.
    ///
    /// Available when no `-u` is selected (`-o` accepts `-f`, `-s`, `-O`,
    /// and `-I`).
    pub fn stdout(self) -> Change<StdoutMode, P, C, S>
    where
        P: TransferStdoutMode,
    {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdoutMode,
            permission_mode: self.permission_mode,
            changelist_mode: self.changelist_mode,
            job_status_mode: self.job_status_mode,
        }
    }
}

// Regular -> remote delete: no `-u`, `-s`, `-O`, or `-I`; `-f` is optional
// (it is folded into the form's built-in `-f`).
#[cfg(not(feature = "lt2015_2"))]
impl<P: TransferRemoteDeleteMode> Change<RegularOperationMode, P, Unselected, Unselected> {
    /// Transitions to the remote delete form.
    ///
    /// Available from the plain edit form as long as neither `-u`, `-s`,
    /// `-O`, nor `-I` is selected. A previously selected `-f` is folded
    /// into the form's built-in `-f`.
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

// Regular -> stdin: the changelist reference must be unset (`-i` takes no
// changelist argument and accepts neither `-O` nor `-I`).
impl<O, S> Change<RegularOperationMode, O, Unselected, S> {
    /// Transitions to the stdin form (`-i`), preserving every selected
    /// option accepted by that form.
    ///
    /// Available only when no `-O` or `-I` is selected (`-i` accepts `-s`,
    /// `-f`, and `-u`, but no changelist reference).
    pub fn stdin(self) -> Change<StdinMode, O, Unselected, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdinMode,
            permission_mode: self.permission_mode,
            changelist_mode: Unselected,
            job_status_mode: self.job_status_mode,
        }
    }
}

// -s within the edit form
impl<O, C> Change<RegularOperationMode, O, C, Unselected> {
    /// Selects `-s`: jobs may be assigned arbitrary status values on
    /// submission.
    pub fn arbitrary_job_status(self) -> Change<RegularOperationMode, O, C, ArbitraryJobStatus> {
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

// -f / -u within the edit form
impl<C, S> Change<RegularOperationMode, Unselected, C, S> {
    /// Selects `-f` (force editing of submitted changelists).
    ///
    /// Mutually exclusive with `-u`.
    pub fn force(self) -> Change<RegularOperationMode, ForceOperationMode, C, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: ForceOperationMode,
            changelist_mode: self.changelist_mode,
            job_status_mode: self.job_status_mode,
        }
    }

    /// Selects `-u` (update a submitted changelist).
    ///
    /// Mutually exclusive with `-f`.
    pub fn update(self) -> Change<RegularOperationMode, UpdateOperationMode, C, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: UpdateOperationMode,
            changelist_mode: self.changelist_mode,
            job_status_mode: self.job_status_mode,
        }
    }
}

// -O / -I within the edit form
impl<O, S> Change<RegularOperationMode, O, Unselected, S> {
    /// Selects `-O`: interpret the changelist number as the original,
    /// pre-renumber number.
    ///
    /// Mutually exclusive with `-I`.
    pub fn original(self) -> Change<RegularOperationMode, O, OriginalChangeListMode, S> {
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            permission_mode: self.permission_mode,
            changelist_mode: OriginalChangeListMode,
            job_status_mode: self.job_status_mode,
        }
    }

    /// Selects `-I`: interpret the changelist number as the Identity field.
    ///
    /// Mutually exclusive with `-O`.
    #[cfg(not(feature = "lt2022_1"))]
    pub fn identity(self) -> Change<RegularOperationMode, O, IdentityChangeListMode, S> {
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

// Regular -> change type / owner transfer: `-s` must be unset; at most one
// of `-f`/`-u`/`-O`/`-I` may be selected and maps to the corresponding
// `Only*` state.
impl<O, C> Change<RegularOperationMode, O, C, Unselected> {
    /// Transitions to the change-type form (`-t type`), preserving a
    /// selected `-f`, `-u`, `-O`, or `-I` as the form's single allowed
    /// extra option.
    ///
    /// Available only when `-s` is not selected (the `-t` / `-U` form does
    /// not accept it), and at most one of `-f`, `-u`, `-O`, `-I` is
    /// selected.
    ///
    /// `visibility` selects the new changelist type.
    #[allow(clippy::type_complexity)]
    pub fn visibility(
        self,
        visibility: Visibility,
    ) -> Change<
        UserOrVisibilityMode,
        <(O, C) as TransferUVMode>::P,
        <(O, C) as TransferUVMode>::C,
        Unselected,
    >
    where
        (O, C): TransferUVMode,
    {
        let (operation, change_ref) = (self.permission_mode, self.changelist_mode).transfer();
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: UserOrVisibilityMode {
                visibility: Some(visibility),
                user: None,
            },
            permission_mode: operation,
            changelist_mode: change_ref,
            job_status_mode: Unselected,
        }
    }

    /// Transitions to the owner-transfer form (`-U user`), preserving a
    /// selected `-f`, `-u`, `-O`, or `-I` as the form's single allowed
    /// extra option.
    ///
    /// Available only when `-s` is not selected (the `-t` / `-U` form does
    /// not accept it), and at most one of `-f`, `-u`, `-O`, `-I` is
    /// selected.
    ///
    /// `user` is the new owner of the empty pending changelist.
    #[allow(clippy::type_complexity)]
    pub fn user(
        self,
        user: impl Into<String>,
    ) -> Change<
        UserOrVisibilityMode,
        <(O, C) as TransferUVMode>::P,
        <(O, C) as TransferUVMode>::C,
        Unselected,
    >
    where
        (O, C): TransferUVMode,
    {
        let (operation, change_ref) = (self.permission_mode, self.changelist_mode).transfer();
        Change {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: UserOrVisibilityMode {
                user: Some(user.into()),
                visibility: None,
            },
            permission_mode: operation,
            changelist_mode: change_ref,
            job_status_mode: Unselected,
        }
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl<F, O, C, S> SubCommand for Change<F, O, C, S>
where
    F: ExclusiveOption,
    O: ExclusiveOption,
    C: ExclusiveOption,
    S: ExclusiveOption,
{
    fn name(&self) -> &str {
        "change"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);
        self.job_status_mode.inject_args(command);
        self.permission_mode.inject_args(command);
        self.changelist_mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

impl<F, O, C, S, I> ParameterizedSpawn<(I,)> for Change<F, O, C, S>
where
    F: ExclusiveOption + TakesChangeList,
    O: ExclusiveOption,
    C: ExclusiveOption,
    S: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 change` with the given `changelist` argument as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    ///
    /// The changelist argument is a changelist number, optionally prefixed
    /// by the form's reference option semantics (for example an original
    /// pre-renumber number under `-O`).
    fn spawn_with(&mut self, (changelist,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(changelist)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// RegularOperationMode accepts an optional changelist: without one it
// opens the editor to create a new pending changelist.
impl<O, C, S> ParameterizedSpawn<()> for Change<RegularOperationMode, O, C, S>
where
    O: ExclusiveOption,
    C: ExclusiveOption,
    S: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 change` (the plain edit form) without a changelist
    /// argument, opening the editor to create a new pending changelist.
    fn spawn_with(&mut self, _: ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// StdoutMode accepts an optional changelist: without one it dumps the
// default changelist specification.
impl<O, C, S> ParameterizedSpawn<()> for Change<StdoutMode, O, C, S>
where
    O: ExclusiveOption,
    C: ExclusiveOption,
    S: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 change -o` without a changelist argument, writing the
    /// default changelist specification to standard output.
    fn spawn_with(&mut self, _: ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<O, S> ParameterizedSpawn<()> for Change<StdinMode, O, Unselected, S>
where
    O: ExclusiveOption,
    S: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 change -i` with a piped standard input stream (and piped
    /// standard output and error streams); write the changelist
    /// specification to the returned [`Child`]'s stdin handle.
    fn spawn_with(&mut self, _: ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<O, S> ParameterizedSpawn<(Stdio,)> for Change<StdinMode, O, Unselected, S>
where
    O: ExclusiveOption,
    S: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 change -i` with the given standard input configuration
    /// and piped standard output and error streams; use the returned
    /// [`Child`] handle to wait for it or interact with it.
    ///
    /// Pass `Stdio::piped()` to obtain a writable `child.stdin` handle and
    /// feed the changelist specification into it; pass `Stdio::inherit()`
    /// to read the specification from the spawning process's own standard
    /// input.
    fn spawn_with(&mut self, (stdio,): (Stdio,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(stdio)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    // Compile-time rejection matrix. Every transition below was verified to
    // fail compilation (trait bound not satisfied or method not found):
    //
    // - `.update().delete()`             (`-u` is not accepted by `-d`)
    // - `.identity().delete()`           (`-I` is not accepted by `-d`)
    // - `.update().stdout()`             (`-u` is not accepted by `-o`)
    // - `.original().stdin()`            (`-O` is not accepted by `-i`)
    // - `.arbitrary_job_status().visibility(..)`   (`-s` rejected by `-t`/`-U`)
    // - `.force().original().visibility(..)`       (`-f` + `-O` together)
    // - `.arbitrary_job_status().user(..)`         (`-s` rejected by `-t`/`-U`)
    // - `.force().original().user(..)`             (`-f` + `-O` together)
    // - `.force().arbitrary_job_status().server_id(..)` (`-s` rejected by remote `-d`)
    // - `.visibility(..).force().update()` (a second option in the `-t`/`-U` form)
    // - `.user(..).original().identity()`  (a second option in the `-t`/`-U` form)
    // - spawning `.stdin()` with a `changelist` argument (no positional arg)

    /// Dry-run checks of the assembled `p4 change` command line; no process
    /// is spawned.
    #[test]
    fn regular_form_injects_all_options() {
        let change = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .force()
            .original();

        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-s", "-f", "-O"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2022_1"))]
    fn regular_form_update_and_identity() {
        let change = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .update()
            .identity();

        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-s", "-u", "-I"]
        );
    }

    #[test]
    fn entry_points_select_form_flags() {
        let delete = Change::new("p4", GlobalOpts::new()).delete();
        assert_eq!(args_of(&delete.setup_command("p4")), ["change", "-d"]);

        let stdout = Change::new("p4", GlobalOpts::new()).stdout();
        assert_eq!(args_of(&stdout.setup_command("p4")), ["change", "-o"]);

        let stdin = Change::new("p4", GlobalOpts::new()).stdin();
        assert_eq!(args_of(&stdin.setup_command("p4")), ["change", "-i"]);

        #[cfg(not(feature = "lt2015_2"))]
        {
            let remote = Change::new("p4", GlobalOpts::new()).server_id("remote-1");
            #[cfg(feature = "lt2022_1")]
            assert_eq!(
                args_of(&remote.setup_command("p4")),
                ["change", "-d", "-f", "--server=remote-1"]
            );
            #[cfg(not(feature = "lt2022_1"))]
            assert_eq!(
                args_of(&remote.setup_command("p4")),
                ["change", "-d", "-f", "--serverid=remote-1"]
            );
        }
    }

    #[test]
    fn arbitrary_job_status_carries_into_delete_stdout_stdin() {
        // -s is accepted by the -d, -o, and -i forms.
        let delete = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .delete();
        assert_eq!(args_of(&delete.setup_command("p4")), ["change", "-d", "-s"]);

        let stdout = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .stdout();
        assert_eq!(args_of(&stdout.setup_command("p4")), ["change", "-o", "-s"]);

        let stdin = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .stdin();
        assert_eq!(args_of(&stdin.setup_command("p4")), ["change", "-i", "-s"]);
    }

    #[test]
    fn regular_to_delete_preserves_compatible_options() {
        let change = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .force()
            .original()
            .delete();

        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-d", "-s", "-f", "-O"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2022_1"))]
    fn regular_to_stdout_preserves_identity() {
        let change = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .force()
            .identity()
            .stdout();

        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-o", "-s", "-f", "-I"]
        );
    }

    #[test]
    fn regular_to_stdin_preserves_update() {
        let change = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .update()
            .stdin();

        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-i", "-s", "-u"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2015_2"))]
    fn regular_to_remote_delete_folds_force_into_builtin() {
        // `-f` selected in the edit form is folded into the remote delete
        // form's built-in `-f`.
        let change = Change::new("p4", GlobalOpts::new())
            .force()
            .server_id("remote-1");

        #[cfg(feature = "lt2022_1")]
        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-d", "-f", "--server=remote-1"]
        );
        #[cfg(not(feature = "lt2022_1"))]
        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-d", "-f", "--serverid=remote-1"]
        );
    }

    #[test]
    fn regular_visibility_maps_to_only_states() {
        // -f maps to the only-force operation
        let forced = Change::new("p4", GlobalOpts::new())
            .force()
            .visibility(Visibility::Restricted);
        assert_eq!(
            args_of(&forced.setup_command("p4")),
            ["change", "-t", "restricted", "-f"]
        );

        // -I maps to the only-identity reference
        #[cfg(not(feature = "lt2022_1"))]
        {
            let identity = Change::new("p4", GlobalOpts::new())
                .identity()
                .visibility(Visibility::Public);
            assert_eq!(
                args_of(&identity.setup_command("p4")),
                ["change", "-t", "public", "-I"]
            );
        }

        // no extra option
        let plain = Change::new("p4", GlobalOpts::new())
            .visibility(Visibility::Restricted)
            .user("maria");
        assert_eq!(
            args_of(&plain.setup_command("p4")),
            ["change", "-t", "restricted", "-U", "maria"]
        );
    }

    #[test]
    fn regular_user_maps_single_option() {
        // -f maps to the only-force operation
        let change = Change::new("p4", GlobalOpts::new())
            .force()
            .user("maria")
            .visibility(Visibility::Public);

        assert_eq!(
            args_of(&change.setup_command("p4")),
            ["change", "-t", "public", "-U", "maria", "-f"]
        );

        // any other single option (`-u`, `-O`, `-I`) maps the same way
        let original = Change::new("p4", GlobalOpts::new())
            .original()
            .user("maria");

        assert_eq!(
            args_of(&original.setup_command("p4")),
            ["change", "-U", "maria", "-O"]
        );
    }

    #[test]
    fn user_or_visibility_form_sets_values_and_selects_one_option() {
        // Both form values can be set from either entry point.
        let both = Change::new("p4", GlobalOpts::new())
            .user("maria")
            .visibility(Visibility::Public);
        assert_eq!(
            args_of(&both.setup_command("p4")),
            ["change", "-t", "public", "-U", "maria"]
        );

        // The form accepts one of `-f`, `-u`, `-O`, `-I`.
        let forced = Change::new("p4", GlobalOpts::new())
            .visibility(Visibility::Restricted)
            .force();
        assert_eq!(
            args_of(&forced.setup_command("p4")),
            ["change", "-t", "restricted", "-f"]
        );

        let updated = Change::new("p4", GlobalOpts::new()).user("maria").update();
        assert_eq!(
            args_of(&updated.setup_command("p4")),
            ["change", "-U", "maria", "-u"]
        );

        let original = Change::new("p4", GlobalOpts::new())
            .user("maria")
            .original();
        assert_eq!(
            args_of(&original.setup_command("p4")),
            ["change", "-U", "maria", "-O"]
        );

        #[cfg(not(feature = "lt2015_2"))]
        {
            let identity = Change::new("p4", GlobalOpts::new())
                .visibility(Visibility::Public)
                .identity();
            assert_eq!(
                args_of(&identity.setup_command("p4")),
                ["change", "-t", "public", "-I"]
            );
        }
    }

    #[test]
    fn options_can_be_selected_inside_forms() {
        let delete = Change::new("p4", GlobalOpts::new())
            .delete()
            .arbitrary_job_status()
            .force()
            .original();
        assert_eq!(
            args_of(&delete.setup_command("p4")),
            ["change", "-d", "-s", "-f", "-O"]
        );

        #[cfg(not(feature = "lt2022_1"))]
        {
            let stdout = Change::new("p4", GlobalOpts::new())
                .stdout()
                .arbitrary_job_status()
                .identity();
            assert_eq!(
                args_of(&stdout.setup_command("p4")),
                ["change", "-o", "-s", "-I"]
            );
        }
        #[cfg(feature = "lt2022_1")]
        {
            let stdout = Change::new("p4", GlobalOpts::new())
                .stdout()
                .arbitrary_job_status()
                .original();
            assert_eq!(
                args_of(&stdout.setup_command("p4")),
                ["change", "-o", "-s", "-O"]
            );
        }

        let stdin = Change::new("p4", GlobalOpts::new())
            .stdin()
            .arbitrary_job_status()
            .update();
        assert_eq!(
            args_of(&stdin.setup_command("p4")),
            ["change", "-i", "-s", "-u"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let change = Change::new("p4", GlobalOpts::new()).delete();

        // Only the command name without any global opts by default.
        assert_eq!(args_of(&change.setup_command("p4")), ["change", "-d"]);
    }

    // Spawn argument counts per form. Each call below mirrors what the
    // corresponding `ParameterizedSpawn` impl assembles: `setup_command`
    // injects the form flags, and the one-argument impls append the
    // changelist positional. The forms that take no positional argument
    // (Regular without a changelist, Stdout without a changelist, and
    // Stdin) are checked against `setup_command` directly.
    #[test]
    fn spawn_argument_counts_per_form() {
        // Regular: 0 or 1 changelist.
        let plain = Change::new("p4", GlobalOpts::new())
            .arbitrary_job_status()
            .force();
        assert_eq!(args_of(&plain.setup_command("p4")), ["change", "-s", "-f"]);
        let mut with_cl = plain.setup_command("p4");
        with_cl.arg("1234");
        assert_eq!(args_of(&with_cl), ["change", "-s", "-f", "1234"]);

        // DeleteMode (local): exactly 1 changelist.
        let delete = Change::new("p4", GlobalOpts::new())
            .delete()
            .arbitrary_job_status()
            .force()
            .original();
        let mut delete_cl = delete.setup_command("p4");
        delete_cl.arg("1234");
        assert_eq!(
            args_of(&delete_cl),
            ["change", "-d", "-s", "-f", "-O", "1234"]
        );

        // DeleteMode (remote): exactly 1 changelist.
        #[cfg(not(feature = "lt2015_2"))]
        {
            let remote = Change::new("p4", GlobalOpts::new()).server_id("edge-1");
            let mut remote_cl = remote.setup_command("p4");
            remote_cl.arg("1234");
            #[cfg(feature = "lt2022_1")]
            assert_eq!(
                args_of(&remote_cl),
                ["change", "-d", "-f", "--server=edge-1", "1234"]
            );
            #[cfg(not(feature = "lt2022_1"))]
            assert_eq!(
                args_of(&remote_cl),
                ["change", "-d", "-f", "--serverid=edge-1", "1234"]
            );
        }

        // Stdout: 0 or 1 changelist.
        #[cfg(not(feature = "lt2022_1"))]
        {
            let stdout = Change::new("p4", GlobalOpts::new())
                .stdout()
                .arbitrary_job_status()
                .force()
                .identity();
            assert_eq!(
                args_of(&stdout.setup_command("p4")),
                ["change", "-o", "-s", "-f", "-I"]
            );
            let mut stdout_cl = stdout.setup_command("p4");
            stdout_cl.arg("1234");
            assert_eq!(
                args_of(&stdout_cl),
                ["change", "-o", "-s", "-f", "-I", "1234"]
            );
        }
        #[cfg(feature = "lt2022_1")]
        {
            let stdout = Change::new("p4", GlobalOpts::new())
                .stdout()
                .arbitrary_job_status()
                .force()
                .original();
            assert_eq!(
                args_of(&stdout.setup_command("p4")),
                ["change", "-o", "-s", "-f", "-O"]
            );
            let mut stdout_cl = stdout.setup_command("p4");
            stdout_cl.arg("1234");
            assert_eq!(
                args_of(&stdout_cl),
                ["change", "-o", "-s", "-f", "-O", "1234"]
            );
        }

        // Stdin: no changelist; stdin is configured separately.
        let stdin = Change::new("p4", GlobalOpts::new())
            .stdin()
            .arbitrary_job_status()
            .update();
        assert_eq!(
            args_of(&stdin.setup_command("p4")),
            ["change", "-i", "-s", "-u"]
        );

        // UserOrVisibility: exactly 1 changelist.
        let uov = Change::new("p4", GlobalOpts::new())
            .visibility(Visibility::Restricted)
            .user("maria")
            .force();
        let mut uov_cl = uov.setup_command("p4");
        uov_cl.arg("1234");
        assert_eq!(
            args_of(&uov_cl),
            ["change", "-t", "restricted", "-U", "maria", "-f", "1234"]
        );
    }
}
