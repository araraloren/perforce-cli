use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::SubCommand;

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// List jobs and the changelists that fix them.
///
#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] fixes [-i -m max -j job -c changelist] [file[revRange] ...]`"
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] fixes [-i] [-m max] [-j job] [-c changelist] [file [revRange]...]`"
)]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2018_1")),
    doc = "`p4 [g-opts] fixes [-i] [-m max] [-j job] [-c changelist] [File Spec [revRange]...]`"
)]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "`p4 [g-opts] fixes [-i] [-m max] [-j job] [-c changelist] [FileSpec [revSpec]]`"
)]
///
/// After a job has been linked to a particular numbered changelist with
/// `p4 fix`, `p4 change`, or `p4 submit`, the job
#[cfg_attr(
    feature = "lt2018_1",
    doc = "is said to have been fixed by the changelist (even if the changelist is still pending)."
)]
#[cfg_attr(
    not(feature = "lt2018_1"),
    doc = "is fixed by the changelist (even if the changelist is still pending)."
)]
/// The `p4 fixes` command lists changelists and the jobs they fix.
///
/// If invoked without arguments, `p4 fixes` displays all fix records. Fix
/// records are displayed in the following format:
///
/// `jobname fixed by change changelist on date by user (status)`
///
/// You can limit the listed fixes by combining the following
#[cfg_attr(feature = "lt2014_2", doc = "flags")]
#[cfg_attr(not(feature = "lt2014_2"), doc = "options")]
/// when calling `p4 fixes`:
///
/// Use the `-c changelist` option to list only the jobs fixed by that
/// pending or submitted changelist.
///
/// Use the `-j job` option to list only those pending or submitted
/// changelists that fix that job.
///
/// Provide one or more file pattern arguments. If you provide a file
/// argument, only submitted changelists affecting files that match the file
#[cfg_attr(
    feature = "lt2018_1",
    doc = "patterns are listed; pending changelists are not included."
)]
#[cfg_attr(
    not(feature = "lt2018_1"),
    doc = "patterns are listed. Pending changelists are not included."
)]
/// If a revision specifier or revision range is included, only submitted
/// changelists that affected files at the given revisions are listed. You
/// can use the `-i`
#[cfg_attr(feature = "lt2014_2", doc = "flag")]
#[cfg_attr(not(feature = "lt2014_2"), doc = "option")]
/// with a file pattern argument to include fixes made by changelists that
/// were integrated into the specified files. Use the `-m max`
#[cfg_attr(feature = "lt2014_2", doc = "flag")]
#[cfg_attr(not(feature = "lt2014_2"), doc = "option")]
/// to limit the output to the first max fixes.
///
#[cfg_attr(
    all(not(feature = "lt2015_2"), feature = "lt2024_2"),
    doc = "",
    doc = "# Note",
    doc = ""
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2015_2")),
    doc = "This command now runs in lockless mode if `db.peeking` is set to `r`."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2018_1")),
    doc = "This command runs in lockless mode if `db.peeking` is set to `r`"
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "",
    doc = "This command runs in lockless mode if `db.peeking` is set to `r`"
)]
///
#[cfg_attr(feature = "lt2024_1", doc = "# Usage Notes")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "# Usage notes")]
///
/// Can File Arguments Use Revision Specifier? Yes
///
/// Can File Arguments Use Revision Range? Yes
///
/// Minimal Access Level Required: `list`
///
#[cfg_attr(
    not(feature = "lt2022_1"),
    doc = "The command returns no output if there is no object to report on."
)]
#[derive(Debug, Clone, Default)]
pub struct Fixes {
    bin: PathBuf,

    global_opts: GlobalOpts,

    include_integrations: bool,

    limit: Option<u64>,

    job_name: Option<String>,

    changelist: Option<String>,
}

impl Fixes {
    /// Creates a new `p4 fixes` command that lists all fix records.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
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
    #[cfg_attr(feature = "lt2015_1", doc = "-i files ...")]
    #[cfg_attr(not(feature = "lt2015_1"), doc = "-i")]
    ///
    /// Include fixes made by changelists that affected files integrated
    /// into the specified files.
    pub fn get_include_integrations(&self) -> bool {
        self.include_integrations
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2015_1", doc = "-i files ...")]
    #[cfg_attr(not(feature = "lt2015_1"), doc = "-i")]
    ///
    /// Include fixes made by changelists that affected files integrated
    /// into the specified files.
    pub fn set_include_integrations(&mut self, v: bool) -> &mut Self {
        self.include_integrations = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2015_1", doc = "-i files ...")]
    #[cfg_attr(not(feature = "lt2015_1"), doc = "-i")]
    ///
    /// Include fixes made by changelists that affected files integrated
    /// into the specified files.
    pub fn include_integrations(mut self, v: bool) -> Self {
        self.include_integrations = v;
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` fixes.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` fixes.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// List only the first `max` fixes.
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -j jobname
    ///
    /// Limit the displayed fixes to those that include the specified job.
    pub fn get_job_name(&self) -> Option<&String> {
        self.job_name.as_ref()
    }

    /// # Description
    ///
    /// -j jobname
    ///
    /// Limit the displayed fixes to those that include the specified job.
    pub fn set_job_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.job_name = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -j jobname
    ///
    /// Limit the displayed fixes to those that include the specified job.
    pub fn job_name(mut self, v: impl Into<String>) -> Self {
        self.job_name = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -c changelist
    ///
    /// Limit the displayed fixes to those that include the specified
    /// changelist.
    pub fn get_changelist(&self) -> Option<&String> {
        self.changelist.as_ref()
    }

    /// # Description
    ///
    /// -c changelist
    ///
    /// Limit the displayed fixes to those that include the specified
    /// changelist.
    pub fn set_changelist(&mut self, v: impl Into<String>) -> &mut Self {
        self.changelist = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -c changelist
    ///
    /// Limit the displayed fixes to those that include the specified
    /// changelist.
    pub fn changelist(mut self, v: impl Into<String>) -> Self {
        self.changelist = Some(v.into());
        self
    }
}

impl SubCommand for Fixes {
    fn name(&self) -> &str {
        "fixes"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.include_integrations {
            command.arg("-i");
        }

        if let Some(limit) = self.limit {
            command.arg("-m").arg(limit.to_string());
        }

        if let Some(job_name) = &self.job_name {
            command.arg("-j").arg(job_name);
        }

        if let Some(changelist) = &self.changelist {
            command.arg("-c").arg(changelist);
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl ParameterizedSpawn<()> for Fixes {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 fixes` without file arguments as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S, I> ParameterizedSpawn<(S,)> for Fixes
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 fixes` for the given file arguments as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (specs,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(specs)
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
        let mut command = Fixes::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    #[test]
    fn without_options() {
        let fixes = Fixes::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&fixes.setup_command("p4")), ["fixes"]);
    }

    #[test]
    fn all_options() {
        let mut fixes = Fixes::new("p4", GlobalOpts::new());
        fixes
            .set_include_integrations(true)
            .set_limit(5)
            .set_job_name("job000141")
            .set_changelist("201");

        assert_eq!(
            args_of(&fixes.setup_command("p4")),
            ["fixes", "-i", "-m", "5", "-j", "job000141", "-c", "201"]
        );
    }
}
