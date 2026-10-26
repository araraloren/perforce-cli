use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] dbverify [ -t db.tablename ] [-U]`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] dbverify [-t db.tablename] [-U]`"
)]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "`p4 [g-opts] dbverify [-t db.tablename] [-U][-v]`"
)]
///
/// Perform low-level verification of the database tables.
///
/// By default, all current tables are verified, which can be computationally
/// expensive and might require scheduled user downtime on large systems;
/// [`Self::table`] restricts verification to a single table.
/// [`Self::less_detailed`] performs the faster `-U` validation, which only
/// looks for tables with non-zero unlock counts and has minimal performance
/// impact.
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "",
    doc = "Use [`Self::verbose`] to request verbose information on the verification."
)]
#[derive(Debug, Clone, Default)]
pub struct DBVerify {
    bin: PathBuf,

    global_opts: GlobalOpts,

    table: Option<String>,

    less_detailed: bool,

    #[cfg(not(feature = "lt2016_1"))]
    verbose: bool,
}

impl DBVerify {
    /// Creates a new `p4 dbverify` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            table: None,
            less_detailed: false,
            #[cfg(not(feature = "lt2016_1"))]
            verbose: false,
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
    #[cfg_attr(feature = "lt2015_1", doc = "-t tablename")]
    #[cfg_attr(not(feature = "lt2015_1"), doc = "-t db.tablename")]
    ///
    /// Restrict verification to the specified table name.
    pub fn get_table(&self) -> Option<&str> {
        self.table.as_deref()
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2015_1", doc = "-t tablename")]
    #[cfg_attr(not(feature = "lt2015_1"), doc = "-t db.tablename")]
    ///
    /// Restrict verification to the specified table name.
    pub fn set_table(&mut self, v: impl Into<String>) -> &mut Self {
        self.table = Some(v.into());
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2015_1", doc = "-t tablename")]
    #[cfg_attr(not(feature = "lt2015_1"), doc = "-t db.tablename")]
    ///
    /// Restrict verification to the specified table name.
    pub fn table(mut self, v: impl Into<String>) -> Self {
        self.set_table(v);
        self
    }

    /// # Description
    ///
    /// -U
    ///
    /// Perform a less-detailed validation.
    ///
    /// Looks for tables with non-zero unlock counts; the presence of a
    /// non-zero unlock count does not positively indicate corruption (and a
    /// zero unlock count does not guarantee data integrity), but this check
    /// has minimal performance impact.
    pub fn get_less_detailed(&self) -> bool {
        self.less_detailed
    }

    /// # Description
    ///
    /// -U
    ///
    /// Perform a less-detailed validation.
    ///
    /// Looks for tables with non-zero unlock counts; the presence of a
    /// non-zero unlock count does not positively indicate corruption (and a
    /// zero unlock count does not guarantee data integrity), but this check
    /// has minimal performance impact.
    pub fn set_less_detailed(&mut self, v: bool) -> &mut Self {
        self.less_detailed = v;
        self
    }

    /// # Description
    ///
    /// -U
    ///
    /// Perform a less-detailed validation.
    ///
    /// Looks for tables with non-zero unlock counts; the presence of a
    /// non-zero unlock count does not positively indicate corruption (and a
    /// zero unlock count does not guarantee data integrity), but this check
    /// has minimal performance impact.
    pub fn less_detailed(mut self, v: bool) -> Self {
        self.set_less_detailed(v);
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Provide verbose information on the verification.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_verbose(&self) -> bool {
        self.verbose
    }

    /// # Description
    ///
    /// -v
    ///
    /// Provide verbose information on the verification.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_verbose(&mut self, v: bool) -> &mut Self {
        self.verbose = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Provide verbose information on the verification.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn verbose(mut self, v: bool) -> Self {
        self.set_verbose(v);
        self
    }
}

impl SubCommand for DBVerify {
    fn name(&self) -> &str {
        "dbverify"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(table) = &self.table {
            command.arg("-t").arg(table);
        }

        if self.less_detailed {
            command.arg("-U");
        }

        #[cfg(not(feature = "lt2016_1"))]
        if self.verbose {
            command.arg("-v");
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl ParameterizedSpawn<()> for DBVerify {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dbverify` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
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
        let mut command = DBVerify::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 dbverify` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let dbverify = DBVerify::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&dbverify.setup_command("p4")), ["dbverify"]);
    }

    #[test]
    fn with_table() {
        let dbverify = DBVerify::new("p4", GlobalOpts::new()).table("db.have");

        assert_eq!(dbverify.get_table(), Some("db.have"));
        assert_eq!(
            args_of(&dbverify.setup_command("p4")),
            ["dbverify", "-t", "db.have"]
        );
    }

    #[test]
    fn less_detailed_flag() {
        let dbverify = DBVerify::new("p4", GlobalOpts::new()).less_detailed(true);

        assert!(dbverify.get_less_detailed());
        assert_eq!(args_of(&dbverify.setup_command("p4")), ["dbverify", "-U"]);
    }

    #[test]
    fn table_and_less_detailed_in_syntax_order() {
        let mut dbverify = DBVerify::new("p4", GlobalOpts::new()).less_detailed(true);
        dbverify.set_table("db.have");

        assert_eq!(
            args_of(&dbverify.setup_command("p4")),
            ["dbverify", "-t", "db.have", "-U"]
        );
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn verbose_flag() {
        let dbverify = DBVerify::new("p4", GlobalOpts::new()).verbose(true);

        assert!(dbverify.get_verbose());
        assert_eq!(args_of(&dbverify.setup_command("p4")), ["dbverify", "-v"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn all_options_in_syntax_order() {
        let dbverify = DBVerify::new("p4", GlobalOpts::new())
            .table("db.have")
            .less_detailed(true)
            .verbose(true);

        assert_eq!(
            args_of(&dbverify.setup_command("p4")),
            ["dbverify", "-t", "db.have", "-U", "-v"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let dbverify = DBVerify::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .table("db.have");

        assert_eq!(
            args_of(&dbverify.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "dbverify", "-t", "db.have"]
        );
    }
}
