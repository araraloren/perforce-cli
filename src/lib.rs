use std::path::{Path, PathBuf};

use crate::cmd::Add;
use crate::cmd::AdminEntry;
#[cfg(not(feature = "lt2016_1"))]
use crate::cmd::Aliases;
use crate::cmd::Annotate;
use crate::cmd::Archive;
use crate::cmd::Attribute;
#[cfg(not(feature = "lt2017_2"))]
use crate::cmd::BackGroundTask;
use crate::cmd::Branch;
use crate::cmd::Branches;
use crate::cmd::CachePurge;
use crate::cmd::Change;
use crate::cmd::Changes;
use crate::cmd::Clean;
use crate::cmd::Client;
use crate::cmd::Clients;
#[cfg(not(feature = "lt2015_1"))]
use crate::cmd::Clone;
use crate::cmd::Configure;
use crate::cmd::Copy;
use crate::cmd::Describe;
use crate::cmd::Diff;
use crate::cmd::Diff2;
use crate::cmd::Edit;
use crate::cmd::FileLog;
use crate::cmd::Print;
use crate::cmd::Sync;
use crate::cmd::Where;
use crate::global::GlobalOpts;

pub mod cmd;
pub mod global;
pub mod spawn;

pub mod prelude {
    pub use crate::cmd::*;
    pub use crate::global::*;
    pub use crate::spawn::*;
}

#[derive(Debug, Clone)]
pub struct P4Cli {
    bin: PathBuf,

    global_opts: GlobalOpts,
}

impl Default for P4Cli {
    fn default() -> Self {
        Self {
            bin: PathBuf::from("p4"),
            global_opts: GlobalOpts::default(),
        }
    }
}

impl P4Cli {
    /// Create a new Perforce command-line client.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
        }
    }

    /// Set the Perforce command path.
    pub fn bin(mut self, bin: impl Into<PathBuf>) -> Self {
        self.set_bin(bin);
        self
    }

    /// Set the global options.
    pub fn global_opts(mut self, global_opts: GlobalOpts) -> Self {
        self.set_global_opts(global_opts);
        self
    }

    /// Get the Perforce command path.
    pub fn get_bin(&self) -> &Path {
        &self.bin
    }

    /// Get the global options.
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// Set the Perforce command path.
    pub fn set_bin(&mut self, bin: impl Into<PathBuf>) {
        self.bin = bin.into();
    }

    /// Set the global options.
    pub fn set_global_opts(&mut self, global_opts: GlobalOpts) {
        self.global_opts = global_opts;
    }

    /// Open files in a client workspace for addition to the depot.
    pub fn cmd_add(&self) -> Add {
        Add::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Perform administrative operations on the server.
    pub fn cmd_admin(&self) -> AdminEntry {
        AdminEntry::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Display command aliases that are currently defined in a .p4aliases file.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn cmd_aliases(&self) -> Aliases {
        Aliases::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Print file lines along with their revisions.
    pub fn cmd_annotate(&self) -> Annotate {
        Annotate::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Archive obsolete revisions to an archive depot.
    pub fn cmd_archive(&self) -> Archive {
        Archive::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Set per-revision attributes on file revisions.
    pub fn cmd_attribute(&self) -> Attribute {
        Attribute::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Run a background command or trigger on the server.
    ///
    /// The returned builder starts with no task target; call
    /// [`BackGroundTask::execute`] (`-e`) or [`BackGroundTask::trigger`]
    /// (`-t`) to select the required target before spawning.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn cmd_bgtask(&self) -> BackGroundTask {
        BackGroundTask::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Create or edit a branch mapping and its view.
    pub fn cmd_branch(&self) -> Branch {
        Branch::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List existing branch mappings.
    pub fn cmd_branches(&self) -> Branches {
        Branches::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Reclaim disk space on a replicated server.
    ///
    /// The returned builder starts with no purge goal and has no effect;
    /// call [`CachePurge::purge_all`] (`-a`), [`CachePurge::free_space`]
    /// (`-f`), [`CachePurge::revision_count`] (`-m`), or
    /// [`CachePurge::size`] (`-s`) to select the required goal before
    /// spawning.
    pub fn cmd_cachepurge(&self) -> CachePurge {
        CachePurge::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Restore client workspace files to match the state of corresponding
    /// depot files.
    ///
    /// Without file arguments, the files in your local working directory are
    /// used; pass file arguments to
    /// [`ParameterizedSpawn::spawn_with`](crate::spawn::ParameterizedSpawn::spawn_with)
    /// to limit the scope.
    pub fn cmd_clean(&self) -> Clean {
        Clean::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Create or edit a client workspace specification and its view.
    ///
    /// The returned builder starts in the [`GeneralMode`](crate::cmd::client::GeneralMode)
    /// state (`p4 client [-f] [-t template] [-T type] [name]`); call
    /// transition methods such as [`Client::delete`](crate::cmd::client::Client::delete)
    /// or [`Client::stdout`](crate::cmd::client::Client::stdout) to select
    /// another syntax form.
    pub fn cmd_client(&self) -> Client {
        Client::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List all client workspaces currently known to the system.
    ///
    /// The returned builder starts with no option selected and spawns the
    /// bare `p4 clients` report as is; set any regular option or filter to
    /// enter the [`RegularMode`](crate::cmd::clients::RegularMode) state, or
    /// call [`Clients::unloaded`](crate::cmd::clients::Clients::unloaded)
    /// (`-U`) to list only unloaded client workspaces.
    pub fn cmd_clients(&self) -> Clients {
        Clients::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List all client workspaces currently known to the system.
    ///
    /// This is an alias for [`Self::cmd_clients`], corresponding to the `p4
    /// workspaces` command.
    pub fn cmd_workspaces(&self) -> Clients {
        Clients::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Clone a new local server from a remote server.
    ///
    /// The returned builder cannot be assembled until a remote port is
    /// selected with [`Clone::port`](crate::cmd::clone::Clone::port) (`-p`)
    /// and exactly one clone target is selected with
    /// [`Clone::remote`](crate::cmd::clone::Clone::remote) (`-r`)
    /// or [`Clone::file`](crate::cmd::clone::Clone::file) (`-f`).
    /// Unlike most commands, `p4 clone` does not accept the global options.
    #[cfg(not(feature = "lt2015_1"))]
    pub fn cmd_clone(&self) -> Clone {
        Clone::new(self.bin.clone())
    }

    /// Set, view, and manage server configuration variables.
    ///
    /// The returned command starts with no operation selected; call
    /// [`Configure::set`], [`Configure::unset`], or
    /// [`Configure::show`] to select the operation.
    #[cfg_attr(not(feature = "lt2019_2"), doc = "")]
    #[cfg_attr(
        not(feature = "lt2019_2"),
        doc = "The [`Configure::history`](Configure::history) operation selects the `p4 configure history` report."
    )]
    #[cfg_attr(not(feature = "lt2023_2"), doc = "")]
    #[cfg_attr(
        not(feature = "lt2023_2"),
        doc = "The [`Configure::patch`](Configure::patch) operation retroactively sets a history comment, and [`Configure::help`](Configure::help) prints the configurables reference."
    )]
    pub fn cmd_configure(&self) -> Configure {
        Configure::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Copy files and/or the stream spec from one location in the depot to
    /// another.
    pub fn cmd_copy(&self) -> Copy {
        Copy::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Create or edit a client workspace specification and its view.
    ///
    /// This is an alias for [`Self::cmd_client`], corresponding to the `p4
    /// workspace` command.
    pub fn cmd_workspace(&self) -> Client {
        Client::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Create or edit a changelist specification.
    pub fn cmd_change(&self) -> Change {
        Change::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Create or edit a changelist specification.
    ///
    /// This is an alias for [`Self::cmd_change`], corresponding to the `p4
    /// changelist` command.
    pub fn cmd_changelist(&self) -> Change {
        Change::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List submitted and pending changelists.
    pub fn cmd_changes(&self) -> Changes {
        Changes::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List submitted and pending changelists.
    ///
    /// This is an alias for [`Self::cmd_changes`], corresponding to the `p4
    /// changelists` command.
    pub fn cmd_changelists(&self) -> Changes {
        Changes::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Display the details of one or more changelists.
    pub fn cmd_describe(&self) -> Describe {
        Describe::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Diff workspace content against depot content, or diff stream specs.
    pub fn cmd_diff(&self) -> Diff {
        Diff::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Compare the content at two depot paths, or compare two stream specs.
    pub fn cmd_diff2(&self) -> Diff2 {
        Diff2::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Open files in a client workspace for edit.
    pub fn cmd_edit(&self) -> Edit {
        Edit::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Print detailed information about the revisions of files.
    pub fn cmd_filelog(&self) -> FileLog {
        FileLog::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Print the contents of depot file revisions.
    pub fn cmd_print(&self) -> Print {
        Print::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Update the client workspace to reflect the contents of the depot.
    pub fn cmd_sync(&self) -> Sync {
        Sync::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Show where a particular file is located, as determined by the client view.
    pub fn cmd_where(&self) -> Where {
        Where::new(self.bin.clone(), self.global_opts.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uses_p4_from_path() {
        let p4 = P4Cli::default();

        assert_eq!(p4.get_bin(), Path::new("p4"));
    }

    #[test]
    fn accepts_custom_bin_path() {
        let p4 = P4Cli::new("C:\\tools\\p4.exe", GlobalOpts::default());

        assert_eq!(p4.get_bin(), Path::new("C:\\tools\\p4.exe"));
    }
}
