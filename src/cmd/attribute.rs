use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};

use crate::cmd::SubCommand;
use crate::global::GlobalOpts;

/// Internal representation of the `-T0` / `-T1` trait storage location.
#[cfg(not(feature = "lt2023_2"))]
#[derive(Debug, Clone, Copy)]
enum TraitStorage {
    /// `-T0`: store the attribute value in the `db.traits` table.
    Table,
    /// `-T1`: store the attribute value in the `trait` depot.
    Depot,
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
#[derive(Debug, Clone, Default)]
pub struct Attribute {
    bin: PathBuf,

    global_opts: GlobalOpts,

    hex: bool,

    submitted_files: bool,

    stdin: bool,

    #[cfg(not(feature = "lt2024_2"))]
    read_from_file: Option<PathBuf>,

    name: Option<String>,

    propagating: bool,

    value: Option<String>,

    #[cfg(not(feature = "lt2023_2"))]
    trait_storage: Option<TraitStorage>,
}

impl SubCommand for Attribute {
    fn name(&self) -> &str {
        "attribute"
    }

    fn inject_local_args(&self, command: &mut Command) {
        if self.hex {
            command.arg("-e");
        }

        if self.submitted_files {
            command.arg("-f");
        }

        if self.propagating {
            command.arg("-p");
        }

        #[cfg(not(feature = "lt2023_2"))]
        match self.trait_storage {
            Some(TraitStorage::Table) => {
                command.arg("-T0");
            }
            Some(TraitStorage::Depot) => {
                command.arg("-T1");
            }
            None => {}
        }

        if self.stdin {
            command.arg("-i");
        }

        #[cfg(not(feature = "lt2024_2"))]
        if let Some(ref file) = self.read_from_file {
            command.arg("-I").arg(file);
        }

        if let Some(ref name) = self.name {
            command.arg("-n").arg(name);
        }

        if let Some(ref value) = self.value {
            command.arg("-v").arg(value);
        }
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl Attribute {
    /// Creates a new `p4 attribute` command.
    ///
    /// `bin` is the path to the Perforce command-line executable.
    pub fn new(bin: impl Into<PathBuf>, global_opts: GlobalOpts) -> Self {
        Self {
            bin: bin.into(),
            global_opts,
            ..Default::default()
        }
    }

    /// Spawns `p4 attribute` for the given files as a child process.
    ///
    /// The child process inherits the standard input, output, and error
    /// streams of the current process, and runs asynchronously; use the
    /// returned [`Child`] handle to wait for it or interact with it.
    ///
    /// This is the general-purpose entry point. For the `-i` option where
    /// the attribute value is read from standard input, use
    /// [`Self::spawn_stdin`] so that the value can be written to the
    /// child's stdin programmatically.
    pub fn spawn<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).args(files).spawn()
    }

    /// Runs `p4 attribute` for the given files to completion and captures
    /// its output.
    ///
    /// Unlike [`Self::spawn`], this method blocks until the command exits and
    /// collects the standard output and error into the returned [`Output`].
    ///
    /// Standard input is set to null, so this method is not suitable for the
    /// `-i` option. Use [`Self::output_with_stdin_value`] instead.
    pub fn output<S: AsRef<OsStr>>(&self, files: &[S]) -> Result<Output, std::io::Error> {
        self.setup_command(&self.bin)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

    /// Spawns `p4 attribute` with the given standard input configuration,
    /// allowing the caller to control how the child's stdin is handled.
    ///
    /// This is intended for use with the `-i` option, which reads the
    /// attribute value from standard input. Only one file argument is
    /// allowed when using `-i`.
    ///
    /// Pass `Stdio::piped()` to obtain a writable `child.stdin` handle and
    /// write the attribute value yourself, then drop it before waiting on
    /// the child.
    pub fn spawn_stdin<S, I>(&self, stdin: I, file: S) -> Result<Child, std::io::Error>
    where
        S: AsRef<OsStr>,
        I: Into<Stdio>,
    {
        self.setup_command(&self.bin).arg(file).stdin(stdin).spawn()
    }

    /// Spawns `p4 attribute` for a single file.
    ///
    /// This is a convenience entry point for command forms that accept only
    /// one file argument, such as:
    ///
    /// `p4 attribute [-f -p [-T0 | -T1]] -I filename -n name file`
    ///
    /// where the attribute value is read from `filename` via the `-I` option
    /// and applied to the single `file`.
    pub fn spawn_file<S: AsRef<OsStr>>(&self, file: S) -> Result<Child, std::io::Error> {
        self.setup_command(&self.bin).arg(file).spawn()
    }

    /// Runs `p4 attribute` for a single file to completion and captures its
    /// output.
    ///
    /// This is the single-file counterpart to [`Self::spawn_file`], intended
    /// for command forms that accept only one file argument, such as:
    ///
    /// `p4 attribute [-f -p [-T0 | -T1]] -I filename -n name file`
    pub fn output_file<S: AsRef<OsStr>>(&self, file: S) -> Result<Output, std::io::Error> {
        self.setup_command(&self.bin)
            .arg(file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }

    /// Runs `p4 attribute` with the given value written to standard input,
    /// capturing the command's output.
    ///
    /// This is a convenience wrapper around the `-i` option: it sets the
    /// `-i` flag, pipes the supplied `value` bytes to the child's standard
    /// input, waits for the command to finish, and returns the captured
    /// output.
    ///
    /// Only one file argument is allowed with the `-i` option.
    pub fn output_with_stdin_value<S, V>(
        mut self,
        file: S,
        value: V,
    ) -> Result<Output, std::io::Error>
    where
        V: AsRef<[u8]>,
        S: AsRef<OsStr>,
    {
        self.stdin = true;
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
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn get_hex(&self) -> bool {
        self.hex
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn set_hex(&mut self, v: bool) -> &mut Self {
        self.hex = v;
        self
    }

    /// # Description
    ///
    /// -e
    ///
    /// Indicates that the value is specified in hex.
    pub fn hex(mut self, v: bool) -> Self {
        self.hex = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set the attribute on submitted files. If a propagating trait is set
    /// on a submitted file, a revision specifier cannot be used, and the
    /// file must not be currently open in any workspace.
    pub fn get_submitted_files(&self) -> bool {
        self.submitted_files
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set the attribute on submitted files. If a propagating trait is set
    /// on a submitted file, a revision specifier cannot be used, and the
    /// file must not be currently open in any workspace.
    pub fn set_submitted_files(&mut self, v: bool) -> &mut Self {
        self.submitted_files = v;
        self
    }

    /// # Description
    ///
    /// -f
    ///
    /// Set the attribute on submitted files. If a propagating trait is set
    /// on a submitted file, a revision specifier cannot be used, and the
    /// file must not be currently open in any workspace.
    pub fn submitted_files(mut self, v: bool) -> Self {
        self.submitted_files = v;
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
    pub fn get_stdin(&self) -> bool {
        self.stdin
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
    pub fn set_stdin(&mut self, v: bool) -> &mut Self {
        self.stdin = v;
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
    pub fn stdin(mut self, v: bool) -> Self {
        self.stdin = v;
        self
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
    #[cfg(not(feature = "lt2024_2"))]
    pub fn get_read_from_file(&self) -> Option<&PathBuf> {
        self.read_from_file.as_ref()
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
    #[cfg(not(feature = "lt2024_2"))]
    pub fn set_read_from_file(&mut self, v: impl Into<PathBuf>) -> &mut Self {
        self.read_from_file = Some(v.into());
        self
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
    #[cfg(not(feature = "lt2024_2"))]
    pub fn read_from_file(mut self, v: impl Into<PathBuf>) -> Self {
        self.read_from_file = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute to set.
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute to set.
    pub fn set_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.name = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -n name
    ///
    /// The name of the attribute to set.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.name = Some(v.into());
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

    /// # Description
    ///
    /// -v value
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "The value of the attribute to set. To clear an attribute, omit the `-v` option."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "The value of the attribute to set. To clear an attribute, omit the `-v` option. The supplied value must be text."
    )]
    pub fn get_value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    /// # Description
    ///
    /// -v value
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "The value of the attribute to set. To clear an attribute, omit the `-v` option."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "The value of the attribute to set. To clear an attribute, omit the `-v` option. The supplied value must be text."
    )]
    pub fn set_value(&mut self, v: impl Into<String>) -> &mut Self {
        self.value = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -v value
    ///
    #[cfg_attr(
        feature = "lt2024_2",
        doc = "The value of the attribute to set. To clear an attribute, omit the `-v` option."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "The value of the attribute to set. To clear an attribute, omit the `-v` option. The supplied value must be text."
    )]
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -T0
    ///
    /// Causes the value to be stored in the `db.traits` table, which is the
    /// implicit default.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn get_trait_storage_table(&self) -> bool {
        matches!(self.trait_storage, Some(TraitStorage::Table))
    }

    /// # Description
    ///
    /// -T0
    ///
    /// Causes the value to be stored in the `db.traits` table, which is the
    /// implicit default.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn set_trait_storage_table(&mut self) -> &mut Self {
        self.trait_storage = Some(TraitStorage::Table);
        self
    }

    /// # Description
    ///
    /// -T0
    ///
    /// Causes the value to be stored in the `db.traits` table, which is the
    /// implicit default.
    #[cfg(not(feature = "lt2023_2"))]
    pub fn trait_storage_table(mut self) -> Self {
        self.trait_storage = Some(TraitStorage::Table);
        self
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
    #[cfg(not(feature = "lt2023_2"))]
    pub fn get_trait_storage_depot(&self) -> bool {
        matches!(self.trait_storage, Some(TraitStorage::Depot))
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
    #[cfg(not(feature = "lt2023_2"))]
    pub fn set_trait_storage_depot(&mut self) -> &mut Self {
        self.trait_storage = Some(TraitStorage::Depot);
        self
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
    #[cfg(not(feature = "lt2023_2"))]
    pub fn trait_storage_depot(mut self) -> Self {
        self.trait_storage = Some(TraitStorage::Depot);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;

    #[test]
    fn without_options() {
        let attr = Attribute::new("p4", GlobalOpts::new());

        assert_eq!(args_of(&attr.setup_command("p4")), ["attribute"]);
    }

    #[test]
    fn set_attribute_with_value() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .name("status")
            .value("approved");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-n", "status", "-v", "approved"]
        );
    }

    #[test]
    fn clear_attribute_omits_value() {
        let attr = Attribute::new("p4", GlobalOpts::new()).name("status");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-n", "status"]
        );
    }

    #[test]
    fn setting_name_or_value_replaces_previous() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .name("status")
            .name("color")
            .value("approved")
            .value("red");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-n", "color", "-v", "red"]
        );
    }

    #[test]
    fn hex_submitted_propagating() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .hex(true)
            .submitted_files(true)
            .propagating(true)
            .name("thumb")
            .value("deadbeef");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            [
                "attribute",
                "-e",
                "-f",
                "-p",
                "-n",
                "thumb",
                "-v",
                "deadbeef"
            ]
        );
    }

    #[test]
    fn stdin_flag() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .stdin(true)
            .name("thumb");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-i", "-n", "thumb"]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn trait_storage_table() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .name("thumb")
            .value("data")
            .trait_storage_table();

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-T0", "-n", "thumb", "-v", "data"]
        );
    }

    #[cfg(not(feature = "lt2023_2"))]
    #[test]
    fn trait_storage_depot() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .name("thumb")
            .value("data")
            .trait_storage_depot();

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-T1", "-n", "thumb", "-v", "data"]
        );
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn read_from_file() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .name("thumb")
            .read_from_file("/tmp/thumb.bin");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            ["attribute", "-I", "/tmp/thumb.bin", "-n", "thumb"]
        );
    }

    #[cfg(not(feature = "lt2024_2"))]
    #[test]
    fn all_modern_options() {
        let attr = Attribute::new("p4", GlobalOpts::new())
            .hex(true)
            .submitted_files(true)
            .propagating(true)
            .trait_storage_depot()
            .read_from_file("/tmp/data.bin")
            .name("thumb");

        assert_eq!(
            args_of(&attr.setup_command("p4")),
            [
                "attribute",
                "-e",
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
        attr.set_name("status").set_value("approved");

        assert_eq!(attr.get_name().unwrap(), &"status".to_string());
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
