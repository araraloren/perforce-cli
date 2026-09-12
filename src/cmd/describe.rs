use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
};

use crate::{cmd::SubCommand, global::GlobalOpts};

#[cfg_attr(
    feature = "lt2014_2",
    doc = "`p4 [g-opts] describe [ -dflags -s -S -f -O ] changelist...`"
)]
#[cfg_attr(
    all(feature = "lt2015_1", not(feature = "lt2014_2")),
    doc = "`p4 [g-opts] describe [ -doptions -s -S -f -O ] changelist...`"
)]
#[cfg_attr(
    all(feature = "lt2015_2", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] describe [-doptions] [-s -S -f -O] changelist …`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_2")),
    doc = "`p4 [g-opts] describe [-doptions] [-s -S -f -O -I] changelist …`"
)]
#[cfg_attr(
    all(feature = "lt2017_1", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] describe [-doptions] [-s -S -f -O -I] changelist ...`"
)]
#[cfg_attr(
    all(feature = "lt2017_2", not(feature = "lt2017_1")),
    doc = "`p4 [g-opts] describe [-doptions] [-f -I -m -O -s -S] changelist ...`"
)]
#[cfg_attr(
    not(feature = "lt2017_2"),
    doc = "`p4 [g-opts] describe [-doptions] [-a -f -I -m -O -s -S] changelist ...`"
)]
///
#[cfg_attr(
    feature = "lt2017_2",
    doc = "Provides information about changelists and the changelists' files."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2017_2")),
    doc = "Provides information about changelists and files in the changelists."
)]
#[cfg_attr(
    not(feature = "lt2019_1"),
    doc = "Provides information about changelists, as well as files in the",
    doc = "changelists, and the path of the open stream, if a stream is open."
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2017_2")),
    doc = "",
    doc = "# Note",
    doc = "",
    doc = "If the depot is of type `graph`, displays a commit description. See",
    doc = "the command-line help for `p4 help-graph describe`."
)]
#[derive(Debug, Clone, Default)]
pub struct Describe {
    bin: PathBuf,

    global_opts: GlobalOpts,

    #[cfg(not(feature = "lt2017_2"))]
    text_only: bool,

    diff_options: Option<String>,

    force_restricted: bool,

    #[cfg(not(feature = "lt2015_2"))]
    use_identity: bool,

    #[cfg(not(feature = "lt2017_1"))]
    limit: Option<u64>,

    original: bool,

    short_output: bool,

    include_shelved_files: bool,
}

impl SubCommand for Describe {
    fn name(&self) -> &str {
        "describe"
    }

    fn inject_local_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2017_2"))]
        {
            if self.text_only {
                command.arg("-a");
            }
        }

        if let Some(diff_options) = &self.diff_options {
            command.arg(format!("-d{diff_options}"));
        }

        if self.force_restricted {
            command.arg("-f");
        }

        #[cfg(not(feature = "lt2015_2"))]
        {
            if self.use_identity {
                command.arg("-I");
            }
        }

        #[cfg(not(feature = "lt2017_1"))]
        {
            if let Some(max) = self.limit {
                command.arg("-m").arg(max.to_string());
            }
        }

        if self.original {
            command.arg("-O");
        }

        if self.short_output {
            command.arg("-s");
        }

        if self.include_shelved_files {
            command.arg("-S");
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl Describe {
    /// Creates a new `p4 describe` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Spawns `p4 describe` for the given changelists as a child process.
    ///
    /// The child process inherits the standard input, output, and error
    /// streams of the current process, and runs asynchronously; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    pub fn spawn<S: AsRef<OsStr>>(&self, changelists: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(changelists).spawn()
    }

    /// Runs `p4 describe` for the given changelists to completion and
    /// captures its output.
    ///
    /// Unlike [`Self::spawn`], this method blocks until the command exits and
    /// collects the standard output and error into the returned [`Output`].
    pub fn output<S: AsRef<OsStr>>(&self, changelists: &[S]) -> Result<Output, std::io::Error> {
        self.setup_command(&self.bin)
            .args(changelists)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
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
    /// -a
    ///
    /// For text files only (ignores binary files):
    ///
    /// - For shelved files, shows the content for "open for add" (pending) files.
    /// - For submitted files, shows the content of added files.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn get_text_only(&self) -> bool {
        self.text_only
    }

    /// # Description
    ///
    /// -a
    ///
    /// For text files only (ignores binary files):
    ///
    /// - For shelved files, shows the content for "open for add" (pending) files.
    /// - For submitted files, shows the content of added files.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn set_text_only(&mut self, v: bool) -> &mut Self {
        self.text_only = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// For text files only (ignores binary files):
    ///
    /// - For shelved files, shows the content for "open for add" (pending) files.
    /// - For submitted files, shows the content of added files.
    #[cfg(not(feature = "lt2017_2"))]
    pub fn text_only(mut self, v: bool) -> Self {
        self.text_only = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2014_2", doc = "-dflags")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "-doptions")]
    ///
    /// Runs the diff routine with one of a subset of the standard UNIX diff
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flags. See the Usage Notes below for a flag listing."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "options. See the Usage Notes below for a option listing."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_1")),
        doc = "options. See Usage Notes for an option listing."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "options. See Usage notes for an option listing."
    )]
    pub fn get_diff_options(&self) -> Option<&str> {
        self.diff_options.as_deref()
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2014_2", doc = "-dflags")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "-doptions")]
    ///
    /// Runs the diff routine with one of a subset of the standard UNIX diff
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flags. See the Usage Notes below for a flag listing."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "options. See the Usage Notes below for a option listing."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_1")),
        doc = "options. See Usage Notes for an option listing."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "options. See Usage notes for an option listing."
    )]
    pub fn set_diff_options(&mut self, v: impl Into<String>) -> &mut Self {
        self.diff_options = Some(v.into());
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2014_2", doc = "-dflags")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "-doptions")]
    ///
    /// Runs the diff routine with one of a subset of the standard UNIX diff
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flags. See the Usage Notes below for a flag listing."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "options. See the Usage Notes below for a option listing."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_1")),
        doc = "options. See Usage Notes for an option listing."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "options. See Usage notes for an option listing."
    )]
    pub fn diff_options(mut self, v: impl Into<String>) -> Self {
        self.diff_options = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the display of descriptions for restricted changelists. This
    #[cfg_attr(feature = "lt2014_2", doc = "flag requires `admin` permission.")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "option requires `admin` permission.")]
    pub fn get_force_restricted(&self) -> bool {
        self.force_restricted
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the display of descriptions for restricted changelists. This
    #[cfg_attr(feature = "lt2014_2", doc = "flag requires `admin` permission.")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "option requires `admin` permission.")]
    pub fn set_force_restricted(&mut self, v: bool) -> &mut Self {
        self.force_restricted = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Force the display of descriptions for restricted changelists. This
    #[cfg_attr(feature = "lt2014_2", doc = "flag requires `admin` permission.")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "option requires `admin` permission.")]
    pub fn force_restricted(mut self, v: bool) -> Self {
        self.force_restricted = v;
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Specifies that the changelist number is the
    #[cfg_attr(feature = "lt2017_1", doc = "Identity field of a changelist.")]
    #[cfg_attr(not(feature = "lt2017_1"), doc = "`Identity` field of a changelist.")]
    #[cfg(not(feature = "lt2015_2"))]
    pub fn get_use_identity(&self) -> bool {
        self.use_identity
    }

    /// # Description
    ///
    /// -I
    ///
    /// Specifies that the changelist number is the
    #[cfg_attr(feature = "lt2017_1", doc = "Identity field of a changelist.")]
    #[cfg_attr(not(feature = "lt2017_1"), doc = "`Identity` field of a changelist.")]
    #[cfg(not(feature = "lt2015_2"))]
    pub fn set_use_identity(&mut self, v: bool) -> &mut Self {
        self.use_identity = v;
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Specifies that the changelist number is the
    #[cfg_attr(feature = "lt2017_1", doc = "Identity field of a changelist.")]
    #[cfg_attr(not(feature = "lt2017_1"), doc = "`Identity` field of a changelist.")]
    #[cfg(not(feature = "lt2015_2"))]
    pub fn use_identity(mut self, v: bool) -> Self {
        self.use_identity = v;
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limits files to the first *max* number of files. The following
    /// example alphabetically lists (and diffs) two files affected by
    /// changelist 765 and two files affected by changelist 987:
    /// `p4 describe -m 2 765 987`
    #[cfg(not(feature = "lt2017_1"))]
    pub fn get_limit(&self) -> Option<u64> {
        self.limit
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limits files to the first *max* number of files. The following
    /// example alphabetically lists (and diffs) two files affected by
    /// changelist 765 and two files affected by changelist 987:
    /// `p4 describe -m 2 765 987`
    #[cfg(not(feature = "lt2017_1"))]
    pub fn set_limit(&mut self, v: u64) -> &mut Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -m max
    ///
    /// Limits files to the first *max* number of files. The following
    /// example alphabetically lists (and diffs) two files affected by
    /// changelist 765 and two files affected by changelist 987:
    /// `p4 describe -m 2 765 987`
    #[cfg(not(feature = "lt2017_1"))]
    pub fn limit(mut self, v: u64) -> Self {
        self.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// -O
    ///
    /// If a changelist was renumbered on submit, and you know only the
    /// original changelist number, use `-O` and the original changelist
    /// number to describe the changelist.
    pub fn get_original(&self) -> bool {
        self.original
    }

    /// # Description
    ///
    /// -O
    ///
    /// If a changelist was renumbered on submit, and you know only the
    /// original changelist number, use `-O` and the original changelist
    /// number to describe the changelist.
    pub fn set_original(&mut self, v: bool) -> &mut Self {
        self.original = v;
        self
    }

    /// # Description
    ///
    /// -O
    ///
    /// If a changelist was renumbered on submit, and you know only the
    /// original changelist number, use `-O` and the original changelist
    /// number to describe the changelist.
    pub fn original(mut self, v: bool) -> Self {
        self.original = v;
        self
    }

    /// # Description
    ///
    /// -s
    ///
    /// Display a shortened output that excludes
    #[cfg_attr(feature = "lt2017_1", doc = "the files' diffs.")]
    #[cfg_attr(not(feature = "lt2017_1"), doc = "the diffs of the files.")]
    pub fn get_short_output(&self) -> bool {
        self.short_output
    }

    /// # Description
    ///
    /// -s
    ///
    /// Display a shortened output that excludes
    #[cfg_attr(feature = "lt2017_1", doc = "the files' diffs.")]
    #[cfg_attr(not(feature = "lt2017_1"), doc = "the diffs of the files.")]
    pub fn set_short_output(&mut self, v: bool) -> &mut Self {
        self.short_output = v;
        self
    }

    /// # Description
    ///
    /// -s
    ///
    /// Display a shortened output that excludes
    #[cfg_attr(feature = "lt2017_1", doc = "the files' diffs.")]
    #[cfg_attr(not(feature = "lt2017_1"), doc = "the diffs of the files.")]
    pub fn short_output(mut self, v: bool) -> Self {
        self.short_output = v;
        self
    }

    /// # Description
    ///
    /// -S
    ///
    #[cfg_attr(
        feature = "lt2017_1",
        doc = "Display files shelved for the specified changelist, including",
        doc = "diffs of those files against their previous depot revision."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2017_1")),
        doc = "Display the names of files shelved for the specified changelist,",
        doc = "including the diff of each file against its previous depot",
        doc = "revision."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Lists files that are shelved for the pending changelist and",
        doc = "displays diffs of the files against their previous revision."
    )]
    pub fn get_include_shelved_files(&self) -> bool {
        self.include_shelved_files
    }

    /// # Description
    ///
    /// -S
    ///
    #[cfg_attr(
        feature = "lt2017_1",
        doc = "Display files shelved for the specified changelist, including",
        doc = "diffs of those files against their previous depot revision."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2017_1")),
        doc = "Display the names of files shelved for the specified changelist,",
        doc = "including the diff of each file against its previous depot",
        doc = "revision."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Lists files that are shelved for the pending changelist and",
        doc = "displays diffs of the files against their previous revision."
    )]
    pub fn set_include_shelved_files(&mut self, v: bool) -> &mut Self {
        self.include_shelved_files = v;
        self
    }

    /// # Description
    ///
    /// -S
    ///
    #[cfg_attr(
        feature = "lt2017_1",
        doc = "Display files shelved for the specified changelist, including",
        doc = "diffs of those files against their previous depot revision."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2017_1")),
        doc = "Display the names of files shelved for the specified changelist,",
        doc = "including the diff of each file against its previous depot",
        doc = "revision."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Lists files that are shelved for the pending changelist and",
        doc = "displays diffs of the files against their previous revision."
    )]
    pub fn include_shelved_files(mut self, v: bool) -> Self {
        self.include_shelved_files = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 describe` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let describe = Describe::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&describe.setup_command("p4")), ["describe"]);
    }

    #[test]
    fn all_local_options() {
        let mut describe = Describe::new("p4", GlobalOpts::new());
        #[cfg(not(feature = "lt2017_2"))]
        describe.set_text_only(true);
        describe.set_diff_options("du").set_force_restricted(true);
        #[cfg(not(feature = "lt2015_2"))]
        describe.set_use_identity(true);
        #[cfg(not(feature = "lt2017_1"))]
        describe.set_limit(2);
        describe
            .set_original(true)
            .set_short_output(true)
            .set_include_shelved_files(true);

        let mut expected = vec!["describe"];
        #[cfg(not(feature = "lt2017_2"))]
        expected.push("-a");
        expected.push("-ddu");
        expected.push("-f");
        #[cfg(not(feature = "lt2015_2"))]
        expected.push("-I");
        #[cfg(not(feature = "lt2017_1"))]
        expected.extend(["-m", "2"]);
        expected.extend(["-O", "-s", "-S"]);

        assert_eq!(args_of(&describe.setup_command("p4")), expected);
    }

    #[cfg(not(feature = "lt2017_1"))]
    #[test]
    fn limit_is_injected_as_separate_args() {
        let mut describe = Describe::new("p4", GlobalOpts::new());
        describe.set_limit(5);

        assert_eq!(
            args_of(&describe.setup_command("p4")),
            ["describe", "-m", "5"]
        );
    }
}
