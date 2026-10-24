use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] counters [-e nameFilter -m max]`"
)]
#[cfg_attr(
    not(feature = "lt2015_1"),
    doc = "`p4 [g-opts] counters [-e nameFilter] [-m max]`"
)]
///
/// Display the list of long-term variables used by P4 Server and associated
/// scripts.
///
/// The bare command reports all counters and their values; [`Self::filter`]
/// restricts the report to counters whose names match a pattern, and
/// [`Self::limit`] caps the number of counters listed.
#[derive(Debug, Clone, Default)]
pub struct Counters {
    bin: PathBuf,

    global_opts: GlobalOpts,

    filter: Option<String>,

    limit: Option<u64>,
}

impl Counters {
    /// Creates a new `p4 counters` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            filter: None,
            limit: None,
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
    /// -e nameFilter
    ///
    /// List counters with a name that matches the `nameFilter` pattern, for
    /// example: `p4 counters -e 'mycounter-*'`
    pub fn get_filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }

    /// # Description
    ///
    /// -e nameFilter
    ///
    /// List counters with a name that matches the `nameFilter` pattern, for
    /// example: `p4 counters -e 'mycounter-*'`
    pub fn set_filter(&mut self, v: impl Into<String>) -> &mut Self {
        self.filter = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -e nameFilter
    ///
    /// List counters with a name that matches the `nameFilter` pattern, for
    /// example: `p4 counters -e 'mycounter-*'`
    pub fn filter(mut self, v: impl Into<String>) -> Self {
        self.set_filter(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` counters.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` counters.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` counters.
    pub fn limit(mut self, v: u64) -> Self {
        self.set_limit(v);
        self
    }
}

impl SubCommand for Counters {
    fn name(&self) -> &str {
        "counters"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(filter) = &self.filter {
            command.arg("-e").arg(filter);
        }

        if let Some(max) = self.limit {
            command.arg("-m").arg(max.to_string());
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl ParameterizedSpawn<()> for Counters {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 counters` as a child process with piped standard output and
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
        let mut command = Counters::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 counters` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let counters = Counters::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&counters.setup_command("p4")), ["counters"]);
    }

    #[test]
    fn with_filter() {
        let counters = Counters::new("p4", GlobalOpts::new()).filter("mycounter-*");

        assert_eq!(counters.get_filter(), Some("mycounter-*"));
        assert_eq!(
            args_of(&counters.setup_command("p4")),
            ["counters", "-e", "mycounter-*"]
        );
    }

    #[test]
    fn with_limit() {
        let counters = Counters::new("p4", GlobalOpts::new()).limit(5);

        assert_eq!(counters.get_limit(), Some(5));
        assert_eq!(
            args_of(&counters.setup_command("p4")),
            ["counters", "-m", "5"]
        );
    }

    #[test]
    fn filter_and_limit_in_syntax_order() {
        let mut counters = Counters::new("p4", GlobalOpts::new()).limit(5);
        counters.set_filter("mycounter-*");

        assert_eq!(
            args_of(&counters.setup_command("p4")),
            ["counters", "-e", "mycounter-*", "-m", "5"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let counters = Counters::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .filter("mycounter-*");

        assert_eq!(
            args_of(&counters.setup_command("p4")),
            [
                "-p",
                "localhost:1666",
                "-q",
                "counters",
                "-e",
                "mycounter-*"
            ]
        );
    }
}
