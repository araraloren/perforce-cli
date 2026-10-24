use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Mode structs
// ---------------------------------------------------------------------------

/// The `-v` (show previous) state of the set and increment forms.
///
/// When selected, the previous value of the counter is displayed after the
/// counter has been set or incremented.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShowPrevious;

/// The set form: `p4 [g-opts] counter [-f -v] countername value`.
///
/// Sets the value of `countername`, creating the counter if it does not
/// already exist. The `B` type parameter tracks whether `-v` has been
/// supplied: [`Unselected`] does not display the previous value;
/// [`ShowPrevious`] does.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetMode<B = Unselected> {
    force: bool,
    show_previous: B,
}

impl ExclusiveOption for SetMode<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
    }
}

impl ExclusiveOption for SetMode<ShowPrevious> {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
        command.arg("-v");
    }
}

/// The delete form: `p4 [g-opts] counter [-f] -d countername`.
///
/// Deletes `countername`, which has the same effect as setting the counter
/// to zero.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeleteMode {
    force: bool,
}

impl ExclusiveOption for DeleteMode {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
        command.arg("-d");
    }
}

/// The increment form: `p4 [g-opts] counter [-f -v] -i countername`.
///
/// Increments `countername` by one and returns the new value. The `B` type
/// parameter tracks whether `-v` has been supplied: [`Unselected`] does not
/// display the previous value; [`ShowPrevious`] does.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncrementMode<B = Unselected> {
    force: bool,
    show_previous: B,
}

impl ExclusiveOption for IncrementMode<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
        command.arg("-i");
    }
}

impl ExclusiveOption for IncrementMode<ShowPrevious> {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
        command.arg("-v");
        command.arg("-i");
    }
}

/// The multi form: `p4 [g-opts] counter [-f] -m [pair list]`.
///
/// Performs a list of operations in a single invocation. To set a counter,
/// use a name and value pair; to delete a counter, use a `-` (hyphen)
/// followed by the name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultiMode {
    force: bool,
}

impl ExclusiveOption for MultiMode {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
        command.arg("-m");
    }
}

/// The compare-and-set form: `p4 [g-opts] counter --from oldvalue --to
/// newvalue countername`.
///
/// Sets `countername` to `newvalue` only if its current value is `oldvalue`.
/// A counter that has never been set or that has been deleted cannot be set
/// using this form.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CASMode {
    old_value: String,
    new_value: String,
}

impl ExclusiveOption for CASMode {
    fn inject_args(&self, command: &mut Command) {
        command
            .arg("--from")
            .arg(&self.old_value)
            .arg("--to")
            .arg(&self.new_value);
    }
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 counter`.
///
/// Access, set, increment, or delete a persistent variable.
///
/// Counters provide long-term variable storage for scripts. Querying a
/// counter that does not exist returns zero; counter names are not stored in
/// the database until set to a nonzero value. Deleting a counter has the same
/// effect as setting it to zero. The compare-and-set form can be used as a
/// building block for higher-level tools and processes.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] counter countername
/// p4 [g-opts] counter [-f -v] countername value
/// p4 [g-opts] counter [-f] -d countername
/// p4 [g-opts] counter [-f -v] -i countername
/// p4 [g-opts] counter [-f] -m [pair list]
/// p4 [g-opts] counter --from oldvalue --to newvalue countername
/// ```
///
/// The `M` type parameter tracks the command form at compile time, starting
/// as [`Unselected`] from [`Counter::new`], which is the bare query form
/// `p4 counter countername`: spawn it with the counter name to read the
/// value. Enter another form with one of [`Self::set`], [`Self::force`],
/// [`Self::show_previous`], [`Self::delete`], [`Self::increment`],
/// [`Self::multi`], or [`Self::cas`]:
/// - [`SetMode`] (entered via [`Self::set`], [`Self::force`], or
///   [`Self::show_previous`]): set or create the counter; can transition to
///   [`DeleteMode`], [`IncrementMode`], or [`MultiMode`] preserving `-f`, and
///   `Counter::show_previous` supplies `-v` by transitioning to the
///   [`ShowPrevious`] state.
/// - [`DeleteMode`] (`-d`, entered via [`Self::delete`]): delete the counter.
/// - [`IncrementMode`] (`-i`, entered via [`Self::increment`]): increment the
///   counter by one and return the new value.
/// - [`MultiMode`] (`-m`, entered via [`Self::multi`]): perform a list of
///   operations in a single invocation.
/// - [`CASMode`] (`--from oldvalue --to newvalue`, entered via
///   [`Self::cas`]): set the counter only if its current value matches.
///
/// `-v` cannot be combined with `-d` or `-m`: [`DeleteMode`] and
/// [`MultiMode`] are only reachable from [`SetMode`] while it is in the
/// [`Unselected`] state.
#[derive(Debug, Clone, Default)]
pub struct Counter<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

impl Counter<Unselected> {
    /// Creates a new `p4 counter` command in the bare query form
    /// `p4 counter countername`.
    ///
    /// `bin` is the path to the Perforce command-line executable. In this
    /// state, spawn with the counter name via [`ParameterizedSpawn::spawn_with`]
    /// to read its value; use one of the builder methods to select another
    /// form.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Enters the set form: `p4 counter [-f -v] countername value`.
    ///
    /// Spawn the returned builder with the counter name and the value to
    /// assign. Use `Counter::force` to supply `-f`, or
    /// `Counter::show_previous` to supply `-v`.
    pub fn set(self) -> Counter<SetMode<Unselected>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SetMode::default(),
        }
    }

    /// Enters the set form with `-f` preset.
    ///
    /// `v` sets the initial state of the `-f` flag, which permits setting or
    /// deleting counters reserved for use by P4 Server. Spawn the returned
    /// builder with the counter name and the value to assign, or transition
    /// to the delete or increment form.
    pub fn force(self, v: bool) -> Counter<SetMode<Unselected>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SetMode {
                force: v,
                show_previous: Unselected,
            },
        }
    }

    /// Enters the set form with `-v` supplied, displaying the previous value
    /// after the set.
    ///
    /// Spawn the returned builder with the counter name and the value to
    /// assign, or transition to the increment form.
    pub fn show_previous(self) -> Counter<SetMode<ShowPrevious>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SetMode {
                force: false,
                show_previous: ShowPrevious,
            },
        }
    }

    /// Enters the delete form: `p4 counter [-f] -d countername`.
    ///
    /// Spawn the returned builder with the counter name to delete.
    pub fn delete(self) -> Counter<DeleteMode> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode::default(),
        }
    }

    /// Enters the increment form: `p4 counter [-f -v] -i countername`.
    ///
    /// Spawn the returned builder with the counter name to increment.
    pub fn increment(self) -> Counter<IncrementMode<Unselected>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: IncrementMode::default(),
        }
    }

    /// Enters the multi form: `p4 counter [-f] -m [pair list]`.
    ///
    /// Spawn the returned builder with the list of name/value pairs and
    /// `-`-prefixed names to delete. Use `Counter::force` on the returned
    /// builder to supply `-f`; transitioning from the set form preserves an
    /// already selected `-f`.
    pub fn multi(self) -> Counter<MultiMode> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: MultiMode::default(),
        }
    }

    /// Enters the compare-and-set form: `p4 counter --from oldvalue --to
    /// newvalue countername`.
    ///
    /// `old_value` is the value the counter must currently hold; `new_value`
    /// is the value to assign when it matches. Spawn the returned builder
    /// with the counter name.
    pub fn cas(
        self,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Counter<CASMode> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: CASMode {
                old_value: old_value.into(),
                new_value: new_value.into(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Shared accessors (available in every mode)
// ---------------------------------------------------------------------------

impl<M> Counter<M> {
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

// ---------------------------------------------------------------------------
// SetMode<Unselected>: force accessor + transitions
// ---------------------------------------------------------------------------

impl Counter<SetMode<Unselected>> {
    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Display the previous value of the specified counter after the counter
    /// has been set.
    ///
    /// Transitions this command to the [`ShowPrevious`] state, which also
    /// permits transitioning to the increment form (`-v` is valid with `-i`
    /// but not with `-d`).
    pub fn show_previous(self) -> Counter<SetMode<ShowPrevious>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SetMode {
                force: self.mode.force,
                show_previous: ShowPrevious,
            },
        }
    }

    /// Transitions to the delete form (`-d`), preserving the `-f` flag.
    ///
    /// This transition is unavailable from the [`ShowPrevious`] state because
    /// `-v` cannot be combined with `-d`.
    pub fn delete(self) -> Counter<DeleteMode> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                force: self.mode.force,
            },
        }
    }

    /// Transitions to the increment form (`-i`), preserving the `-f` flag.
    pub fn increment(self) -> Counter<IncrementMode<Unselected>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: IncrementMode {
                force: self.mode.force,
                show_previous: Unselected,
            },
        }
    }

    /// Transitions to the multi form (`-m`), preserving the `-f` flag.
    ///
    /// This transition is unavailable from the [`ShowPrevious`] state because
    /// `-v` cannot be combined with `-m`.
    pub fn multi(self) -> Counter<MultiMode> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: MultiMode {
                force: self.mode.force,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// SetMode<ShowPrevious>: force accessor + transition
// ---------------------------------------------------------------------------

impl Counter<SetMode<ShowPrevious>> {
    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }

    /// Transitions to the increment form (`-i`), preserving the `-f` and `-v`
    /// flags.
    pub fn increment(self) -> Counter<IncrementMode<ShowPrevious>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: IncrementMode {
                force: self.mode.force,
                show_previous: ShowPrevious,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// DeleteMode: force accessor
// ---------------------------------------------------------------------------

impl Counter<DeleteMode> {
    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }
}

// ---------------------------------------------------------------------------
// IncrementMode: force accessor
// ---------------------------------------------------------------------------

impl<B> Counter<IncrementMode<B>> {
    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }
}

impl Counter<IncrementMode<Unselected>> {
    /// # Description
    ///
    /// -v
    ///
    /// Display the previous value of the specified counter after the counter
    /// has been incremented.
    ///
    /// Transitions this command to the [`ShowPrevious`] state.
    pub fn show_previous(self) -> Counter<IncrementMode<ShowPrevious>> {
        Counter {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: IncrementMode {
                force: self.mode.force,
                show_previous: ShowPrevious,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// MultiMode: force accessor
// ---------------------------------------------------------------------------

impl Counter<MultiMode> {
    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set or delete counters that are reserved for use by P4 Server (listed
    /// in `p4 help counters`). Never set the change counter to a value that
    /// is lower than its current value.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }
}

// ---------------------------------------------------------------------------
// CASMode: old and new value accessors
// ---------------------------------------------------------------------------

impl Counter<CASMode> {
    /// # Description
    ///
    /// --from oldvalue
    ///
    /// The value the counter must currently hold for the new value to be
    /// assigned.
    pub fn get_old_value(&self) -> &str {
        &self.mode.old_value
    }

    /// # Description
    ///
    /// --from oldvalue
    ///
    /// The value the counter must currently hold for the new value to be
    /// assigned.
    pub fn set_old_value(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.old_value = v.into();
        self
    }

    /// # Description
    ///
    /// --from oldvalue
    ///
    /// The value the counter must currently hold for the new value to be
    /// assigned.
    pub fn old_value(mut self, v: impl Into<String>) -> Self {
        self.mode.old_value = v.into();
        self
    }

    /// # Description
    ///
    /// --to newvalue
    ///
    /// The value to assign to the counter when its current value matches
    /// `oldvalue`.
    pub fn get_new_value(&self) -> &str {
        &self.mode.new_value
    }

    /// # Description
    ///
    /// --to newvalue
    ///
    /// The value to assign to the counter when its current value matches
    /// `oldvalue`.
    pub fn set_new_value(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.new_value = v.into();
        self
    }

    /// # Description
    ///
    /// --to newvalue
    ///
    /// The value to assign to the counter when its current value matches
    /// `oldvalue`.
    pub fn new_value(mut self, v: impl Into<String>) -> Self {
        self.mode.new_value = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> SubCommand for Counter<M> {
    fn name(&self) -> &str {
        "counter"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

// Query form: exactly 1 countername.
impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Counter<Unselected> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counter countername` with the given counter name as a child
    /// process with piped standard output and error streams; use the returned
    /// [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (countername,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(countername)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Set form: countername and value.
impl<S, A, B> ParameterizedSpawn<(A, B)> for Counter<SetMode<S>>
where
    SetMode<S>: ExclusiveOption,
    A: AsRef<OsStr>,
    B: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counter [-f -v] countername value` with the given counter
    /// name and value as a child process with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (countername, value): (A, B)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(countername)
            .arg(value)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Delete form: exactly 1 countername.
impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Counter<DeleteMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counter [-f] -d countername` with the given counter name as
    /// a child process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (countername,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(countername)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Increment form: exactly 1 countername.
impl<S, I> ParameterizedSpawn<(I,)> for Counter<IncrementMode<S>>
where
    IncrementMode<S>: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counter [-f -v] -i countername` with the given counter name
    /// as a child process with piped standard output and error streams; use
    /// the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (countername,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(countername)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Multi form: the pair list.
impl<S, I> ParameterizedSpawn<(S,)> for Counter<MultiMode>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counter [-f] -m` with the given pair list as a child
    /// process with piped standard output and error streams; use the returned
    /// [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (pairs,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(pairs)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Compare-and-set form: exactly 1 countername.
impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Counter<CASMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counter --from oldvalue --to newvalue countername` with the
    /// given counter name as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (countername,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(countername)
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
        let mut command = Counter::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    // Compile-time rejection: Counter<SetMode<ShowPrevious>> has no
    // `delete()` method (-v cannot be combined with -d).
    // Compile-time rejection: Counter<CASMode> and Counter<DeleteMode> have
    // no `force()`/`show_previous()` method.

    /// Dry-run checks of the assembled `p4 counter` command line; no process
    /// is spawned.
    #[test]
    fn query_form_without_options() {
        let counter = Counter::new("p4", GlobalOpts::new());
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter"]);
    }

    #[test]
    fn set_form_from_unselected() {
        let counter = Counter::new("p4", GlobalOpts::new()).set();
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter"]);
    }

    #[test]
    fn set_form_with_force() {
        let counter = Counter::new("p4", GlobalOpts::new()).force(true);
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter", "-f"]);
    }

    #[test]
    fn set_form_with_show_previous() {
        let counter = Counter::new("p4", GlobalOpts::new()).show_previous();
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter", "-v"]);
    }

    #[test]
    fn set_form_with_force_and_show_previous() {
        let counter = Counter::new("p4", GlobalOpts::new())
            .set()
            .force(true)
            .show_previous();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-v"]
        );
    }

    #[test]
    fn delete_form_from_unselected() {
        let counter = Counter::new("p4", GlobalOpts::new()).delete();
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter", "-d"]);
    }

    #[test]
    fn delete_form_preserves_force_from_set() {
        let counter = Counter::new("p4", GlobalOpts::new()).force(true).delete();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-d"]
        );
    }

    #[test]
    fn delete_form_force_can_be_set_directly() {
        let counter = Counter::new("p4", GlobalOpts::new()).delete().force(true);
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-d"]
        );
    }

    #[test]
    fn increment_form_from_unselected() {
        let counter = Counter::new("p4", GlobalOpts::new()).increment();
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter", "-i"]);
    }

    #[test]
    fn increment_form_preserves_force_from_set() {
        let counter = Counter::new("p4", GlobalOpts::new())
            .force(true)
            .increment();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-i"]
        );
    }

    #[test]
    fn increment_form_preserves_force_and_show_previous() {
        let counter = Counter::new("p4", GlobalOpts::new())
            .show_previous()
            .force(true)
            .increment();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-v", "-i"]
        );
    }

    #[test]
    fn increment_form_show_previous_can_be_set_directly() {
        let counter = Counter::new("p4", GlobalOpts::new())
            .increment()
            .show_previous();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-v", "-i"]
        );
    }

    #[test]
    fn multi_form_without_force() {
        let counter = Counter::new("p4", GlobalOpts::new()).multi();
        assert_eq!(args_of(&counter.setup_command("p4")), ["counter", "-m"]);
    }

    #[test]
    fn multi_form_preserves_force_from_set() {
        let counter = Counter::new("p4", GlobalOpts::new()).force(true).multi();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-m"]
        );
    }

    #[test]
    fn multi_form_with_force() {
        let counter = Counter::new("p4", GlobalOpts::new()).multi().force(true);
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "-f", "-m"]
        );
    }

    #[test]
    fn cas_form_injects_from_and_to_before_the_name() {
        let counter = Counter::new("p4", GlobalOpts::new()).cas("3", "4");
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "--from", "3", "--to", "4"]
        );
    }

    #[test]
    fn cas_value_accessors_replace_values() {
        let mut counter = Counter::new("p4", GlobalOpts::new()).cas("3", "4");
        assert_eq!(counter.get_old_value(), "3");
        assert_eq!(counter.get_new_value(), "4");

        counter.set_old_value("5").set_new_value("6");
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "--from", "5", "--to", "6"]
        );

        let counter = counter.old_value("7").new_value("8");
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["counter", "--from", "7", "--to", "8"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let global_opts = GlobalOpts::new().port("localhost:1666").quiet_mode(true);

        let counter = Counter::new("p4", global_opts.clone()).delete();
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "counter", "-d"]
        );

        let counter = Counter::new("p4", global_opts.clone()).multi().force(true);
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "counter", "-f", "-m"]
        );

        let counter = Counter::new("p4", global_opts).cas("3", "4");
        assert_eq!(
            args_of(&counter.setup_command("p4")),
            [
                "-p",
                "localhost:1666",
                "-q",
                "counter",
                "--from",
                "3",
                "--to",
                "4"
            ]
        );
    }

    #[test]
    fn global_opts_accessors_replace_the_stored_options() {
        let mut counter = Counter::new("p4", GlobalOpts::new());
        counter.set_global_opts(GlobalOpts::new().port("localhost:1666"));

        assert_eq!(
            args_of(&counter.setup_command("p4")),
            ["-p", "localhost:1666", "counter"]
        );

        let counter = counter.global_opts(GlobalOpts::new().quiet_mode(true));
        assert_eq!(args_of(&counter.setup_command("p4")), ["-q", "counter"]);
    }
}
