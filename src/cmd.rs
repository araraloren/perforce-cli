pub mod add;
pub mod admin;
#[cfg(not(feature = "lt2016_1"))]
pub mod aliases;
pub mod annotate;
pub mod archive;
pub mod attribute;
pub mod changes;
pub mod describe;
pub mod edit;
pub mod filelog;
pub mod r#where;

pub use add::Add;
pub use admin::AdminEntry;
#[cfg(not(feature = "lt2016_1"))]
pub use aliases::Aliases;
pub use annotate::Annotate;
pub use archive::Archive;
pub use attribute::Attribute;
pub use changes::Changes;
pub use describe::Describe;
pub use edit::Edit;
pub use filelog::FileLog;
pub use r#where::Where;

use std::{ffi::OsStr, process::Command};

use crate::global::GlobalOpts;

/// Long output mode for changelist descriptions shared by commands such as
/// `p4 changes` and `p4 filelog`.
///
/// By default, only the first 30 (or 31) characters of the description are
/// shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongOutput {
    /// Show the full text of each changelist description (`-l`).
    Default,
    /// Show the full text truncated at 250 characters (`-L`).
    Truncated,
}

impl LongOutput {
    /// Returns the command-line flag for this long output mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            LongOutput::Default => "-l",
            LongOutput::Truncated => "-L",
        }
    }
}

pub trait SubCommand {
    fn name(&self) -> &str;

    fn inject_local_args(&self, command: &mut Command);

    fn global_opts(&self) -> Option<&GlobalOpts> {
        None
    }

    fn inject_args(&self, command: &mut Command) {
        // inject global opts
        if let Some(global_opts) = self.global_opts() {
            global_opts.setup_args(command);
        };
        // inject local opts
        self.inject_local_args(command.arg(self.name()));
    }

    fn setup_command<S: AsRef<OsStr>>(&self, bin: S) -> Command {
        let mut cmd = Command::new(bin);

        self.inject_args(&mut cmd);
        cmd
    }
}

/// Test-only helper: collects the arguments assembled on a [`Command`] as
/// strings for easy comparison.
#[cfg(test)]
pub(crate) fn args_of(command: &Command) -> Vec<String> {
    command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}
