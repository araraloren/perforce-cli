use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Options that only the submitted-files syntax form accepts (`-k` and
/// `-m depth`).
///
/// `p4 fetch -s shelf` rejects both, so they live in their own type state:
/// once any of them is selected the command has entered
/// `Fetch<Submitted<M, SubmittedOnlyOpts, S>>`, and [`Fetch::shelved`] is no
/// longer reachable.
#[derive(Debug, Clone, Default)]
pub struct SubmittedOnlyOpts {
    /// `-k`: do not sync the client workspace after fetching.
    suppress_workspace_sync: bool,

    /// `-m depth`: number of revisions of each file to fetch.
    depth: Option<u64>,
}

/// `-n` mode of `p4 fetch`: show what `p4 fetch` would do without actually
/// fetching files or altering the database.
#[derive(Debug, Clone, Default)]
pub struct DryRun;

/// Tangent-relocate mode of `p4 fetch`: after fetching from a remote server,
/// fetch fixes from the remote server and relocate them in a single tangent
/// depot.
///
/// The CLI flag is `-u` in 2015.1 and `-t` from 2015.2 onward; the injected
/// argument is selected at compile time.
#[derive(Debug, Clone, Default)]
pub struct TangentRelocate;

/// `-S stream` target of `p4 fetch`: fetch the spec and history of the
/// specific stream.
#[derive(Debug, Clone, Default)]
pub struct Stream {
    name: String,
}

impl ExclusiveOption for DryRun {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-n");
    }
}
impl ExclusiveOption for TangentRelocate {
    fn inject_args(&self, command: &mut Command) {
        #[cfg(feature = "lt2015_2")]
        command.arg("-u");
        #[cfg(not(feature = "lt2015_2"))]
        command.arg("-t");
    }
}

impl ExclusiveOption for Stream {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-S").arg(&self.name);
    }
}

/// First syntax form of `p4 fetch`: fetch submitted files.
///
#[cfg_attr(
    feature = "lt2015_2",
    doc = "`p4 [g-opts] fetch [-r remotespec] [-m depth] [-v -k] [-n | -u] [-S stream | filespec]`"
)]
#[cfg_attr(
    all(feature = "lt2016_1", not(feature = "lt2015_2")),
    doc = "`p4 [g-opts] fetch [-r remotespec] [-m depth] [-v -k] [-n | -t] [-S stream | filespec]`"
)]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2016_1")),
    doc = "`p4 [g-opts] fetch [-r remotespec] [-m depth] [-v -k] [-n | -t] [-Ox] [-S stream | filespec]`"
)]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "`p4 [g-opts] fetch [-r remotespec] [-m depth] [-v -k] [-n | -t] [-Ox] [-S stream | FileSpec[revSpec]]`"
)]
/// The `M` type parameter tracks the fetch mode ([`DryRun`] or
/// [`TangentRelocate`]), the `O` type parameter the submitted-only options
/// (`-m depth` / `-k`), and the `S` type parameter the target (`-S stream`).
#[derive(Debug, Clone, Default)]
pub struct Submitted<M = Unselected, O = Unselected, S = Unselected> {
    mode: M,
    opts: O,
    target: S,
}

/// Second syntax form of `p4 fetch`: fetch a shelved changelist.
///
/// `p4 [g-opts] fetch [-r remotespec] [-v] [-n] [-Ox] -s shelf`
#[cfg(not(feature = "lt2016_1"))]
#[derive(Debug, Clone, Default)]
pub struct Shelved {
    shelf: String,
    dry_run: bool,
}

/// `p4 fetch` — Copy files from a remote server into a local server.
///
/// The `M` type parameter tracks the syntax form: it starts as
/// [`Submitted`] with both its mode and target [`Unselected`], and
/// [`Fetch::shelved`] switches to the [`Shelved`] form.
#[derive(Debug, Clone, Default)]
pub struct Fetch<M> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    mode: M,

    remote_spec: Option<String>,

    #[cfg(not(feature = "lt2020_2"))]
    remote_user: Option<String>,

    verbose: bool,

    #[cfg(not(feature = "lt2016_1"))]
    output_changelist: bool,

    #[cfg(not(feature = "lt2016_1"))]
    output_files: bool,

    #[cfg(not(feature = "lt2016_1"))]
    output_integration: bool,
}

impl Fetch<Submitted> {
    /// Creates a new `p4 fetch` command in the first syntax form (fetch
    /// submitted files), with no mode (`-n` or tangent-relocate), no
    /// submitted-only option (`-m depth` / `-k`), and no target
    /// (`-S stream` / filespec) selected.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            mode: Submitted {
                mode: Unselected,
                opts: Unselected,
                target: Unselected,
            },
            remote_spec: None,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: None,
            verbose: false,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: false,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: false,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: false,
        }
    }

    /// Select the second syntax form: fetch a shelved changelist.
    ///
    /// Only available while the submitted-only options (`-m depth` / `-k`),
    /// the tangent-relocate mode, and the `-S stream` target have not been
    /// selected: `p4 fetch -s shelf` accepts none of them, and the type state
    /// makes them unreachable here. The remaining options (`-r`,
    /// `--remote-user`, `-v`, `-Ox`) carry over.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn shelved(self, shelf: impl Into<String>) -> Fetch<Shelved> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Shelved {
                shelf: shelf.into(),
                dry_run: false,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }
}

/// `-n` is the only mode both syntax forms accept, so dry-run stays on the
/// path to the shelved form and carries over as `-n`.
#[cfg(not(feature = "lt2016_1"))]
impl Fetch<Submitted<DryRun>> {
    /// Select the second syntax form: show what fetching a shelved changelist
    /// would do without actually fetching files or altering the database.
    pub fn shelved(self, shelf: impl Into<String>) -> Fetch<Shelved> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Shelved {
                shelf: shelf.into(),
                dry_run: true,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }
}

/// Mode selection, available only while no mode has been chosen.
///
/// `-n` and tangent-relocate are mutually exclusive in the synopsis, so
/// choosing one moves `M` away from [`Unselected`] and the alternatives
/// become unreachable.
impl<O, S> Fetch<Submitted<Unselected, O, S>> {
    /// Select `-n` mode: show what `p4 fetch` would do without actually
    /// fetching files or altering the database.
    pub fn dry_run(self) -> Fetch<Submitted<DryRun, O, S>> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Submitted {
                mode: DryRun,
                target: self.mode.target,
                opts: self.mode.opts,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }

    /// Select tangent-relocate mode: after fetching from a remote server,
    /// fetch fixes from the remote server and relocate them in a single
    /// tangent depot.
    ///
    /// In 2015.1 this injects `-u`; from 2015.2 onward it injects `-t`.
    pub fn tangent_relocate(self) -> Fetch<Submitted<TangentRelocate, O, S>> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Submitted {
                mode: TangentRelocate,
                target: self.mode.target,
                opts: self.mode.opts,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }
}

impl<M: ExclusiveOption, O, S: ExclusiveOption> Fetch<Submitted<M, O, S>> {
    /// Select `-S stream` target: fetch the spec and history of the specific
    /// stream.
    pub fn stream(self, name: impl Into<String>) -> Fetch<Submitted<M, O, Stream>> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Submitted {
                mode: self.mode.mode,
                target: Stream { name: name.into() },
                opts: self.mode.opts,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }

    /// Inject the options common to both states of the `O` type parameter,
    /// followed by the selected mode and target.
    fn inject_submitted_args(&self, command: &mut Command, depth: Option<u64>, no_sync: bool) {
        if let Some(remote_spec) = &self.remote_spec {
            command.arg("-r").arg(remote_spec);
        }

        #[cfg(not(feature = "lt2020_2"))]
        if let Some(remote_user) = &self.remote_user {
            command.arg("--remote-user").arg(remote_user);
        }

        if let Some(depth) = depth {
            command.arg("-m").arg(depth.to_string());
        }

        if self.verbose {
            command.arg("-v");
        }

        if no_sync {
            command.arg("-k");
        }

        self.mode.mode.inject_args(command);

        #[cfg(not(feature = "lt2016_1"))]
        {
            if self.output_changelist {
                command.arg("-Oc");
            }

            if self.output_files {
                command.arg("-Of");
            }

            if self.output_integration {
                command.arg("-Oi");
            }
        }

        self.mode.target.inject_args(command);
    }
}

/// Submitted-only options, selectable while none of them has been set.
///
/// Choosing either one moves the command into the [`SubmittedOnlyOpts`]
/// state; from there [`Fetch::shelved`] is no longer reachable, because
/// `p4 fetch -s shelf` accepts neither `-m` nor `-k`.
impl<M: ExclusiveOption, S: ExclusiveOption> Fetch<Submitted<M, Unselected, S>> {
    /// # Description
    ///
    /// -m depth
    ///
    /// Limits the number of revisions of each file that is fetched. By
    /// default all revisions are fetched.
    ///
    /// The value of `depth` corresponds to the number of revisions of each
    /// file that is fetched. For example, to fetch only the most recent
    /// revision of each file, use a depth of `1`.
    pub fn depth(self, v: u64) -> Fetch<Submitted<M, SubmittedOnlyOpts, S>> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Submitted {
                mode: self.mode.mode,
                opts: SubmittedOnlyOpts {
                    suppress_workspace_sync: false,
                    depth: Some(v),
                },
                target: self.mode.target,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }

    /// # Description
    ///
    /// -k
    ///
    /// Do not sync the client workspace after fetching.
    ///
    /// Selecting this option moves the command into the [`SubmittedOnlyOpts`]
    /// state, where [`Fetch::shelved`] is no longer reachable.
    pub fn suppress_workspace_sync(self, v: bool) -> Fetch<Submitted<M, SubmittedOnlyOpts, S>> {
        Fetch {
            bin: self.bin,
            global_opts: self.global_opts,
            mode: Submitted {
                mode: self.mode.mode,
                opts: SubmittedOnlyOpts {
                    suppress_workspace_sync: v,
                    depth: None,
                },
                target: self.mode.target,
            },
            remote_spec: self.remote_spec,
            #[cfg(not(feature = "lt2020_2"))]
            remote_user: self.remote_user,
            verbose: self.verbose,
            #[cfg(not(feature = "lt2016_1"))]
            output_changelist: self.output_changelist,
            #[cfg(not(feature = "lt2016_1"))]
            output_files: self.output_files,
            #[cfg(not(feature = "lt2016_1"))]
            output_integration: self.output_integration,
        }
    }
}

impl<M: ExclusiveOption, S: ExclusiveOption> Fetch<Submitted<M, SubmittedOnlyOpts, S>> {
    /// # Description
    ///
    /// -m depth
    ///
    /// Limits the number of revisions of each file that is fetched. By
    /// default all revisions are fetched.
    ///
    /// The value of `depth` corresponds to the number of revisions of each
    /// file that is fetched. For example, to fetch only the most recent
    /// revision of each file, use a depth of `1`.
    pub fn get_depth(&self) -> Option<u64> {
        self.mode.opts.depth
    }

    /// # Description
    ///
    /// -m depth
    ///
    /// Limits the number of revisions of each file that is fetched. By
    /// default all revisions are fetched.
    ///
    /// The value of `depth` corresponds to the number of revisions of each
    /// file that is fetched. For example, to fetch only the most recent
    /// revision of each file, use a depth of `1`.
    pub fn set_depth(&mut self, v: u64) -> &mut Self {
        self.mode.opts.depth = Some(v);
        self
    }

    /// # Description
    ///
    /// -m depth
    ///
    /// Limits the number of revisions of each file that is fetched. By
    /// default all revisions are fetched.
    ///
    /// The value of `depth` corresponds to the number of revisions of each
    /// file that is fetched. For example, to fetch only the most recent
    /// revision of each file, use a depth of `1`.
    pub fn depth(mut self, v: u64) -> Self {
        self.set_depth(v);
        self
    }

    /// # Description
    ///
    /// -k
    ///
    /// Do not sync the client workspace after fetching.
    pub fn get_suppress_workspace_sync(&self) -> bool {
        self.mode.opts.suppress_workspace_sync
    }

    /// # Description
    ///
    /// -k
    ///
    /// Do not sync the client workspace after fetching.
    pub fn set_suppress_workspace_sync(&mut self, v: bool) -> &mut Self {
        self.mode.opts.suppress_workspace_sync = v;
        self
    }

    /// # Description
    ///
    /// -k
    ///
    /// Do not sync the client workspace after fetching.
    pub fn suppress_workspace_sync(mut self, v: bool) -> Self {
        self.set_suppress_workspace_sync(v);
        self
    }
}

#[cfg(not(feature = "lt2016_1"))]
impl Fetch<Shelved> {
    /// # Description
    ///
    /// -n
    ///
    /// Show what `p4 fetch` would do without actually fetching files or
    /// altering the database.
    pub fn get_dry_run(&self) -> bool {
        self.mode.dry_run
    }

    /// # Description
    ///
    /// -n
    ///
    /// Show what `p4 fetch` would do without actually fetching files or
    /// altering the database.
    pub fn set_dry_run(&mut self, v: bool) -> &mut Self {
        self.mode.dry_run = v;
        self
    }

    /// # Description
    ///
    /// -n
    ///
    /// Show what `p4 fetch` would do without actually fetching files or
    /// altering the database.
    pub fn dry_run(mut self, v: bool) -> Self {
        self.set_dry_run(v);
        self
    }
}

impl<M> Fetch<M> {
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
    /// See [Global options](GlobalOpts).
    pub fn get_global_opts(&self) -> &GlobalOpts {
        &self.global_opts
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn set_global_opts(&mut self, v: GlobalOpts) -> &mut Self {
        self.global_opts = v;
        self
    }

    /// # Description
    ///
    /// g-opts
    ///
    /// See [Global options](GlobalOpts).
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.global_opts = v;
        self
    }

    /// # Description
    ///
    /// -r remotespec
    ///
    /// A remote spec previously defined on the local server with `p4 remote`.
    /// Use the remote spec to limit what `p4 fetch` fetches.
    ///
    /// The `p4 remote` command places no limits on the configuration of a
    /// remote. Using a remote spec with `p4 fetch` overrides the remote
    /// spec's configuration.
    pub fn get_remote_spec(&self) -> Option<&str> {
        self.remote_spec.as_deref()
    }

    /// # Description
    ///
    /// -r remotespec
    ///
    /// A remote spec previously defined on the local server with `p4 remote`.
    /// Use the remote spec to limit what `p4 fetch` fetches.
    ///
    /// The `p4 remote` command places no limits on the configuration of a
    /// remote. Using a remote spec with `p4 fetch` overrides the remote
    /// spec's configuration.
    pub fn set_remote_spec(&mut self, v: impl Into<String>) -> &mut Self {
        self.remote_spec = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -r remotespec
    ///
    /// A remote spec previously defined on the local server with `p4 remote`.
    /// Use the remote spec to limit what `p4 fetch` fetches.
    ///
    /// The `p4 remote` command places no limits on the configuration of a
    /// remote. Using a remote spec with `p4 fetch` overrides the remote
    /// spec's configuration.
    pub fn remote_spec(mut self, v: impl Into<String>) -> Self {
        self.set_remote_spec(v);
        self
    }

    /// # Description
    ///
    /// --remote-user
    ///
    /// The `Helix Server` user to authenticate as when connecting to the
    /// remote server.
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "The `Helix Server` user to authenticate as when connecting to the remote server."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "The `P4 Server` user to authenticate as when connecting to the remote server."
    )]
    ///
    /// This is useful when the remote server has a different set of users
    /// than the local server, or when you want to perform the fetch as a
    /// different user than the one you are logged in as locally.
    #[cfg(not(feature = "lt2020_2"))]
    pub fn get_remote_user(&self) -> Option<&str> {
        self.remote_user.as_deref()
    }

    /// # Description
    ///
    /// --remote-user
    ///
    /// The `Helix Server` user to authenticate as when connecting to the
    /// remote server.
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "The `Helix Server` user to authenticate as when connecting to the remote server."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "The `P4 Server` user to authenticate as when connecting to the remote server."
    )]
    ///
    /// This is useful when the remote server has a different set of users
    /// than the local server, or when you want to perform the fetch as a
    /// different user than the one you are logged in as locally.
    #[cfg(not(feature = "lt2020_2"))]
    pub fn set_remote_user(&mut self, v: impl Into<String>) -> &mut Self {
        self.remote_user = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --remote-user
    ///
    /// The `Helix Server` user to authenticate as when connecting to the
    /// remote server.
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "The `Helix Server` user to authenticate as when connecting to the remote server."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "The `P4 Server` user to authenticate as when connecting to the remote server."
    )]
    ///
    /// This is useful when the remote server has a different set of users
    /// than the local server, or when you want to perform the fetch as a
    /// different user than the one you are logged in as locally.
    #[cfg(not(feature = "lt2020_2"))]
    pub fn remote_user(mut self, v: impl Into<String>) -> Self {
        self.set_remote_user(v);
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Show verbose output during the fetch process.
    pub fn get_verbose(&self) -> bool {
        self.verbose
    }

    /// # Description
    ///
    /// -v
    ///
    /// Show verbose output during the fetch process.
    pub fn set_verbose(&mut self, v: bool) -> &mut Self {
        self.verbose = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Show verbose output during the fetch process.
    pub fn verbose(mut self, v: bool) -> Self {
        self.set_verbose(v);
        self
    }

    /// # Description
    ///
    /// -Oc
    ///
    /// Output the changelist.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_output_changelist(&self) -> bool {
        self.output_changelist
    }

    /// # Description
    ///
    /// -Oc
    ///
    /// Output the changelist.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_output_changelist(&mut self, v: bool) -> &mut Self {
        self.output_changelist = v;
        self
    }

    /// # Description
    ///
    /// -Oc
    ///
    /// Output the changelist.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn output_changelist(mut self, v: bool) -> Self {
        self.set_output_changelist(v);
        self
    }

    /// # Description
    ///
    /// -Of
    ///
    /// Output the files.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_output_files(&self) -> bool {
        self.output_files
    }

    /// # Description
    ///
    /// -Of
    ///
    /// Output the files.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_output_files(&mut self, v: bool) -> &mut Self {
        self.output_files = v;
        self
    }

    /// # Description
    ///
    /// -Of
    ///
    /// Output the files.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn output_files(mut self, v: bool) -> Self {
        self.set_output_files(v);
        self
    }

    /// # Description
    ///
    /// -Oi
    ///
    /// Output the integration records.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn get_output_integration(&self) -> bool {
        self.output_integration
    }

    /// # Description
    ///
    /// -Oi
    ///
    /// Output the integration records.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn set_output_integration(&mut self, v: bool) -> &mut Self {
        self.output_integration = v;
        self
    }

    /// # Description
    ///
    /// -Oi
    ///
    /// Output the integration records.
    #[cfg(not(feature = "lt2016_1"))]
    pub fn output_integration(mut self, v: bool) -> Self {
        self.set_output_integration(v);
        self
    }
}

impl<M: ExclusiveOption, S: ExclusiveOption> SubCommand for Fetch<Submitted<M, Unselected, S>> {
    fn name(&self) -> &str {
        "fetch"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.inject_submitted_args(command, None, false);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption, S: ExclusiveOption> SubCommand
    for Fetch<Submitted<M, SubmittedOnlyOpts, S>>
{
    fn name(&self) -> &str {
        "fetch"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.inject_submitted_args(
            command,
            self.mode.opts.depth,
            self.mode.opts.suppress_workspace_sync,
        );
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

#[cfg(not(feature = "lt2016_1"))]
impl SubCommand for Fetch<Shelved> {
    fn name(&self) -> &str {
        "fetch"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(remote_spec) = &self.remote_spec {
            command.arg("-r").arg(remote_spec);
        }

        #[cfg(not(feature = "lt2020_2"))]
        if let Some(remote_user) = &self.remote_user {
            command.arg("--remote-user").arg(remote_user);
        }

        if self.verbose {
            command.arg("-v");
        }

        if self.mode.dry_run {
            command.arg("-n");
        }

        #[cfg(not(feature = "lt2016_1"))]
        {
            if self.output_changelist {
                command.arg("-Oc");
            }

            if self.output_files {
                command.arg("-Of");
            }

            if self.output_integration {
                command.arg("-Oi");
            }
        }

        command.arg("-s").arg(&self.mode.shelf);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<M: ExclusiveOption, O, S: ExclusiveOption> ParameterizedSpawn<()> for Fetch<Submitted<M, O, S>>
where
    Self: SubCommand,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 fetch` as a child process with piped standard output and
    /// error streams; use the returned [`Child`] handle to wait for it or
    /// interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<M: ExclusiveOption, O, S, I> ParameterizedSpawn<(S,)> for Fetch<Submitted<M, O, Unselected>>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
    Self: SubCommand,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 fetch` for the given file specifications as a child process
    /// with piped standard output and error streams; use the returned
    /// [`Child`] handle to wait for it or interact with it.
    fn spawn_with(&mut self, (specs,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(specs)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(not(feature = "lt2016_1"))]
impl ParameterizedSpawn<()> for Fetch<Shelved> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 fetch` as a child process with piped standard output and
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
        let mut command = Fetch::new("p4", GlobalOpts::new());
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    #[test]
    fn submitted_bare() {
        let fetch = Fetch::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch"]);
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn shelved_bare() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).shelved("42");

        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-s", "42"]);
    }

    #[test]
    fn submitted_with_remote_spec() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).remote_spec("origin");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-r", "origin"]
        );
    }

    #[test]
    fn submitted_with_depth() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).depth(1);

        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-m", "1"]);
    }

    #[test]
    fn submitted_with_no_sync() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).suppress_workspace_sync(true);

        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-k"]);
    }

    #[test]
    fn submitted_with_verbose() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).verbose(true);

        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-v"]);
    }

    #[test]
    fn submitted_tangent_relocate() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).tangent_relocate();

        #[cfg(feature = "lt2015_2")]
        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-u"]);
        #[cfg(not(feature = "lt2015_2"))]
        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-t"]);
    }

    #[test]
    fn submitted_dry_run() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).dry_run();

        assert_eq!(args_of(&fetch.setup_command("p4")), ["fetch", "-n"]);
    }

    #[test]
    fn submitted_stream_target() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).stream("//stream/dev");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-S", "//stream/dev"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2020_2"))]
    fn submitted_with_remote_user() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).remote_user("bruno");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "--remote-user", "bruno"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn submitted_output_flags() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .output_changelist(true)
            .output_files(true)
            .output_integration(true);

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-Oc", "-Of", "-Oi"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn shelved_dry_run() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .shelved("42")
            .dry_run(true);

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-n", "-s", "42"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn dry_run_then_shelved() {
        let fetch = Fetch::new("p4", GlobalOpts::new()).dry_run().shelved("42");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-n", "-s", "42"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2020_2"))]
    fn shelved_with_remote_user() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .shelved("42")
            .remote_user("bruno");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "--remote-user", "bruno", "-s", "42"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn shelved_with_remote_spec() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .shelved("42")
            .remote_spec("origin");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-r", "origin", "-s", "42"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn shelved_output_flags() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .shelved("42")
            .output_changelist(true)
            .output_files(true);

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-Oc", "-Of", "-s", "42"]
        );
    }

    #[test]
    fn transitions_preserve_options() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .remote_spec("origin")
            .depth(5)
            .suppress_workspace_sync(true)
            .tangent_relocate();

        #[cfg(feature = "lt2015_2")]
        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-r", "origin", "-m", "5", "-k", "-u"]
        );
        #[cfg(not(feature = "lt2015_2"))]
        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-r", "origin", "-m", "5", "-k", "-t"]
        );
    }

    #[test]
    #[cfg(not(feature = "lt2016_1"))]
    fn shelved_carries_over_options() {
        let fetch = Fetch::new("p4", GlobalOpts::new())
            .remote_spec("origin")
            .verbose(true)
            .shelved("42");

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["fetch", "-r", "origin", "-v", "-s", "42"]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let fetch = Fetch::new(
            "p4",
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        );

        assert_eq!(
            args_of(&fetch.setup_command("p4")),
            ["-p", "localhost:1666", "-q", "fetch"]
        );
    }
}
