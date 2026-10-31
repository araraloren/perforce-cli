use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// The `-f`/`-l` output-format state of `p4 export`.
///
/// Selected with the `format_non_text` (`-f`) or `limit` (`-l`) builder
/// before choosing a checkpoint or journal. These options apply to both
/// checkpoint and journal exports, but not to raw journal output (`-r`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Format {
    format_non_text: bool,

    limit: Option<u64>,
}

/// The `-r` raw-journal state of `p4 export`.
///
/// Selected with the `raw_journal` builder. Raw output applies to journals
/// only, so this state must be followed by a journal selection (`-j`) before
/// the command can be spawned.
#[derive(Debug, Clone, Copy, Default)]
pub struct RawJournal;

/// Checkpoint selection (`-c token`) of `p4 export`.
///
/// Wraps the output-format state `M` ([`Unselected`] or [`Format`]). Only
/// commands in this state, or in the [`Journal`] state, can be spawned.
#[derive(Debug, Clone, Default)]
pub struct CheckPoint<M = Unselected> {
    token: String,

    mode: M,
}

/// Journal selection (`-j token`) of `p4 export`.
///
/// Wraps the output-format state `M` ([`Unselected`], [`Format`], or
/// [`RawJournal`]). Only commands in this state, or in the [`CheckPoint`]
/// state, can be spawned.
#[derive(Debug, Clone, Default)]
pub struct Journal<M = Unselected> {
    token: String,

    mode: M,
}

/// Type-safe builder for `p4 export`.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] export -c token [-J prefix] [-f] [-l lines] [-F filter]
/// [-T tableexcludelist] [-P filterpattern]
///
/// p4 [g-opts] export -j token [-J prefix] [-f] [-l lines] [-F filter]
/// [-T tableexcludelist] [-P filterpattern]
///
/// p4 [g-opts] export -j token [-J prefix] -r [-F filter]
/// [-T tableexcludelist] [-P filterpattern]
/// ```
///
/// Extract journal or checkpoint records.
///
#[cfg_attr(
    feature = "lt2017_1",
    doc = "This command reports checkpoint and journal metadata from a Perforce server. With no options, the records are reported in tagged form."
)]
#[cfg_attr(
    all(feature = "lt2017_2", not(feature = "lt2017_1")),
    doc = "This command reports checkpoint and journal metadata from a Helix Core server. With no options, the records are reported in tagged form."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2017_2")),
    doc = "This command reports checkpoint and journal metadata from a Helix Server server. With no options, the records are reported in tagged form."
)]
#[cfg_attr(
    all(feature = "lt2021_1", not(feature = "lt2019_1")),
    doc = "This command reports checkpoint and journal metadata from a Helix server server. With no options, the records are reported in tagged form."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2021_1")),
    doc = "This command reports checkpoint and journal metadata from a Helix Server server. With no options, the records are reported in tagged form."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2024_1")),
    doc = "This command reports checkpoint and journal metadata from a Helix Core Server server. With no options, the records are reported in tagged form."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "This command reports checkpoint and journal metadata from a P4 Server server. With no options, the records are reported in tagged form."
)]
///
/// Some fields are added to the tagged output to indicate either
/// transactional consistency, or to indicate the end of the journal.
///
#[cfg_attr(
    feature = "lt2014_2",
    doc = "To filter database tables out of the exported data, use the `-T` flag with a list of tables whose data you wish to exclude. To exclude data from multiple tables, separate the table names by spaces or commas. The table names must begin with \"db.\", following the naming convention used for database files in the server root directory. If you separate the table exclusion list with spaces, you must enclose the list in quotes."
)]
#[cfg_attr(
    not(feature = "lt2014_2"),
    doc = "To filter database tables out of the exported data, use the `-T` option with a list of tables whose data you wish to exclude. To exclude data from multiple tables, separate the table names by spaces or commas. The table names must begin with \"db.\", following the naming convention used for database files in the server root directory. If you separate the table exclusion list with spaces, you must enclose the list in quotes."
)]
///
#[cfg_attr(feature = "lt2024_1", doc = "# Usage Notes")]
#[cfg_attr(not(feature = "lt2024_1"), doc = "# Usage notes")]
///
/// Can File Arguments Use Revision Specifier? N/A
///
/// Can File Arguments Use Revision Range? N/A
///
/// Minimal Access Level Required: `super`
#[cfg_attr(not(feature = "lt2022_2"), doc = "available to an service user")]
///
/// Compressed journals or checkpoints are not supported.
///
/// The `M` type parameter tracks the option state at compile time.
/// [`Unselected`] selects neither `-c` nor `-j`; `format_non_text` (`-f`) and
/// `limit` (`-l`) enter the [`Format`] state, and `raw_journal` (`-r`)
/// enters the [`RawJournal`] state. Only `checkpoint` (`-c`) and `journal`
/// (`-j`) enter the [`CheckPoint`] and [`Journal`] states, which are the
/// only states implementing [`ParameterizedSpawn`], so a partially
/// configured command cannot be spawned.
#[derive(Debug, Clone, Default)]
pub struct Export<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    prefix: Option<String>,

    filter: Option<String>,

    excluded_tables: Option<Vec<String>>,

    filter_patterns: Option<Vec<String>>,

    mode: M,
}

// ---------------------------------------------------------------------------
// Mode selection
// ---------------------------------------------------------------------------

impl Export<Unselected> {
    /// Creates a new `p4 export` command.
    ///
    /// `bin` is the path to the Perforce command-line executable. The
    /// resulting command has neither `-c` nor `-j` selected and cannot be
    /// spawned until a checkpoint or journal is selected.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            prefix: None,
            filter: None,
            excluded_tables: None,
            filter_patterns: None,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    /// `-c token`
    ///
    /// Specifies a checkpoint number or position token of the form
    /// `checkpointnum#byteoffset`.
    ///
    /// Transitions this command to the [`CheckPoint`] state, which spawns.
    pub fn checkpoint(self, token: impl Into<String>) -> Export<CheckPoint<Unselected>> {
        self.map_mode(|mode| CheckPoint {
            token: token.into(),
            mode,
        })
    }

    /// # Description
    ///
    /// `-j token`
    ///
    /// Specify a journal number or position token of the form
    /// `journalnum/byteoffset`.
    ///
    /// Transitions this command to the [`Journal`] state, which spawns.
    pub fn journal(self, token: impl Into<String>) -> Export<Journal<Unselected>> {
        self.map_mode(|mode| Journal {
            token: token.into(),
            mode,
        })
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// Format the output so that non-textual datatypes are formatted
    /// appropriately.
    ///
    /// Transitions this command to the [`Format`] state; a checkpoint or
    /// journal must still be selected before spawning.
    pub fn format_non_text(self) -> Export<Format> {
        self.with_mode(Format {
            format_non_text: true,
            limit: None,
        })
    }

    /// # Description
    ///
    /// `-l lines`
    ///
    /// Limit output to the specified number of lines of journal records.
    ///
    /// Transitions this command to the [`Format`] state; a checkpoint or
    /// journal must still be selected before spawning.
    pub fn limit(self, lines: u64) -> Export<Format> {
        self.with_mode(Format {
            format_non_text: false,
            limit: Some(lines),
        })
    }

    /// # Description
    ///
    /// `-r`
    ///
    /// Display raw journal output; this argument applies to journals only.
    ///
    /// Transitions this command to the [`RawJournal`] state; a journal must
    /// still be selected with `journal` (`-j`) before spawning.
    pub fn raw_journal(self) -> Export<RawJournal> {
        self.with_mode(RawJournal)
    }
}

impl Export<Format> {
    /// # Description
    ///
    /// `-f`
    ///
    /// Format the output so that non-textual datatypes are formatted
    /// appropriately.
    pub fn get_format_non_text(&self) -> bool {
        self.mode.format_non_text
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// Format the output so that non-textual datatypes are formatted
    /// appropriately.
    pub fn set_format_non_text(&mut self, v: bool) -> &mut Self {
        self.mode.format_non_text = v;
        self
    }

    /// # Description
    ///
    /// `-f`
    ///
    /// Format the output so that non-textual datatypes are formatted
    /// appropriately.
    pub fn format_non_text(mut self, v: bool) -> Self {
        self.mode.format_non_text = v;
        self
    }

    /// # Description
    ///
    /// `-l lines`
    ///
    /// Limit output to the specified number of lines of journal records.
    pub fn get_limit(&self) -> Option<u64> {
        self.mode.limit
    }

    /// # Description
    ///
    /// `-l lines`
    ///
    /// Limit output to the specified number of lines of journal records.
    pub fn set_limit(&mut self, v: Option<u64>) -> &mut Self {
        self.mode.limit = v;
        self
    }

    /// # Description
    ///
    /// `-l lines`
    ///
    /// Limit output to the specified number of lines of journal records.
    pub fn limit(mut self, v: u64) -> Self {
        self.mode.limit = Some(v);
        self
    }

    /// # Description
    ///
    /// `-c token`
    ///
    /// Specifies a checkpoint number or position token of the form
    /// `checkpointnum#byteoffset`.
    ///
    /// Transitions this command to the [`CheckPoint`] state, which spawns.
    pub fn checkpoint(self, token: impl Into<String>) -> Export<CheckPoint<Format>> {
        self.map_mode(|mode| CheckPoint {
            token: token.into(),
            mode,
        })
    }

    /// # Description
    ///
    /// `-j token`
    ///
    /// Specify a journal number or position token of the form
    /// `journalnum/byteoffset`.
    ///
    /// Transitions this command to the [`Journal`] state, which spawns.
    pub fn journal(self, token: impl Into<String>) -> Export<Journal<Format>> {
        self.map_mode(|mode| Journal {
            token: token.into(),
            mode,
        })
    }
}

impl Export<RawJournal> {
    /// # Description
    ///
    /// `-j token`
    ///
    /// Specify a journal number or position token of the form
    /// `journalnum/byteoffset`.
    ///
    /// Transitions this command to the [`Journal`] state, which spawns.
    pub fn journal(self, token: impl Into<String>) -> Export<Journal<RawJournal>> {
        self.map_mode(|mode| Journal {
            token: token.into(),
            mode,
        })
    }
}

// ---------------------------------------------------------------------------
// Common options, command path, and state helpers
// ---------------------------------------------------------------------------

impl<M> Export<M> {
    fn with_mode<N>(self, mode: N) -> Export<N> {
        Export {
            bin: self.bin,
            global_opts: self.global_opts,
            prefix: self.prefix,
            filter: self.filter,
            excluded_tables: self.excluded_tables,
            filter_patterns: self.filter_patterns,
            mode,
        }
    }

    fn map_mode<N>(self, f: impl FnOnce(M) -> N) -> Export<N> {
        Export {
            bin: self.bin,
            global_opts: self.global_opts,
            prefix: self.prefix,
            filter: self.filter,
            excluded_tables: self.excluded_tables,
            filter_patterns: self.filter_patterns,
            mode: f(self.mode),
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
    /// `-J prefix`
    ///
    /// Specifies a filename prefix for the journal, such as that used with
    /// `p4d -jc prefix`.
    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// # Description
    ///
    /// `-J prefix`
    ///
    /// Specifies a filename prefix for the journal, such as that used with
    /// `p4d -jc prefix`.
    pub fn set_prefix(&mut self, v: impl Into<String>) -> &mut Self {
        self.prefix = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-J prefix`
    ///
    /// Specifies a filename prefix for the journal, such as that used with
    /// `p4d -jc prefix`.
    pub fn prefix(mut self, v: impl Into<String>) -> Self {
        self.set_prefix(v);
        self
    }

    /// # Description
    ///
    /// `-F filter`
    ///
    /// Limit output to records that match the specified filter pattern. For
    /// example, `-F "table = db.configure"`.
    pub fn get_filter(&self) -> Option<&str> {
        self.filter.as_deref()
    }

    /// # Description
    ///
    /// `-F filter`
    ///
    /// Limit output to records that match the specified filter pattern. For
    /// example, `-F "table = db.configure"`.
    pub fn set_filter(&mut self, v: impl Into<String>) -> &mut Self {
        self.filter = Some(v.into());
        self
    }

    /// # Description
    ///
    /// `-F filter`
    ///
    /// Limit output to records that match the specified filter pattern. For
    /// example, `-F "table = db.configure"`.
    pub fn filter(mut self, v: impl Into<String>) -> Self {
        self.set_filter(v);
        self
    }

    /// # Description
    ///
    /// `-T tableexcludelist`
    ///
    /// Supply a list of database tables (for example, `db.have` and
    /// `db.client`) to exclude from export. Limit output to records that
    /// match the specified filter pattern. For example,
    /// `-T db.have,db.client` or `-T "db.have db.client"`.
    pub fn get_excluded_tables(&self) -> Option<&[String]> {
        self.excluded_tables.as_deref()
    }

    /// # Description
    ///
    /// `-T tableexcludelist`
    ///
    /// Supply a list of database tables (for example, `db.have` and
    /// `db.client`) to exclude from export. Limit output to records that
    /// match the specified filter pattern. For example,
    /// `-T db.have,db.client` or `-T "db.have db.client"`.
    pub fn set_excluded_tables(&mut self, v: Vec<String>) -> &mut Self {
        self.excluded_tables = Some(v);
        self
    }

    /// # Description
    ///
    /// `-T tableexcludelist`
    ///
    /// Supply a list of database tables (for example, `db.have` and
    /// `db.client`) to exclude from export. Limit output to records that
    /// match the specified filter pattern. For example,
    /// `-T db.have,db.client` or `-T "db.have db.client"`.
    pub fn excluded_tables(mut self, v: Vec<String>) -> Self {
        self.set_excluded_tables(v);
        self
    }

    /// # Description
    ///
    /// `-P filterpattern`
    ///
    /// Limit output to records that match the specified filter pattern.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "Multiple filter patterns can be specified with multiple `-P` flags."
    )]
    #[cfg_attr(
        not(feature = "lt2014_2"),
        doc = "Multiple filter patterns can be specified with multiple `-P` options."
    )]
    ///
    /// Each `filterpattern` begins with two characters and a colon, and
    /// specifies either a client filter or a depot filter, as well as whether
    /// the pattern is to be included or excluded, using the syntax:
    ///
    /// ```text
    /// -Pic://client/pattern - client records to include
    /// -Pxc://client/pattern - client records to exclude
    /// -Pif://depot/pattern - depot records to include
    /// -Pxf://depot/pattern - depot records to exclude
    /// ```
    ///
    /// The first character specifies whether the records are included or
    /// excluded (`i` or `x`), the second character specifies whether the
    /// records are client workspace-related or depot-file related (`c` or
    /// `f`), the colon is a separator, and the remainder of the
    /// `filterpattern` denotes either a client workspace view or a depot file
    /// path.
    #[cfg_attr(
        feature = "lt2024_1",
        doc = "The mechanism by which this filtering is implemented is the same as that which is used by the `ClientDataFilter:` and `RevisionDataFilter:` fields in the `p4 server` form."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "The mechanism by which this filtering is implemented is the same as that which is used by the `ClientDataFilter:` and `RevisionDataFilter` fields in the `p4 server` form."
    )]
    pub fn get_filter_patterns(&self) -> Option<&[String]> {
        self.filter_patterns.as_deref()
    }

    /// # Description
    ///
    /// `-P filterpattern`
    ///
    /// Limit output to records that match the specified filter pattern.
    /// This option can be specified more than once; each call appends
    /// another pattern.
    ///
    /// See `get_filter_patterns` for the pattern syntax.
    pub fn set_filter_pattern(&mut self, v: impl Into<String>) -> &mut Self {
        self.filter_patterns
            .get_or_insert_with(Vec::new)
            .push(v.into());
        self
    }

    /// # Description
    ///
    /// `-P filterpattern`
    ///
    /// Limit output to records that match the specified filter pattern.
    /// This option can be specified more than once; each call appends
    /// another pattern.
    ///
    /// See `get_filter_patterns` for the pattern syntax.
    pub fn filter_pattern(mut self, v: impl Into<String>) -> Self {
        self.set_filter_pattern(v);
        self
    }
}

impl<M> Export<CheckPoint<M>> {
    /// Get the checkpoint number or position token.
    pub fn get_token(&self) -> &str {
        &self.mode.token
    }

    /// Set the checkpoint number or position token.
    pub fn set_token(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.token = v.into();
        self
    }

    /// Set the checkpoint number or position token.
    pub fn token(mut self, v: impl Into<String>) -> Self {
        self.set_token(v);
        self
    }
}

impl<M> Export<Journal<M>> {
    /// Get the journal number or position token.
    pub fn get_token(&self) -> &str {
        &self.mode.token
    }

    /// Set the journal number or position token.
    pub fn set_token(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.token = v.into();
        self
    }

    /// Set the journal number or position token.
    pub fn token(mut self, v: impl Into<String>) -> Self {
        self.set_token(v);
        self
    }
}

// ---------------------------------------------------------------------------
// Argument injection
// ---------------------------------------------------------------------------

impl ExclusiveOption for Format {
    fn inject_args(&self, command: &mut Command) {
        if self.format_non_text {
            command.arg("-f");
        }

        if let Some(lines) = self.limit {
            command.arg("-l").arg(lines.to_string());
        }
    }
}

impl ExclusiveOption for RawJournal {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-r");
    }
}

impl<M: ExclusiveOption> ExclusiveOption for CheckPoint<M> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-c").arg(&self.token);
        self.mode.inject_args(command);
    }
}

impl<M: ExclusiveOption> ExclusiveOption for Journal<M> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-j").arg(&self.token);
        self.mode.inject_args(command);
    }
}

impl<M: ExclusiveOption> SubCommand for Export<M> {
    fn name(&self) -> &str {
        "export"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.mode.inject_args(command);

        if let Some(prefix) = &self.prefix {
            command.arg("-J").arg(prefix);
        }

        if let Some(filter) = &self.filter {
            command.arg("-F").arg(filter);
        }

        if let Some(tables) = &self.excluded_tables {
            command.arg("-T").arg(tables.join(","));
        }

        if let Some(patterns) = &self.filter_patterns {
            for pattern in patterns {
                command.arg("-P").arg(pattern);
            }
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

// ---------------------------------------------------------------------------
// Spawning
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> ParameterizedSpawn<()> for Export<CheckPoint<M>> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 export -c token` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<M: ExclusiveOption> ParameterizedSpawn<()> for Export<Journal<M>> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 export -j token` as a child process with piped standard
    /// output and error streams; use the returned [`Child`] handle to wait
    /// for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// Global options
// ---------------------------------------------------------------------------

impl<M: ExclusiveOption> Export<M> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    fn args(cmd: &impl SubCommand) -> Vec<String> {
        args_of(&cmd.setup_command("p4"))
    }

    /// Dry-run checks of the assembled `p4 export` command line; no process
    /// is spawned.
    #[test]
    fn without_selection_has_no_options() {
        let cmd = Export::new("p4", GlobalOpts::new());

        assert_eq!(args(&cmd), ["export"]);
    }

    #[test]
    fn format_state_injects_flags_before_selection() {
        assert_eq!(
            args(&Export::new("p4", GlobalOpts::new()).format_non_text()),
            ["export", "-f"]
        );
        assert_eq!(
            args(&Export::new("p4", GlobalOpts::new()).limit(12)),
            ["export", "-l", "12"]
        );
    }

    #[test]
    fn raw_journal_state_injects_flag_before_selection() {
        assert_eq!(
            args(&Export::new("p4", GlobalOpts::new()).raw_journal()),
            ["export", "-r"]
        );
    }

    #[test]
    fn bare_checkpoint_and_journal() {
        assert_eq!(
            args(&Export::new("p4", GlobalOpts::new()).checkpoint("3")),
            ["export", "-c", "3"]
        );
        assert_eq!(
            args(&Export::new("p4", GlobalOpts::new()).journal("7")),
            ["export", "-j", "7"]
        );
    }

    #[test]
    fn checkpoint_with_all_options() {
        let cmd = Export::new("p4", GlobalOpts::new())
            .prefix("backup")
            .format_non_text()
            .limit(10)
            .filter("table = db.configure")
            .excluded_tables(vec!["db.have".to_string(), "db.client".to_string()])
            .filter_pattern("ic://client/depot/main")
            .filter_pattern("xf://depot/secret")
            .checkpoint("3");

        assert_eq!(
            args(&cmd),
            [
                "export",
                "-c",
                "3",
                "-f",
                "-l",
                "10",
                "-J",
                "backup",
                "-F",
                "table = db.configure",
                "-T",
                "db.have,db.client",
                "-P",
                "ic://client/depot/main",
                "-P",
                "xf://depot/secret",
            ]
        );
    }

    #[test]
    fn raw_journal_with_options() {
        let cmd = Export::new("p4", GlobalOpts::new())
            .prefix("backup")
            .excluded_tables(vec!["db.have".to_string()])
            .filter_pattern("if://depot/main")
            .raw_journal()
            .journal("7");

        assert_eq!(
            args(&cmd),
            [
                "export",
                "-j",
                "7",
                "-r",
                "-J",
                "backup",
                "-T",
                "db.have",
                "-P",
                "if://depot/main",
            ]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let cmd = Export::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .journal("7");

        assert_eq!(
            args(&cmd),
            ["-p", "localhost:1666", "-q", "export", "-j", "7"]
        );
    }

    #[test]
    fn token_accessors_update_selection() {
        let mut cmd = Export::new("p4", GlobalOpts::new()).checkpoint("3");
        assert_eq!(cmd.get_token(), "3");

        cmd.set_token("12#4");
        assert_eq!(cmd.get_token(), "12#4");
        assert_eq!(args(&cmd), ["export", "-c", "12#4"]);

        let cmd = cmd.token("5");
        assert_eq!(args(&cmd), ["export", "-c", "5"]);
    }

    #[test]
    fn format_accessors_update_options() {
        let mut cmd = Export::new("p4", GlobalOpts::new()).format_non_text();
        assert!(cmd.get_format_non_text());
        assert_eq!(cmd.get_limit(), None);

        cmd.set_format_non_text(false);
        cmd.set_limit(Some(5));
        assert!(!cmd.get_format_non_text());
        assert_eq!(cmd.get_limit(), Some(5));
        assert_eq!(args(&cmd), ["export", "-l", "5"]);

        let cmd = cmd.limit(9).format_non_text(true);
        assert_eq!(args(&cmd), ["export", "-f", "-l", "9"]);
    }

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut cmd = Export::new("p4", GlobalOpts::new()).journal("7");
        assert_eq!(cmd.get_bin(), Path::new("p4"));

        cmd.set_bin("/usr/local/bin/p4");
        assert_eq!(cmd.get_bin(), Path::new("/usr/local/bin/p4"));

        let cmd = cmd.bin("p4.exe");
        assert_eq!(cmd.get_bin(), Path::new("p4.exe"));
    }
}
