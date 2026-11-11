use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The changelist selected with `-c changelist`.
///
/// Entered with [`Fix::changelist`]; this is the only state that can be
/// spawned, because `-c changelist` is required by every `p4 fix` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changelist {
    changelist: String,
}

impl ExclusiveOption for Changelist {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-c").arg(&self.changelist);
    }
}

/// Marker for a selected `-c changelist`: the only state that can spawn.
pub trait ChangelistSelected: ExclusiveOption {}

impl ChangelistSelected for Changelist {}

/// Link jobs to the changelists that fix them.
///
/// `p4 [g-opts] fix [-d] [-s status] -c changelist jobName ...`
///
/// The `p4 fix` command links jobs (descriptions of work to be done) to a
/// changelist (a set of changes to files that does the work described by a
/// job).
///
/// If the changelist has not yet been submitted, the job appears on the
/// `p4 submit` or `p4 change` form for the changelist to which it's linked,
/// and under normal circumstances, the status of the job is changed to
/// `closed` when the changelist is submitted. If the changelist has already
/// been submitted when you run `p4 fix`, the job's status is changed to a
/// default status (typically `closed`) immediately.
///
/// To change a job status to something other than the default status
/// (typically `closed`) when you submit a changelist, supply the `-s` option
/// to `p4 fix`, `p4 submit`, or `p4 change`.
///
/// Because described work can be fixed over multiple changelists, one job
/// can be linked to multiple changelists. Because a single changelist might
/// fix ten bugs, multiple jobs can be linked to the same changelist. You can
/// do this in one command execution by providing multiple jobs as arguments
/// to `p4 fix`.
///
#[cfg_attr(feature = "lt2024_1", doc = "# Usage Notes")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "# Usage notes")]
///
/// Can File Arguments Use Revision Specifier? N/A
///
/// Can File Arguments Use Revision Range? N/A
///
/// Minimal Access Level Required: `open`
///
#[cfg_attr(
    feature = "lt2017_2",
    doc = "Because the format of jobs can be changed from site to site, it is possible that the jobs on your system no longer have a `Status:` field. If so, you can still link jobs to changelists with `p4 fix`, but Perforce will not change any of the job fields' values when the changelist is submitted."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2017_2")),
    doc = "Because the format of jobs can be changed from site to site, it is possible that the jobs on your system no longer have a `Status:` field. If so, you can still link jobs to changelists with `p4 fix`, but Helix Server will not change any of the job fields' values when the changelist is submitted."
)]
#[cfg_attr(
    all(feature = "lt2021_1", not(feature = "lt2019_1")),
    doc = "Because the format of jobs can be changed from site to site, it is possible that the jobs on your system no longer have a `Status:` field. If so, you can still link jobs to changelists with `p4 fix`, but Helix server will not change any of the job fields' values when the changelist is submitted."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2021_1")),
    doc = "Because the format of jobs can be changed from site to site, it is possible that the jobs on your system no longer have a `Status:` field. If so, you can still link jobs to changelists with `p4 fix`, but Helix Server will not change any of the job fields' values when the changelist is submitted."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2024_1")),
    doc = "Because the format of jobs can be changed from site to site, it is possible that the jobs on your system no longer have a `Status` field. If so, you can still link jobs to changelists with `p4 fix`, but Helix Core Server will not change any of the job fields' values when the changelist is submitted."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "Because the format of jobs can be changed from site to site, it is possible that the jobs on your system no longer have a `Status` field. If so, you can still link jobs to changelists with `p4 fix`, but P4 Server will not change any of the job fields' values when the changelist is submitted."
)]
///
/// You can change a fixed or unfixed job's status at any time by editing the
/// job with `p4 job`.
///
/// Another way to fix (or unfix) a job is to add it to (or delete it from)
/// the
#[cfg_attr(feature = "lt2024_1", doc = "`Jobs:` field")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "`Jobs` field")]
/// of an unsubmitted changelist's `p4 submit` or `p4 change` form.
///
/// You can't `p4 fix` a job to the default changelist; instead, add the job
/// to the
#[cfg_attr(feature = "lt2024_1", doc = "`Jobs:` field")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "`Jobs` field")]
/// of the default changelist's `p4 submit` form when submitting it to the
/// depot.
///
/// If you use `p4 fix -s status` on a job, and then use the `-s` option with
/// `p4 submit` or `p4 change`, the
#[cfg_attr(feature = "lt2024_1", doc = "`Jobs:` field")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "`Jobs` field")]
/// of the changelist's form will also require a status value (the default
/// value being the one specified by `p4 fix -s status`). The job(s) will be
/// assigned the specified status upon successful submission of the
/// changelist. If no status value is specified in the form, the error
/// message:
///
/// `Wrong number of words for field 'Jobs'.`
///
/// is displayed.
///
/// `p4 fix -s status`, `p4 submit -s`, and `p4 change -s` are intended for
/// use in conjunction with defect tracking systems.
///
/// Under normal circumstances, end users do not use these commands, and use
/// `p4 submit` and `p4 change` without the `-s` option. In this case, only
/// the job number is required in the
#[cfg_attr(
    feature = "lt2024_1",
    doc = "`Jobs:` field, and each job's status is set to a default value (typically `closed`) on completion of the submit."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "`Jobs` field, and each job's status is set to a default value (typically `closed`) on completion of the submit."
)]
///
/// The `C` type parameter tracks the required `-c changelist` selection at
/// compile time: it starts as [`Unselected`], and [`Fix::changelist`] moves
/// it to [`Changelist`]. Only the selected state implements
/// [`ParameterizedSpawn`], so a command without `-c changelist` cannot be
/// spawned.
#[derive(Debug, Clone, Default)]
pub struct Fix<C = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    changelist: C,

    delete_fix_record: bool,

    status: Option<String>,
}

impl Fix<Unselected> {
    /// Creates a new `p4 fix` command with no `-c changelist` selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. Use
    /// [`Self::changelist`] to select the required changelist before
    /// spawning.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// # Description
    ///
    /// -c changelist
    ///
    #[cfg_attr(not(feature = "lt2015_1"), doc = "The changelist to mark as fixed.")]
    ///
    /// Transitions this command to the [`Changelist`] state, which spawns.
    pub fn changelist(self, changelist: impl Into<String>) -> Fix<Changelist> {
        Fix {
            bin: self.bin,
            global_opts: self.global_opts,
            changelist: Changelist {
                changelist: changelist.into(),
            },
            delete_fix_record: self.delete_fix_record,
            status: self.status,
        }
    }
}

impl<C> Fix<C> {
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
    /// -d
    ///
    /// Delete the fix record for the specified job at the specified
    /// changelist. The job's status will not change.
    pub fn get_delete_fix_record(&self) -> bool {
        self.delete_fix_record
    }

    /// # Description
    ///
    /// -d
    ///
    /// Delete the fix record for the specified job at the specified
    /// changelist. The job's status will not change.
    pub fn set_delete_fix_record(&mut self, v: bool) -> &mut Self {
        self.delete_fix_record = v;
        self
    }

    /// # Description
    ///
    /// -d
    ///
    /// Delete the fix record for the specified job at the specified
    /// changelist. The job's status will not change.
    pub fn delete_fix_record(mut self, v: bool) -> Self {
        self.delete_fix_record = v;
        self
    }

    /// # Description
    ///
    /// -s status
    ///
    /// Upon submission of the changelist, change the job's status to
    /// `status`, rather than the default value `closed` (or some other value
    /// as defined in the `Presets:` of field 102 of the `p4 jobspec` form).
    ///
    /// If the changelist to which you're linking the job been submitted, the
    /// status value is immediately reflected in the job's status.
    ///
    /// If the changelist is `pending`, the job status is changed on
    /// submission of the changelist, provided that the
    #[cfg_attr(feature = "lt2014_2", doc = "`-s` flag")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "`-s` option")]
    /// is also supplied to `p4 submit` and the desired status appears next to
    /// the job in the `p4 submit` form's
    #[cfg_attr(feature = "lt2024_1", doc = "`Jobs:` field.")]
    #[cfg_attr(not(feature = "lt2024_1"), doc = "`Jobs` field.")]
    ///
    /// To leave a job unchanged, use the special status of `same`.
    pub fn get_status(&self) -> Option<&String> {
        self.status.as_ref()
    }

    /// # Description
    ///
    /// -s status
    ///
    /// Upon submission of the changelist, change the job's status to
    /// `status`, rather than the default value `closed` (or some other value
    /// as defined in the `Presets:` of field 102 of the `p4 jobspec` form).
    ///
    /// If the changelist to which you're linking the job been submitted, the
    /// status value is immediately reflected in the job's status.
    ///
    /// If the changelist is `pending`, the job status is changed on
    /// submission of the changelist, provided that the
    #[cfg_attr(feature = "lt2014_2", doc = "`-s` flag")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "`-s` option")]
    /// is also supplied to `p4 submit` and the desired status appears next to
    /// the job in the `p4 submit` form's
    #[cfg_attr(feature = "lt2024_1", doc = "`Jobs:` field.")]
    #[cfg_attr(not(feature = "lt2024_1"), doc = "`Jobs` field.")]
    ///
    /// To leave a job unchanged, use the special status of `same`.
    pub fn set_status(&mut self, v: impl Into<String>) -> &mut Self {
        self.status = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -s status
    ///
    /// Upon submission of the changelist, change the job's status to
    /// `status`, rather than the default value `closed` (or some other value
    /// as defined in the `Presets:` of field 102 of the `p4 jobspec` form).
    ///
    /// If the changelist to which you're linking the job been submitted, the
    /// status value is immediately reflected in the job's status.
    ///
    /// If the changelist is `pending`, the job status is changed on
    /// submission of the changelist, provided that the
    #[cfg_attr(feature = "lt2014_2", doc = "`-s` flag")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "`-s` option")]
    /// is also supplied to `p4 submit` and the desired status appears next to
    /// the job in the `p4 submit` form's
    #[cfg_attr(feature = "lt2024_1", doc = "`Jobs:` field.")]
    #[cfg_attr(not(feature = "lt2024_1"), doc = "`Jobs` field.")]
    ///
    /// To leave a job unchanged, use the special status of `same`.
    pub fn status(mut self, v: impl Into<String>) -> Self {
        self.status = Some(v.into());
        self
    }
}

impl<C: ExclusiveOption> SubCommand for Fix<C> {
    fn name(&self) -> &str {
        "fix"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.delete_fix_record {
            command.arg("-d");
        }

        if let Some(status) = &self.status {
            command.arg("-s").arg(status);
        }

        self.changelist.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<C: ChangelistSelected, S, I> ParameterizedSpawn<(S,)> for Fix<C>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 fix` for the given jobs as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with(&mut self, (jobs,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(jobs)
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
        let mut command = Fix::new("p4", GlobalOpts::new()).changelist("201");
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    /// Dry-run checks of the assembled `p4 fix` command line; no process is
    /// spawned.
    #[test]
    fn changelist_only() {
        let fix = Fix::new("p4", GlobalOpts::new()).changelist("201");

        assert_eq!(args_of(&fix.setup_command("p4")), ["fix", "-c", "201"]);
    }

    #[test]
    fn all_options() {
        let mut fix = Fix::new("p4", GlobalOpts::new());
        fix.set_delete_fix_record(true).set_status("suspended");
        let fix = fix.changelist("201");

        assert_eq!(
            args_of(&fix.setup_command("p4")),
            ["fix", "-d", "-s", "suspended", "-c", "201"]
        );
    }
}
