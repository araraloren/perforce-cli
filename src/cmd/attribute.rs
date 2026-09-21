use std::ffi::OsStr;
use std::io::Write;
#[cfg(not(feature = "lt2024_2"))]
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};

use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

/// Default value source of `p4 attribute`: set or clear attributes with
/// `-n name [-v value]` pairs.
///
/// Each entry is an attribute name together with the value to set, or `None`
/// to clear the attribute by omitting `-v`.
#[derive(Debug, Clone, Default)]
pub struct Standard {
    values: Option<Vec<(String, Option<String>)>>,

    hex: bool,
}

impl ExclusiveOption for Standard {
    fn inject_args(&self, command: &mut Command) {
        if self.hex {
            command.arg("-e");
        }

        if let Some(pairs) = &self.values {
            for (name, value) in pairs {
                command.arg("-n").arg(name);

                if let Some(value) = value {
                    command.arg("-v").arg(value);
                }
            }
        }
    }
}

/// Read the attribute value from standard input (`-i`).
///
/// Entered with [`Attribute::read_from_stdin`]; only one file argument is
/// allowed in this state.
#[derive(Debug, Clone, Default)]
pub struct FromStdin {
    hex: bool,

    name: String,
}

impl ExclusiveOption for FromStdin {
    fn inject_args(&self, command: &mut Command) {
        if self.hex {
            command.arg("-e");
        }

        command.arg("-i").arg("-n").arg(&self.name);
    }
}

/// Read the attribute value from a file (`-I filename`).
///
/// Entered with [`Attribute::read_from_file`]; `-e` is not available and only
/// one file argument is allowed in this state.
#[cfg(not(feature = "lt2024_2"))]
#[derive(Debug, Clone, Default)]
pub struct FromFile {
    name: String,

    file: PathBuf,
}

#[cfg(not(feature = "lt2024_2"))]
impl ExclusiveOption for FromFile {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-I").arg(&self.file).arg("-n").arg(&self.name);
    }
}

/// Trait storage location of the `-T0` / `-T1` option group of
/// `p4 attribute`.
#[cfg(not(feature = "lt2023_2"))]
pub mod storage {
    /// `-T0`: store the attribute value in the `db.traits` table.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DatabaseTraits;

    /// `-T1`: store the attribute value in the `trait` depot.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct TraitDepot;
}

#[cfg(not(feature = "lt2023_2"))]
impl ExclusiveOption for storage::DatabaseTraits {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-T0");
    }
}

#[cfg(not(feature = "lt2023_2"))]
impl ExclusiveOption for storage::TraitDepot {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-T1");
    }
}

#[cfg_attr(
    feature = "lt2023_2",
    doc = "`p4 [g-opts] attribute [-e -f -p] -n name [-v value] files ...`: set per-revision attributes on revisions.\n\n`p4 [g-opts] attribute [-e -f -p] -i -n name file`"
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2023_2")),
    doc = "`p4 [g-opts] attribute [-e -f -p] -n name [-v value [-T0 | -T1]] files ...`: set per-revision attributes on revisions.\n\n`p4 [g-opts] attribute [-e -f -p [-T0 | -T1]] -i -n name file`"
)]
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "`p4 [g-opts] attribute [-e -f -p] -n name [-v value [-T0 | -T1]] files ...`: set per-revision attributes on revisions.\n\n`p4 [g-opts] attribute [-e -f -p [-T0 | -T1]] -i -n name file`\n\n`p4 attribute [-f -p [-T0 | -T1]] -I filename -n name file`"
)]
///
/// The `S` type parameter tracks the value source: [`Standard`] for the
/// default `-n`/`-v` set-or-clear form, [`FromStdin`] for the `-i` form,
#[cfg_attr(
    not(feature = "lt2024_2"),
    doc = "[`FromFile`] for the `-I filename` form, and"
)]
/// and the `T` type parameter tracks the `-T0`/`-T1` trait storage location.
#[derive(Debug, Clone, Default)]
pub struct Attribute<S = Standard, T = Unselected> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    source: S,

    storage: T,

    on_submitted_files: bool,

    propagating: bool,
}

impl<S: ExclusiveOption, T: ExclusiveOption> SubCommand for Attribute<S, T> {
    fn name(&self) -> &str {
        "attribute"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.on_submitted_files {
            command.arg("-f");
        }

        if self.propagating {
            command.arg("-p");
        }

        self.storage.inject_args(command);

        self.source.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl Attribute<Standard, Unselected> {
    /// Creates a new `p4 attribute` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            on_submitted_files: false,
            propagating: false,
            source: Standard::default(),
            storage: Unselected,
        }
    }
}

impl<T: ExclusiveOption> ParameterizedSpawn for Attribute<Standard, T> {
    type Input<'a> = &'a [&'a OsStr];
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 attribute` for the given files as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with<'a>(&mut self, files: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<T: ExclusiveOption> Attribute<Standard, T> {
    /// # Description
    ///
    /// -n name -v value
    ///
    /// Sets the attribute `name` to `value` on the given files.
    #[cfg_attr(not(feature = "lt2024_2"), doc = "The supplied value must be text.")]
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.source
            .values
            .get_or_insert_with(Vec::new)
            .push((name.into(), Some(value.into())));
        self
    }

    /// # Description
    ///
    /// -n name
    ///
    /// Clears the attribute `name` on the given files by omitting the `-v`
    /// option.
    pub fn clear(&mut self, name: impl Into<String>) -> &mut Self {
        self.source
            .values
            .get_or_insert_with(Vec::new)
            .push((name.into(), None));
        self
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn get_hex(&self) -> bool {
        self.source.hex
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn set_hex(&mut self, v: bool) -> &mut Self {
        self.source.hex = v;
        self
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn hex(mut self, v: bool) -> Self {
        self.source.hex = v;
        self
    }

    /// # Description
    ///
    /// -i
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "Read an attribute value from the standard input. Only one file argument is allowed when using this option."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "Read an attribute value from the standard input. Only one file argument is allowed when using this option. This option supports both textual and binary content."
    )]
    ///
    /// Transitions this command to the [`FromStdin`] state; any `-n`/`-v`
    /// pairs set with [`Self::set`] or [`Self::clear`] are discarded, while
    /// `-e` is preserved.
    pub fn read_from_stdin(self, name: String) -> Attribute<FromStdin, T> {
        Attribute {
            bin: self.bin,
            global_opts: self.global_opts,
            on_submitted_files: self.on_submitted_files,
            propagating: self.propagating,
            source: FromStdin {
                hex: self.source.hex,
                name,
            },
            storage: self.storage,
        }
    }

    /// # Description
    ///
    /// -I filename
    ///
    /// Read the attribute value from a file. The file can have textual or
    /// binary content. Use this when the attribute data exceeds 250
    /// megabytes, which might cause the command to fail with the `Rpc buffer
    /// too big` error. The following are not allowed with this option:
    ///
    /// - Using the `-e` option to specify the value as hex.
    /// - More than one file argument.
    /// - Setting more than one trait value.
    ///
    /// To display attributes set with this option, use the `p4 print -T`
    /// command instead of the `p4 fstat -Oa` command because `p4 print -T`
    /// can handle larger non-encoded binary data.
    ///
    /// Transitions this command to the [`FromFile`] state; any `-n`/`-v`
    /// pairs set with [`Self::set`] or [`Self::clear`], as well as `-e`, are
    /// discarded.
    #[cfg(not(feature = "lt2024_2"))]
    pub fn read_from_file(self, name: String, file: PathBuf) -> Attribute<FromFile, T> {
        Attribute {
            bin: self.bin,
            global_opts: self.global_opts,
            on_submitted_files: self.on_submitted_files,
            propagating: self.propagating,
            source: FromFile { name, file },
            storage: self.storage,
        }
    }
}

impl<T: ExclusiveOption> Attribute<FromStdin, T> {
    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn get_hex(&self) -> bool {
        self.source.hex
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn set_hex(&mut self, v: bool) -> &mut Self {
        self.source.hex = v;
        self
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn hex(mut self, v: bool) -> Self {
        self.source.hex = v;
        self
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute read from standard input.
    pub fn get_name(&self) -> &str {
        &self.source.name
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute read from standard input.
    pub fn set_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.source.name = v.into();
        self
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute read from standard input.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.source.name = v.into();
        self
    }

    /// Runs `p4 attribute -i` for a single file, writing `value` to the
    /// child's standard input, and captures the command's output.
    ///
    /// Unlike [`ParameterizedSpawn::spawn_with`], this method handles writing
    /// the attribute value to the child's standard input itself, so it is the
    /// most convenient way to submit values through standard input.
    ///
    /// Only one file argument is allowed in the [`FromStdin`] state.
    pub fn output<S, V>(&self, file: S, value: V) -> Result<Output, std::io::Error>
    where
        S: AsRef<OsStr>,
        V: AsRef<[u8]>,
    {
        let mut child = self
            .setup_command(&self.bin)
            .arg(file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(value.as_ref())?;
        }
        drop(child.stdin.take());

        child.wait_with_output()
    }
}

impl<T: ExclusiveOption> ParameterizedSpawn for Attribute<FromStdin, T> {
    type Input<'a> = (&'a OsStr, Stdio);
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 attribute -i` for a single file as a child process with the
    /// given standard input configuration and piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    ///
    /// Pass `Stdio::piped()` to obtain a writable `child.stdin` handle and
    /// write the attribute value yourself, then drop it before waiting on
    /// the child. Only one file argument is allowed in the [`FromStdin`]
    /// state.
    fn spawn_with<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .arg(input.0)
            .stdin(input.1)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(not(feature = "lt2024_2"))]
impl<T: ExclusiveOption> Attribute<FromFile, T> {
    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute read from the file.
    pub fn get_name(&self) -> &str {
        &self.source.name
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute read from the file.
    pub fn set_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.source.name = v.into();
        self
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute read from the file.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.source.name = v.into();
        self
    }

    /// # Description
    ///
    /// -I filename
    ///
    /// The file the attribute value is read from.
    pub fn get_file(&self) -> &Path {
        &self.source.file
    }
}

#[cfg(not(feature = "lt2024_2"))]
impl<T: ExclusiveOption> ParameterizedSpawn for Attribute<FromFile, T> {
    type Input<'a> = &'a OsStr;
    type Output<'a> = Child;
    type Error = std::io::Error;

    /// Spawns `p4 attribute -I` for a single file as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    ///
    /// Only one file argument is allowed in the [`FromFile`] state.
    fn spawn_with<'a>(&mut self, file: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        self.setup_command(&self.bin)
            .arg(file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

impl<S: ExclusiveOption, T: ExclusiveOption> Attribute<S, T> {
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
    /// -f
    ///
    /// Set the attribute on submitted files. If a propagating trait is set
    /// on a submitted file, a revision specifier cannot be used, and the
    /// file must not be currently open in any workspace.
    pub fn get_on_submitted_files(&self) -> bool {
        self.on_submitted_files
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set the attribute on submitted files. If a propagating trait is set
    /// on a submitted file, a revision specifier cannot be used, and the
    /// file must not be currently open in any workspace.
    pub fn set_on_submitted_files(&mut self, v: bool) -> &mut Self {
        self.on_submitted_files = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set the attribute on submitted files. If a propagating trait is set
    /// on a submitted file, a revision specifier cannot be used, and the
    /// file must not be currently open in any workspace.
    pub fn on_submitted_files(mut self, v: bool) -> Self {
        self.on_submitted_files = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    #[cfg_attr(
        feature = "lt2021_2",
        doc = "Create a propagating attribute: an attribute whose value is propagated to subsequent revisions whenever the file is opened with `p4 add`, `p4 edit`, or `p4 delete`."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Create a *propagating attribute*: an attribute whose value is propagated to subsequent revisions whenever the file is opened. Relevant commands include `p4 copy`, `p4 delete`, `p4 edit`, `p4 integrate`, `p4 reconcile`, `p4 resolve`, `p4 shelve`, `p4 submit`, and `p4 unshelve`."
    )]
    pub fn get_propagating(&self) -> bool {
        self.propagating
    }

    /// # Description
    ///
    /// -p
    ///
    #[cfg_attr(
        feature = "lt2021_2",
        doc = "Create a propagating attribute: an attribute whose value is propagated to subsequent revisions whenever the file is opened with `p4 add`, `p4 edit`, or `p4 delete`."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Create a *propagating attribute*: an attribute whose value is propagated to subsequent revisions whenever the file is opened. Relevant commands include `p4 copy`, `p4 delete`, `p4 edit`, `p4 integrate`, `p4 reconcile`, `p4 resolve`, `p4 shelve`, `p4 submit`, and `p4 unshelve`."
    )]
    pub fn set_propagating(&mut self, v: bool) -> &mut Self {
        self.propagating = v;
        self
    }

    /// # Description
    ///
    /// -p
    ///
    #[cfg_attr(
        feature = "lt2021_2",
        doc = "Create a propagating attribute: an attribute whose value is propagated to subsequent revisions whenever the file is opened with `p4 add`, `p4 edit`, or `p4 delete`."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Create a *propagating attribute*: an attribute whose value is propagated to subsequent revisions whenever the file is opened. Relevant commands include `p4 copy`, `p4 delete`, `p4 edit`, `p4 integrate`, `p4 reconcile`, `p4 resolve`, `p4 shelve`, `p4 submit`, and `p4 unshelve`."
    )]
    pub fn propagating(mut self, v: bool) -> Self {
        self.propagating = v;
        self
    }
}

#[cfg(not(feature = "lt2023_2"))]
impl<S: ExclusiveOption, T: ExclusiveOption> Attribute<S, T> {
    /// # Description
    ///
    /// -T0
    ///
    /// Causes the value to be stored in the `db.traits` table, which is the
    /// implicit default.
    pub fn store_in_database_traits(self) -> Attribute<S, storage::DatabaseTraits> {
        Attribute {
            bin: self.bin,
            global_opts: self.global_opts,
            on_submitted_files: self.on_submitted_files,
            propagating: self.propagating,
            source: self.source,
            storage: storage::DatabaseTraits,
        }
    }

    /// # Description
    ///
    /// -T1
    ///
    /// Causes the value to be stored in the `trait` depot instead of the
    /// `db.traits` table, even if the size of the attribute value is less
    /// than the size specified by the `trait.storagedepot.min` configurable.
    /// However, if `trait.storagedepot.min` is unset or set to `0`, the
    /// attribute value is stored in the `db.traits` table.
    pub fn store_in_trait_depot(self) -> Attribute<S, storage::TraitDepot> {
        Attribute {
            bin: self.bin,
            global_opts: self.global_opts,
            on_submitted_files: self.on_submitted_files,
            propagating: self.propagating,
            source: self.source,
            storage: storage::TraitDepot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    /// Dry-run checks of the assembled `p4 attribute` command line; no
    /// process is spawned.
    #[test]
    fn without_options() {
        let attr = Attribute::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&attr.setup_command("p4")), ["attribute"]);
    }

    #[test]
    fn set_attribute_with_value() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set("status", "approved");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-n", "status", "-v", "approved"]
        );
    }

    #[test]
    fn clear_attribute_omits_value() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.clear("status");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-n", "status"]
        );
    }

    #[test]
    fn multiple_set_and_clear_pairs_keep_order() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set("color", "red")
            .clear("status")
            .set("owner", "alice");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            [
                "attribute",
                "-n",
                "color",
                "-v",
                "red",
                "-n",
                "status",
                "-n",
                "owner",
                "-v",
                "alice"
            ]
        );
    }

    #[test]
    fn hex_submitted_propagating() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set_hex(true)
            .set_on_submitted_files(true)
            .set_propagating(true)
            .set("thumb", "deadbeef");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            [
                "attribute",
                "-f",
                "-p",
                "-e",
                "-n",
                "thumb",
                "-v",
                "deadbeef"
            ]
        );
    }

    #[test]
    fn read_from_stdin_state() {
        let attr = Attribute::new("p4", GlobalOpts::new()).read_from_stdin("thumb".to_string());

        assert_eq!(attr.get_name(), "thumb");
        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-i", "-n", "thumb"]
        );
    }

    #[test]
    fn stdin_name_can_be_replaced() {
        let mut attr = Attribute::new("p4", GlobalOpts::new()).read_from_stdin("old".to_string());
        attr.set_name("thumb");

        assert_eq!(attr.get_name(), "thumb");

        let attr = attr.name("icon");
        assert_eq!(attr.get_name(), "icon");
        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-i", "-n", "icon"]
        );
    }

    #[test]
    fn stdin_preserves_flags_and_hex() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set_hex(true)
            .set_on_submitted_files(true)
            .set_propagating(true)
            .set("discarded", "value");

        let attr = attr.read_from_stdin("thumb".to_string());

        assert!(attr.get_hex());
        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-f", "-p", "-e", "-i", "-n", "thumb"]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn store_in_database_traits() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set("thumb", "data");

        let attr = attr.store_in_database_traits();

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-T0", "-n", "thumb", "-v", "data"]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn store_in_trait_depot() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set("thumb", "data");

        let attr = attr.store_in_trait_depot();

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-T1", "-n", "thumb", "-v", "data"]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn stdin_with_trait_depot() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .read_from_stdin("thumb".to_string())
            .store_in_trait_depot();

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-T1", "-i", "-n", "thumb"]
        );
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn read_from_file_state() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .read_from_file("thumb".to_string(), PathBuf::from("/tmp/thumb.bin"));

        assert_eq!(attr.get_name(), "thumb");
        assert_eq!(attr.get_file(), Path::new("/tmp/thumb.bin"));
        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-I", "/tmp/thumb.bin", "-n", "thumb"]
        );
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn all_modern_options() {
        let mut attr = Attribute::new("p4", GlobalOpts::new());
        attr.set_on_submitted_files(true).set_propagating(true);

        let attr = attr
            .store_in_trait_depot()
            .read_from_file("thumb".to_string(), PathBuf::from("/tmp/data.bin"));

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            [
                "attribute",
                "-f",
                "-p",
                "-T1",
                "-I",
                "/tmp/data.bin",
                "-n",
                "thumb"
            ]
        );
    }

    #[test]
    fn set_style_with_global_opts() {
        let mut attr = Attribute::new("p4", GlobalOpts::new().port("localhost:1666"));
        attr.set("status", "approved");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            [
                "-p",
                "localhost:1666",
                "attribute",
                "-n",
                "status",
                "-v",
                "approved"
            ]
        );
    }
}
