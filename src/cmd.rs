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
pub mod print;
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
pub use print::Print;
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

/// The default "no option selected" variant, shared by every
/// [`ExclusiveOption`] group.
///
/// It is a zero-sized type whose [`ExclusiveOption::inject_args`] is a no-op,
/// so any mutually exclusive option group can use it as its default type
/// parameter instead of defining its own empty variant.
#[derive(Debug, Clone, Copy, Default)]
pub struct Unselected;

impl ExclusiveOption for Unselected {
    fn inject_args(&self, _: &mut Command) {}
}

/// Marker trait for a mutually exclusive option group.
///
/// Some Perforce commands accept a set of options where only one may be
/// selected at a time (for example `p4 admin checkpoint [-z | -Z]` or
/// `p4 admin updatespecdepot [-a | -s type]`). Each variant of such a group
/// implements this trait to inject its own CLI arguments; the selected
/// variant is encoded in a type parameter so that the alternatives are
/// unavailable at compile time.
///
/// Variants may carry their own data (for example the `type` argument of
/// `-s type`) and inject any number of arguments, keeping the trait open to
/// option groups more complex than a single flag.
pub trait ExclusiveOption {
    #[allow(unused_variables)]
    /// Inject the CLI arguments corresponding to this selection into
    /// `command`.
    fn inject_args(&self, command: &mut Command) {}
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
