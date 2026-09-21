use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Internal representation of the `-T` / `--tab=N` tab stop setting.
#[cfg(not(feature = "lt2016_1"))]
#[derive(Debug, Clone, Copy)]
enum Tab {
    /// `-T`: align output to the default tab stop of 8.
    Default,
    /// `--tab=N`: align output to a tab stop of N.
    Custom(u32),
}

/// Variants of the `[-i | -I]` mutually exclusive option group of
/// `p4 annotate`.
pub mod follow {
    /// Follow file history across branches (`-i`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Branches;

    /// Follow integrations into the file (`-I`).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Integrations;
}

impl ExclusiveOption for follow::Branches {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-i");
    }
}

impl ExclusiveOption for follow::Integrations {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-I");
    }
}

#[cfg_attr(
    feature = "lt2014_2",
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -d flag] file[revRange] ...`: print file lines along with their revisions."
)]
#[cfg_attr(
    all(feature = "lt2015_1", not(feature = "lt2014_2")),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -d options] file[revRange] ...`: print file lines along with their revisions."
)]
#[cfg_attr(
    all(feature = "lt2015_2", not(feature = "lt2015_1")),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t] [-d options] file[revRange] ...`: print file lines along with their revisions."
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_2")),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -u] [-d options] file[revRange] ...`: print file lines along with their revisions."
)]
#[cfg_attr(
    all(feature = "lt2018_1", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -u -T] [-d options] file[revRange] ...`: print file lines along with their revisions."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2018_1")),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -u -T] [-d options] File Spec[revSpec]`: print file lines along with their revisions."
)]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2019_1")),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -T -u] [-d options] File Spec[revSpec]`: print file lines along with their revisions."
)]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "`p4 [g-opts] annotate [-a -c -i -I -q -t -T -u] [-d options] FileSpec[revSpec]`: print file lines along with their revisions."
)]
#[derive(Debug, Clone, Default)]
pub struct Annotate<F = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    all_lines: bool,

    changelist_number: bool,

    diff_options: Option<String>,

    follow: F,

    quiet_mode: bool,

    force_binary: bool,

    #[cfg(not(feature = "lt2015_2"))]
    user_date: bool,

    #[cfg(not(feature = "lt2016_1"))]
    tab: Option<Tab>,
}

impl Annotate<Unselected> {
    /// Creates a new `p4 annotate` command.
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
    /// -i
    ///
    /// Follow file history across branches. If a file was created by
    /// branching,
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce includes revisions up to the branch point."
    )]
    #[cfg_attr(
        not(feature = "lt2017_2"),
        doc = "the output includes revisions up to the branch point."
    )]
    /// The use of the -i option implies the -c option. The -i option cannot be
    /// combined with -I.
    pub fn follow_branches(self) -> Annotate<follow::Branches> {
        Annotate {
            bin: self.bin,
            global_opts: self.global_opts,
            all_lines: self.all_lines,
            changelist_number: self.changelist_number,
            diff_options: self.diff_options,
            follow: follow::Branches,
            quiet_mode: self.quiet_mode,
            force_binary: self.force_binary,
            #[cfg(not(feature = "lt2015_2"))]
            user_date: self.user_date,
            #[cfg(not(feature = "lt2016_1"))]
            tab: self.tab,
        }
    }

    /// # Description
    ///
    /// -I
    ///
    /// Follow integrations into the file. If a line was introduced into the
    /// file by a merge, the source of the merge is indicated as the changelist
    /// that introduced the line.
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "If that source was itself the result of an integration, that",
        doc = "source will be used instead, and so on. The use of the -I option",
        doc = "implies the -c option. The -I option cannot be combined with -i."
    )]
    #[cfg_attr(
        not(feature = "lt2017_2"),
        doc = "If that source was itself the result of an integration, that",
        doc = "source will be used instead. The use of the -I option implies the",
        doc = "-c option. The -I option cannot be combined with -i."
    )]
    pub fn follow_integrations(self) -> Annotate<follow::Integrations> {
        Annotate {
            bin: self.bin,
            global_opts: self.global_opts,
            all_lines: self.all_lines,
            changelist_number: self.changelist_number,
            diff_options: self.diff_options,
            follow: follow::Integrations,
            quiet_mode: self.quiet_mode,
            force_binary: self.force_binary,
            #[cfg(not(feature = "lt2015_2"))]
            user_date: self.user_date,
            #[cfg(not(feature = "lt2016_1"))]
            tab: self.tab,
        }
    }
}

impl<F: ExclusiveOption> ParameterizedSpawn for Annotate<F> {
    type Input<'a> = &'a [&'a OsStr];
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 annotate` for the given files as a child process with piped
    /// standard output and error streams; use the returned [`Child`] handle
    /// to wait for it or interact with it.
    fn spawn_with<'a>(&mut self, files: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<F: ExclusiveOption> Annotate<F> {
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
    /// All lines, including deleted lines and lines no longer present at the
    /// head revision, are included. Each line includes a starting and ending
    /// revision.
    pub fn get_all_lines(&self) -> bool {
        self.all_lines
    }

    /// # Description
    ///
    /// -a
    ///
    /// All lines, including deleted lines and lines no longer present at the
    /// head revision, are included. Each line includes a starting and ending
    /// revision.
    pub fn set_all_lines(&mut self, v: bool) -> &mut Self {
        self.all_lines = v;
        self
    }

    /// # Description
    ///
    /// -a
    ///
    /// All lines, including deleted lines and lines no longer present at the
    /// head revision, are included. Each line includes a starting and ending
    /// revision.
    pub fn all_lines(mut self, v: bool) -> Self {
        self.all_lines = v;
        self
    }

    /// # Description
    ///
    /// -c
    ///
    /// Display the changelist number, rather than the revision number,
    /// associated with each line. If you use the -a option and the -c option
    /// together, each line includes a starting and ending changelist number.
    pub fn get_changelist_number(&self) -> bool {
        self.changelist_number
    }

    /// # Description
    ///
    /// -c
    ///
    /// Display the changelist number, rather than the revision number,
    /// associated with each line. If you use the -a option and the -c option
    /// together, each line includes a starting and ending changelist number.
    pub fn set_changelist_number(&mut self, v: bool) -> &mut Self {
        self.changelist_number = v;
        self
    }

    /// # Description
    ///
    /// -c
    ///
    /// Display the changelist number, rather than the revision number,
    /// associated with each line. If you use the -a option and the -c option
    /// together, each line includes a starting and ending changelist number.
    pub fn changelist_number(mut self, v: bool) -> Self {
        self.changelist_number = v;
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2014_2", doc = "-d flag")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "-d options")]
    ///
    /// Runs the diff routine with one of a subset of the standard UNIX diff
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flags. See the Usage Notes below for a listing of these",
        doc = "flags."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "options. See Usage Notes below for a listing of these",
        doc = "options."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_1")),
        doc = "options. See Usage Notes for a listing of these options."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "options. See Usage notes for a listing of these options."
    )]
    pub fn get_diff_options(&self) -> Option<&str> {
        self.diff_options.as_deref()
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2014_2", doc = "-d flag")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "-d options")]
    ///
    /// Runs the diff routine with one of a subset of the standard UNIX diff
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flags. See the Usage Notes below for a listing of these",
        doc = "flags."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "options. See Usage Notes below for a listing of these",
        doc = "options."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_1")),
        doc = "options. See Usage Notes for a listing of these options."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "options. See Usage notes for a listing of these options."
    )]
    pub fn set_diff_options(&mut self, v: impl Into<String>) -> &mut Self {
        self.diff_options = Some(v.into());
        self
    }

    /// # Description
    ///
    #[cfg_attr(feature = "lt2014_2", doc = "-d flag")]
    #[cfg_attr(not(feature = "lt2014_2"), doc = "-d options")]
    ///
    /// Runs the diff routine with one of a subset of the standard UNIX diff
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "flags. See the Usage Notes below for a listing of these",
        doc = "flags."
    )]
    #[cfg_attr(
        all(feature = "lt2015_1", not(feature = "lt2014_2")),
        doc = "options. See Usage Notes below for a listing of these",
        doc = "options."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2015_1")),
        doc = "options. See Usage Notes for a listing of these options."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "options. See Usage notes for a listing of these options."
    )]
    pub fn diff_options(mut self, v: impl Into<String>) -> Self {
        self.diff_options = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Quiet mode; suppress the one-line header for each file."
    )]
    #[cfg_attr(
        not(feature = "lt2017_2"),
        doc = "Quiet mode, which suppresses the one-line header for each file."
    )]
    pub fn get_quiet_mode(&self) -> bool {
        self.quiet_mode
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Quiet mode; suppress the one-line header for each file."
    )]
    #[cfg_attr(
        not(feature = "lt2017_2"),
        doc = "Quiet mode, which suppresses the one-line header for each file."
    )]
    pub fn set_quiet_mode(&mut self, v: bool) -> &mut Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Quiet mode; suppress the one-line header for each file."
    )]
    #[cfg_attr(
        not(feature = "lt2017_2"),
        doc = "Quiet mode, which suppresses the one-line header for each file."
    )]
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Force `p4 annotate` to display non-text (binary) files.
    pub fn get_force_binary(&self) -> bool {
        self.force_binary
    }

    /// # Description
    ///
    /// -t
    ///
    /// Force `p4 annotate` to display non-text (binary) files.
    pub fn set_force_binary(&mut self, v: bool) -> &mut Self {
        self.force_binary = v;
        self
    }

    /// # Description
    ///
    /// -t
    ///
    /// Force `p4 annotate` to display non-text (binary) files.
    pub fn force_binary(mut self, v: bool) -> Self {
        self.force_binary = v;
        self
    }

    /// # Description
    ///
    /// -u
    ///
    /// Display the name of the user who modified the change and the date when
    /// the modification occurred.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn get_user_date(&self) -> bool {
        self.user_date
    }

    /// # Description
    ///
    /// -u
    ///
    /// Display the name of the user who modified the change and the date when
    /// the modification occurred.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn set_user_date(&mut self, v: bool) -> &mut Self {
        self.user_date = v;
        self
    }

    /// # Description
    ///
    /// -u
    ///
    /// Display the name of the user who modified the change and the date when
    /// the modification occurred.
    #[cfg(not(feature = "lt2015_2"))]
    pub fn user_date(mut self, v: bool) -> Self {
        self.user_date = v;
        self
    }

    /// # Description
    ///
    /// -T | `--tab=N`
    ///
    /// Align output to a tab stop of 8. You can specify a different tab value
    /// using the `--tab` option and specifying the desired value for N.
    ///
    /// Returns the tab stop currently set, or `None` if no tab option is set.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_tab(&self) -> Option<u32> {
        match self.tab {
            Some(Tab::Default) => Some(8),
            Some(Tab::Custom(value)) => Some(value),
            None => None,
        }
    }

    /// # Description
    ///
    /// `--tab=N`
    ///
    /// Align output to a tab stop of N. Use [`Self::set_default_tab`] to
    /// align output to the default tab stop of 8 with `-T`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_tab(&mut self, v: u32) -> &mut Self {
        self.tab = Some(Tab::Custom(v));
        self
    }

    /// # Description
    ///
    /// `--tab=N`
    ///
    /// Align output to a tab stop of N. Use [`Self::default_tab`] to align
    /// output to the default tab stop of 8 with `-T`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn tab(mut self, v: u32) -> Self {
        self.tab = Some(Tab::Custom(v));
        self
    }

    /// # Description
    ///
    /// -T
    ///
    /// Align output to a tab stop of 8. Use [`Self::set_tab`] to align output
    /// to a different tab stop with `--tab=N`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_default_tab(&mut self) -> &mut Self {
        self.tab = Some(Tab::Default);
        self
    }

    /// # Description
    ///
    /// -T
    ///
    /// Align output to a tab stop of 8. Use [`Self::tab`] to align output to
    /// a different tab stop with `--tab=N`.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn default_tab(mut self) -> Self {
        self.tab = Some(Tab::Default);
        self
    }
}

impl<F: ExclusiveOption> SubCommand for Annotate<F> {
    fn name(&self) -> &str {
        "annotate"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.all_lines {
            command.arg("-a");
        }

        if self.changelist_number {
            command.arg("-c");
        }

        if let Some(diff_options) = &self.diff_options {
            command.arg(format!("-d{diff_options}"));
        }

        self.follow.inject_args(command);

        if self.quiet_mode {
            command.arg("-q");
        }

        if self.force_binary {
            command.arg("-t");
        }

        #[cfg(not(feature = "lt2015_2"))]
        {
            if self.user_date {
                command.arg("-u");
            }
        }

        #[cfg(not(feature = "lt2016_1"))]
        {
            if let Some(tab) = self.tab {
                match tab {
                    Tab::Default => {
                        command.arg("-T");
                    }
                    Tab::Custom(value) => {
                        command.arg(format!("--tab={value}"));
                    }
                }
            }
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 annotate` command line; no process
    /// is spawned.
    #[test]
    fn without_options() {
        let annotate = Annotate::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&annotate.setup_command("p4")), ["annotate"]);
    }

    #[test]
    fn all_options() {
        let mut annotate = Annotate::new("p4", GlobalOpts::new());
        annotate
            .set_all_lines(true)
            .set_changelist_number(true)
            .set_diff_options("Su2")
            .set_quiet_mode(true)
            .set_force_binary(true);
        #[cfg(not(feature = "lt2015_2"))]
        annotate.set_user_date(true);

        // `-i` and `-I` are mutually exclusive; select the branch-following
        // variant here.
        #[cfg_attr(feature = "lt2016_1", allow(unused_mut))]
        let mut annotate = annotate.follow_branches();

        // `mut` is only needed when the tab block below is compiled in.
        #[cfg_attr(feature = "lt2016_1", allow(unused_mut))]
        let mut expected = vec!["annotate", "-a", "-c", "-dSu2", "-i", "-q", "-t"];
        #[cfg(not(feature = "lt2015_2"))]
        expected.push("-u");
        #[cfg(not(feature = "lt2016_1"))]
        {
            annotate.set_default_tab();
            expected.push("-T");
        }

        assert_eq!(args_of(&annotate.setup_command("p4")), expected);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn default_tab_uses_short_flag() {
        let mut annotate = Annotate::new("p4", GlobalOpts::new());
        annotate.set_default_tab();

        assert_eq!(annotate.get_tab(), Some(8));
        assert_eq!(args_of(&annotate.setup_command("p4")), ["annotate", "-T"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn custom_tab_uses_long_flag() {
        let mut annotate = Annotate::new("p4", GlobalOpts::new());
        annotate.set_tab(4);

        assert_eq!(annotate.get_tab(), Some(4));
        assert_eq!(
            args_of(&annotate.setup_command("p4")),
            ["annotate", "--tab=4"]
        );
    }
}
