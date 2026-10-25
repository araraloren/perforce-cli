use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// `p4 [g-opts] dbschema [tablename[:tableversion]]`
#[cfg_attr(
    not(feature = "lt2020_2"),
    doc = "",
    doc = "`p4 [g-opts] dbschema [-A] [tablename] ...`"
)]
///
/// Reports information about the database structure in which the server
/// stores metadata. The command is intended for systems integrators and
/// returns tagged output.
///
#[cfg_attr(
    feature = "lt2020_2",
    doc = "Spawn with at most one `tablename`, optionally followed by `:tableversion`; by default all current tables are reported."
)]
#[cfg_attr(
    not(feature = "lt2020_2"),
    doc = "Spawn with a list of `tablename` entries, each optionally followed by `:tableversion`; by default all current tables are reported. Use [`Self::all_versions`] to supply `-A` and report all versions instead."
)]
#[derive(Debug, Clone, Default)]
pub struct DBSchema {
    bin: PathBuf,

    global_opts: GlobalOpts,

    #[cfg(not(feature = "lt2020_2"))]
    all_versions: bool,
}

impl DBSchema {
    /// Creates a new `p4 dbschema` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            #[cfg(not(feature = "lt2020_2"))]
            all_versions: false,
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

    /// # Description
    ///
    /// -A
    ///
    /// Display information about all versions.
    #[cfg(not(feature = "lt2020_2"))]
    pub fn get_all_versions(&self) -> bool {
        self.all_versions
    }

    /// # Description
    ///
    /// -A
    ///
    /// Display information about all versions.
    #[cfg(not(feature = "lt2020_2"))]
    pub fn set_all_versions(&mut self, v: bool) -> &mut Self {
        self.all_versions = v;
        self
    }

    /// # Description
    ///
    /// -A
    ///
    /// Display information about all versions.
    #[cfg(not(feature = "lt2020_2"))]
    pub fn all_versions(mut self, v: bool) -> Self {
        self.all_versions = v;
        self
    }
}

impl SubCommand for DBSchema {
    fn name(&self) -> &str {
        "dbschema"
    }

    fn inject_local_args(&self, _: &mut Command) {}

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// No table arguments.
impl ParameterizedSpawn<()> for DBSchema {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbschema` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        let mut command = self.setup_command(&self.bin);
        #[cfg(not(feature = "lt2020_2"))]
        if self.all_versions {
            command.arg("-A");
        }
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// 2020.2+: a list of tables.
#[cfg(not(feature = "lt2020_2"))]
impl<S, I> ParameterizedSpawn<(S,)> for DBSchema
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbschema` for the given tables as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (tables,): (S,)) -> Result<Self::Output, Self::Error> {
        let mut command = self.setup_command(&self.bin);
        if self.all_versions {
            command.arg("-A");
        }
        command
            .args(tables)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// Up to 2020.1: at most one table.
#[cfg(feature = "lt2020_2")]
impl<I: AsRef<OsStr>> ParameterizedSpawn<(I,)> for DBSchema {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbschema` for the given table as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (table,): (I,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .arg(table)
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
        let mut command = DBSchema::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 dbschema` command line; no process
    /// is spawned.
    #[test]
    fn without_arguments() {
        let dbschema = DBSchema::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&dbschema.setup_command("p4")), ["dbschema"]);
    }

    #[cfg(not(feature = "lt2020_2"))]
    #[test]
    fn all_versions_accessors() {
        let mut dbschema = DBSchema::new("p4", GlobalOpts::new());
        assert!(!dbschema.get_all_versions());

        dbschema.set_all_versions(true);
        assert!(dbschema.get_all_versions());

        let dbschema = dbschema.all_versions(false);
        assert!(!dbschema.get_all_versions());
    }

    #[test]
    fn global_opts_are_injected_once() {
        let dbschema = DBSchema::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        );

        assert_eq!(
            args_of(&dbschema.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "dbschema"]
        );
    }
}
