use std::path::{Path, PathBuf};

use crate::cmd::Add;
use crate::cmd::AdminEntry;
#[cfg(not(feature = "lt2016_1"))]
use crate::cmd::Aliases;
use crate::cmd::Annotate;
use crate::cmd::Archive;
use crate::cmd::Attribute;
use crate::cmd::Changes;
use crate::cmd::Edit;
use crate::cmd::FileLog;
use crate::global::GlobalOpts;

pub mod cmd;
pub mod global;

pub mod prelude {
    pub use crate::cmd::*;
    pub use crate::global::*;
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
    pub fn add(&self) -> Add {
        Add::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Perform administrative operations on the server.
    pub fn admin(&self) -> AdminEntry {
        AdminEntry::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Display command aliases that are currently defined in a .p4aliases file.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn aliases(&self) -> Aliases {
        Aliases::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Print file lines along with their revisions.
    pub fn annotate(&self) -> Annotate {
        Annotate::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Archive obsolete revisions to an archive depot.
    pub fn archive(&self) -> Archive {
        Archive::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Set per-revision attributes on file revisions.
    pub fn attribute(&self) -> Attribute {
        Attribute::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List submitted and pending changelists.
    pub fn changes(&self) -> Changes {
        Changes::new(self.bin.clone(), self.global_opts.clone())
    }

    /// List submitted and pending changelists.
    ///
    /// This is an alias for [`Self::changes`], corresponding to the `p4
    /// changelists` command.
    pub fn changelists(&self) -> Changes {
        Changes::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Open files in a client workspace for edit.
    pub fn edit(&self) -> Edit {
        Edit::new(self.bin.clone(), self.global_opts.clone())
    }

    /// Print detailed information about the revisions of files.
    pub fn filelog(&self) -> FileLog {
        FileLog::new(self.bin.clone(), self.global_opts.clone())
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
