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

/// The general form: `p4 branch [-f] branchspec`.
///
/// Create or edit a branch mapping. The `-f` flag permits administrators to
/// change the modification date of the branch mapping.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneralMode {
    force: bool,
}

/// The delete form: `p4 branch -d [-f] branchspec`.
///
/// Delete the named branch mapping. The `-f` flag permits P4 Server
/// administrators to delete branches they don't own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeleteMode {
    force: bool,
}

/// The stdin form: `p4 branch -i [-f]`.
///
/// Read the branch mapping from standard input without invoking an editor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StdinMode {
    force: bool,
}

/// The stdout form state in which `-S stream` has been supplied.
///
/// Carries the stream name and unlocks the `-P parent` option, which
/// requires `-S` and evaluates the stream as if it were a child of the
/// given parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamSelected {
    stream: String,

    parent: Option<String>,
}

/// The stdout form: `p4 branch [-S stream [-P parent]] -o placeholder`.
///
/// Write the branch mapping to standard output without invoking an editor.
/// The optional `-S stream` displays the mapping generated for the specified
/// stream; `-P parent` requires `-S` and evaluates the stream as if it were
/// a child of the given parent.
///
/// The `S` type parameter tracks whether `-S` has been supplied: the
/// [`Unselected`] state carries no stream, and supplying a stream with
/// `Branch::stream` transitions to the [`StreamSelected`] state, which
/// unlocks the `-P` option.
#[derive(Debug, Clone, Default)]
pub struct StdoutMode<S = Unselected> {
    stream_opts: S,
}

// ---------------------------------------------------------------------------
// ExclusiveOption implementations
// ---------------------------------------------------------------------------

impl ExclusiveOption for GeneralMode {
    fn inject_args(&self, command: &mut Command) {
        if self.force {
            command.arg("-f");
        }
    }
}

impl ExclusiveOption for DeleteMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-d");
        if self.force {
            command.arg("-f");
        }
    }
}

impl ExclusiveOption for StdinMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-i");
        if self.force {
            command.arg("-f");
        }
    }
}

impl ExclusiveOption for StdoutMode<Unselected> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-o");
    }
}

impl ExclusiveOption for StdoutMode<StreamSelected> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-S").arg(&self.stream_opts.stream);
        if let Some(parent) = &self.stream_opts.parent {
            command.arg("-P").arg(parent);
        }
        command.arg("-o");
    }
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 branch`.
///
/// Create or edit a branch mapping and its view.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] branch [-f] branchspec
/// p4 [g-opts] branch -d [-f] branchspec
/// p4 [g-opts] branch -i [-f]
/// p4 [g-opts] branch [-S stream] -o placeholderForStreamBranchSpecName
/// p4 [g-opts] branch [-S stream -P parent] -o placeholderForStreamBranchSpecName
/// ```
///
/// The `M` type parameter tracks the command form at compile time, starting
/// as [`Unselected`] from [`Branch::new`]. Enter a form with one of
/// [`Self::force`], [`Self::delete`], [`Self::stdin`], or [`Self::stdout`]:
/// - [`GeneralMode`] (entered via [`Self::force`]): create or edit a branch
///   mapping; can transition to [`DeleteMode`] or [`StdinMode`] preserving
///   the `-f` flag.
/// - [`DeleteMode`] (`-d`, entered via [`Self::delete`]): delete a branch
///   mapping.
/// - [`StdinMode`] (`-i`, entered via [`Self::stdin`]): read the branch
///   mapping from standard input.
/// - [`StdoutMode`] (`-o`, entered via [`Self::stdout`]): write the branch
///   mapping to standard output; `Branch::stream` supplies `-S` and
///   transitions to the [`StreamSelected`] state, which unlocks
///   `Branch::parent` (`-P` requires `-S`).
#[derive(Debug, Clone, Default)]
pub struct Branch<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,
}

// ---------------------------------------------------------------------------
// Constructor
// ---------------------------------------------------------------------------

impl Branch<Unselected> {
    /// Creates a new `p4 branch` command with no form selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. Use
    /// [`Self::force`], [`Self::delete`], [`Self::stdin`], or
    /// [`Self::stdout`] to select a form before spawning.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Enters the general form: `p4 branch [-f] branchspec`.
    ///
    /// `v` sets the initial state of the `-f` flag.
    pub fn force(self, v: bool) -> Branch<GeneralMode> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: GeneralMode { force: v },
        }
    }

    /// Enters the delete form: `p4 branch -d [-f] branchspec`.
    pub fn delete(self) -> Branch<DeleteMode> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode { force: false },
        }
    }

    /// Enters the stdin form: `p4 branch -i [-f]`.
    pub fn stdin(self) -> Branch<StdinMode> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdinMode { force: false },
        }
    }

    /// Enters the stdout form: `p4 branch [-S stream [-P parent]] -o
    /// placeholder`.
    ///
    /// Use `Branch::stream` on the entered form to supply `-S`; that
    /// transitions to the [`StreamSelected`] state and unlocks
    /// `Branch::parent` (`-P` requires `-S`).
    pub fn stdout(self) -> Branch<StdoutMode> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdoutMode::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Shared accessors
// ---------------------------------------------------------------------------

impl<M> Branch<M> {
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
// GeneralMode: force accessor + transitions
// ---------------------------------------------------------------------------

impl Branch<GeneralMode> {
    /// # Description
    ///
    /// -f
    ///
    /// Force option. Combined with `-d`, permits P4 Server administrators to
    /// delete branches they don't own. Also permits administrators to change
    /// the modification date of the branch mapping (the `Update` field is
    /// writable when using the `-f` option).
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force option. Combined with `-d`, permits P4 Server administrators to
    /// delete branches they don't own. Also permits administrators to change
    /// the modification date of the branch mapping (the `Update` field is
    /// writable when using the `-f` option).
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force option. Combined with `-d`, permits P4 Server administrators to
    /// delete branches they don't own. Also permits administrators to change
    /// the modification date of the branch mapping (the `Update` field is
    /// writable when using the `-f` option).
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }

    /// Transitions to the delete form (`-d`), preserving the `-f` flag.
    pub fn delete(self) -> Branch<DeleteMode> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: DeleteMode {
                force: self.mode.force,
            },
        }
    }

    /// Transitions to the stdin form (`-i`), preserving the `-f` flag.
    pub fn stdin(self) -> Branch<StdinMode> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdinMode {
                force: self.mode.force,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// DeleteMode: force accessor
// ---------------------------------------------------------------------------

impl Branch<DeleteMode> {
    /// # Description
    ///
    /// -f
    ///
    /// Force option. Permits P4 Server administrators to delete branches they
    /// don't own.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force option. Permits P4 Server administrators to delete branches they
    /// don't own.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force option. Permits P4 Server administrators to delete branches they
    /// don't own.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }
}

// ---------------------------------------------------------------------------
// StdinMode: force accessor
// ---------------------------------------------------------------------------

impl Branch<StdinMode> {
    /// # Description
    ///
    /// -f
    ///
    /// Force option. Permits administrators to change the modification date of
    /// the branch mapping.
    pub fn get_force(&self) -> bool {
        self.mode.force
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force option. Permits administrators to change the modification date of
    /// the branch mapping.
    pub fn set_force(&mut self, v: bool) -> &mut Self {
        self.mode.force = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force option. Permits administrators to change the modification date of
    /// the branch mapping.
    pub fn force(mut self, v: bool) -> Self {
        self.mode.force = v;
        self
    }
}

// ---------------------------------------------------------------------------
// StdoutMode<Unselected>: -S transition
// ---------------------------------------------------------------------------

impl Branch<StdoutMode<Unselected>> {
    /// # Description
    ///
    /// -S stream
    ///
    /// Display the mapping generated for the specified stream. This option
    /// enables you to see how change is propagated between the stream and its
    /// parent.
    ///
    /// Transitions this command to the [`StreamSelected`] state, which
    /// unlocks the `-P` option.
    pub fn stream(self, v: impl Into<String>) -> Branch<StdoutMode<StreamSelected>> {
        Branch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: StdoutMode {
                stream_opts: StreamSelected {
                    stream: v.into(),
                    parent: None,
                },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// StdoutMode<StreamSelected>: stream and parent accessors
// ---------------------------------------------------------------------------

impl Branch<StdoutMode<StreamSelected>> {
    /// # Description
    ///
    /// -S stream
    ///
    /// The stream whose generated mapping is displayed.
    pub fn get_stream(&self) -> &str {
        &self.mode.stream_opts.stream
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// The stream whose generated mapping is displayed.
    pub fn set_stream(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.stream_opts.stream = v.into();
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// The stream whose generated mapping is displayed. Preserves any `-P`
    /// parent already set.
    pub fn stream(mut self, v: impl Into<String>) -> Self {
        self.mode.stream_opts.stream = v.into();
        self
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// For a specified stream, display the mapping that is generated by
    /// hypothetically treating the stream as if it were a child of the
    /// specified parent.
    pub fn get_parent(&self) -> Option<&String> {
        self.mode.stream_opts.parent.as_ref()
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// For a specified stream, display the mapping that is generated by
    /// hypothetically treating the stream as if it were a child of the
    /// specified parent.
    pub fn set_parent(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.stream_opts.parent = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -P parent
    ///
    /// For a specified stream, display the mapping that is generated by
    /// hypothetically treating the stream as if it were a child of the
    /// specified parent.
    pub fn parent(mut self, v: impl Into<String>) -> Self {
        self.mode.stream_opts.parent = Some(v.into());
        self
    }
}

// ---------------------------------------------------------------------------
// SubCommand
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> SubCommand for Branch<M> {
    fn name(&self) -> &str {
        "branch"
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

// General form: 0 args (open editor) or 1 branchspec.
impl ParameterizedSpawn<()> for Branch<GeneralMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branch` (the general form) without a branchspec argument,
    /// opening the editor to create a new branch mapping.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Branch<GeneralMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branch` with the given `branchspec` argument as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (branchspec,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(branchspec)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Delete form: exactly 1 branchspec.
impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for Branch<DeleteMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branch -d` with the given `branchspec` argument as a child
    /// process with piped standard output and error streams; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (branchspec,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(branchspec)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Stdin form: no positional argument; stdin is piped.
impl ParameterizedSpawn<()> for Branch<StdinMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branch -i` with a piped standard input stream (and piped
    /// standard output and error streams); write the branch mapping
    /// specification to the returned [`Child`]'s stdin handle.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl ParameterizedSpawn<(Stdio,)> for Branch<StdinMode> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branch -i` with the given standard input configuration and
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    ///
    /// Pass `Stdio::piped()` to obtain a writable `child.stdin` handle and
    /// feed the branch mapping specification into it; pass `Stdio::inherit()`
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

// Stdout form: exactly 1 placeholder.
impl<S, I> ParameterizedSpawn<(I,)> for Branch<StdoutMode<S>>
where
    StdoutMode<S>: ExclusiveOption,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 branch [-S stream [-P parent]] -o` with the given placeholder
    /// argument as a child process with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (placeholder,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(placeholder)
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
        let mut command = Branch::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    // Compile-time rejection: Branch<Unselected> has no spawn impl.
    // Compile-time rejection: Branch<StdoutMode> has no `force()` method
    // (stdout form does not accept -f).
    // Compile-time rejection: Branch<StdoutMode> (no -S yet) has no
    // `parent()`/`set_parent()`/`get_parent()` method (-P requires -S).

    /// Dry-run checks of the assembled `p4 branch` command line; no process
    /// is spawned.
    #[test]
    fn general_form_without_force() {
        let branch = Branch::new("p4", GlobalOpts::new()).force(false);
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch"]);
    }

    #[test]
    fn general_form_with_force() {
        let branch = Branch::new("p4", GlobalOpts::new()).force(true);
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch", "-f"]);
    }

    #[test]
    fn delete_form_from_unselected() {
        let branch = Branch::new("p4", GlobalOpts::new()).delete();
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch", "-d"]);
    }

    #[test]
    fn delete_form_preserves_force_from_general() {
        let branch = Branch::new("p4", GlobalOpts::new()).force(true).delete();
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch", "-d", "-f"]);
    }

    #[test]
    fn stdin_form_from_unselected() {
        let branch = Branch::new("p4", GlobalOpts::new()).stdin();
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch", "-i"]);
    }

    #[test]
    fn stdin_form_preserves_force_from_general() {
        let branch = Branch::new("p4", GlobalOpts::new()).force(true).stdin();
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch", "-i", "-f"]);
    }

    #[test]
    fn stdout_form_from_unselected() {
        let branch = Branch::new("p4", GlobalOpts::new()).stdout();
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch", "-o"]);
    }

    #[test]
    fn stdout_with_stream() {
        let branch = Branch::new("p4", GlobalOpts::new())
            .stdout()
            .stream("//depotstream1/5dev1");
        assert_eq!(
            args_of(&branch.setup_command("p4")),
            ["branch", "-S", "//depotstream1/5dev1", "-o"]
        );
    }

    #[test]
    fn stdout_with_stream_and_parent() {
        let branch = Branch::new("p4", GlobalOpts::new())
            .stdout()
            .stream("//stream1/5dev1")
            .parent("//stream1/main5");
        assert_eq!(
            args_of(&branch.setup_command("p4")),
            [
                "branch",
                "-S",
                "//stream1/5dev1",
                "-P",
                "//stream1/main5",
                "-o"
            ]
        );
    }

    #[test]
    fn renaming_stream_preserves_parent() {
        let branch = Branch::new("p4", GlobalOpts::new())
            .stdout()
            .stream("//stream1/5dev1")
            .parent("//stream1/main5")
            .stream("//stream2/5dev2");
        assert_eq!(
            args_of(&branch.setup_command("p4")),
            [
                "branch",
                "-S",
                "//stream2/5dev2",
                "-P",
                "//stream1/main5",
                "-o"
            ]
        );
    }

    #[test]
    fn force_can_be_set_inside_general_delete_and_stdin() {
        let general = Branch::new("p4", GlobalOpts::new())
            .force(false)
            .force(true);
        assert_eq!(args_of(&general.setup_command("p4")), ["branch", "-f"]);

        let delete = Branch::new("p4", GlobalOpts::new()).delete().force(true);
        assert_eq!(args_of(&delete.setup_command("p4")), ["branch", "-d", "-f"]);

        let stdin = Branch::new("p4", GlobalOpts::new()).stdin().force(true);
        assert_eq!(args_of(&stdin.setup_command("p4")), ["branch", "-i", "-f"]);
    }

    #[test]
    fn force_can_be_unset() {
        let branch = Branch::new("p4", GlobalOpts::new())
            .force(true)
            .force(false);
        assert_eq!(args_of(&branch.setup_command("p4")), ["branch"]);
    }

    #[test]
    fn setters_update_the_same_command() {
        let mut branch = Branch::new("p4", GlobalOpts::new()).force(false);
        branch.set_force(true);
        assert!(branch.get_force());

        let mut stdout = Branch::new("p4", GlobalOpts::new())
            .stdout()
            .stream("//depot/main");
        stdout.set_parent("//depot/dev");
        assert_eq!(stdout.get_stream(), "//depot/main");
        assert_eq!(stdout.get_parent(), Some(&String::from("//depot/dev")));
    }

    #[test]
    fn global_opts_are_injected_once() {
        let branch = Branch::new("p4", GlobalOpts::new().port("localhost:1666")).force(false);
        assert_eq!(
            args_of(&branch.setup_command("p4")),
            ["-p", "localhost:1666", "branch"]
        );
    }

    // Spawn argument counts per form.
    #[test]
    fn spawn_argument_counts_per_form() {
        // General: 0 or 1 branchspec.
        let plain = Branch::new("p4", GlobalOpts::new()).force(false);
        assert_eq!(args_of(&plain.setup_command("p4")), ["branch"]);
        let mut with_bs = plain.setup_command("p4");
        with_bs.arg("mybranch");
        assert_eq!(args_of(&with_bs), ["branch", "mybranch"]);

        // Delete: exactly 1 branchspec.
        let delete = Branch::new("p4", GlobalOpts::new()).force(true).delete();
        let mut delete_bs = delete.setup_command("p4");
        delete_bs.arg("oldbranch");
        assert_eq!(args_of(&delete_bs), ["branch", "-d", "-f", "oldbranch"]);

        // Stdin: no positional; stdin is configured separately.
        let stdin = Branch::new("p4", GlobalOpts::new()).stdin();
        assert_eq!(args_of(&stdin.setup_command("p4")), ["branch", "-i"]);

        // Stdout: exactly 1 placeholder.
        let stdout = Branch::new("p4", GlobalOpts::new()).stdout();
        let mut stdout_ph = stdout.setup_command("p4");
        stdout_ph.arg("placeholder1");
        assert_eq!(args_of(&stdout_ph), ["branch", "-o", "placeholder1"]);

        // Stdout with stream and parent: exactly 1 placeholder.
        let stream = Branch::new("p4", GlobalOpts::new())
            .stdout()
            .stream("//depotstream1/5dev1")
            .parent("//stream1/main5");
        let mut stream_ph = stream.setup_command("p4");
        stream_ph.arg("placeholder2");
        assert_eq!(
            args_of(&stream_ph),
            [
                "branch",
                "-S",
                "//depotstream1/5dev1",
                "-P",
                "//stream1/main5",
                "-o",
                "placeholder2"
            ]
        );
    }
}
