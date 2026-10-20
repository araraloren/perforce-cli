use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The optional positional argument of `p4 configure show` and
/// `p4 configure history`.
///
/// The argument selects what the report covers: all servers
/// ([`Target::AllServers`]), a named server ([`Target::Server`]), or a
/// single configuration variable ([`Target::Variable`]). When no target is
/// selected, the report covers the current server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Reports the configuration variables for all servers known to the
    /// system; injected as the `allservers` keyword.
    AllServers,

    /// Reports the settings of the named server.
    Server(String),

    /// Reports the setting(s) of the specified variable.
    Variable(String),
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 configure`.
///
#[cfg_attr(
    feature = "lt2019_2",
    doc = "Set and manage server configuration variables."
)]
#[cfg_attr(
    all(not(feature = "lt2019_2"), feature = "lt2024_1"),
    doc = "Set, view, and manage server configuration variables."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "Set, view, and manage server configuration variables, including comments."
)]
///
/// Configuration variables are used to control and customize the behavior
/// of the Perforce service.
///
/// The `T` type parameter tracks the selected operation at compile time: it
/// starts as [`Unselected`], in which the command has no effect and cannot
/// be spawned. Use one of the builder methods to select the operation:
/// [`set`](Self::set), [`unset`](Self::unset), or [`show`](Self::show)
#[cfg_attr(not(feature = "lt2019_2"), doc = ", [`history`](Self::history)")]
#[cfg_attr(feature = "lt2023_2", doc = ".")]
#[cfg_attr(
    not(feature = "lt2023_2"),
    doc = ", [`patch`](Self::patch), or [`help`](Self::help)."
)]
#[derive(Debug, Clone, Default)]
pub struct Configure<T = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    sub_command: T,
}

impl Configure<Unselected> {
    /// Creates a new `p4 configure` command with no operation selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. The
    /// command has no effect in this state; use one of the builder methods
    /// to select the operation before spawning.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            sub_command: Unselected,
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

    /// Selects the `set` operation.
    ///
    /// The `variable` and `value` are required; the optional `server_id`
    /// prefix and the `--comment` refinement are set on the returned
    /// builder.
    pub fn set(
        self,
        variable: impl Into<String>,
        value: impl Into<String>,
    ) -> Configure<SetVariable> {
        Configure {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: SetVariable {
                server_id: None,
                variable: variable.into(),
                value: value.into(),
                ..Default::default()
            },
        }
    }

    /// Selects the `unset` operation.
    ///
    /// The `variable` is required; the optional `server_id` prefix and the
    /// `--comment` refinement are set on the returned builder.
    pub fn unset(self, variable: impl Into<String>) -> Configure<UnSetVariable> {
        Configure {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: UnSetVariable {
                server_id: None,
                variable: variable.into(),
                ..Default::default()
            },
        }
    }

    /// Selects the `show` operation.
    ///
    /// The returned builder spawns the bare `p4 configure show` report; use
    /// [`allservers`](Configure::allservers), [`server`](Configure::server),
    /// or [`variable`](Configure::variable) to restrict the report.
    pub fn show(self) -> Configure<Show> {
        Configure {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: Show::default(),
        }
    }

    /// Selects the `history` operation.
    ///
    /// The returned builder spawns the bare `p4 configure history` report;
    /// use [`allservers`](Configure::allservers),
    /// [`server`](Configure::server), or [`variable`](Configure::variable)
    /// to restrict the report.
    #[cfg(not(feature = "lt2019_2"))]
    pub fn history(self) -> Configure<History> {
        Configure {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: History::default(),
        }
    }

    /// Selects the `p4 configure history` comment patch form.
    ///
    /// Sets the `comment` on the history of `variable`, optionally targeting
    /// a specific history entry with [`iteration`](Configure::iteration).
    #[cfg(not(feature = "lt2023_2"))]
    pub fn patch(
        self,
        variable: impl Into<String>,
        comment: impl Into<String>,
    ) -> Configure<Patch> {
        Configure {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: Patch {
                iteration: None,
                comment: comment.into(),
                variable: variable.into(),
            },
        }
    }

    /// Selects the `help` operation.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn help(self) -> Configure<Help> {
        Configure {
            bin: self.bin,
            global_opts: self.global_opts,
            sub_command: Help,
        }
    }
}

// ---------------------------------------------------------------------------
// Carrier
// ---------------------------------------------------------------------------

impl<T: SubCommand> Configure<T> {
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

    /// Spawns the assembled `p4 configure` command as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_piped(&mut self) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<T: SubCommand> SubCommand for Configure<T> {
    fn name(&self) -> &str {
        "configure"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.sub_command.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// configure set
// ---------------------------------------------------------------------------

/// The `p4 configure set` operation.
///
/// Sets the named variable to the provided value.
///
/// # Syntax
///
/// ```text
#[cfg_attr(
    feature = "lt2020_1",
    doc = "p4 [g-opts] configure set [P4NAME#|serverid#]variable=value",
    doc = "```"
)]
#[cfg_attr(
    all(not(feature = "lt2020_1"), feature = "lt2023_2"),
    doc = "p4 [g-opts] configure set [serverid#]variable=value",
    doc = "```"
)]
#[cfg_attr(
    not(feature = "lt2023_2"),
    doc = "p4 [g-opts] configure set [--comment=comment] [serverid#]variable=value",
    doc = "```"
)]
#[derive(Debug, Clone, Default)]
pub struct SetVariable {
    /// `[serverid#]`
    server_id: Option<String>,

    /// `variable`
    variable: String,

    /// `value`
    value: String,

    /// `--comment=comment`
    #[cfg(not(feature = "lt2023_2"))]
    comment: Option<String>,
}

impl Configure<SetVariable> {
    /// Returns the server whose configuration is being set, if any.
    pub fn get_server_id(&self) -> Option<&str> {
        self.sub_command.server_id.as_deref()
    }

    /// Sets the `[serverid#]` prefix.
    pub fn set_server_id(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.server_id = Some(v.into());
        self
    }

    /// Sets the `[serverid#]` prefix.
    pub fn server_id(mut self, v: impl Into<String>) -> Self {
        self.set_server_id(v);
        self
    }

    /// Returns the name of the variable being set.
    pub fn get_variable(&self) -> &str {
        &self.sub_command.variable
    }

    /// Sets the name of the variable being set.
    pub fn set_variable(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.variable = v.into();
        self
    }

    /// Sets the name of the variable being set.
    pub fn variable(mut self, v: impl Into<String>) -> Self {
        self.set_variable(v);
        self
    }

    /// Returns the value being assigned.
    pub fn get_value(&self) -> &str {
        &self.sub_command.value
    }

    /// Sets the value to assign.
    pub fn set_value(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.value = v.into();
        self
    }

    /// Sets the value to assign.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.set_value(v);
        self
    }

    /// Returns the comment attached to the change, if any.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn get_comment(&self) -> Option<&str> {
        self.sub_command.comment.as_deref()
    }

    /// Sets `--comment=comment`.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn set_comment(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.comment = Some(v.into());
        self
    }

    /// Sets `--comment=comment`.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn comment(mut self, v: impl Into<String>) -> Self {
        self.set_comment(v);
        self
    }
}

impl SubCommand for SetVariable {
    fn name(&self) -> &str {
        "set"
    }

    fn inject_local_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2023_2"))]
        {
            if let Some(comment) = &self.comment {
                command.arg(format!("--comment={}", comment));
            }
        }

        let variable = match &self.server_id {
            Some(server_id) => format!("{}#{}", server_id, self.variable),
            None => self.variable.clone(),
        };
        command.arg(format!("{}={}", variable, self.value));
    }
}

impl ParameterizedSpawn<()> for Configure<SetVariable> {
    type Output = Child;
    type Error = std::io::Error;

    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.spawn_piped()
    }
}

// ---------------------------------------------------------------------------
// configure unset
// ---------------------------------------------------------------------------

/// The `p4 configure unset` operation.
///
/// Unsets the named configuration variable.
///
/// # Syntax
///
/// ```text
#[cfg_attr(
    feature = "lt2020_1",
    doc = "p4 [g-opts] configure unset [P4NAME#|serverid#]variable",
    doc = "```"
)]
#[cfg_attr(
    all(not(feature = "lt2020_1"), feature = "lt2023_2"),
    doc = "p4 [g-opts] configure unset [serverid#]variable",
    doc = "```"
)]
#[cfg_attr(
    not(feature = "lt2023_2"),
    doc = "p4 [g-opts] configure unset [--comment=comment] [serverid#]variable",
    doc = "```"
)]
#[derive(Debug, Clone, Default)]
pub struct UnSetVariable {
    /// `[serverid#]`
    server_id: Option<String>,

    /// `variable`
    variable: String,

    /// `--comment=comment`
    #[cfg(not(feature = "lt2023_2"))]
    comment: Option<String>,
}

impl Configure<UnSetVariable> {
    /// Returns the server whose configuration is being unset, if any.
    pub fn get_server_id(&self) -> Option<&str> {
        self.sub_command.server_id.as_deref()
    }

    /// Sets the `[serverid#]` prefix.
    pub fn set_server_id(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.server_id = Some(v.into());
        self
    }

    /// Sets the `[serverid#]` prefix.
    pub fn server_id(mut self, v: impl Into<String>) -> Self {
        self.set_server_id(v);
        self
    }

    /// Returns the name of the variable being unset.
    pub fn get_variable(&self) -> &str {
        &self.sub_command.variable
    }

    /// Sets the name of the variable being unset.
    pub fn set_variable(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.variable = v.into();
        self
    }

    /// Sets the name of the variable being unset.
    pub fn variable(mut self, v: impl Into<String>) -> Self {
        self.set_variable(v);
        self
    }

    /// Returns the comment attached to the change, if any.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn get_comment(&self) -> Option<&str> {
        self.sub_command.comment.as_deref()
    }

    /// Sets `--comment=comment`.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn set_comment(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.comment = Some(v.into());
        self
    }

    /// Sets `--comment=comment`.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn comment(mut self, v: impl Into<String>) -> Self {
        self.set_comment(v);
        self
    }
}

impl SubCommand for UnSetVariable {
    fn name(&self) -> &str {
        "unset"
    }

    fn inject_local_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2023_2"))]
        {
            if let Some(comment) = &self.comment {
                command.arg(format!("--comment={}", comment));
            }
        }

        let variable = match &self.server_id {
            Some(server_id) => format!("{}#{}", server_id, self.variable),
            None => self.variable.clone(),
        };
        command.arg(variable);
    }
}

impl ParameterizedSpawn<()> for Configure<UnSetVariable> {
    type Output = Child;
    type Error = std::io::Error;

    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.spawn_piped()
    }
}

// ---------------------------------------------------------------------------
// configure show
// ---------------------------------------------------------------------------

/// The `p4 configure show` operation.
///
/// Displays the current configuration of this server.
///
/// # Syntax
///
/// ```text
#[cfg_attr(
    feature = "lt2020_1",
    doc = "p4 [g-opts] configure show [allservers | P4NAME | variable]",
    doc = "```"
)]
#[cfg_attr(
    all(not(feature = "lt2020_1"), feature = "lt2023_2"),
    doc = "p4 [g-opts] configure show [allservers | variable]",
    doc = "```"
)]
#[cfg_attr(
    not(feature = "lt2023_2"),
    doc = "p4 [g-opts] configure show [allservers | serverid | variable]",
    doc = "```"
)]
#[derive(Debug, Clone, Default)]
pub struct Show {
    /// The optional positional target of the report.
    target: Option<Target>,
}

impl Configure<Show> {
    /// Returns the selected target, if any.
    pub fn get_target(&self) -> Option<&Target> {
        self.sub_command.target.as_ref()
    }

    /// Sets the positional target of the report.
    pub fn set_target(&mut self, target: Option<Target>) -> &mut Self {
        self.sub_command.target = target;
        self
    }

    /// Sets the positional target of the report.
    pub fn target(mut self, target: Option<Target>) -> Self {
        self.set_target(target);
        self
    }

    /// Restricts the report to all servers known to the system.
    pub fn allservers(mut self) -> Self {
        self.sub_command.target = Some(Target::AllServers);
        self
    }

    /// Restricts the report to the named server.
    pub fn server(mut self, v: impl Into<String>) -> Self {
        self.sub_command.target = Some(Target::Server(v.into()));
        self
    }

    /// Restricts the report to the specified variable.
    pub fn variable(mut self, v: impl Into<String>) -> Self {
        self.sub_command.target = Some(Target::Variable(v.into()));
        self
    }
}

impl SubCommand for Show {
    fn name(&self) -> &str {
        "show"
    }

    fn inject_local_args(&self, command: &mut Command) {
        match &self.target {
            Some(Target::AllServers) => {
                command.arg("allservers");
            }
            Some(Target::Server(name)) => {
                command.arg(name);
            }
            Some(Target::Variable(name)) => {
                command.arg(name);
            }
            None => {}
        }
    }
}

impl ParameterizedSpawn<()> for Configure<Show> {
    type Output = Child;
    type Error = std::io::Error;

    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.spawn_piped()
    }
}

// ---------------------------------------------------------------------------
// configure history
// ---------------------------------------------------------------------------

/// The `p4 configure history` operation.
///
/// Displays the history of configurables' values recorded by the server.
///
/// # Syntax
///
/// ```text
#[cfg_attr(
    feature = "lt2020_1",
    doc = "p4 [g-opts] configure history [allservers | P4NAME | variable]",
    doc = "```"
)]
#[cfg_attr(
    all(not(feature = "lt2020_1"), feature = "lt2023_2"),
    doc = "p4 [g-opts] configure history [allservers | variable]",
    doc = "```"
)]
#[cfg_attr(
    not(feature = "lt2023_2"),
    doc = "p4 [g-opts] configure history [allservers | serverid | variable]",
    doc = "```"
)]
#[cfg(not(feature = "lt2019_2"))]
#[derive(Debug, Clone, Default)]
pub struct History {
    /// The optional positional target of the report.
    target: Option<Target>,
}

#[cfg(not(feature = "lt2019_2"))]
impl Configure<History> {
    /// Returns the selected target, if any.
    pub fn get_target(&self) -> Option<&Target> {
        self.sub_command.target.as_ref()
    }

    /// Sets the positional target of the report.
    pub fn set_target(&mut self, target: Option<Target>) -> &mut Self {
        self.sub_command.target = target;
        self
    }

    /// Sets the positional target of the report.
    pub fn target(mut self, target: Option<Target>) -> Self {
        self.set_target(target);
        self
    }

    /// Restricts the report to all servers known to the system.
    pub fn allservers(mut self) -> Self {
        self.sub_command.target = Some(Target::AllServers);
        self
    }

    /// Restricts the report to the named server.
    pub fn server(mut self, v: impl Into<String>) -> Self {
        self.sub_command.target = Some(Target::Server(v.into()));
        self
    }

    /// Restricts the report to the specified variable.
    pub fn variable(mut self, v: impl Into<String>) -> Self {
        self.sub_command.target = Some(Target::Variable(v.into()));
        self
    }
}

#[cfg(not(feature = "lt2019_2"))]
impl SubCommand for History {
    fn name(&self) -> &str {
        "history"
    }

    fn inject_local_args(&self, command: &mut Command) {
        match &self.target {
            Some(Target::AllServers) => {
                command.arg("allservers");
            }
            Some(Target::Server(name)) => {
                command.arg(name);
            }
            Some(Target::Variable(name)) => {
                command.arg(name);
            }
            None => {}
        }
    }
}

#[cfg(not(feature = "lt2019_2"))]
impl ParameterizedSpawn<()> for Configure<History> {
    type Output = Child;
    type Error = std::io::Error;

    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.spawn_piped()
    }
}

// ---------------------------------------------------------------------------
// configure history --comment (comment patch form)
// ---------------------------------------------------------------------------

/// The `p4 configure history` comment patch operation.
///
/// Passing the long option `--comment` sets the comment for the most recent
/// history; the optional `--iteration=N` option can be used to retroactively
/// set a comment on a specific history entry.
///
/// # Syntax
///
/// ```text
/// p4 configure history [--iteration=N] --comment=comment variable
/// ```
#[cfg(not(feature = "lt2023_2"))]
#[derive(Debug, Clone, Default)]
pub struct Patch {
    /// `--iteration=N`
    iteration: Option<u64>,

    /// `--comment=comment`
    comment: String,

    /// `variable`
    variable: String,
}

#[cfg(not(feature = "lt2023_2"))]
impl Configure<Patch> {
    /// Returns the targeted history entry, if any.
    pub fn get_iteration(&self) -> Option<u64> {
        self.sub_command.iteration
    }

    /// Sets `--iteration=N`.
    pub fn set_iteration(&mut self, v: u64) -> &mut Self {
        self.sub_command.iteration = Some(v);
        self
    }

    /// Sets `--iteration=N`.
    pub fn iteration(mut self, v: u64) -> Self {
        self.set_iteration(v);
        self
    }

    /// Returns the comment being set.
    pub fn get_comment(&self) -> &str {
        &self.sub_command.comment
    }

    /// Sets the comment being set.
    pub fn set_comment(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.comment = v.into();
        self
    }

    /// Sets the comment being set.
    pub fn comment(mut self, v: impl Into<String>) -> Self {
        self.set_comment(v);
        self
    }

    /// Returns the name of the variable whose history is patched.
    pub fn get_variable(&self) -> &str {
        &self.sub_command.variable
    }

    /// Sets the name of the variable whose history is patched.
    pub fn set_variable(&mut self, v: impl Into<String>) -> &mut Self {
        self.sub_command.variable = v.into();
        self
    }

    /// Sets the name of the variable whose history is patched.
    pub fn variable(mut self, v: impl Into<String>) -> Self {
        self.set_variable(v);
        self
    }
}

#[cfg(not(feature = "lt2023_2"))]
impl SubCommand for Patch {
    fn name(&self) -> &str {
        "history"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(iteration) = self.iteration {
            command.arg(format!("--iteration={}", iteration));
        }

        command
            .arg(format!("--comment={}", self.comment))
            .arg(&self.variable);
    }
}

#[cfg(not(feature = "lt2023_2"))]
impl ParameterizedSpawn<()> for Configure<Patch> {
    type Output = Child;
    type Error = std::io::Error;

    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.spawn_piped()
    }
}

// ---------------------------------------------------------------------------
// configure help
// ---------------------------------------------------------------------------

/// The `p4 configure help` operation.
///
/// Shows details about all the documented and supported configurables. The
/// output is best redirected to a file because it has about 4,000 lines.
#[cfg(not(feature = "lt2023_2"))]
#[derive(Debug, Clone, Default)]
pub struct Help;

#[cfg(not(feature = "lt2023_2"))]
impl SubCommand for Help {
    fn name(&self) -> &str {
        "help"
    }

    fn inject_local_args(&self, _command: &mut Command) {}
}

#[cfg(not(feature = "lt2023_2"))]
impl ParameterizedSpawn<()> for Configure<Help> {
    type Output = Child;
    type Error = std::io::Error;

    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.spawn_piped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    fn entry() -> Configure<Unselected> {
        Configure::new("p4", GlobalOpts::default())
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn set_injects_options_in_syntax_order() {
        let command = entry()
            .set("monitor", "2")
            .server_id("edge1")
            .comment("why");

        assert_eq!(
            args_of(&command.setup_command("p4")),
            ["configure", "set", "--comment=why", "edge1#monitor=2"]
        );
    }

    #[test]
    fn set_without_optional_parts() {
        let command = entry().set("monitor", "2");

        assert_eq!(
            args_of(&command.setup_command("p4")),
            ["configure", "set", "monitor=2"]
        );
    }

    #[test]
    fn set_accessors_reflect_changes() {
        let mut command = entry().set("monitor", "2");

        command.set_variable("security").set_value("4");
        assert_eq!(command.get_variable(), "security");
        assert_eq!(command.get_value(), "4");
        assert_eq!(command.get_server_id(), None);

        command.set_server_id("edge1");
        assert_eq!(command.get_server_id(), Some("edge1"));
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn unset_injects_options_in_syntax_order() {
        let command = entry()
            .unset("net.parallel.max")
            .server_id("edge1")
            .comment("unused");

        assert_eq!(
            args_of(&command.setup_command("p4")),
            [
                "configure",
                "unset",
                "--comment=unused",
                "edge1#net.parallel.max"
            ]
        );
    }

    #[test]
    fn show_target_variants() {
        let bare = entry().show();
        assert_eq!(args_of(&bare.setup_command("p4")), ["configure", "show"]);
        assert_eq!(bare.get_target(), None);

        let all = entry().show().allservers();
        assert_eq!(
            args_of(&all.setup_command("p4")),
            ["configure", "show", "allservers"]
        );

        let server = entry().show().server("paris-edge");
        assert_eq!(
            args_of(&server.setup_command("p4")),
            ["configure", "show", "paris-edge"]
        );

        let variable = entry().show().variable("monitor");
        assert_eq!(
            args_of(&variable.setup_command("p4")),
            ["configure", "show", "monitor"]
        );
        assert_eq!(
            variable.get_target(),
            Some(&Target::Variable("monitor".to_string()))
        );
    }

    #[test]
    fn show_set_target_roundtrip() {
        let mut command = entry().show();
        command.set_target(Some(Target::Server("headquarters".to_string())));

        assert_eq!(
            args_of(&command.setup_command("p4")),
            ["configure", "show", "headquarters"]
        );

        command.set_target(None);
        assert_eq!(args_of(&command.setup_command("p4")), ["configure", "show"]);
    }

    #[cfg(not(feature = "lt2019_2"))]
    #[test]
    fn history_target_variants() {
        let bare = entry().history();
        assert_eq!(args_of(&bare.setup_command("p4")), ["configure", "history"]);

        let all = entry().history().allservers();
        assert_eq!(
            args_of(&all.setup_command("p4")),
            ["configure", "history", "allservers"]
        );

        let variable = entry().history().variable("monitor");
        assert_eq!(
            args_of(&variable.setup_command("p4")),
            ["configure", "history", "monitor"]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn patch_injects_options_in_syntax_order() {
        let command = entry().patch("server.allowfetch", "for DVCS").iteration(2);

        assert_eq!(
            args_of(&command.setup_command("p4")),
            [
                "configure",
                "history",
                "--iteration=2",
                "--comment=for DVCS",
                "server.allowfetch"
            ]
        );
        assert_eq!(command.get_iteration(), Some(2));
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn patch_without_iteration() {
        let command = entry().patch("server.allowfetch", "for DVCS");

        assert_eq!(
            args_of(&command.setup_command("p4")),
            [
                "configure",
                "history",
                "--comment=for DVCS",
                "server.allowfetch"
            ]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn help_injects_no_local_arguments() {
        let command = entry().help();

        assert_eq!(args_of(&command.setup_command("p4")), ["configure", "help"]);
    }

    #[test]
    fn global_opts_are_injected_before_the_subcommand() {
        let mut global_opts = GlobalOpts::default();
        global_opts.set_user("bruno");

        let command = Configure::new("p4", global_opts).set("monitor", "2");

        assert_eq!(
            args_of(&command.setup_command("p4")),
            ["-u", "bruno", "configure", "set", "monitor=2"]
        );
    }
}
