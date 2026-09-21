use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// `p4 [g-opts] aliases`: display command aliases that are currently defined
/// in a `.p4aliases` file.
#[derive(Debug, Clone, Default)]
pub struct Aliases {
    bin: PathBuf,

    global_opts: GlobalOpts,
}

impl SubCommand for Aliases {
    fn name(&self) -> &str {
        "aliases"
    }

    fn inject_local_args(&self, _: &mut Command) {}

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl ParameterizedSpawn for Aliases {
    type Input<'a> = ();
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 aliases` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with<'a>(&mut self, (): Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl Aliases {
    /// Creates a new `p4 aliases` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
        }
    }

    /// # Description
    ///
    /// g-opts
    ///
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run check of the assembled `p4 aliases` command line; no process is
    /// spawned.
    #[test]
    fn without_options() {
        let aliases = Aliases::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&aliases.setup_command("p4")), ["aliases"]);
    }

    #[test]
    fn with_global_opts() {
        let aliases = Aliases::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        );

        assert_eq!(
            args_of(&aliases.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "aliases"]
        );
    }
}
