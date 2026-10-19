use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// Clone target: `-r remotespec` vs `-f filespec`
// ---------------------------------------------------------------------------

/// The `-r remotespec` target state: use the remote spec named `remotespec`,
/// installed on the remote server, as a template for the clone and stream
/// setup.
///
/// Entered with [`Clone::remote`]; mutually exclusive with
/// [`FileSpecMode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSpecMode {
    source: String,
}

/// The `-f filespec` target state: use the *filespec* in the remote server as
/// the path to clone.
///
/// Entered with [`Clone::file`]; mutually exclusive with
/// [`RemoteSpecMode`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSpecMode {
    source: String,
}

impl ExclusiveOption for RemoteSpecMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-r").arg(&self.source);
    }
}

impl ExclusiveOption for FileSpecMode {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-f").arg(&self.source);
    }
}

/// Marker for a selected clone target: either [`RemoteSpecMode`] (`-r`) or
/// [`FileSpecMode`] (`-f`).
///
/// Exactly one of `-r` / `-f` is required, so the command can only be
/// assembled and spawned once [`struct@Clone`] has transitioned into one of these
/// states.
pub trait CloneTarget: ExclusiveOption {}

impl CloneTarget for RemoteSpecMode {}
impl CloneTarget for FileSpecMode {}

// ---------------------------------------------------------------------------
// Remote port: `-p port`
// ---------------------------------------------------------------------------

/// The `-p port` state: the address (`P4PORT`) of the remote server to clone
/// from, modelled as an ordinary option because `p4 clone` does not accept
/// the global options.
///
/// Entered with [`Clone::port`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    port: String,
}

// ---------------------------------------------------------------------------
// Command struct
// ---------------------------------------------------------------------------

/// Type-safe builder for `p4 clone`.
///
#[cfg_attr(
    feature = "lt2017_1",
    doc = "Clone a new local Perforce server from a remote server."
)]
#[cfg_attr(
    all(feature = "lt2017_2", not(feature = "lt2017_1")),
    doc = "Clone a new local Helix Core server from a remote server."
)]
#[cfg_attr(
    all(feature = "lt2019_1", not(feature = "lt2017_2")),
    doc = "Clone a new local Helix Server from a remote server."
)]
#[cfg_attr(
    all(feature = "lt2021_1", not(feature = "lt2019_1")),
    doc = "Clone a new local Helix server from a remote server."
)]
#[cfg_attr(
    all(feature = "lt2024_1", not(feature = "lt2021_1")),
    doc = "Clone a new local Helix Server from a remote server."
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2024_1")),
    doc = "Clone a new local Helix Core Server from a remote server."
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "Clone a new local P4 Server from a remote server."
)]
///
/// When you clone from a remote server, you copy the portion of its contents
/// that you want to work with into your local server.
///
/// Unlike most commands, `p4 clone` does not accept the full set of global
/// options: only `-u user`, `-d dir`, and `-c client` are recognized before
/// the subcommand, so they are modelled as ordinary options here rather than
/// through [`GlobalOpts`](crate::global::GlobalOpts).
///
/// # Syntax
///
/// ```text
/// p4 [-u user] [-d dir] [-c client] clone [-m depth] [-v] -p port -r remote
/// p4 [-u user] [-d dir] [-c client] clone [-m depth] [-v] -p port -f filespec
/// ```
///
/// The `P` type parameter tracks the remote port (`-p`) at compile time and
/// the `M` type parameter tracks the clone target (`-r` / `-f`); both start
/// as [`Unselected`], in which the command cannot be assembled. Select a port
/// with [`Self::port`] and exactly one target with [`Self::remote`] or
/// [`Self::file`] before assembling the command.
#[derive(Debug, Clone)]
pub struct Clone<P = Unselected, M = Unselected> {
    bin: PathBuf,

    /// `-u user`
    user: Option<String>,

    /// `-d dir`
    dir: Option<PathBuf>,

    /// `-c client`
    client: Option<String>,

    /// `-m depth`
    depth: Option<u64>,

    /// `-v`
    verbose: bool,

    port: P,

    mode: M,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl Clone<Unselected, Unselected> {
    /// Creates a new `p4 clone` command with no remote port and no clone
    /// target selected.
    ///
    /// `bin` is the path to the Perforce command-line executable. The command
    /// cannot be assembled in this state; select a remote port with
    /// [`Self::port`] (`-p`) and exactly one target with
    /// [`Self::remote`] (`-r`) or [`Self::file`] (`-f`) first.
    pub fn new(bin: impl Into<PathBuf>) -> Self {
        Self {
            bin: bin.into(),
            user: None,
            dir: None,
            client: None,
            depth: None,
            verbose: false,
            port: Unselected,
            mode: Unselected,
        }
    }
}

// ---------------------------------------------------------------------------
// Common options: available in every state
// ---------------------------------------------------------------------------

impl<P, M> Clone<P, M> {
    /// Returns the path to the Perforce command-line executable.
    pub fn get_bin(&self) -> &Path {
        self.bin.as_path()
    }

    /// Sets the path to the Perforce command-line executable.
    pub fn set_bin(&mut self, bin: impl Into<PathBuf>) -> &mut Self {
        self.bin = bin.into();
        self
    }

    /// Sets the path to the Perforce command-line executable.
    pub fn bin(mut self, bin: impl Into<PathBuf>) -> Self {
        self.bin = bin.into();
        self
    }

    /// Returns the value of `-u user`, if set.
    ///
    /// This is one of the few options that `p4 clone` recognizes *before* the
    /// subcommand, in place of the full set of global options.
    pub fn get_user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// Sets the value of `-u user`.
    pub fn set_user(&mut self, v: impl Into<String>) -> &mut Self {
        self.user = Some(v.into());
        self
    }

    /// Sets `-u user`.
    ///
    /// # Description
    ///
    /// -u user
    ///
    #[cfg_attr(feature = "lt2017_2", doc = "Specifies the Perforce user.")]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2017_2")),
        doc = "Specifies the Helix Server user."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Specifies the Helix server user."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Specifies the Helix Server user."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Specifies the Helix Core Server user."
    )]
    #[cfg_attr(not(feature = "lt2024_2"), doc = "Specifies the P4 Server user.")]
    pub fn user(mut self, v: impl Into<String>) -> Self {
        self.user = Some(v.into());
        self
    }

    /// Returns the value of `-d dir`, if set.
    ///
    /// This is one of the few options that `p4 clone` recognizes *before* the
    /// subcommand, in place of the full set of global options.
    pub fn get_dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// Sets the value of `-d dir`.
    pub fn set_dir(&mut self, v: impl Into<PathBuf>) -> &mut Self {
        self.dir = Some(v.into());
        self
    }

    /// Sets `-d dir`.
    ///
    /// # Description
    ///
    /// -d dir
    ///
    #[cfg_attr(
        feature = "lt2017_1",
        doc = "Specifies the directory in which the new Perforce server is initialized. If not specified, defaults to the current directory."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2017_1")),
        doc = "Specifies the directory in which the new Helix Core server is initialized. If not specified, defaults to the current directory."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2017_2")),
        doc = "Specifies the directory in which the new Helix Server is initialized. If not specified, defaults to the current directory."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Specifies the directory in which the new Helix server is initialized. If not specified, defaults to the current directory."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Specifies the directory in which the new Helix Server is initialized. If not specified, defaults to the current directory."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Specifies the directory in which the new Helix Core Server is initialized. If not specified, defaults to the current directory."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Specifies the directory in which the new P4 Server is initialized. If not specified, defaults to the current directory."
    )]
    pub fn dir(mut self, v: impl Into<PathBuf>) -> Self {
        self.dir = Some(v.into());
        self
    }

    /// Returns the value of `-c client`, if set.
    ///
    /// This is one of the few options that `p4 clone` recognizes *before* the
    /// subcommand, in place of the full set of global options.
    pub fn get_client(&self) -> Option<&str> {
        self.client.as_deref()
    }

    /// Sets the value of `-c client`.
    pub fn set_client(&mut self, v: impl Into<String>) -> &mut Self {
        self.client = Some(v.into());
        self
    }

    /// Sets `-c client`.
    ///
    /// # Description
    ///
    /// -c client
    ///
    /// Specifies the client name. If not specified, defaults to the name
    /// established with the `p4 init` command.
    pub fn client(mut self, v: impl Into<String>) -> Self {
        self.client = Some(v.into());
        self
    }

    /// Returns the value of `-m depth`, if set.
    pub fn get_depth(&self) -> Option<u64> {
        self.depth
    }

    /// Sets the value of `-m depth`.
    pub fn set_depth(&mut self, v: u64) -> &mut Self {
        self.depth = Some(v);
        self
    }

    /// Sets `-m depth`.
    ///
    /// # Description
    ///
    /// -m depth
    ///
    /// Specifies the maximum number of revisions of each file to clone; a
    /// *shallow* clone.
    pub fn depth(mut self, v: u64) -> Self {
        self.depth = Some(v);
        self
    }

    /// Returns whether `-v` verbose mode is enabled.
    pub fn get_verbose(&self) -> bool {
        self.verbose
    }

    /// Sets whether `-v` verbose mode is enabled.
    pub fn set_verbose(&mut self, v: bool) -> &mut Self {
        self.verbose = v;
        self
    }

    /// Enables `-v` verbose mode when `v` is `true`.
    ///
    /// # Description
    ///
    /// -v
    ///
    /// Enables verbose mode.
    pub fn verbose(mut self, v: bool) -> Self {
        self.verbose = v;
        self
    }
}

// ---------------------------------------------------------------------------
// Port selection: `-p port`
// ---------------------------------------------------------------------------

impl<M> Clone<Unselected, M> {
    /// Sets `-p port` and transitions this command to the [`Port`] state.
    ///
    /// # Description
    ///
    /// -p port
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Specifies the address (`P4PORT`) of the remote server you wish to clone from."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "Specifies the address (`P4PORT`) of the remote server you wish to clone from.",
        doc = "",
        doc = "This flag is optional. If not specified, `p4 clone` uses the remote server specified by the `P4PORT` environment variable."
    )]
    ///
    /// A remote port is required before the command can be assembled.
    pub fn port(self, v: impl Into<String>) -> Clone<Port, M> {
        Clone {
            bin: self.bin,
            user: self.user,
            dir: self.dir,
            client: self.client,
            depth: self.depth,
            verbose: self.verbose,
            port: Port { port: v.into() },
            mode: self.mode,
        }
    }
}

impl<M> Clone<Port, M> {
    /// Returns the value of `-p port`.
    pub fn get_port(&self) -> &str {
        &self.port.port
    }

    /// Sets the value of `-p port`.
    pub fn set_port(&mut self, v: impl Into<String>) -> &mut Self {
        self.port.port = v.into();
        self
    }

    /// Sets `-p port`.
    ///
    /// # Description
    ///
    /// -p port
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Specifies the address (`P4PORT`) of the remote server you wish to clone from."
    )]
    #[cfg_attr(
        not(feature = "lt2016_1"),
        doc = "Specifies the address (`P4PORT`) of the remote server you wish to clone from.",
        doc = "",
        doc = "This flag is optional. If not specified, `p4 clone` uses the remote server specified by the `P4PORT` environment variable."
    )]
    pub fn port(mut self, v: impl Into<String>) -> Self {
        self.port.port = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// Clone target selection: `-r remotespec` vs `-f filespec`
// ---------------------------------------------------------------------------

impl<P> Clone<P, Unselected> {
    /// Sets `-r remotespec` and transitions this command to the
    /// [`RemoteSpecMode`] state.
    ///
    /// # Description
    ///
    /// -r remotespec
    ///
    /// Specifies the remote spec call `remotespec` installed on the remote
    /// server to use as a template for the clone and stream setup.
    ///
    /// This is mutually exclusive with `-f filespec`
    /// ([`Self::file`]).
    pub fn remote(self, v: impl Into<String>) -> Clone<P, RemoteSpecMode> {
        Clone {
            bin: self.bin,
            user: self.user,
            dir: self.dir,
            client: self.client,
            depth: self.depth,
            verbose: self.verbose,
            port: self.port,
            mode: RemoteSpecMode { source: v.into() },
        }
    }

    /// Sets `-f filespec` and transitions this command to the
    /// [`FileSpecMode`] state.
    ///
    /// # Description
    ///
    /// -f filespec
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Specifies a `filepath` in the remote server to use as the path to clone. Perforce will use this path to determine the stream setup in the local server. Specifying the filepath also creates a default remote spec called `origin`."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2016_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. Perforce uses this path to determine the stream setup in the local server. The Helix Versioning Engine also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2017_2")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Versioning Engine uses this path to determine the stream setup in the local server. Helix Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core Server uses this path to determine the stream setup in the local server. Helix Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core server uses this path to determine the stream setup in the local server. Helix server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core Server uses this path to determine the stream setup in the local server. Helix Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core Server uses this path to determine the stream setup in the local server. Helix Core Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The P4 Server uses this path to determine the stream setup in the local server. P4 Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    ///
    /// This is mutually exclusive with `-r remotespec`
    /// ([`Self::remote`]).
    pub fn file(self, v: impl Into<String>) -> Clone<P, FileSpecMode> {
        Clone {
            bin: self.bin,
            user: self.user,
            dir: self.dir,
            client: self.client,
            depth: self.depth,
            verbose: self.verbose,
            port: self.port,
            mode: FileSpecMode { source: v.into() },
        }
    }
}

impl<P> Clone<P, RemoteSpecMode> {
    /// Returns the value of `-r remotespec`.
    pub fn get_source(&self) -> &str {
        &self.mode.source
    }

    /// Sets the value of `-r remotespec`.
    pub fn set_source(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.source = v.into();
        self
    }

    /// Sets `-r remotespec`.
    ///
    /// # Description
    ///
    /// -r remotespec
    ///
    /// Specifies the remote spec call `remotespec` installed on the remote
    /// server to use as a template for the clone and stream setup.
    pub fn source(mut self, v: impl Into<String>) -> Self {
        self.mode.source = v.into();
        self
    }
}

impl<P> Clone<P, FileSpecMode> {
    /// Returns the value of `-f filespec`.
    pub fn get_source(&self) -> &str {
        &self.mode.source
    }

    /// Sets the value of `-f filespec`.
    pub fn set_source(&mut self, v: impl Into<String>) -> &mut Self {
        self.mode.source = v.into();
        self
    }

    /// Sets `-f filespec`.
    ///
    /// # Description
    ///
    /// -f filespec
    ///
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "Specifies a `filepath` in the remote server to use as the path to clone. Perforce will use this path to determine the stream setup in the local server. Specifying the filepath also creates a default remote spec called `origin`."
    )]
    #[cfg_attr(
        all(feature = "lt2017_2", not(feature = "lt2016_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. Perforce uses this path to determine the stream setup in the local server. The Helix Versioning Engine also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2017_2")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Versioning Engine uses this path to determine the stream setup in the local server. Helix Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core Server uses this path to determine the stream setup in the local server. Helix Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core server uses this path to determine the stream setup in the local server. Helix server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core Server uses this path to determine the stream setup in the local server. Helix Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The Helix Core Server uses this path to determine the stream setup in the local server. Helix Core Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Specifies a `filespec` in the remote server to use as the path to clone. The P4 Server uses this path to determine the stream setup in the local server. P4 Server also uses this file specification to determine the stream setup in the personal server. Specifying the filespec also creates a default remote spec called `origin`.",
        doc = "",
        doc = "The `-f` preceding the filespec is optional. You can simply issue the command `p4 clone //file/path`."
    )]
    pub fn source(mut self, v: impl Into<String>) -> Self {
        self.mode.source = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// SubCommand: only after a port (`-p`) and a target (`-r` / `-f`) are selected
// ---------------------------------------------------------------------------

impl<M: CloneTarget> SubCommand for Clone<Port, M> {
    fn name(&self) -> &str {
        "clone"
    }

    /// Injects the options that precede the subcommand name (`-u`, `-d`,
    /// `-c`), then the subcommand name, then the remaining options.
    ///
    /// `p4 clone` does not accept the global options, so no
    /// [`GlobalOpts`](crate::global::GlobalOpts) are injected.
    fn inject_args(&self, command: &mut Command) {
        if let Some(user) = &self.user {
            command.arg("-u").arg(user);
        }

        if let Some(dir) = &self.dir {
            command.arg("-d").arg(dir);
        }

        if let Some(client) = &self.client {
            command.arg("-c").arg(client);
        }

        self.inject_local_args(command.arg(self.name()));
    }

    fn inject_local_args(&self, command: &mut Command) {
        if let Some(depth) = self.depth {
            command.arg("-m").arg(depth.to_string());
        }

        if self.verbose {
            command.arg("-v");
        }

        command.arg("-p").arg(&self.port.port);

        self.mode.inject_args(command);
    }
}

// ---------------------------------------------------------------------------
// Spawning: only after a port (`-p`) and a target (`-r` / `-f`) are selected
// ---------------------------------------------------------------------------

impl<M: CloneTarget> ParameterizedSpawn<()> for Clone<Port, M> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 clone` as a child process with piped standard output and
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
        let mut command = Clone::new("p4").port("perforce:1666").remote("origin");
        assert_eq!(command.get_bin(), Path::new("p4"));

        command.set_bin("/usr/local/bin/p4");
        assert_eq!(command.get_bin(), Path::new("/usr/local/bin/p4"));

        let command = command.bin("p4.exe");
        assert_eq!(command.get_bin(), Path::new("p4.exe"));
    }

    #[test]
    fn getters_reflect_selected_values() {
        let mut command = Clone::new("p4").port("perforce:1666").remote("origin");

        assert_eq!(command.get_user(), None);
        assert_eq!(command.get_dir(), None);
        assert_eq!(command.get_client(), None);
        assert_eq!(command.get_depth(), None);
        assert!(!command.get_verbose());
        assert_eq!(command.get_port(), "perforce:1666");
        assert_eq!(command.get_source(), "origin");

        command.set_user("bruno");
        command.set_dir("Ace");
        command.set_client("bruno-ws");
        command.set_depth(5);
        command.set_verbose(true);
        command.set_port("perforce:1667");
        command.set_source("upstream");

        assert_eq!(command.get_user(), Some("bruno"));
        assert_eq!(command.get_dir(), Some(Path::new("Ace")));
        assert_eq!(command.get_client(), Some("bruno-ws"));
        assert_eq!(command.get_depth(), Some(5));
        assert!(command.get_verbose());
        assert_eq!(command.get_port(), "perforce:1667");
        assert_eq!(command.get_source(), "upstream");
    }

    /// Dry-run checks of the assembled `p4 clone` command line; no process is
    /// spawned.
    #[test]
    fn port_and_remote_only() {
        let clone = Clone::new("p4").port("perforce:1666").remote("origin");

        assert_eq!(
            args_of(&clone.setup_command("p4")),
            ["clone", "-p", "perforce:1666", "-r", "origin"]
        );
    }

    #[test]
    fn port_and_file_only() {
        let clone = Clone::new("p4")
            .port("perforce:1666")
            .file("//depot/main/...");

        assert_eq!(
            args_of(&clone.setup_command("p4")),
            ["clone", "-p", "perforce:1666", "-f", "//depot/main/..."]
        );
        assert_eq!(clone.get_source(), "//depot/main/...");
    }

    /// The two required selections are independent of each other: the target
    /// may be chosen before the port.
    #[test]
    fn selection_order_is_independent() {
        let check = |clone: Clone<Port, RemoteSpecMode>| {
            assert_eq!(
                args_of(&clone.setup_command("p4")),
                ["clone", "-p", "perforce:1666", "-r", "origin"]
            );
        };

        check(Clone::new("p4").port("perforce:1666").remote("origin"));
        check(Clone::new("p4").remote("origin").port("perforce:1666"));
    }

    #[test]
    fn all_options_in_syntax_order() {
        let clone = Clone::new("p4")
            .user("bruno")
            .dir("Ace")
            .client("bruno-ws")
            .depth(5)
            .verbose(true)
            .port("perforce:1666")
            .file("//depot/main/...");

        assert_eq!(
            args_of(&clone.setup_command("p4")),
            [
                "-u",
                "bruno",
                "-d",
                "Ace",
                "-c",
                "bruno-ws",
                "clone",
                "-m",
                "5",
                "-v",
                "-p",
                "perforce:1666",
                "-f",
                "//depot/main/..."
            ]
        );
    }

    #[test]
    fn verbose_flag_is_omitted_when_disabled() {
        let clone = Clone::new("p4")
            .port("perforce:1666")
            .remote("origin")
            .verbose(false);

        assert_eq!(
            args_of(&clone.setup_command("p4")),
            ["clone", "-p", "perforce:1666", "-r", "origin"]
        );
    }

    /// `-p`, `-m`, and `-v` are common options: they may be set before the
    /// port and target are selected.
    #[test]
    fn common_options_before_required_selections() {
        let clone = Clone::new("p4")
            .depth(1)
            .verbose(true)
            .port("perforce:1666")
            .remote("origin");

        assert_eq!(
            args_of(&clone.setup_command("p4")),
            [
                "clone",
                "-m",
                "1",
                "-v",
                "-p",
                "perforce:1666",
                "-r",
                "origin"
            ]
        );
    }
}
