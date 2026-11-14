use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The unload-depot form of `p4 files` (`-U`).
///
/// This mode is mutually exclusive with the stream-components form. Pass the
/// unload file specification to [`ParameterizedSpawn::spawn_with`].
#[derive(Debug, Clone, Copy, Default)]
pub struct UnloadDepot {}

impl ExclusiveOption for UnloadDepot {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-U");
    }
}

/// The stream-components form of `p4 files` (`--streamviews`).
///
/// This mode is mutually exclusive with [`UnloadDepot`].
#[cfg(not(feature = "lt2022_2"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct StreamView {}

#[cfg(not(feature = "lt2022_2"))]
impl ExclusiveOption for StreamView {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--streamviews");
    }
}

/// Provide information about files in the depot without accessing their
/// contents.
///
#[cfg_attr(
    feature = "lt2015_1",
    doc = "`p4 [g-opts] files [-a] [-A] [-e] [-m max] file[revRange] ...`",
    doc = "",
    doc = "`p4 [g-opts] files -U unloadfile ...`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] files [-a -A -e] [-m max] file[revRange]...`",
    doc = "",
    doc = "`p4 [g-opts] files -U unloadfile ...`"
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] files [-a -A -e] [-i][-m max] file[revRange]...`",
    doc = "",
    doc = "`p4 [g-opts] files -U unloadfile ...`"
)]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2018_1")),
    doc = "`p4 [g-opts] files [-a -A -e] [-i][-m max] [[File Spec][revSpec]]`",
    doc = "",
    doc = "`p4 [g-opts] files -U unload[File Spec]`"
)]
#[cfg_attr(
    all(feature = "lt2026_1", not(feature = "lt2022_2")),
    doc = "`p4 [g-opts] files [-a -A -e] [-i][-m max] FileSpec[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] files [-a] [-e] [-m max] [--streamviews] FileSpec[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] files -U unload[FileSpec]`"
)]
#[cfg_attr(
    not(feature = "lt2026_1"),
    doc = "`p4 [g-opts] files [-a -A -e] [-i] [-m max] FileSpec[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] files [-a] [-e] [-m max] [--streamviews] FileSpec[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] files -U unload[FileSpec]`",
    doc = "",
    doc = "`p4 [g-opts] files [-a -A -e -i -m max] [[-U] | [--streamviews]] [--ignore-changeview] FileSpec[revSpec]`"
)]
///
#[cfg_attr(
    all(not(feature = "lt2017_1"), feature = "lt2018_1"),
    doc = "",
    doc = "# Note",
    doc = ""
)]
#[cfg_attr(
    all(not(feature = "lt2017_1"), feature = "lt2017_2"),
    doc = "The output is different for files in depots of type `graph`. See the final example in the section Examples."
)]
#[cfg_attr(
    all(not(feature = "lt2017_2"), feature = "lt2018_1"),
    doc = "The output is different for files in depots of type `graph`. See the final example in the section Examples and for syntax, at the command-line, type `p4 help-graph files`"
)]
///
/// This command lists each file that matches the file patterns provided as
/// arguments. If a revision specifier is given, the files are described at
/// the given revision. One file is listed per line, and the format of each
/// line is:
///
/// `depot-file-location#rev - action change changelist (filetype)`
///
/// where:
///
#[cfg_attr(
    feature = "lt2014_2",
    doc = "- *depot-file-location* is the file's location relative to the top of the depot"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2014_2")),
    doc = "- *depot-file-location* is the file's location relative to the top of the depot,"
)]
#[cfg_attr(
    not(feature = "lt2016_1"),
    doc = "- *depot-file-location* is the file’s location relative to the top of the depot,"
)]
///
#[cfg_attr(
    feature = "lt2014_2",
    doc = "- *rev* is the revision number of the head revision of that file"
)]
#[cfg_attr(
    not(feature = "lt2014_2"),
    doc = "- *rev* is the revision number of the head revision of that file,"
)]
///
/// - *action* is the action taken at the head revision: `add`, `edit`,
///   `delete`, `branch`, `move/add`, `move/delete`, `integrate`, `import`,
#[cfg_attr(feature = "lt2014_2", doc = "  `purge`, or `archive`.")]
#[cfg_attr(not(feature = "lt2014_2"), doc = "  `purge`, or `archive`,")]
///
/// - *changelist* is the number of the changelist in which the revision was
///   submitted, and
///
/// - *filetype* is the
#[cfg_attr(feature = "lt2017_2", doc = "Perforce")]
#[cfg_attr(
    all(not(feature = "lt2017_2"), feature = "lt2019_1"),
    doc = "Helix Server"
)]
#[cfg_attr(
    all(not(feature = "lt2019_1"), feature = "lt2021_1"),
    doc = "Helix server"
)]
#[cfg_attr(
    all(not(feature = "lt2021_1"), feature = "lt2024_1"),
    doc = "Helix Server"
)]
#[cfg_attr(
    all(not(feature = "lt2024_1"), feature = "lt2024_2"),
    doc = "Helix Core Server"
)]
#[cfg_attr(not(feature = "lt2024_2"), doc = "P4 Server")]
/// file type of this file at the head revision.
///
/// Unlike most
#[cfg_attr(feature = "lt2017_2", doc = "Perforce")]
#[cfg_attr(
    all(not(feature = "lt2017_2"), feature = "lt2019_1"),
    doc = "Helix Server"
)]
#[cfg_attr(
    all(not(feature = "lt2019_1"), feature = "lt2021_1"),
    doc = "Helix server"
)]
#[cfg_attr(
    all(not(feature = "lt2021_1"), feature = "lt2024_1"),
    doc = "Helix Server"
)]
#[cfg_attr(
    all(not(feature = "lt2024_1"), feature = "lt2024_2"),
    doc = "Helix Core Server"
)]
#[cfg_attr(not(feature = "lt2024_2"), doc = "P4 Server")]
#[cfg_attr(
    feature = "lt2018_1",
    doc = "commands, `p4 files` reports on any file in the depot; it is not limited to only those files that are visible through the client view."
)]
#[cfg_attr(
    not(feature = "lt2018_1"),
    doc = "commands, `p4 files` reports on any file in the depot. It is not limited to only those files that are visible through the client view."
)]
/// If a file pattern on the command line is given in client syntax, only files
/// in the client workspace are shown.
///
/// The `M` type parameter isolates [`UnloadDepot`] (`-U`) from
#[cfg_attr(feature = "lt2022_2", doc = "the regular form at compile time.")]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "[`StreamView`] (`--streamviews`) at compile time."
)]
///
#[cfg_attr(feature = "lt2024_1", doc = "# Usage Notes")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "# Usage notes")]
///
#[cfg_attr(
    feature = "lt2026_1",
    doc = "Can File Arguments Use Revision Specifier? Yes",
    doc = "",
    doc = "Can File Arguments Use Revision Range? Yes",
    doc = "",
    doc = "Minimal Access Level Required: `list`"
)]
#[cfg_attr(
    not(feature = "lt2026_1"),
    doc = "Can file arguments use revision specifier? Yes",
    doc = "",
    doc = "Can file arguments use revision range? Yes",
    doc = "",
    doc = "Minimal access level required: `list`"
)]
///
#[cfg_attr(
    feature = "lt2018_1",
    doc = "The specified revision can be a revision range; in this case, only those files with revisions within the specified range are listed, and by default, only the highest revision in that range is listed. (To display information for all files within a revision range, use `p4 files -a`.)"
)]
#[cfg_attr(
    not(feature = "lt2018_1"),
    doc = "The specified revision can be a revision range. In this case, only those files with revisions within the specified range are listed, and by default, only the highest revision in that range is listed. (To display information for all files within a revision range, use `p4 files -a`.)"
)]
///
#[cfg_attr(
    feature = "lt2024_1",
    doc = "Because the output of `p4 files` can be quite large when called with highly non-restrictive file arguments (for example, `p4 files //depot/...` prints information about all the files in the depot), it may be subject to a `maxresults` limitation as set in `p4 group`."
)]
#[cfg_attr(
    not(feature = "lt2024_1"),
    doc = "The output of `p4 files` can be large when called with non-restrictive file arguments. For example, `p4 files //depot/...` prints information about all the files in the depot. However, `p4 files` commands can be subject to a `maxresults` limitation that is set in `p4 group`."
)]
#[derive(Debug, Clone, Default)]
pub struct Files<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    all: bool,

    archive_depot_only: bool,

    exclude_deleted: bool,

    #[cfg(not(feature = "lt2016_1"))]
    case_insensitive: bool,

    limit: Option<u64>,

    #[cfg(not(feature = "lt2025_2"))]
    ignore_changeview: bool,

    mode: M,
}

impl Files<Unselected> {
    /// Creates a new `p4 files` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// # Description
    ///
    /// -U unloadfile
    ///
    /// List only files in the unload depot. See `p4 unload` for details.
    ///
    /// Transitions this command to the [`UnloadDepot`] state. This is
    /// mutually exclusive with the stream-components form.
    pub fn unload_depot(self) -> Files<UnloadDepot> {
        Files {
            bin: self.bin,
            global_opts: self.global_opts,
            all: self.all,
            archive_depot_only: self.archive_depot_only,
            exclude_deleted: self.exclude_deleted,
            #[cfg(not(feature = "lt2016_1"))]
            case_insensitive: self.case_insensitive,
            limit: self.limit,
            #[cfg(not(feature = "lt2025_2"))]
            ignore_changeview: self.ignore_changeview,
            mode: UnloadDepot::default(),
        }
    }

    /// # Description
    ///
    /// `--streamviews`
    ///
    /// Report files and directories from stream Paths and Components, where
    /// imported files are represented as symbolic links under the stream depot
    /// path.
    #[cfg_attr(
        all(not(feature = "lt2022_2"), feature = "lt2024_1"),
        doc = "See the Viewing imported files example under Stream Components > in Helix Core Command-Line (P4) Guide."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_1"), feature = "lt2024_2"),
        doc = "See the Viewing imported files example under Stream Components > in the Helix Core Command-Line (P4) Guide."
    )]
    #[cfg_attr(
        all(not(feature = "lt2024_2"), feature = "lt2025_2"),
        doc = "See the Viewing imported files example under Stream Components > in the P4 CLI Documentation."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "See the Viewing imported files example under Stream components."
    )]
    ///
    /// Transitions this command to the [`StreamView`] state. This is mutually
    /// exclusive with the unload-depot form.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn stream_view(self) -> Files<StreamView> {
        Files {
            bin: self.bin,
            global_opts: self.global_opts,
            all: self.all,
            archive_depot_only: self.archive_depot_only,
            exclude_deleted: self.exclude_deleted,
            #[cfg(not(feature = "lt2016_1"))]
            case_insensitive: self.case_insensitive,
            limit: self.limit,
            #[cfg(not(feature = "lt2025_2"))]
            ignore_changeview: self.ignore_changeview,
            mode: StreamView::default(),
        }
    }
}

impl<M: ExclusiveOption> Files<M> {
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
        self.set_global_opts(v);
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, list all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn get_all(&self) -> bool {
        self.all
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, list all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn set_all(&mut self, v: bool) -> &mut Self {
        self.all = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For each file, list all revisions within a specified revision range,
    /// rather than only the highest revision in the range.
    pub fn all(mut self, v: bool) -> Self {
        self.set_all(v);
        self
    }

    /// # Description
    ///
    /// -A
    ///
    /// Limit output to files in archive depots.
    pub fn get_archive_depot_only(&self) -> bool {
        self.archive_depot_only
    }

    /// # Description
    ///
    /// -A
    ///
    /// Limit output to files in archive depots.
    pub fn set_archive_depot_only(&mut self, v: bool) -> &mut Self {
        self.archive_depot_only = v;
        self
    }

    /// # Description
    ///
    /// -A
    ///
    /// Limit output to files in archive depots.
    pub fn archive_depot_only(mut self, v: bool) -> Self {
        self.set_archive_depot_only(v);
        self
    }

    /// # Description
    ///
    /// -e
    ///
    #[cfg_attr(
        feature = "lt2026_1",
        doc = "Exclude deleted, purged, or archived files; the files that remain are those available for syncing or integration."
    )]
    #[cfg_attr(
        not(feature = "lt2026_1"),
        doc = "Exclude deleted, purged, or archived files. The remaining files are available for syncing or integration."
    )]
    pub fn get_exclude_deleted(&self) -> bool {
        self.exclude_deleted
    }

    /// # Description
    ///
    /// -e
    ///
    #[cfg_attr(
        feature = "lt2026_1",
        doc = "Exclude deleted, purged, or archived files; the files that remain are those available for syncing or integration."
    )]
    #[cfg_attr(
        not(feature = "lt2026_1"),
        doc = "Exclude deleted, purged, or archived files. The remaining files are available for syncing or integration."
    )]
    pub fn set_exclude_deleted(&mut self, v: bool) -> &mut Self {
        self.exclude_deleted = v;
        self
    }

    /// # Description
    ///
    /// -e
    ///
    #[cfg_attr(
        feature = "lt2026_1",
        doc = "Exclude deleted, purged, or archived files; the files that remain are those available for syncing or integration."
    )]
    #[cfg_attr(
        not(feature = "lt2026_1"),
        doc = "Exclude deleted, purged, or archived files. The remaining files are available for syncing or integration."
    )]
    pub fn exclude_deleted(mut self, v: bool) -> Self {
        self.set_exclude_deleted(v);
        self
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the file argument when listing files in a case
    /// sensitive server.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the file argument when listing files in a case
    /// sensitive server.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_case_insensitive(&mut self, v: bool) -> &mut Self {
        self.case_insensitive = v;
        self
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the file argument when listing files in a case
    /// sensitive server.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn case_insensitive(mut self, v: bool) -> Self {
        self.set_case_insensitive(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limit output to the first `max` files.
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limit output to the first `max` files.
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limit output to the first `max` files.
    pub fn limit(mut self, v: u64) -> Self {
        self.set_limit(v);
        self
    }

    /// # Description
    ///
    /// `--ignore-changeview`
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// [`p4 client`](crate::cmd::Client).
    #[cfg(not(feature = "lt2025_2"))]
    pub fn get_ignore_changeview(&self) -> bool {
        self.ignore_changeview
    }

    /// # Description
    ///
    /// `--ignore-changeview`
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// [`p4 client`](crate::cmd::Client).
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_ignore_changeview(&mut self, v: bool) -> &mut Self {
        self.ignore_changeview = v;
        self
    }

    /// # Description
    ///
    /// `--ignore-changeview`
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// [`p4 client`](crate::cmd::Client).
    #[cfg(not(feature = "lt2025_2"))]
    pub fn ignore_changeview(mut self, v: bool) -> Self {
        self.set_ignore_changeview(v);
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Files<M> {
    fn name(&self) -> &str {
        "files"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.all {
            command.arg("-a");
        }

        if self.archive_depot_only {
            command.arg("-A");
        }

        if self.exclude_deleted {
            command.arg("-e");
        }

        #[cfg(not(feature = "lt2016_1"))]
        if self.case_insensitive {
            command.arg("-i");
        }

        if let Some(limit) = self.limit {
            command.arg("-m").arg(limit.to_string());
        }

        self.mode.inject_args(command);

        #[cfg(not(feature = "lt2025_2"))]
        if self.ignore_changeview {
            command.arg("--ignore-changeview");
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M, S, I> ParameterizedSpawn<(S,)> for Files<M>
where
    M: ExclusiveOption,
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 files` for the given file specifications as a child process
    /// with piped standard output and error streams.
    fn spawn_with(&mut self, (files,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    fn args(cmd: &impl SubCommand) -> Vec<String> {
        args_of(&cmd.setup_command("p4"))
    }

    #[test]
    fn plain_form() {
        let cmd = Files::new("p4", GlobalOpts::new());
        assert_eq!(args(&cmd), ["files"]);
    }

    #[test]
    fn options_inject_in_synopsis_order() {
        #[allow(unused_mut)]
        let mut cmd = Files::new("p4", GlobalOpts::new())
            .all(true)
            .archive_depot_only(true)
            .exclude_deleted(true)
            .limit(5);

        #[cfg(not(feature = "lt2016_1"))]
        cmd.set_case_insensitive(true);
        #[cfg(not(feature = "lt2025_2"))]
        cmd.set_ignore_changeview(true);

        #[cfg(feature = "lt2016_1")]
        assert_eq!(args(&cmd), ["files", "-a", "-A", "-e", "-m", "5"]);
        #[cfg(all(not(feature = "lt2016_1"), feature = "lt2025_2"))]
        assert_eq!(args(&cmd), ["files", "-a", "-A", "-e", "-i", "-m", "5"]);
        #[cfg(not(feature = "lt2025_2"))]
        assert_eq!(
            args(&cmd),
            [
                "files",
                "-a",
                "-A",
                "-e",
                "-i",
                "-m",
                "5",
                "--ignore-changeview"
            ]
        );
    }

    #[test]
    fn unload_depot_form() {
        let cmd = Files::new("p4", GlobalOpts::new()).unload_depot();
        assert_eq!(args(&cmd), ["files", "-U"]);
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn stream_view_form() {
        let cmd = Files::new("p4", GlobalOpts::new()).stream_view();
        assert_eq!(args(&cmd), ["files", "--streamviews"]);
    }

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut cmd = Files::new("p4", GlobalOpts::new());
        assert_eq!(cmd.get_bin(), Path::new("p4"));

        cmd.set_bin("p4-custom");
        assert_eq!(cmd.get_bin(), Path::new("p4-custom"));

        let cmd = cmd.bin("p4-other");
        assert_eq!(cmd.get_bin(), Path::new("p4-other"));
    }
}
