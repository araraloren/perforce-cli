use std::{
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Option dimensions: `-f` (force), `-t` (template), `-T` (workspace type)
// ---------------------------------------------------------------------------

/// The `-f` (force) flag is set.
///
/// Entered with [`Client::force`]; valid in the [`GeneralMode`],
/// [`DeleteMode`],
#[cfg_attr(not(feature = "lt2022_1"), doc = "[`SwitchViewMode`],")]
/// and [`StdinMode`] states.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ForceSet;

impl ExclusiveOption for ForceSet {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-f");
    }
}

/// The `-t template` option is set: copy client workspace `template`'s view
/// and options into the `View:` and `Options:` field of this workspace.
///
/// Entered with [`Client::template`]; valid in the [`GeneralMode`] and
/// [`StdoutMode`] states.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateSet {
    template: String,
}

impl ExclusiveOption for TemplateSet {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-t").arg(&self.template);
    }
}

/// The `-T type` option is set: create the client workspace with a specific
/// [`WorkSpaceType`].
///
/// Entered with [`Client::workspace_type`]; valid in the [`GeneralMode`]
/// and [`StdoutMode`] states.
#[cfg(not(feature = "lt2016_2"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TypeSet {
    workspace_type: WorkSpaceType,
}

#[cfg(not(feature = "lt2016_2"))]
impl ExclusiveOption for TypeSet {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-T").arg(self.workspace_type.as_str());
    }
}

/// The type of a client workspace, set with `-T type` or the `Type:` field.
///
/// By default, clients are [`Writable`](WorkSpaceType::Writable).
#[cfg(not(feature = "lt2016_2"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WorkSpaceType {
    /// `writeable`: for general use; the have list is tracked in the
    /// central `db.have` table.
    #[default]
    Writable,
    /// `readonly`: for short lived clients used in build automation scripts
    /// that do not edit or submit files.
    Readonly,
    /// `partitioned`: for short lived clients used in build automation
    /// scripts that do edit or submit files.
    Partitioned,
    /// `partitioned-jnl`: like `partitioned`, but journaled, checkpointed,
    /// and recoverable.
    #[cfg(not(feature = "lt2024_1"))]
    PartitionedJnl,
    /// `graph`: for use with depots and repos of type graph and Git
    /// Connector.
    #[cfg(not(feature = "lt2024_1"))]
    Graph,
}

#[cfg(not(feature = "lt2016_2"))]
impl WorkSpaceType {
    fn as_str(&self) -> &'static str {
        match self {
            WorkSpaceType::Writable => "writeable",
            WorkSpaceType::Readonly => "readonly",
            WorkSpaceType::Partitioned => "partitioned",
            #[cfg(not(feature = "lt2024_1"))]
            WorkSpaceType::PartitionedJnl => "partitioned-jnl",
            #[cfg(not(feature = "lt2024_1"))]
            WorkSpaceType::Graph => "graph",
        }
    }
}

// ---------------------------------------------------------------------------
// Command modes
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2016_2",
    doc = "The default form: `p4 client [-f] [-t template] [name]`.",
    doc = "",
    doc = "Create or edit a client workspace specification. This is the initial state of [`Client`]; from here the command can transition into any of the other modes, subject to the options already set:",
    doc = "",
    doc = "- with `-f` set, [`Client::delete`] and [`Client::stdin`] remain available (both of them accept `-f`)",
    doc = "- with `-t` set, [`Client::stdout`] remains available (the `-o` form accepts it)"
)]
#[cfg_attr(
    all(feature = "lt2022_1", not(feature = "lt2016_2")),
    doc = "The default form: `p4 client [-f] [-t template] [-T type] [name]`.",
    doc = "",
    doc = "Create or edit a client workspace specification. This is the initial state of [`Client`]; from here the command can transition into any of the other modes, subject to the options already set:",
    doc = "",
    doc = "- with `-f` set, [`Client::delete`] and [`Client::stdin`] remain available (both of them accept `-f`)",
    doc = "- with `-t` / `-T` set, [`Client::stdout`] remains available (the `-o` form accepts both)"
)]
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "The default form: `p4 client [-f] [-t template] [-T type] [name]`.",
    doc = "",
    doc = "Create or edit a client workspace specification. This is the initial state of [`Client`]; from here the command can transition into any of the other modes, subject to the options already set:",
    doc = "",
    doc = "- with `-f` set, [`Client::delete`], [`Client::switch_view_stream`], [`Client::switch_view_template`], [`Client::switch_view`], and [`Client::stdin`] remain available (all of them accept `-f`)",
    doc = "- with `-t` / `-T` set, [`Client::stdout`] remains available (the `-o` form accepts both)"
)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GeneralMode;

/// How `-d -f` treats shelved changes of the deleted client workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShelvedHandling {
    /// `-Fs`: the client is deleted and the shelved changes are left
    /// intact.
    Keep,
    /// `-Fd`: the client and the shelved changes are both deleted.
    #[cfg(not(feature = "lt2024_1"))]
    Delete,
}

/// Marker: the `--serverid=X` option is set on the delete form.
#[cfg(not(feature = "lt2015_2"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerIdSet;

/// Marker: the `-Fs` option is set; the client is deleted and the shelved
/// changes are left intact.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShelvedKept;

/// Marker: the `-Fd` option is set; the client and the shelved changes
/// are both deleted.
#[cfg(not(feature = "lt2024_1"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShelvedDeleted;

/// Tracks whether the `--serverid=X` option is present on the delete form.
#[doc(hidden)]
pub trait ServerIdState {
    /// Whether the `--serverid=X` option has been selected.
    const SET: bool;
}

impl ServerIdState for Unselected {
    const SET: bool = false;
}

#[cfg(not(feature = "lt2015_2"))]
impl ServerIdState for ServerIdSet {
    const SET: bool = true;
}

/// Tracks the shelved-changes handling (`-Fs` / `-Fd`) on the delete form.
#[doc(hidden)]
pub trait ShelvedState {
    /// The selected shelved-changes handling, if any.
    fn handling() -> Option<ShelvedHandling>;

    /// Inject the `-Fs` / `-Fd` argument, if selected.
    fn inject_shelved(command: &mut Command);
}

impl ShelvedState for Unselected {
    fn handling() -> Option<ShelvedHandling> {
        None
    }

    fn inject_shelved(_: &mut Command) {}
}

impl ShelvedState for ShelvedKept {
    fn handling() -> Option<ShelvedHandling> {
        Some(ShelvedHandling::Keep)
    }

    fn inject_shelved(command: &mut Command) {
        command.arg("-Fs");
    }
}

#[cfg(not(feature = "lt2024_1"))]
impl ShelvedState for ShelvedDeleted {
    fn handling() -> Option<ShelvedHandling> {
        Some(ShelvedHandling::Delete)
    }

    fn inject_shelved(command: &mut Command) {
        command.arg("-Fd");
    }
}

/// The shelved states with which `--serverid=X` may be combined: no
/// shelved option, or `-Fs`. The documented form is
/// `p4 client -d -f --serverid=X [-Fs] name`, so `-Fd` is excluded.
#[doc(hidden)]
pub trait ServerIdCompatible: ShelvedState {}

impl ServerIdCompatible for Unselected {}
impl ServerIdCompatible for ShelvedKept {}

/// Marks the `(serverid state, shelved state, force state)` triples of the
/// delete form that match a documented command line and are therefore
/// spawnable:
///
/// - `(Unselected, Unselected, Unselected)` — `p4 client -d name`
/// - `(Unselected, Unselected, ForceSet)` — `p4 client -d -f name`
/// - `(Unselected, ShelvedKept, ForceSet)` — `p4 client -d -f -Fs name`
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "- `(Unselected, ShelvedDeleted, ForceSet)` — `p4 client -d -f -Fd name`"
)]
#[cfg_attr(
    not(feature = "lt2015_2"),
    doc = "- `(ServerIdSet, Unselected, ForceSet)` — `p4 client -d -f --serverid=X name`",
    doc = "- `(ServerIdSet, ShelvedKept, ForceSet)` — `p4 client -d -f --serverid=X -Fs name`"
)]
///
/// States where an option requiring `-f` is selected without `-f` are
/// reachable as intermediate builder states (the options may be selected
/// in any order) but are not marked here, so they cannot be spawned until
/// [`Client::force`] is called.
#[doc(hidden)]
pub trait ValidDeleteCombination {}

impl ValidDeleteCombination for (Unselected, Unselected, Unselected) {}
impl ValidDeleteCombination for (Unselected, Unselected, ForceSet) {}
impl ValidDeleteCombination for (Unselected, ShelvedKept, ForceSet) {}
#[cfg(not(feature = "lt2024_1"))]
impl ValidDeleteCombination for (Unselected, ShelvedDeleted, ForceSet) {}
#[cfg(not(feature = "lt2015_2"))]
impl ValidDeleteCombination for (ServerIdSet, Unselected, ForceSet) {}
#[cfg(not(feature = "lt2015_2"))]
impl ValidDeleteCombination for (ServerIdSet, ShelvedKept, ForceSet) {}

#[cfg_attr(
    feature = "lt2015_2",
    doc = "The delete form: `p4 client -d [-f [-Fs]] name`.",
    doc = "",
    doc = "The `S` type parameter tracks the shelved handling: [`Unselected`] or [`ShelvedKept`] (`-Fs`).",
    doc = "",
    doc = "The options may be selected in any order (for example `keep_shelved().force()` is equivalent to `force().keep_shelved()`); a state where `-Fs` is set without `-f` is an intermediate state that cannot be spawned until [`Client::force`] is called. Entered with [`Client::delete`], or directly from the [`GeneralMode`] state via [`Client::keep_shelved`]."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2015_2")),
    doc = "The delete form: `p4 client -d [-f [-Fs]] name` and `p4 client -d -f --serverid=X [-Fs] name`.",
    doc = "",
    doc = "The two type parameters encode the documented compatibility matrix at compile time:",
    doc = "",
    doc = "- `I` tracks `--serverid=X`: [`Unselected`] or [`ServerIdSet`]",
    doc = "- `S` tracks the shelved handling: [`Unselected`] or [`ShelvedKept`] (`-Fs`)",
    doc = "",
    doc = "`--serverid=X` can only be combined with `-Fs`; the illegal combinations have no reachable state. The options may be selected in any order (for example `keep_shelved().force()` is equivalent to `force().keep_shelved()`); a state where `-Fs` or `--serverid=X` is set without `-f` is an intermediate state that cannot be spawned until [`Client::force`] is called. Entered with [`Client::delete`], or directly from the [`GeneralMode`] state via [`Client::server_id`] and [`Client::keep_shelved`]."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "The delete form: `p4 client -d [-f [-Fs|-Fd]] name` and `p4 client -d -f --serverid=X [-Fs] name`.",
    doc = "",
    doc = "The two type parameters encode the documented compatibility matrix at compile time:",
    doc = "",
    doc = "- `I` tracks `--serverid=X`: [`Unselected`] or [`ServerIdSet`]",
    doc = "- `S` tracks the shelved handling: [`Unselected`], [`ShelvedKept`] (`-Fs`), or [`ShelvedDeleted`] (`-Fd`)",
    doc = "",
    doc = "`-Fs` and `-Fd` are mutually exclusive, and `--serverid=X` can only be combined with `-Fs`, never with `-Fd`; the illegal combinations have no reachable state. The options may be selected in any order (for example `delete_shelved().force()` is equivalent to `force().delete_shelved()`); a state where `-Fs`, `-Fd`, or `--serverid=X` is set without `-f` is an intermediate state that cannot be spawned until [`Client::force`] is called. Entered with [`Client::delete`], or directly from the [`GeneralMode`] state via [`Client::server_id`], [`Client::keep_shelved`], and [`Client::delete_shelved`]."
)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeleteMode<I = Unselected, S = Unselected> {
    server_id: Option<String>,
    serverid_state: PhantomData<I>,
    shelved: S,
}

#[cfg_attr(
    feature = "lt2016_2",
    doc = "The output form: `p4 client -o [-t template] [name]`."
)]
#[cfg_attr(
    not(feature = "lt2016_2"),
    doc = "The output form: `p4 client -o [-t template] [-T type] [name]`."
)]
///
/// Write the client workspace specification to standard output. Entered
/// with [`Client::stdout`];
#[cfg_attr(
    feature = "lt2016_2",
    doc = "carries the `-t` option over from [`GeneralMode`]."
)]
#[cfg_attr(
    not(feature = "lt2016_2"),
    doc = "carries the `-t` and `-T` options over from [`GeneralMode`]."
)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StdoutMode;

/// The `-o` part of the stream form: display the workspace specification
/// that would have been created for the stream.
///
/// Entered with [`Client::stdout`] in the [`StreamMode`] state; unlocks
/// the `-c change` option.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StdoutOutput {
    change: Option<String>,
}

impl ExclusiveOption for StdoutOutput {
    fn inject_args(&self, command: &mut Command) {
        if let Some(change) = &self.change {
            command.arg("-c").arg(change);
        }
        command.arg("-o");
    }
}

/// The stream form: `p4 client -S stream [[-c change] -o] [name]`.
///
/// Associate the workspace with the specified stream, which is used to
/// generate its workspace view. Entered with [`Client::stream`]; before
/// `-o` is selected, [`Client::switch_view`] can add `-s` to switch the
/// stream associated with an existing workspace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamMode<O = Unselected> {
    stream: String,
    output: O,
}

/// Switch the workspace view to a stream: `p4 client -s [-f] -S stream
/// [name]`.
///
/// Entered with [`Client::switch_view_stream`] or [`Client::switch_view`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamSwitch {
    stream: String,
}

impl ExclusiveOption for StreamSwitch {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-S").arg(&self.stream);
    }
}

/// Switch the workspace view to the view defined for another workspace:
/// `p4 client -s [-f] -t template [name]`.
///
/// Entered with [`Client::switch_view_template`] or [`Client::switch_view`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateSwitch {
    template: String,
}

impl ExclusiveOption for TemplateSwitch {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-t").arg(&self.template);
    }
}

#[cfg_attr(
    feature = "lt2022_1",
    doc = "The switch-view form: `p4 client -s (-S stream | -t template) [name]`."
)]
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "The switch-view form: `p4 client -s [-f] (-S stream | -t template) [name]`."
)]
///
/// Switching views is not allowed in a client that has opened files.
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "The `-f` option can be used with `-s` to force switching with opened files."
)]
///
/// View switching has no effect on files in a client workspace until
/// `p4 sync` is run.
///
/// Entered with [`Client::switch_view_stream`] or [`Client::switch_view`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SwitchViewMode<V> {
    view: V,
}

/// Marks the `-f` states that may be carried into the switch-view form:
/// always [`Unselected`]; [`ForceSet`] where the selected feature set
/// accepts `-f` on the `-s` form.
#[doc(hidden)]
pub trait SwitchViewForceCompatible: ExclusiveOption {}

impl SwitchViewForceCompatible for Unselected {}

#[cfg(not(feature = "lt2022_1"))]
impl SwitchViewForceCompatible for ForceSet {}

/// The input form: `p4 client -i [-f]`.
///
/// Read the client workspace specification from standard input. Entered
/// with [`Client::stdin`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StdinMode;

// ---------------------------------------------------------------------------
// Mode argument injection
// ---------------------------------------------------------------------------

/// Injects the mode-specific arguments in syntax order, given the
/// orthogonal option dimensions `F` (`-f`), `T` (`-t`), and `W` (`-T`).
pub trait ClientMode<F, T, W> {
    /// Inject the mode flag(s) and the carried options into `command`.
    fn inject_mode_args(&self, command: &mut Command, force: &F, template: &T, workspace_type: &W);
}

impl<F, T, W> ClientMode<F, T, W> for GeneralMode
where
    F: ExclusiveOption,
    T: ExclusiveOption,
    W: ExclusiveOption,
{
    fn inject_mode_args(&self, command: &mut Command, force: &F, template: &T, workspace_type: &W) {
        force.inject_args(command);
        template.inject_args(command);
        workspace_type.inject_args(command);
    }
}

impl<I, S, F> ClientMode<F, Unselected, Unselected> for DeleteMode<I, S>
where
    I: ServerIdState,
    S: ShelvedState,
    F: ExclusiveOption,
{
    fn inject_mode_args(&self, command: &mut Command, force: &F, _: &Unselected, _: &Unselected) {
        command.arg("-d");
        force.inject_args(command);

        if I::SET
            && let Some(server_id) = &self.server_id
        {
            command.arg(format!("--serverid={server_id}"));
        }

        S::inject_shelved(command);
    }
}

impl<T, W> ClientMode<Unselected, T, W> for StdoutMode
where
    T: ExclusiveOption,
    W: ExclusiveOption,
{
    fn inject_mode_args(
        &self,
        command: &mut Command,
        _: &Unselected,
        template: &T,
        workspace_type: &W,
    ) {
        command.arg("-o");
        template.inject_args(command);
        workspace_type.inject_args(command);
    }
}

impl<O> ClientMode<Unselected, Unselected, Unselected> for StreamMode<O>
where
    O: ExclusiveOption,
{
    fn inject_mode_args(
        &self,
        command: &mut Command,
        _: &Unselected,
        _: &Unselected,
        _: &Unselected,
    ) {
        command.arg("-S").arg(&self.stream);
        self.output.inject_args(command);
    }
}

impl<F> ClientMode<F, Unselected, Unselected> for SwitchViewMode<StreamSwitch>
where
    F: ExclusiveOption,
{
    fn inject_mode_args(&self, command: &mut Command, force: &F, _: &Unselected, _: &Unselected) {
        command.arg("-s");
        force.inject_args(command);
        self.view.inject_args(command);
    }
}

impl<F> ClientMode<F, Unselected, Unselected> for SwitchViewMode<TemplateSwitch>
where
    F: ExclusiveOption,
{
    fn inject_mode_args(&self, command: &mut Command, force: &F, _: &Unselected, _: &Unselected) {
        command.arg("-s");
        force.inject_args(command);
        self.view.inject_args(command);
    }
}

impl<F> ClientMode<F, Unselected, Unselected> for StdinMode
where
    F: ExclusiveOption,
{
    fn inject_mode_args(&self, command: &mut Command, force: &F, _: &Unselected, _: &Unselected) {
        command.arg("-i");
        force.inject_args(command);
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2015_2",
    doc = "```text",
    doc = "p4 client [-f] [-t template] [clientname]",
    doc = "p4 client -o [-t template] [clientname]",
    doc = "p4 client -d [-f [-Fs]] clientname",
    doc = "p4 client -s [-S stream | -t clientname] clientname",
    doc = "p4 client -S stream [[-c change] -o] [clientname]",
    doc = "p4 client -i [-f]",
    doc = "```"
)]
#[cfg_attr(
    all(feature = "lt2016_2", not(feature = "lt2015_2")),
    doc = "```text",
    doc = "p4 client [-f] [-t template] [clientname]",
    doc = "p4 client -o [-t template] [clientname]",
    doc = "p4 client -d [-f [-Fs]] clientname",
    doc = "p4 client -s [-S stream | -t clientname] clientname",
    doc = "p4 client -S stream [[-c change] -o] [clientname]",
    doc = "p4 client -i [-f]",
    doc = "p4 client -d -f --serverid=serverid [-Fs]",
    doc = "```"
)]
#[cfg_attr(
    all(feature = "lt2022_1", not(feature = "lt2016_2")),
    doc = "```text",
    doc = "p4 client [-f] [-t template] [-T type] [clientname]",
    doc = "p4 client -o [-t template] [-T type] [clientname]",
    doc = "p4 client -d [-f [-Fs]] clientname",
    doc = "p4 client -s [-S stream | -t clientname] clientname",
    doc = "p4 client -S stream [[-c change] -o] [clientname]",
    doc = "p4 client -i [-f]",
    doc = "p4 client -d -f --serverid=serverid [-Fs]",
    doc = "```"
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2022_1")),
    doc = "```text",
    doc = "p4 client [-f] [-t template] [-T type] [name]",
    doc = "p4 client -d [-f [-Fs]] name",
    doc = "p4 client -o [-t template] [-T type] [name]",
    doc = "p4 client -S stream [[-c change] -o] [name]",
    doc = "p4 client -s [-f] -S stream [name]",
    doc = "p4 client -s [-f] -t template [name]",
    doc = "p4 client -i [-f]",
    doc = "p4 client -d -f --serverid=X [-Fs] name",
    doc = "```"
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "```text",
    doc = "p4 client [-f] [-t template] [-T type] [name]",
    doc = "p4 client -d [-f [-Fs|-Fd]] name",
    doc = "p4 client -o [-t template] [-T type] [name]",
    doc = "p4 client -S stream [[-c change] -o] [name]",
    doc = "p4 client -s [-f] -S stream [name]",
    doc = "p4 client -s [-f] -t template [name]",
    doc = "p4 client -i [-f]",
    doc = "p4 client -d -f --serverid=X [-Fs] name",
    doc = "```"
)]
///
/// Create or edit a client workspace specification and its view.
///
#[cfg_attr(
    feature = "lt2016_2",
    doc = "The type parameters track the mode `M` and the orthogonal option dimensions `F` (`-f`) and `T` (`-t template`) at compile time, so a transition is only available when every option already set is valid in the target form:",
    doc = "",
    doc = "- [`Self::delete`] requires `-t` to be unset and ends in the terminal [`DeleteMode`] state, from which no further transitions are possible",
    doc = "- [`Self::stdout`] requires `-f` to be unset; `-t` is carried into the [`StdoutMode`] state",
    doc = "- [`Self::stream`] requires `-f` / `-t` to be unset",
    doc = "- [`Self::switch_view_stream`] / [`Self::switch_view_template`] / [`Self::switch_view`] switch the view: for the stream form `-t` must be unset, while the template form requires `-t` to be set",
    doc = "- the `-s -S stream` form has a second entry path: select the stream with [`Self::stream`] and then [`Self::switch_view`] (only before `-o` is selected)",
    doc = "- [`Self::stdin`] requires `-t` to be unset; `-f` is carried into the [`StdinMode`] state"
)]
#[cfg_attr(
    all(feature = "lt2022_1", not(feature = "lt2016_2")),
    doc = "The type parameters track the mode `M` and the orthogonal option dimensions `F` (`-f`), `T` (`-t template`), and `W` (`-T type`) at compile time, so a transition is only available when every option already set is valid in the target form:",
    doc = "",
    doc = "- [`Self::delete`] requires `-t` / `-T` to be unset and ends in the terminal [`DeleteMode`] state, from which no further transitions are possible",
    doc = "- [`Self::stdout`] requires `-f` to be unset; `-t` / `-T` are carried into the [`StdoutMode`] state",
    doc = "- [`Self::stream`] requires `-f` / `-t` / `-T` to be unset",
    doc = "- [`Self::switch_view_stream`] / [`Self::switch_view_template`] / [`Self::switch_view`] require `-T` to be unset; for the stream form `-t` must also be unset, while the template form requires `-t` to be set",
    doc = "- the `-s -S stream` form has a second entry path: select the stream with [`Self::stream`] and then [`Self::switch_view`] (only before `-o` is selected)",
    doc = "- [`Self::stdin`] requires `-t` / `-T` to be unset; `-f` is carried into the [`StdinMode`] state"
)]
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "The type parameters track the mode `M` and the orthogonal option dimensions `F` (`-f`), `T` (`-t template`), and `W` (`-T type`) at compile time, so a transition is only available when every option already set is valid in the target form:",
    doc = "",
    doc = "- [`Self::delete`] requires `-t` / `-T` to be unset and ends in the terminal [`DeleteMode`] state, from which no further transitions are possible",
    doc = "- [`Self::stdout`] requires `-f` to be unset; `-t` / `-T` are carried into the [`StdoutMode`] state",
    doc = "- [`Self::stream`] requires `-f` / `-t` / `-T` to be unset",
    doc = "- [`Self::switch_view_stream`] / [`Self::switch_view_template`] / [`Self::switch_view`] require `-T` to be unset; for the stream form `-t` must also be unset, while the template form requires `-t` to be set. `-f` is carried into the [`SwitchViewMode`] state",
    doc = "- the `-s -S stream` form has a second entry path: select the stream with [`Self::stream`] and then [`Self::switch_view`] (only before `-o` is selected); `-f` is added afterwards if needed",
    doc = "- [`Self::stdin`] requires `-t` / `-T` to be unset; `-f` is carried into the [`StdinMode`] state"
)]
#[derive(Debug, Clone, Default)]
pub struct Client<M = GeneralMode, F = Unselected, T = Unselected, W = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,

    force: F,

    template: T,

    workspace_type: W,
}

impl<M, F, T, W> SubCommand for Client<M, F, T, W>
where
    M: ClientMode<F, T, W>,
{
    fn name(&self) -> &str {
        "client"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode
            .inject_mode_args(command, &self.force, &self.template, &self.workspace_type);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning: GeneralMode — `p4 client [-f] [-t template] [-T type] [name]`
// ---------------------------------------------------------------------------

impl<F, T, W> ParameterizedSpawn<()> for Client<GeneralMode, F, T, W>
where
    F: ExclusiveOption,
    T: ExclusiveOption,
    W: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client` for the workspace named by `P4CLIENT` as a child
    /// process; the editor configured by `P4EDITOR` is invoked on the
    /// client workspace specification form.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<F, T, W, I> ParameterizedSpawn<(I,)> for Client<GeneralMode, F, T, W>
where
    F: ExclusiveOption,
    T: ExclusiveOption,
    W: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client` for the named workspace as a child process; the
    /// editor configured by `P4EDITOR` is invoked on the client workspace
    /// specification form.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (name,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// Spawning: StdoutMode — `p4 client -o [-t template] [-T type] [name]`
// ---------------------------------------------------------------------------

impl<T, W> ParameterizedSpawn<()> for Client<StdoutMode, Unselected, T, W>
where
    T: ExclusiveOption,
    W: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -o` for the workspace named by `P4CLIENT` as a
    /// child process, writing the client workspace specification to
    /// standard output.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<T, W, I> ParameterizedSpawn<(I,)> for Client<StdoutMode, Unselected, T, W>
where
    T: ExclusiveOption,
    W: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -o` for the named workspace as a child process,
    /// writing the client workspace specification to standard output.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (name,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// Spawning: DeleteMode — `p4 client -d [-f [-Fs|-Fd]] name`
// ---------------------------------------------------------------------------

impl<I, S, F, A> ParameterizedSpawn<(A,)> for Client<DeleteMode<I, S>, F, Unselected, Unselected>
where
    I: ServerIdState,
    S: ShelvedState,
    F: ExclusiveOption,
    (I, S, F): ValidDeleteCombination,
    A: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -d` for the named workspace as a child process.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (name,): (A,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// Spawning: StreamMode — `p4 client -S stream [[-c change] -o] [name]`
// ---------------------------------------------------------------------------

impl<O> ParameterizedSpawn<()> for Client<StreamMode<O>, Unselected, Unselected, Unselected>
where
    O: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -S stream` for the workspace named by `P4CLIENT`
    /// as a child process, associating the workspace with the stream.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<O, I> ParameterizedSpawn<(I,)> for Client<StreamMode<O>, Unselected, Unselected, Unselected>
where
    O: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -S stream` for the named workspace as a child
    /// process, associating the workspace with the stream.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (name,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// Spawning: SwitchViewMode — `p4 client -s [-f] (-S stream | -t template)
// [name]`
// ---------------------------------------------------------------------------

impl<V, F> ParameterizedSpawn<()> for Client<SwitchViewMode<V>, F, Unselected, Unselected>
where
    SwitchViewMode<V>: ClientMode<F, Unselected, Unselected>,
    F: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -s` for the workspace named by `P4CLIENT` as a
    /// child process, switching its view.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<V, F, I> ParameterizedSpawn<(I,)> for Client<SwitchViewMode<V>, F, Unselected, Unselected>
where
    SwitchViewMode<V>: ClientMode<F, Unselected, Unselected>,
    F: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -s` for the named workspace as a child process,
    /// switching its view.
    ///
    /// The child process has its standard output and error streams piped;
    /// use the returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (name,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// Spawning: StdinMode — `p4 client -i [-f]`
// ---------------------------------------------------------------------------

impl<F> ParameterizedSpawn<()> for Client<StdinMode, F, Unselected, Unselected>
where
    F: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -i` with a piped standard input stream (and piped
    /// standard output and error streams); write the client workspace
    /// specification to the returned [`Child`]'s stdin handle.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<F> ParameterizedSpawn<(Stdio,)> for Client<StdinMode, F, Unselected, Unselected>
where
    F: ExclusiveOption,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 client -i` with the given standard input configuration
    /// and piped standard output and error streams; use the returned
    /// [`Child`] handle to wait for it or interact with it.
    ///
    /// Pass `Stdio::piped()` to obtain a writable `child.stdin` handle and
    /// feed the client workspace specification into it; pass
    /// `Stdio::inherit()` to read the specification from the spawning
    /// process's own standard input.
    fn spawn_with(&mut self, (stdio,): (Stdio,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(stdio)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// General accessors and option builders
// ---------------------------------------------------------------------------

impl Client<GeneralMode, Unselected, Unselected, Unselected> {
    /// Creates a new `p4 client` command in the [`GeneralMode`] state.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: GeneralMode,
            force: Unselected,
            template: Unselected,
            workspace_type: Unselected,
        }
    }
}

impl<M, F, T, W> Client<M, F, T, W> {
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

    /// Get the global options.
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// Set the global options.
    pub fn set_global_opts(&mut self, v: GlobalOpts) -> &mut Self {
        self.global_opts = v;
        self
    }

    /// Set the global options.
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.set_global_opts(v);
        self
    }
}

// `-f` is valid in the GeneralMode, DeleteMode, SwitchViewMode, and
// StdinMode states.

impl<T, W> Client<GeneralMode, Unselected, T, W> {
    /// # Description
    ///
    /// -f
    ///
    /// Allows the last modification date, which is normally read-only, to
    /// be set. Administrators can use the `-f` option to delete or modify
    /// locked workspaces owned by other users. Use of this option requires
    /// `admin` access granted by `p4 protect`.
    pub fn force(self) -> Client<GeneralMode, ForceSet, T, W> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: ForceSet,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

impl<I, S> Client<DeleteMode<I, S>, Unselected, Unselected, Unselected>
where
    I: ServerIdState,
    S: ShelvedState,
{
    /// # Description
    ///
    /// -f
    ///
    /// The combination of `-d` with `-f` allows the P4 Server
    /// administrator to delete a locked workspace that is owned by other
    /// user. This also reverts any files opened on the workspace.
    ///
    #[cfg_attr(
        feature = "lt2015_2",
        doc = "`-f` is required by the `-Fs` option, but it may be selected before or after it; calling `force` on a state carrying it completes the documented form and makes the command spawnable."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_2")),
        doc = "`-f` is required by the `-Fs` and `--serverid=X` options, but it may be selected before or after them; calling `force` on a state carrying those options completes the documented form and makes the command spawnable."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "`-f` is required by the `-Fs`, `-Fd`, and `--serverid=X` options, but it may be selected before or after them; calling `force` on a state carrying those options completes the documented form and makes the command spawnable."
    )]
    pub fn force(self) -> Client<DeleteMode<I, S>, ForceSet, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: ForceSet,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

#[cfg(not(feature = "lt2022_1"))]
impl<V> Client<SwitchViewMode<V>, Unselected, Unselected, Unselected> {
    /// # Description
    ///
    /// -f
    ///
    /// The `-f` option can be used with `-s` to force switching with
    /// opened files.
    pub fn force(self) -> Client<SwitchViewMode<V>, ForceSet, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: ForceSet,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

impl Client<StdinMode, Unselected, Unselected, Unselected> {
    /// # Description
    ///
    /// -f
    ///
    /// Allows the last modification date, which is normally read-only, to
    /// be set. Use of this option requires `admin` access granted by
    /// `p4 protect`.
    pub fn force(self) -> Client<StdinMode, ForceSet, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: ForceSet,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// `-t` is valid in the GeneralMode and StdoutMode states.

impl<F, W> Client<GeneralMode, F, Unselected, W> {
    /// # Description
    ///
    /// -t name
    ///
    /// Copy client workspace `template`'s view and options into the
    /// `View:` and `Options:` field of this workspace. If you specify a
    /// default client template using the `template.client` configurable,
    /// you do not have to specify this option.
    pub fn template(self, template: impl Into<String>) -> Client<GeneralMode, F, TemplateSet, W> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: self.force,
            template: TemplateSet {
                template: template.into(),
            },
            workspace_type: self.workspace_type,
        }
    }
}

impl<W> Client<StdoutMode, Unselected, Unselected, W> {
    /// # Description
    ///
    /// -t name
    ///
    /// Copy client workspace `template`'s view and options into the
    /// `View:` and `Options:` field of this workspace.
    pub fn template(
        self,
        template: impl Into<String>,
    ) -> Client<StdoutMode, Unselected, TemplateSet, W> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: self.force,
            template: TemplateSet {
                template: template.into(),
            },
            workspace_type: self.workspace_type,
        }
    }
}

// `-T` is valid in the GeneralMode and StdoutMode states.

#[cfg(not(feature = "lt2016_2"))]
impl<F, T> Client<GeneralMode, F, T, Unselected> {
    /// # Description
    ///
    /// -T type
    ///
    /// By default, clients are `writeable`. When you create a client
    /// workspace, use `-T type` to set a specific type.
    pub fn workspace_type(
        self,
        workspace_type: WorkSpaceType,
    ) -> Client<GeneralMode, F, T, TypeSet> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: self.force,
            template: self.template,
            workspace_type: TypeSet { workspace_type },
        }
    }
}

#[cfg(not(feature = "lt2016_2"))]
impl<T> Client<StdoutMode, Unselected, T, Unselected> {
    /// # Description
    ///
    /// -T type
    ///
    /// By default, clients are `writeable`. When you create a client
    /// workspace, use `-T type` to set a specific type.
    pub fn workspace_type(
        self,
        workspace_type: WorkSpaceType,
    ) -> Client<StdoutMode, Unselected, T, TypeSet> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: self.mode,
            force: self.force,
            template: self.template,
            workspace_type: TypeSet { workspace_type },
        }
    }
}

// Option accessors for the set states.

impl<M, F, W> Client<M, F, TemplateSet, W> {
    /// Get the template client workspace name set with `-t`.
    pub fn get_template(&self) -> &str {
        &self.template.template
    }

    /// # Description
    ///
    /// -t name
    ///
    /// Copy client workspace `template`'s view and options into the
    /// `View:` and `Options:` field of this workspace.
    pub fn set_template(&mut self, template: impl Into<String>) -> &mut Self {
        self.template.template = template.into();
        self
    }
}

#[cfg(not(feature = "lt2016_2"))]
impl<M, F, T> Client<M, F, T, TypeSet> {
    /// Get the client workspace type set with `-T`.
    pub fn get_workspace_type(&self) -> WorkSpaceType {
        self.workspace_type.workspace_type
    }

    /// # Description
    ///
    /// -T type
    ///
    /// By default, clients are `writeable`. When you create a client
    /// workspace, use `-T type` to set a specific type.
    pub fn set_workspace_type(&mut self, workspace_type: WorkSpaceType) -> &mut Self {
        self.workspace_type.workspace_type = workspace_type;
        self
    }
}

// ---------------------------------------------------------------------------
// Mode transitions out of GeneralMode
// ---------------------------------------------------------------------------

impl<F> Client<GeneralMode, F, Unselected, Unselected> {
    /// # Description
    ///
    /// -d name
    ///
    /// Delete the specified client workspace whether or not the workspace
    /// is owned by the user. The workspace must be unlocked and must have
    /// no opened files or pending changes.
    ///
    /// This transitions into the terminal [`DeleteMode`] state, from which
    /// no further mode transitions are possible.
    pub fn delete(self) -> Client<DeleteMode<Unselected, Unselected>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: None,
                serverid_state: PhantomData,
                shelved: Unselected,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }

    /// # Description
    ///
    /// -i
    ///
    /// Read the client workspace specification from standard input.
    ///
    /// This transitions into the [`StdinMode`] state, carrying the `-f`
    /// option.
    pub fn stdin(self) -> Client<StdinMode, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdinMode,
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// The `-f` states accepted on the switch-view form are tracked by
// [`SwitchViewForceCompatible`].

impl<F> Client<GeneralMode, F, Unselected, Unselected>
where
    F: SwitchViewForceCompatible,
{
    /// # Description
    ///
    /// -s -S stream
    ///
    /// Switch the workspace view to the specified stream.
    ///
    /// This transitions into the [`SwitchViewMode`] state.
    #[cfg_attr(
        not(feature = "lt2022_1"),
        doc = "",
        doc = "The `-f` option may be selected before this transition, or added afterwards with [`Client::force`]."
    )]
    pub fn switch_view_stream(
        self,
        stream: impl Into<String>,
    ) -> Client<SwitchViewMode<StreamSwitch>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SwitchViewMode {
                view: StreamSwitch {
                    stream: stream.into(),
                },
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }

    /// # Description
    ///
    /// -s -t template
    ///
    /// Switch the workspace view to the view defined for another
    /// workspace.
    ///
    /// This transitions into the [`SwitchViewMode`] state.
    #[cfg_attr(
        not(feature = "lt2022_1"),
        doc = "",
        doc = "The `-f` option may be selected before this transition, or added afterwards with [`Client::force`]."
    )]
    pub fn switch_view_template(
        self,
        template: impl Into<String>,
    ) -> Client<SwitchViewMode<TemplateSwitch>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SwitchViewMode {
                view: TemplateSwitch {
                    template: template.into(),
                },
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

impl<F> Client<GeneralMode, F, TemplateSet, Unselected>
where
    F: SwitchViewForceCompatible,
{
    /// # Description
    ///
    /// -s -t template
    ///
    /// Switch the workspace view to the view defined for the template
    /// workspace already selected with [`Client::template`].
    ///
    /// This transitions into the [`SwitchViewMode`] state.
    #[cfg_attr(
        not(feature = "lt2022_1"),
        doc = "",
        doc = "The `-f` option may be selected before this transition, or added afterwards with [`Client::force`]."
    )]
    pub fn switch_view(self) -> Client<SwitchViewMode<TemplateSwitch>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SwitchViewMode {
                view: TemplateSwitch {
                    template: self.template.template,
                },
            },
            force: self.force,
            template: Unselected,
            workspace_type: self.workspace_type,
        }
    }
}

impl<T, W> Client<GeneralMode, Unselected, T, W> {
    /// # Description
    ///
    /// -o
    ///
    /// Write the client workspace specification to standard output.
    ///
    /// This transitions into the [`StdoutMode`] state, carrying the `-t`
    /// and `-T` options.
    pub fn stdout(self) -> Client<StdoutMode, Unselected, T, W> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdoutMode,
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

impl Client<GeneralMode, Unselected, Unselected, Unselected> {
    /// # Description
    ///
    /// -S stream
    ///
    /// Associates the workspace with the specified stream, which is used
    /// to generate its workspace view.
    ///
    /// This transitions into the [`StreamMode`] state.
    pub fn stream(
        self,
        stream: impl Into<String>,
    ) -> Client<StreamMode<Unselected>, Unselected, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StreamMode {
                stream: stream.into(),
                output: Unselected,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// One-shot transitions from `GeneralMode` into a delete form. Since
// `--serverid`, `-Fs`, and `-Fd` only occur on a `-d -f` form, selecting
// any of them implies `-d` and transitions directly into the
// [`DeleteMode`] state, carrying along `-f` if it was already selected.
// `-f` may also be selected afterwards via [`Client::force`]; until then
// the state is an intermediate, non-spawnable state.

impl<F> Client<GeneralMode, F, Unselected, Unselected>
where
    F: ExclusiveOption,
{
    /// # Description
    ///
    /// -d [-f] --serverid=X
    ///
    /// If you try to forcibly delete a client bound to another server, you
    /// need to specify the `--serverid` option and specify the server id
    /// of the other server. This ensures that you do not accidentally
    /// delete the client believing it to be connected to your own server.
    /// This variant of the `p4 client` command must be issued directly to
    /// the commit server.
    ///
    /// Selecting this option implies `-d`; `-f` may be selected before or
    /// after it. The resulting command is spawnable only once `-f` is set.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn server_id(
        self,
        server_id: impl Into<String>,
    ) -> Client<DeleteMode<ServerIdSet, Unselected>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: Some(server_id.into()),
                serverid_state: PhantomData,
                shelved: Unselected,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }

    /// # Description
    ///
    /// -d [-f] -Fs
    ///
    /// Allows the deletion with `-d` of a client even when that client
    /// contains shelved changes. The client is deleted and the shelved
    /// changes are left intact. (You must use the `-f` option with the
    /// `-Fs` option.)
    ///
    /// Selecting this option implies `-d`; `-f` may be selected before or
    /// after it. The resulting command is spawnable only once `-f` is set.
    pub fn keep_shelved(
        self,
    ) -> Client<DeleteMode<Unselected, ShelvedKept>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: None,
                serverid_state: PhantomData,
                shelved: ShelvedKept,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }

    /// # Description
    ///
    /// -d [-f] -Fd
    ///
    /// Allows the deletion with `-d` of a client even when that client
    /// contains shelved changes. The client and the shelved changes are
    /// both deleted. (You must use the `-f` option with the `-Fd`
    /// option.)
    ///
    /// Selecting this option implies `-d`; `-f` may be selected before or
    /// after it. The resulting command is spawnable only once `-f` is set.
    #[cfg(not(feature = "lt2024_1"))]
    pub fn delete_shelved(
        self,
    ) -> Client<DeleteMode<Unselected, ShelvedDeleted>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: None,
                serverid_state: PhantomData,
                shelved: ShelvedDeleted,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// ---------------------------------------------------------------------------
// DeleteMode options
// ---------------------------------------------------------------------------

// The `--serverid=X` transition: available when it has not been selected
// yet, and only in shelved states compatible with the documented form
// `p4 client -d -f --serverid=X [-Fs] name` (never after `-Fd`).

#[cfg(not(feature = "lt2015_2"))]
impl<S, F> Client<DeleteMode<Unselected, S>, F, Unselected, Unselected>
where
    S: ServerIdCompatible,
    F: ExclusiveOption,
{
    /// # Description
    ///
    /// --serverid=X
    ///
    /// If you try to forcibly delete a client bound to another server, you
    /// need to specify the `--serverid` option and specify the server id
    /// of the other server. This ensures that you do not accidentally
    /// delete the client believing it to be connected to your own server.
    /// This variant of the `p4 client` command must be issued directly to
    /// the commit server.
    ///
    /// Only the `-Fs` shelved handling can be combined with `--serverid`;
    /// this transition is not available once `-Fd` has been selected. The
    /// required `-f` may be selected before or afterwards.
    pub fn server_id(
        self,
        server_id: impl Into<String>,
    ) -> Client<DeleteMode<ServerIdSet, S>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: Some(server_id.into()),
                serverid_state: PhantomData,
                shelved: self.mode.shelved,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// The `--serverid=X` value is always present in this state.

#[cfg(not(feature = "lt2015_2"))]
impl<S, F> Client<DeleteMode<ServerIdSet, S>, F, Unselected, Unselected>
where
    S: ServerIdCompatible,
    F: ExclusiveOption,
{
    /// Get the server ID set with `--serverid=X`.
    pub fn get_server_id(&self) -> &str {
        self.mode.server_id.as_deref().expect("server ID is set")
    }

    /// # Description
    ///
    /// --serverid=X
    ///
    /// Update the server ID of the other server to which the deleted
    /// client workspace is bound.
    pub fn set_server_id(&mut self, server_id: impl Into<String>) -> &mut Self {
        self.mode.server_id = Some(server_id.into());
        self
    }
}

// Shelved handling getters are available in every delete state.

impl<I, S, F> Client<DeleteMode<I, S>, F, Unselected, Unselected>
where
    I: ServerIdState,
    S: ShelvedState,
    F: ExclusiveOption,
{
    /// Get how shelved changes of the deleted client workspace are
    /// handled.
    pub fn get_shelved(&self) -> Option<ShelvedHandling> {
        S::handling()
    }
}

// The `-Fs` transition: available when neither `-Fs` nor `-Fd` is set,
// regardless of whether `--serverid=X` is set (the documented
// `--serverid=X [-Fs]` form accepts it).

impl<I, F> Client<DeleteMode<I, Unselected>, F, Unselected, Unselected>
where
    I: ServerIdState,
    F: ExclusiveOption,
{
    /// # Description
    ///
    /// -Fs
    ///
    /// Allows the deletion with `-d` of a client even when that client
    /// contains shelved changes. The client is deleted and the shelved
    /// changes are left intact. (You must use the `-f` option with the
    /// `-Fs` option.)
    ///
    /// After this transition neither `-Fs` nor `-Fd` can be selected
    /// again; the required `-f` may be selected before or afterwards.
    pub fn keep_shelved(self) -> Client<DeleteMode<I, ShelvedKept>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: self.mode.server_id,
                serverid_state: PhantomData,
                shelved: ShelvedKept,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// The `-Fd` transition: only available without `--serverid=X` and without
// a previously selected shelved handling.

#[cfg(not(feature = "lt2024_1"))]
impl<F> Client<DeleteMode<Unselected, Unselected>, F, Unselected, Unselected>
where
    F: ExclusiveOption,
{
    /// # Description
    ///
    /// -Fd
    ///
    /// Allows the deletion with `-d` of a client even when that client
    /// contains shelved changes. The client and the shelved changes are
    /// both deleted. (You must use the `-f` option with the `-Fd`
    /// option.)
    ///
    /// This is the terminal shelved state: it cannot be combined with
    /// `--serverid=X`, and neither `-Fs` nor `-Fd` can be selected
    /// again; the required `-f` may be selected before or afterwards.
    pub fn delete_shelved(
        self,
    ) -> Client<DeleteMode<Unselected, ShelvedDeleted>, F, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                server_id: None,
                serverid_state: PhantomData,
                shelved: ShelvedDeleted,
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

// ---------------------------------------------------------------------------
// StreamMode options
// ---------------------------------------------------------------------------

impl<O> Client<StreamMode<O>, Unselected, Unselected, Unselected> {
    /// Get the stream set with `-S`.
    pub fn get_stream(&self) -> &str {
        &self.mode.stream
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Associates the workspace with the specified stream, which is used
    /// to generate its workspace view.
    pub fn set_stream(&mut self, stream: impl Into<String>) -> &mut Self {
        self.mode.stream = stream.into();
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Associates the workspace with the specified stream, which is used
    /// to generate its workspace view.
    pub fn stream(mut self, stream: impl Into<String>) -> Self {
        self.set_stream(stream);
        self
    }
}

// Adding `-s` to a selected `-S stream` reaches the switch form
// `p4 client -s [-f] -S stream [name]`; this is only possible before the
// `-o` option is selected, since the `-S stream -o` form carries no `-s`.

impl Client<StreamMode<Unselected>, Unselected, Unselected, Unselected> {
    /// # Description
    ///
    /// -s -S stream
    ///
    /// Switch the workspace view to the stream already selected with
    /// [`Client::stream`], changing the stream associated with the
    /// workspace.
    ///
    /// This transitions into the [`SwitchViewMode`] state; the
    /// intermediate `-S stream` form does not accept `-f`.
    #[cfg_attr(
        not(feature = "lt2022_1"),
        doc = "The optional `-f` can be added after this transition with [`Client::force`]."
    )]
    pub fn switch_view(
        self,
    ) -> Client<SwitchViewMode<StreamSwitch>, Unselected, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: SwitchViewMode {
                view: StreamSwitch {
                    stream: self.mode.stream,
                },
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }

    /// # Description
    ///
    /// -o
    ///
    /// When used with `-S stream`, displays the workspace specification
    /// that would have been created for the stream.
    ///
    /// Unlocks the [`Client::change`] option.
    pub fn stdout(self) -> Client<StreamMode<StdoutOutput>, Unselected, Unselected, Unselected> {
        Client {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StreamMode {
                stream: self.mode.stream,
                output: StdoutOutput { change: None },
            },
            force: self.force,
            template: self.template,
            workspace_type: self.workspace_type,
        }
    }
}

impl Client<StreamMode<StdoutOutput>, Unselected, Unselected, Unselected> {
    /// Get the changelist number set with `-c`.
    pub fn get_change(&self) -> Option<&str> {
        self.mode.output.change.as_deref()
    }

    /// # Description
    ///
    /// -c change
    ///
    /// When used with `-S stream` and `-o`, displays the workspace
    /// specification that would have been created for a stream at the
    /// moment the change was submitted.
    pub fn set_change(&mut self, change: impl Into<String>) -> &mut Self {
        self.mode.output.change = Some(change.into());
        self
    }

    /// # Description
    ///
    /// -c change
    ///
    /// When used with `-S stream` and `-o`, displays the workspace
    /// specification that would have been created for a stream at the
    /// moment the change was submitted.
    pub fn change(mut self, change: impl Into<String>) -> Self {
        self.set_change(change);
        self
    }
}

// ---------------------------------------------------------------------------
// SwitchViewMode accessors
// ---------------------------------------------------------------------------

impl<F> Client<SwitchViewMode<StreamSwitch>, F, Unselected, Unselected> {
    /// Get the stream the workspace view is switched to.
    pub fn get_stream(&self) -> &str {
        &self.mode.view.stream
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Switch the workspace view to the specified stream.
    pub fn set_stream(&mut self, stream: impl Into<String>) -> &mut Self {
        self.mode.view.stream = stream.into();
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// Switch the workspace view to the specified stream.
    pub fn stream(mut self, stream: impl Into<String>) -> Self {
        self.set_stream(stream);
        self
    }
}

impl<F> Client<SwitchViewMode<TemplateSwitch>, F, Unselected, Unselected> {
    /// Get the template workspace whose view is switched to.
    pub fn get_template(&self) -> &str {
        &self.mode.view.template
    }

    /// # Description
    ///
    /// -t template
    ///
    /// Switch the workspace view to the view defined for another
    /// workspace.
    pub fn set_template(&mut self, template: impl Into<String>) -> &mut Self {
        self.mode.view.template = template.into();
        self
    }

    /// # Description
    ///
    /// -t template
    ///
    /// Switch the workspace view to the view defined for another
    /// workspace.
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.set_template(template);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 client` command line; no process
    /// is spawned.
    #[test]
    fn general_mode_without_options() {
        let client = Client::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&client.setup_command("p4")), ["client"]);
    }

    #[test]
    fn general_mode_with_force_and_template() {
        let client = Client::new("p4", GlobalOpts::new()).force().template("tpl");

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-f", "-t", "tpl"]
        );
        assert_eq!(client.get_template(), "tpl");

        // The `set_*` variants mutate an already-selected option.
        let mut client = Client::new("p4", GlobalOpts::new()).template("tmp");
        client.set_template("tpl");
        assert_eq!(client.get_template(), "tpl");
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-t", "tpl"]
        );
    }

    #[cfg(not(feature = "lt2016_2"))]
    #[test]
    fn general_mode_with_all_options() {
        let client = Client::new("p4", GlobalOpts::new())
            .force()
            .template("tpl")
            .workspace_type(WorkSpaceType::Readonly);

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-f", "-t", "tpl", "-T", "readonly"]
        );
        assert_eq!(client.get_template(), "tpl");
        assert_eq!(client.get_workspace_type(), WorkSpaceType::Readonly);
    }

    #[cfg(not(feature = "lt2016_2"))]
    #[test]
    fn workspace_type_values() {
        for (ty, s) in [
            (WorkSpaceType::Writable, "writeable"),
            (WorkSpaceType::Readonly, "readonly"),
            (WorkSpaceType::Partitioned, "partitioned"),
            #[cfg(not(feature = "lt2024_1"))]
            (WorkSpaceType::PartitionedJnl, "partitioned-jnl"),
            #[cfg(not(feature = "lt2024_1"))]
            (WorkSpaceType::Graph, "graph"),
        ] {
            let client = Client::new("p4", GlobalOpts::new()).workspace_type(ty);
            assert_eq!(args_of(&client.setup_command("p4")), ["client", "-T", s]);
        }
    }

    #[test]
    fn stdout_mode_carries_template() {
        let client = Client::new("p4", GlobalOpts::new())
            .template("tpl")
            .stdout();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-o", "-t", "tpl"]
        );
    }

    #[cfg(not(feature = "lt2016_2"))]
    #[test]
    fn stdout_mode_carries_template_and_type() {
        let client = Client::new("p4", GlobalOpts::new())
            .template("tpl")
            .workspace_type(WorkSpaceType::Partitioned)
            .stdout();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-o", "-t", "tpl", "-T", "partitioned"]
        );
    }

    #[test]
    fn delete_mode_is_terminal() {
        let client = Client::new("p4", GlobalOpts::new()).delete();

        assert_eq!(args_of(&client.setup_command("p4")), ["client", "-d"]);
    }

    #[test]
    fn delete_mode_with_force_and_keep_shelved() {
        let client = Client::new("p4", GlobalOpts::new())
            .delete()
            .force()
            .keep_shelved();

        assert_eq!(client.get_shelved(), Some(ShelvedHandling::Keep));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "-Fs"]
        );
    }

    #[cfg(not(feature = "lt2015_2"))]
    #[test]
    fn delete_mode_with_force_and_server_id() {
        // `--serverid=X` followed by `-Fs`: the documented
        // `-d -f --serverid=X [-Fs]` form.
        let mut client = Client::new("p4", GlobalOpts::new())
            .delete()
            .force()
            .server_id("edge.1667");

        assert_eq!(client.get_server_id(), "edge.1667");
        assert_eq!(client.get_shelved(), None);
        client.set_server_id("edge.42");

        let client = client.keep_shelved();

        assert_eq!(client.get_server_id(), "edge.42");
        assert_eq!(client.get_shelved(), Some(ShelvedHandling::Keep));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "--serverid=edge.42", "-Fs"]
        );

        // `-Fs` selected first, then `--serverid=X`: the same documented
        // form reached in the opposite order.
        let client = Client::new("p4", GlobalOpts::new())
            .delete()
            .force()
            .keep_shelved()
            .server_id("edge.1667");

        assert_eq!(client.get_server_id(), "edge.1667");
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"]
        );
    }

    #[cfg(not(feature = "lt2024_1"))]
    #[test]
    fn delete_mode_with_force_and_delete_shelved() {
        // `-Fd` without `--serverid=X`: the `-d -f -Fd` form.
        let client = Client::new("p4", GlobalOpts::new())
            .delete()
            .force()
            .delete_shelved();

        assert_eq!(client.get_shelved(), Some(ShelvedHandling::Delete));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "-Fd"]
        );
    }

    #[test]
    fn force_set_in_general_mode_can_delete() {
        let client = Client::new("p4", GlobalOpts::new()).force().delete();

        assert_eq!(args_of(&client.setup_command("p4")), ["client", "-d", "-f"]);
    }

    #[test]
    fn force_unlocks_keep_shelved_one_shot() {
        let client = Client::new("p4", GlobalOpts::new()).force().keep_shelved();

        assert_eq!(client.get_shelved(), Some(ShelvedHandling::Keep));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "-Fs"]
        );
    }

    #[cfg(not(feature = "lt2015_2"))]
    #[test]
    fn force_unlocks_server_id_one_shot() {
        let client = Client::new("p4", GlobalOpts::new())
            .force()
            .server_id("edge.1667")
            .keep_shelved();

        assert_eq!(client.get_server_id(), "edge.1667");
        assert_eq!(client.get_shelved(), Some(ShelvedHandling::Keep));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "--serverid=edge.1667", "-Fs"]
        );
    }

    #[cfg(not(feature = "lt2024_1"))]
    #[test]
    fn force_unlocks_delete_shelved_one_shot() {
        let client = Client::new("p4", GlobalOpts::new())
            .force()
            .delete_shelved();

        assert_eq!(client.get_shelved(), Some(ShelvedHandling::Delete));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-d", "-f", "-Fd"]
        );
    }

    #[test]
    fn stream_mode() {
        let client = Client::new("p4", GlobalOpts::new()).stream("//stream/main");

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-S", "//stream/main"]
        );
        assert_eq!(client.get_stream(), "//stream/main");

        let client = client.stdout();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-S", "//stream/main", "-o"]
        );

        let client = client.change("4321");

        assert_eq!(client.get_change(), Some("4321"));
        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-S", "//stream/main", "-c", "4321", "-o"]
        );
    }

    #[cfg(not(feature = "lt2022_1"))]
    #[test]
    fn switch_view_stream_carries_force() {
        let client = Client::new("p4", GlobalOpts::new())
            .force()
            .switch_view_stream("//stream/dev");

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-s", "-f", "-S", "//stream/dev"]
        );
        assert_eq!(client.get_stream(), "//stream/dev");
    }

    #[test]
    fn switch_view_reuses_stream() {
        // `-S stream` followed by `switch_view`: the documented
        // `-s -S stream` form reached on the two-step path.
        let client = Client::new("p4", GlobalOpts::new())
            .stream("//stream/dev")
            .switch_view();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-s", "-S", "//stream/dev"]
        );
        assert_eq!(client.get_stream(), "//stream/dev");

        // The stream value remains editable on the intermediate form.
        let client = Client::new("p4", GlobalOpts::new())
            .stream("old")
            .stream("//stream/dev")
            .switch_view();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-s", "-S", "//stream/dev"]
        );
    }

    #[cfg(not(feature = "lt2022_1"))]
    #[test]
    fn switch_view_reuses_stream_then_force() {
        // The intermediate `-S` form rejects `-f`; it is added after the
        // switch.
        let client = Client::new("p4", GlobalOpts::new())
            .stream("//stream/dev")
            .switch_view()
            .force();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-s", "-f", "-S", "//stream/dev"]
        );
    }

    #[test]
    fn switch_view_template() {
        let client = Client::new("p4", GlobalOpts::new()).switch_view_template("tpl");

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-s", "-t", "tpl"]
        );
        assert_eq!(client.get_template(), "tpl");
    }

    #[test]
    fn switch_view_reuses_template() {
        let client = Client::new("p4", GlobalOpts::new())
            .template("tpl")
            .switch_view();

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["client", "-s", "-t", "tpl"]
        );
        assert_eq!(client.get_template(), "tpl");
    }

    #[test]
    fn stdin_mode() {
        let client = Client::new("p4", GlobalOpts::new()).stdin();

        assert_eq!(args_of(&client.setup_command("p4")), ["client", "-i"]);

        let client = Client::new("p4", GlobalOpts::new()).force().stdin();

        assert_eq!(args_of(&client.setup_command("p4")), ["client", "-i", "-f"]);
    }

    #[test]
    fn global_opts_are_injected_once() {
        let client = Client::new("p4", GlobalOpts::new().port("localhost:1666"));

        assert_eq!(
            args_of(&client.setup_command("p4")),
            ["-p", "localhost:1666", "client"]
        );
    }
}
