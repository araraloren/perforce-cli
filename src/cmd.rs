pub mod add;
pub mod admin;
#[cfg(not(feature = "lt2016_1"))]
pub mod aliases;
pub mod annotate;
pub mod archive;
pub mod attribute;
pub mod changes;
pub mod describe;
pub mod edit;
pub mod filelog;
pub mod print;
pub mod sync;
pub mod r#where;

pub use add::Add;
pub use admin::AdminEntry;
#[cfg(not(feature = "lt2016_1"))]
pub use aliases::Aliases;
pub use annotate::Annotate;
pub use archive::Archive;
pub use attribute::Attribute;
pub use changes::Changes;
pub use describe::Describe;
pub use edit::Edit;
pub use filelog::FileLog;
pub use print::Print;
pub use sync::Sync;
pub use r#where::Where;

use std::{ffi::OsStr, process::Command};

use crate::global::GlobalOpts;

/// Long output mode for changelist descriptions shared by commands such as
/// `p4 changes` and `p4 filelog`.
///
/// By default, only the first 30 (or 31) characters of the description are
/// shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongOutput {
    /// Show the full text of each changelist description (`-l`).
    Default,
    /// Show the full text truncated at 250 characters (`-L`).
    Truncated,
}

impl LongOutput {
    /// Returns the command-line flag for this long output mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            LongOutput::Default => "-l",
            LongOutput::Truncated => "-L",
        }
    }
}

/// Output format of the diff routine passed via `-doptions`.
///
/// This is one of two orthogonal dimensions of [`DiffOptions`] (the other
/// being [`WhitespaceHandling`]). Exactly one format may be selected; the
/// [`Default`](DiffFormat::Default) variant corresponds to no format flag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiffFormat {
    /// Default diff output format (no format flag).
    #[default]
    Default,
    /// Context output format (`-dc[num]`), showing `num` lines of context
    /// around the changes.
    Context(Option<u32>),
    /// RCS output format (`-dn`), showing additions and deletions with
    /// associated line ranges.
    Rcs,
    /// Summary output format (`-ds`), showing only the number of chunks and
    /// lines added, deleted, or changed.
    Summary,
    /// Unified output format (`-du[num]`), showing added and deleted lines
    /// with `num` lines of context, in a form compatible with `patch(1)`.
    Unified(Option<u32>),
}

/// Whitespace handling of the diff routine passed via `-doptions`.
///
/// This is one of two orthogonal dimensions of [`DiffOptions`] (the other
/// being [`DiffFormat`]). The [`IgnoreChangesWithinWhitespace`] and
/// [`IgnoreAllWhitespace`] variants each imply [`IgnoreLineEndings`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WhitespaceHandling {
    /// No whitespace handling.
    #[default]
    None,
    /// Ignore line-ending (CR/LF) convention when finding diffs (`-dl`).
    IgnoreLineEndings,
    /// Ignore changes made within whitespace (`-db`); implies `-dl`.
    IgnoreChangesWithinWhitespace,
    /// Ignore whitespace altogether (`-dw`); implies `-dl`.
    IgnoreAllWhitespace,
}

/// Diff routine options passed via `-doptions`, shared by commands such as
/// `p4 diff`, `p4 diff2`, `p4 describe`, and `p4 annotate`.
///
/// The structured ([`Typed`](DiffOptions::Typed)) options split into two
/// orthogonal dimensions — [`DiffFormat`] (output format) and
/// [`WhitespaceHandling`] — each of which is internally mutually exclusive.
/// All options are assembled with a [`DiffOptionsBuilder`], whose mode type
/// parameter tracks at compile time whether structured or raw options are
/// being built; both builder states convert into this type via [`From`], and
/// command methods accept `impl Into<DiffOptions>` so a builder can be
/// passed directly.
///
/// # Examples
///
/// ```
/// use perforce_cli::cmd::{DiffOptions, DiffOptionsBuilder};
///
/// // `-dub`: unified format, ignore changes within whitespace
/// let opts = DiffOptions::from(
///     DiffOptionsBuilder::unified(None).ignore_changes_within_whitespace(),
/// );
///
/// // `-dc3`: context format with 3 lines of context
/// let opts = DiffOptions::from(DiffOptionsBuilder::context(Some(3)));
///
/// // `-d-C 25`: pass `-C 25` straight to an external diff program
/// let opts = DiffOptions::from(DiffOptionsBuilder::raw("-C 25"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOptions {
    /// Structured diff options (the documented subset of the standard UNIX
    /// diff flags).
    Typed {
        format: DiffFormat,
        whitespace: WhitespaceHandling,
    },
    /// Raw option string passed directly to the underlying diff routine
    /// (useful with an external diff program configured via `P4DIFF`).
    Raw(String),
}

impl Default for DiffOptions {
    fn default() -> Self {
        DiffOptions::Typed {
            format: DiffFormat::Default,
            whitespace: WhitespaceHandling::None,
        }
    }
}

impl DiffOptions {
    /// Returns the selected output format, or `None` if this is a
    /// [`Raw`](DiffOptions::Raw) value.
    pub fn format(&self) -> Option<DiffFormat> {
        match self {
            DiffOptions::Typed { format, .. } => Some(*format),
            DiffOptions::Raw(_) => None,
        }
    }

    /// Returns the selected whitespace handling, or `None` if this is a
    /// [`Raw`](DiffOptions::Raw) value.
    pub fn whitespace_handling(&self) -> Option<WhitespaceHandling> {
        match self {
            DiffOptions::Typed { whitespace, .. } => Some(*whitespace),
            DiffOptions::Raw(_) => None,
        }
    }

    /// Returns the raw option string, or `None` if this is a
    /// [`Typed`](DiffOptions::Typed) value.
    pub fn raw_str(&self) -> Option<&str> {
        match self {
            DiffOptions::Typed { .. } => None,
            DiffOptions::Raw(s) => Some(s),
        }
    }

    /// Injects the `-doptions` argument into `command`.
    ///
    /// If this is a [`Typed`](DiffOptions::Typed) value with both the format
    /// and whitespace handling at their defaults, no `-d` argument is emitted.
    pub fn inject_arg(&self, command: &mut Command) {
        use DiffFormat::*;
        use WhitespaceHandling::*;

        let (format, whitespace) = match self {
            DiffOptions::Typed { format, whitespace } => (*format, *whitespace),
            DiffOptions::Raw(s) => {
                command.arg(format!("-d{s}"));
                return;
            }
        };

        if matches!(format, Default) && matches!(whitespace, None) {
            return;
        }

        let mut s = String::from("-d");

        match format {
            Default => {}
            Context(num) => {
                s.push('c');
                if let Some(n) = num {
                    s.push_str(&n.to_string());
                }
            }
            Rcs => s.push('n'),
            Summary => s.push('s'),
            Unified(num) => {
                s.push('u');
                if let Some(n) = num {
                    s.push_str(&n.to_string());
                }
            }
        }

        match whitespace {
            None => {}
            IgnoreLineEndings => s.push('l'),
            IgnoreChangesWithinWhitespace => s.push('b'),
            IgnoreAllWhitespace => s.push('w'),
        }

        command.arg(s);
    }
}

impl From<String> for DiffOptions {
    fn from(s: String) -> Self {
        DiffOptions::Raw(s)
    }
}

/// The structured ([`Typed`](DiffOptions::Typed)) state of a
/// [`DiffOptionsBuilder`], holding the selected [`DiffFormat`] and
/// [`WhitespaceHandling`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiffOptionsTypedMode {
    format: DiffFormat,
    whitespace: WhitespaceHandling,
}

/// The raw ([`Raw`](DiffOptions::Raw)) state of a [`DiffOptionsBuilder`],
/// holding the option string passed verbatim to the underlying diff routine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffOptionsRawMode {
    raw: String,
}

/// Builder for [`DiffOptions`], shared by commands such as `p4 diff`,
/// `p4 diff2`, `p4 describe`, and `p4 annotate`.
///
/// The mode type parameter tracks at compile time which kind of options are
/// being assembled. The format constructors ([`DiffOptionsBuilder::context`],
/// [`DiffOptionsBuilder::rcs`], [`DiffOptionsBuilder::summary`],
/// [`DiffOptionsBuilder::unified`], and [`DiffOptionsBuilder::new`]) produce
/// the [`DiffOptionsTypedMode`] state, where the whitespace builders
/// ([`DiffOptionsBuilder::ignore_line_endings`],
/// [`DiffOptionsBuilder::ignore_changes_within_whitespace`],
/// [`DiffOptionsBuilder::ignore_all_whitespace`]) become available;
/// [`DiffOptionsBuilder::raw`] produces the [`DiffOptionsRawMode`] state,
/// which offers no further options — the two states never mix.
///
/// Both states convert into [`DiffOptions`] via [`From`], so command methods
/// that accept `impl Into<DiffOptions>` take a builder directly.
///
/// # Examples
///
/// ```
/// use perforce_cli::cmd::DiffOptionsBuilder;
///
/// // `-dub`: unified format, ignore changes within whitespace
/// let builder =
///     DiffOptionsBuilder::unified(None).ignore_changes_within_whitespace();
///
/// // `-dc3`: context format with 3 lines of context
/// let builder = DiffOptionsBuilder::context(Some(3));
///
/// // `-d-C 25`: pass `-C 25` straight to an external diff program
/// let builder = DiffOptionsBuilder::raw("-C 25");
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiffOptionsBuilder<M> {
    mode: M,
}

impl DiffOptionsBuilder<DiffOptionsTypedMode> {
    /// Creates a new builder in the [`DiffOptionsTypedMode`] state with the
    /// default format and no whitespace handling; the resulting
    /// [`DiffOptions`] emits no `-d` flag.
    pub fn new() -> Self {
        Self {
            mode: DiffOptionsTypedMode::default(),
        }
    }

    /// `-dc[num]`: context output format with `num` lines of context.
    pub fn context(num: Option<u32>) -> Self {
        Self {
            mode: DiffOptionsTypedMode {
                format: DiffFormat::Context(num),
                whitespace: WhitespaceHandling::None,
            },
        }
    }

    /// `-dn`: RCS output format.
    pub fn rcs() -> Self {
        Self {
            mode: DiffOptionsTypedMode {
                format: DiffFormat::Rcs,
                whitespace: WhitespaceHandling::None,
            },
        }
    }

    /// `-ds`: summary output format.
    pub fn summary() -> Self {
        Self {
            mode: DiffOptionsTypedMode {
                format: DiffFormat::Summary,
                whitespace: WhitespaceHandling::None,
            },
        }
    }

    /// `-du[num]`: unified output format with `num` lines of context.
    pub fn unified(num: Option<u32>) -> Self {
        Self {
            mode: DiffOptionsTypedMode {
                format: DiffFormat::Unified(num),
                whitespace: WhitespaceHandling::None,
            },
        }
    }

    /// `-dl`: ignore line-ending (CR/LF) convention when finding diffs.
    pub fn ignore_line_endings(mut self) -> Self {
        self.mode.whitespace = WhitespaceHandling::IgnoreLineEndings;
        self
    }

    /// `-db`: ignore changes made within whitespace (implies `-dl`).
    pub fn ignore_changes_within_whitespace(mut self) -> Self {
        self.mode.whitespace = WhitespaceHandling::IgnoreChangesWithinWhitespace;
        self
    }

    /// `-dw`: ignore whitespace altogether (implies `-dl`).
    pub fn ignore_all_whitespace(mut self) -> Self {
        self.mode.whitespace = WhitespaceHandling::IgnoreAllWhitespace;
        self
    }
}

impl DiffOptionsBuilder<DiffOptionsRawMode> {
    /// Passes an arbitrary option string directly to the underlying diff
    /// routine (for use with an external diff program).
    ///
    /// The string is appended to `-d` verbatim. For example,
    /// `DiffOptionsBuilder::raw("-C 25")` produces `-d-C 25`.
    pub fn raw(s: impl Into<String>) -> Self {
        Self {
            mode: DiffOptionsRawMode { raw: s.into() },
        }
    }
}

impl From<DiffOptionsBuilder<DiffOptionsTypedMode>> for DiffOptions {
    fn from(builder: DiffOptionsBuilder<DiffOptionsTypedMode>) -> Self {
        DiffOptions::Typed {
            format: builder.mode.format,
            whitespace: builder.mode.whitespace,
        }
    }
}

impl From<DiffOptionsBuilder<DiffOptionsRawMode>> for DiffOptions {
    fn from(builder: DiffOptionsBuilder<DiffOptionsRawMode>) -> Self {
        DiffOptions::Raw(builder.mode.raw)
    }
}

/// The default "no option selected" variant, shared by every
/// [`ExclusiveOption`] group.
///
/// It is a zero-sized type whose [`ExclusiveOption::inject_args`] is a no-op,
/// so any mutually exclusive option group can use it as its default type
/// parameter instead of defining its own empty variant.
#[derive(Debug, Clone, Copy, Default)]
pub struct Unselected;

impl ExclusiveOption for Unselected {
    fn inject_args(&self, _: &mut Command) {}
}

/// Marker trait for a mutually exclusive option group.
///
/// Some Perforce commands accept a set of options where only one may be
/// selected at a time (for example `p4 admin checkpoint [-z | -Z]` or
/// `p4 admin updatespecdepot [-a | -s type]`). Each variant of such a group
/// implements this trait to inject its own CLI arguments; the selected
/// variant is encoded in a type parameter so that the alternatives are
/// unavailable at compile time.
///
/// Variants may carry their own data (for example the `type` argument of
/// `-s type`) and inject any number of arguments, keeping the trait open to
/// option groups more complex than a single flag.
pub trait ExclusiveOption {
    #[allow(unused_variables)]
    /// Inject the CLI arguments corresponding to this selection into
    /// `command`.
    fn inject_args(&self, command: &mut Command) {}
}

pub trait SubCommand {
    fn name(&self) -> &str;

    fn inject_local_args(&self, command: &mut Command);

    fn global_opts(&self) -> Option<&GlobalOpts> {
        None
    }

    fn inject_args(&self, command: &mut Command) {
        // inject global opts
        if let Some(global_opts) = self.global_opts() {
            global_opts.setup_args(command);
        };
        // inject local opts
        self.inject_local_args(command.arg(self.name()));
    }

    fn setup_command<S: AsRef<OsStr>>(&self, bin: S) -> Command {
        let mut cmd = Command::new(bin);

        self.inject_args(&mut cmd);
        cmd
    }
}

/// Test-only helper: collects the arguments assembled on a [`Command`] as
/// strings for easy comparison.
#[cfg(test)]
pub(crate) fn args_of(command: &Command) -> Vec<String> {
    command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

#[cfg(test)]
mod diff_options_tests {
    use super::*;

    fn injected(opts: impl Into<DiffOptions>) -> Vec<String> {
        let mut cmd = Command::new("p4");
        opts.into().inject_arg(&mut cmd);
        args_of(&cmd)
    }

    #[test]
    fn default_emits_nothing() {
        assert!(injected(DiffOptionsBuilder::new()).is_empty());
        assert!(injected(DiffOptions::default()).is_empty());
    }

    #[test]
    fn unified_format() {
        assert_eq!(injected(DiffOptionsBuilder::unified(None)), ["-du"]);
    }

    #[test]
    fn unified_format_with_context() {
        assert_eq!(injected(DiffOptionsBuilder::unified(Some(3))), ["-du3"]);
    }

    #[test]
    fn context_format() {
        assert_eq!(injected(DiffOptionsBuilder::context(None)), ["-dc"]);
        assert_eq!(injected(DiffOptionsBuilder::context(Some(5))), ["-dc5"]);
    }

    #[test]
    fn rcs_format() {
        assert_eq!(injected(DiffOptionsBuilder::rcs()), ["-dn"]);
    }

    #[test]
    fn summary_format() {
        assert_eq!(injected(DiffOptionsBuilder::summary()), ["-ds"]);
    }

    #[test]
    fn whitespace_only() {
        assert_eq!(
            injected(DiffOptionsBuilder::new().ignore_line_endings()),
            ["-dl"]
        );
    }

    #[test]
    fn unified_with_ignore_changes_within_whitespace() {
        assert_eq!(
            injected(DiffOptionsBuilder::unified(None).ignore_changes_within_whitespace()),
            ["-dub"]
        );
    }

    #[test]
    fn context_with_ignore_all_whitespace() {
        assert_eq!(
            injected(DiffOptionsBuilder::context(Some(2)).ignore_all_whitespace()),
            ["-dc2w"]
        );
    }

    #[test]
    fn raw_passthrough() {
        assert_eq!(injected(DiffOptionsBuilder::raw("-C 25")), ["-d-C 25"]);
        assert_eq!(injected(DiffOptionsBuilder::raw("--brief")), ["-d--brief"]);
    }

    #[test]
    fn getters_reflect_variant() {
        let typed = DiffOptions::from(DiffOptionsBuilder::unified(Some(3)).ignore_line_endings());
        assert_eq!(typed.format(), Some(DiffFormat::Unified(Some(3))));
        assert_eq!(
            typed.whitespace_handling(),
            Some(WhitespaceHandling::IgnoreLineEndings)
        );
        assert_eq!(typed.raw_str(), None);

        let raw = DiffOptions::from(DiffOptionsBuilder::raw("abc"));
        assert_eq!(raw.format(), None);
        assert_eq!(raw.whitespace_handling(), None);
        assert_eq!(raw.raw_str(), Some("abc"));
    }
}
