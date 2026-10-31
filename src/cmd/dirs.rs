use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Workspace view form (-C -D -H -S stream -i --ignore-changeview)
// ---------------------------------------------------------------------------

/// The client-workspace-view form of `p4 dirs`.
///
/// Lists directories according to the current client view (`-C`), the client
/// have list (`-H`), or a stream (`-S stream`), optionally ignoring the case
/// of the directory argument (`-i`) and removing the changelist limit on
/// depot paths (`--ignore-changeview`).
#[derive(Debug, Clone, Default)]
pub struct WorkspaceView {
    client_mapped_only: bool,

    client_have_list_only: bool,

    #[cfg(not(feature = "lt2016_1"))]
    case_insensitive: bool,

    #[cfg(not(feature = "lt2025_2"))]
    ignore_changeview: bool,

    stream: Option<String>,
}

// ---------------------------------------------------------------------------
// Stream view form (-D --streamviews)
// ---------------------------------------------------------------------------

/// The stream-components form of `p4 dirs`.
///
/// Reports files and directories from stream paths and components with
/// `--streamviews`, where imported files are represented as symbolic links
/// under the stream depot path.
#[cfg(not(feature = "lt2022_2"))]
#[derive(Debug, Clone, Default)]
pub struct StreamView {}

// ---------------------------------------------------------------------------
// Mode dispatch
// ---------------------------------------------------------------------------

impl ExclusiveOption for WorkspaceView {
    fn inject_args(&self, command: &mut Command) {
        if self.client_mapped_only {
            command.arg("-C");
        }

        if self.client_have_list_only {
            command.arg("-H");
        }

        if let Some(stream) = &self.stream {
            command.arg("-S").arg(stream);
        }

        #[cfg(not(feature = "lt2016_1"))]
        if self.case_insensitive {
            command.arg("-i");
        }

        #[cfg(not(feature = "lt2025_2"))]
        if self.ignore_changeview {
            command.arg("--ignore-changeview");
        }
    }
}

#[cfg(not(feature = "lt2022_2"))]
impl ExclusiveOption for StreamView {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--streamviews");
    }
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

#[cfg_attr(
    feature = "lt2016_1",
    doc = "`p4 [g-opts] dirs [-C -D -H] [-S stream] depot_directory[revRange]...`"
)]
#[cfg_attr(
    all(not(feature = "lt2016_1"), feature = "lt2017_2"),
    doc = "`p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directory[revRange]...`"
)]
#[cfg_attr(
    all(not(feature = "lt2017_2"), feature = "lt2018_1"),
    doc = "`p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directory[revRange]...`",
    doc = "",
    doc = "`p4 [g-opts] dirs [-S stream] [-i] depot_directory[revRange]...`"
)]
#[cfg_attr(
    all(not(feature = "lt2018_1"), feature = "lt2022_2"),
    doc = "`p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directories*[revSpec]`"
)]
#[cfg_attr(
    all(not(feature = "lt2022_2"), feature = "lt2026_1"),
    doc = "`p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directories*[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] dirs [-D] [--streamviews] depot_directories*[revSpec]`"
)]
#[cfg_attr(
    not(feature = "lt2026_1"),
    doc = "`p4 [g-opts] dirs [-C -D -H] [-S stream] [-i] depot_directories*[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] dirs [-D] [--streamviews] depot_directories*[revSpec]`",
    doc = "",
    doc = "`p4 [g-opts] dirs [-D] [[-C] [-H] [-S stream] [-i] [--ignore-changeview] | [--streamviews]] depot_directories*[revSpec]`"
)]
///
/// Lists the immediate subdirectories of the specified depot directories.
///
/// `p4 dirs` is meant to be used in scripts rather than from the command line.
#[cfg_attr(
    feature = "lt2018_1",
    doc = "",
    doc = "The `depot_directory` argument must be provided in depot or local syntax and must end with the `*` wildcard."
)]
#[cfg_attr(
    all(not(feature = "lt2018_1"), feature = "lt2025_2"),
    doc = "",
    doc = "The `depot_directories` argument must be provided in depot or local syntax and must end with the `*` wildcard."
)]
#[cfg_attr(
    not(feature = "lt2025_2"),
    doc = "",
    doc = "If you use streams, you must provide the `depot_directories` argument in depot syntax. Otherwise, you can use depot or local syntax. The argument must end with the `*` wildcard."
)]
///
/// `p4 dirs` only lists the immediate subdirectories of the directory
/// arguments.
///
/// By default, only subdirectories that contain at least one undeleted file
/// are returned; [`Self::include_deleted`] (`-D`) also lists subdirectories
/// that contain only deleted files.
///
#[cfg_attr(
    feature = "lt2018_2",
    doc = "If you include a revision specifier or revision range as part of a directory argument, then the only subdirectories returned are those that contain at least one file revision that matches the given specifier."
)]
#[cfg_attr(
    not(feature = "lt2018_2"),
    doc = "If you include a revision specifier or revision range as part of a directory argument, the only subdirectories returned are those that contain at least one file revision that matches the given specifier."
)]
///
/// # Usage notes
///
#[cfg_attr(
    feature = "lt2024_1",
    doc = "Perforce does not track directories in its database. Therefore, the `...` wildcard is not supported."
)]
#[cfg_attr(
    all(not(feature = "lt2024_1"), feature = "lt2024_2"),
    doc = "Helix Core Server does not track directories in its database. Therefore, the `...` wildcard is not supported."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "P4 Server does not track directories in its database. Therefore, the `...` wildcard is not supported."
)]
#[derive(Debug, Clone, Default)]
pub struct Dirs<M = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    include_deleted: bool,

    mode: M,
}

impl Dirs<Unselected> {
    /// Creates a new `p4 dirs` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            include_deleted: false,
            mode: Unselected,
        }
    }

    /// # Description
    ///
    /// -C
    ///
    /// Display only those directories that are mapped through the current
    /// client view.
    ///
    /// Enters the [`WorkspaceView`] form with [`WorkspaceView`]'s
    /// [`client_mapped_only`](Dirs::client_mapped_only) flag set.
    pub fn client_mapped_only(self) -> Dirs<WorkspaceView> {
        Dirs {
            bin: self.bin,
            global_opts: self.global_opts,
            include_deleted: self.include_deleted,
            mode: WorkspaceView {
                client_mapped_only: true,
                ..Default::default()
            },
        }
    }

    /// # Description
    ///
    /// -H
    ///
    /// Include only those directories that contain files on the current
    /// client workspace's `p4 have` list.
    ///
    /// Enters the [`WorkspaceView`] form with [`WorkspaceView`]'s
    /// [`client_have_list_only`](Dirs::client_have_list_only) flag set.
    pub fn client_have_list_only(self) -> Dirs<WorkspaceView> {
        Dirs {
            bin: self.bin,
            global_opts: self.global_opts,
            include_deleted: self.include_deleted,
            mode: WorkspaceView {
                client_have_list_only: true,
                ..Default::default()
            },
        }
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List directories mapped for the specified stream.
    ///
    /// Enters the [`WorkspaceView`] form with the stream selected.
    pub fn stream(self, v: impl Into<String>) -> Dirs<WorkspaceView> {
        Dirs {
            bin: self.bin,
            global_opts: self.global_opts,
            include_deleted: self.include_deleted,
            mode: WorkspaceView {
                stream: Some(v.into()),
                ..Default::default()
            },
        }
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the directory argument when listing directories in
    /// a case-sensitive server.
    ///
    /// Enters the [`WorkspaceView`] form with [`WorkspaceView`]'s
    /// [`case_insensitive`](Dirs::case_insensitive) flag set.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn case_insensitive(self) -> Dirs<WorkspaceView> {
        Dirs {
            bin: self.bin,
            global_opts: self.global_opts,
            include_deleted: self.include_deleted,
            mode: WorkspaceView {
                case_insensitive: true,
                ..Default::default()
            },
        }
    }

    /// # Description
    ///
    /// `--ignore-changeview`
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// [`p4 client`](crate::cmd::Client).
    ///
    /// Enters the [`WorkspaceView`] form with [`WorkspaceView`]'s
    /// [`ignore_changeview`](Dirs::ignore_changeview) flag set.
    #[cfg(not(feature = "lt2025_2"))]
    pub fn ignore_changeview(self) -> Dirs<WorkspaceView> {
        Dirs {
            bin: self.bin,
            global_opts: self.global_opts,
            include_deleted: self.include_deleted,
            mode: WorkspaceView {
                ignore_changeview: true,
                ..Default::default()
            },
        }
    }

    /// # Description
    ///
    /// `--streamviews`
    ///
    /// Report files and directories from stream paths and components, where
    /// imported files are represented as symbolic links under the stream
    /// depot path.
    ///
    /// Enters the [`StreamView`] form.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn stream_view(self) -> Dirs<StreamView> {
        Dirs {
            bin: self.bin,
            global_opts: self.global_opts,
            include_deleted: self.include_deleted,
            mode: StreamView::default(),
        }
    }
}

impl Dirs<WorkspaceView> {
    /// # Description
    ///
    /// -C
    ///
    /// Display only those directories that are mapped through the current
    /// client view.
    pub fn get_client_mapped_only(&self) -> bool {
        self.mode.client_mapped_only
    }

    /// # Description
    ///
    /// -C
    ///
    /// Display only those directories that are mapped through the current
    /// client view.
    pub fn set_client_mapped_only(&mut self, v: bool) -> &mut Self {
        self.mode.client_mapped_only = v;
        self
    }

    /// # Description
    ///
    /// -C
    ///
    /// Display only those directories that are mapped through the current
    /// client view.
    pub fn client_mapped_only(mut self, v: bool) -> Self {
        self.set_client_mapped_only(v);
        self
    }

    /// # Description
    ///
    /// -H
    ///
    /// Include only those directories that contain files on the current
    /// client workspace's `p4 have` list.
    pub fn get_client_have_list_only(&self) -> bool {
        self.mode.client_have_list_only
    }

    /// # Description
    ///
    /// -H
    ///
    /// Include only those directories that contain files on the current
    /// client workspace's `p4 have` list.
    pub fn set_client_have_list_only(&mut self, v: bool) -> &mut Self {
        self.mode.client_have_list_only = v;
        self
    }

    /// # Description
    ///
    /// -H
    ///
    /// Include only those directories that contain files on the current
    /// client workspace's `p4 have` list.
    pub fn client_have_list_only(mut self, v: bool) -> Self {
        self.set_client_have_list_only(v);
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List directories mapped for the specified stream.
    pub fn get_stream(&self) -> Option<&str> {
        self.mode.stream.as_deref()
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List directories mapped for the specified stream.
    pub fn set_stream(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.stream = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -S stream
    ///
    /// List directories mapped for the specified stream.
    pub fn stream(mut self, v: impl Into<String>) -> Self {
        self.set_stream(v);
        self
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the directory argument when listing directories in
    /// a case-sensitive server.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_case_insensitive(&self) -> bool {
        self.mode.case_insensitive
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the directory argument when listing directories in
    /// a case-sensitive server.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_case_insensitive(&mut self, v: bool) -> &mut Self {
        self.mode.case_insensitive = v;
        self
    }

    /// # Description
    ///
    /// -i
    ///
    /// Ignore the case of the directory argument when listing directories in
    /// a case-sensitive server.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn case_insensitive(mut self, v: bool) -> Self {
        self.set_case_insensitive(v);
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
        self.mode.ignore_changeview
    }

    /// # Description
    ///
    /// `--ignore-changeview`
    ///
    /// Remove the changelist limit on depot paths. See ChangeView in
    /// [`p4 client`](crate::cmd::Client).
    #[cfg(not(feature = "lt2025_2"))]
    pub fn set_ignore_changeview(&mut self, v: bool) -> &mut Self {
        self.mode.ignore_changeview = v;
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

impl<M: ExclusiveOption> Dirs<M> {
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
    /// -D
    ///
    /// Include subdirectories that contain only deleted files. By default,
    /// these directories are not displayed.
    pub fn get_include_deleted(&self) -> bool {
        self.include_deleted
    }

    /// # Description
    ///
    /// -D
    ///
    /// Include subdirectories that contain only deleted files. By default,
    /// these directories are not displayed.
    pub fn set_include_deleted(&mut self, v: bool) -> &mut Self {
        self.include_deleted = v;
        self
    }

    /// # Description
    ///
    /// -D
    ///
    /// Include subdirectories that contain only deleted files. By default,
    /// these directories are not displayed.
    pub fn include_deleted(mut self, v: bool) -> Self {
        self.set_include_deleted(v);
        self
    }
}

impl<M: ExclusiveOption> SubCommand for Dirs<M> {
    fn name(&self) -> &str {
        "dirs"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.include_deleted {
            command.arg("-D");
        }

        self.mode.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption> ParameterizedSpawn<()> for Dirs<M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dirs` without directory arguments as a child process with
    /// piped standard output and error streams.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<M, S, I> ParameterizedSpawn<(S,)> for Dirs<M>
where
    M: ExclusiveOption,
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 dirs` for the given depot directories as a child process
    /// with piped standard output and error streams.
    fn spawn_with(&mut self, (directories,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(directories)
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
        let cmd = Dirs::new("p4", GlobalOpts::new());
        assert_eq!(args(&cmd), ["dirs"]);
    }

    #[test]
    fn include_deleted_is_shared_by_every_form() {
        let cmd = Dirs::new("p4", GlobalOpts::new()).include_deleted(true);
        assert_eq!(args(&cmd), ["dirs", "-D"]);

        let cmd = Dirs::new("p4", GlobalOpts::new())
            .client_mapped_only()
            .include_deleted(true);
        assert_eq!(args(&cmd), ["dirs", "-D", "-C"]);

        #[cfg(not(feature = "lt2022_2"))]
        {
            let cmd = Dirs::new("p4", GlobalOpts::new())
                .stream_view()
                .include_deleted(true);
            assert_eq!(args(&cmd), ["dirs", "-D", "--streamviews"]);
        }
    }

    #[test]
    fn workspace_view_arguments() {
        let cmd = Dirs::new("p4", GlobalOpts::new())
            .client_mapped_only()
            .client_have_list_only(true)
            .stream("main")
            .include_deleted(true);

        assert_eq!(args(&cmd), ["dirs", "-D", "-C", "-H", "-S", "main"]);
    }

    #[test]
    fn stream_accessors() {
        let mut cmd = Dirs::new("p4", GlobalOpts::new()).stream("release");
        assert_eq!(cmd.get_stream(), Some("release"));

        cmd.set_stream("main");
        assert_eq!(cmd.get_stream(), Some("main"));
        assert_eq!(args(&cmd), ["dirs", "-S", "main"]);
    }

    #[cfg(not(feature = "lt2016_1"))]
    #[test]
    fn case_insensitive_flag() {
        let cmd = Dirs::new("p4", GlobalOpts::new()).case_insensitive();
        assert_eq!(args(&cmd), ["dirs", "-i"]);

        let mut cmd = Dirs::new("p4", GlobalOpts::new()).client_mapped_only();
        assert!(!cmd.get_case_insensitive());

        cmd.set_case_insensitive(true);
        assert!(cmd.get_case_insensitive());
        assert_eq!(args(&cmd), ["dirs", "-C", "-i"]);
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn streamviews_flag() {
        let cmd = Dirs::new("p4", GlobalOpts::new()).stream_view();
        assert_eq!(args(&cmd), ["dirs", "--streamviews"]);
    }

    #[cfg(not(feature = "lt2025_2"))]
    #[test]
    fn ignore_changeview_flag() {
        let cmd = Dirs::new("p4", GlobalOpts::new()).ignore_changeview();
        assert_eq!(args(&cmd), ["dirs", "--ignore-changeview"]);

        let mut cmd = Dirs::new("p4", GlobalOpts::new()).client_mapped_only();
        assert!(!cmd.get_ignore_changeview());

        cmd.set_ignore_changeview(true);
        assert!(cmd.get_ignore_changeview());
        assert_eq!(args(&cmd), ["dirs", "-C", "--ignore-changeview"]);
    }

    #[test]
    fn global_opts_are_injected_once() {
        let cmd = Dirs::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .stream("main");

        assert_eq!(
            args(&cmd),
            ["-p", "localhost:1666", "-q", "dirs", "-S", "main"]
        );
    }
}
