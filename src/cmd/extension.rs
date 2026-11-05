#[cfg(not(feature = "lt2020_1"))]
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{ExclusiveOption, SubCommand, Unselected};
use crate::global::GlobalOpts;
use crate::spawn::ParameterizedSpawn;

// ---------------------------------------------------------------------------
// ExtensionEntry
// ---------------------------------------------------------------------------

/// Entry point of `p4 extension`, holding the binary and global options.
///
/// # Description
///
/// The following operations are available: [`sample`](Self::sample),
/// [`package`](Self::package), [`install`](Self::install),
/// [`delete`](Self::delete), [`configure`](Self::configure),
#[cfg_attr(not(feature = "lt2020_1"), doc = " [`run`](Self::run),")]
/// or [`list`](Self::list).
///
#[cfg_attr(
    feature = "lt2020_1",
    doc = "`p4 extension` manages the installation, versioning, and configuration of Helix Core Server Extensions. This command requires super access granted by p4 protect.",
    doc = "",
    doc = "For server extensions, see p4 help serverextensionintro and p4 help serverextensiondev. See also Helix Core Extensions Developer Guide."
)]
#[cfg_attr(
    all(feature = "lt2020_2", not(feature = "lt2020_1")),
    doc = "`p4 extension` manages the installation, versioning, and configuration of Helix Core Server Extensions.",
    doc = "",
    doc = "Extensions provide a way to customize the behavior of the Helix Core Server with user-supplied logic. Extensions are self-contained packages of third-party code and assets run within the Helix Core Server. Extensions can be used for many use cases, such as change submission validation, form validation, external authentication, external job fix integration, external archive integration, and command policies. For a list of events that Extensions can register for, see the list of trigger types in the output of p4 help triggers.",
    doc = "",
    doc = "Extensions coexist with triggers (see p4 triggers), but offer more functionality. The Helix Core Server runs Extensions natively, without relying on external processes, so Extensions provide a portable, versioned runtime with automatic replication and a programmatic API.",
    doc = "",
    doc = "Extensions are versioned within the Helix Core Server and are stored in a special depot, named .p4-extensions by default. The extension depot is only accessible to the super user and by read-only commands. The extension depot is created automatically.",
    doc = "",
    doc = "Installing or upgrading an Extension creates a changelist with information about pre- and post-install versions.",
    doc = "",
    doc = "For more information about Extensions, see p4 help serverextensionintro and Helix Core Extensions Developer Guide."
)]
#[cfg_attr(
    all(feature = "lt2025_1", not(feature = "lt2020_2")),
    doc = "`p4 extension` manages the installation, versioning, and configuration of Helix Core Server Extensions.",
    doc = "",
    doc = "Extensions are a means for customizing parts of the Helix Core with user-supplied logic. Extensions are self-contained packages of third-party code and assets run within the Helix Core Server, used for change submission validation, form validation, external authentication, external job fix integration, external archive integration, and command policies. For a list of events that Extensions can register for, see the list of trigger types in the output of p4 help triggers.",
    doc = "",
    doc = "The Helix Core Server runs Extensions natively, without relying on external processes, so it can provide a portable, versioned runtime with automatic replication and a programmatic API.",
    doc = "",
    doc = "Extensions coexist with triggers (see p4 triggers), but Extensions offer more functionality.",
    doc = "",
    doc = "Extensions are versioned within the Helix Core Server and are stored in a special depot, named .p4-extensions by default. The extension depot is only accessible to the super user and by read-only commands. The extension depot is created automatically.",
    doc = "",
    doc = "Installing or upgrading an Extension creates a changelist with information about pre- and post-install versions.",
    doc = "",
    doc = "The p4 extension command requires super access granted by p4 protect. Users who are members of groups specified in the ExtAllowedGroups field of an Extension's global configuration and who are either a depot or repo owner can create or configure instances of file-based Extensions to operate on the sections of the server they manage.",
    doc = ""
)]
#[cfg_attr(
    all(feature = "lt2024_2", not(feature = "lt2020_2")),
    doc = "For more information about Extensions, see p4 help serverextensionintro and Helix Core Extensions Developer Guide."
)]
#[cfg_attr(
    all(feature = "lt2025_1", not(feature = "lt2024_2")),
    doc = "For more information about Extensions, see p4 help serverextensionintro and P4 Extensions Documentation."
)]
#[cfg_attr(
    not(feature = "lt2025_1"),
    doc = "Create and configure P4 Extensions.",
    doc = "",
    doc = "Extensions are a means for customizing parts of the P4 with user-supplied logic. Extensions are self-contained packages of third-party code and assets that run within the P4 Server. Typical use cases are change submission validation, form validation, external authentication, external job fix integration, external archive integration, and command policies.",
    doc = "",
    doc = "An Extension can implement custom commands that a user can run. For example, `p4 extension --run myInstanceConfig validateFileSize pathToFile` where validateFileSize represents a custom command and pathToFile represents an argument that is relevant to the custom command.",
    doc = "",
    doc = "Extensions coexist with p4 triggers, but offer more functionality. To learn more, see Advantages over triggers in the Extension overview at P4 Server Administration Documentation.",
    doc = "",
    doc = "The P4 Server runs extensions natively, without relying on external processes. Extensions run in a portable, versioned runtime with automatic replication and a programmatic API, which is documented at P4 Server Administration Documentation.",
    doc = "",
    doc = "The p4 extension command requires super access granted by p4 protect. Users who are members of groups specified in the ExtAllowedGroups field of an extension's global configuration and who are either a depot or repo owner can create or configure instances of file-based extensions to operate on the sections of the server they manage.",
    doc = "",
    doc = "Extensions are versioned within the P4 Server and are stored in the built-in .p4-extensions depot. The extension depot is only accessible to the super user and by read-only commands.",
    doc = "",
    doc = "Installing or upgrading an extension creates a changelist with information about pre/post install versions. Multiple revisions of an extension can be active at any given time and can be referenced by their revision number."
)]
pub struct ExtensionEntry {
    bin: PathBuf,

    global_opts: GlobalOpts,
}

impl ExtensionEntry {
    /// Creates an `ExtensionEntry` with the given p4 binary path and global
    /// options.
    pub(crate) fn new(bin: PathBuf, global_opts: GlobalOpts) -> Self {
        Self { bin, global_opts }
    }

    /// # Description
    ///
    /// --sample name
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Create a skeleton of a new Extension in a directory.  Pass it the name of the Extension to be created."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "Creates a skeleton of a new Extension.  The argument is the name of the Extension to create as well as the name of the local directory where the files are placed.  The sample Extension has placeholder values that should be replaced when developing the code. (See Naming rules.)"
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_2")),
        doc = "Create a skeleton of a new Extension in a directory.  Pass it the name of the Extension to be created. (See Naming rules.) `p4 extension --sample myExt` This allows you to then use the `--package` option."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Create a skeleton of a new Extension in a directory.  Pass it the name of the Extension to be created. (See Naming rules.) `p4 extension --sample ExtName` This allows you to then use the `--package` option."
    )]
    pub fn sample(self, name: impl Into<String>) -> Extension<Sample> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Sample { name: name.into() },
        }
    }

    /// # Description
    ///
    /// --package dir
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "The `--package` flag creates a packaged Extension file from the named directory of files.  The resulting file has the same name as the directory, but with the .p4-extension suffix."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "Creates a packaged Extension file from the named directory of files, preparing it for `--install` on the server.  The resulting file has the same name as the directory, but with the .p4-extension suffix. Packaging is done client-side and requires a version 2020.1 or later command-line client."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_2")),
        doc = "The `--package` option creates a packaged Extension file from the named directory of files.  For example, `p4 extension --package myExt` The resulting file has the same name as the directory, but with the .p4-extension suffix. An Extension must be packaged before it can be installed by using the `--install` option."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "The `--package` option creates a packaged Extension file from the named directory of files.  For example, `p4 extension --package ExtName` The resulting file has the same name as the directory, but with the .p4-extension suffix. An Extension must be packaged before it can be installed by using the `--install` option."
    )]
    pub fn package(self, dir: impl Into<String>) -> Extension<Package> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Package {
                dir: dir.into(),
                sign: None,
            },
        }
    }

    /// # Description
    ///
    /// --install file
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "The full name of an Extension file on the client. The command installs it server-side. Packaged Extensions have the p4-extension file suffix."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "The `--install` flag is passed the full name of a packaged Extension file residing on the client and installs it server-side, where it can then be instantiated with `--configure`."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_2")),
        doc = "The full name of an Extension file on the client. The command installs it server-side. Packaged Extensions have the p4-extension file suffix. For example, `p4 extension --install myExt.p4-extension --yes`",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "To actually perform the install or delete, the `--yes` option is required. Without `--yes`, the p4 extension command merely reports what it would do without actually performing the install or delete."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "The full name of an Extension file on the client. The command installs it server-side. Packaged Extensions have the p4-extension file suffix. For example, `p4 extension --install ExtName.p4-extension --yes`",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "To actually perform the install or delete, the `--yes` option is required. Without `--yes`, the p4 extension command merely reports what it would do without actually performing the install or delete."
    )]
    pub fn install(self, file: impl Into<String>) -> Extension<Install<Unselected>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Install {
                file: file.into(),
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// --delete extFQN
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Deletes an extension, its configurations, or both. To remove one specific revision and its instances, include the `--revision` option. To remove only the named instance config, include the `--name` option. To delete file-based instances by the part of the depot they affect, include the `--path` option. To remove all revisions of an extension and its configurations, omit the `--name` option."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "Deletes an extension, its configurations, or both. To remove the named instance config, include the `--name` option. For example: `p4 extension --delete myCompany::fileSizeCheck --name=fileSizeCheck-instance --yes` To remove all revisions of an extension and its configurations, omit the `--name` option. For example: `p4 extension --delete myCompany::fileSizeCheck` To remove one specific revision and its instances, include the `--revision` option. For example: `p4 extension --delete myCompany::fileSizeCheck --name=fileSizeCheck-instance --revision=2 --yes` To delete file-based instances registered under the specified path, use the `--path` option. For example: `p4 extension --delete ExampleInc::extD1 --path=//depot/... --yes`"
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_2")),
        doc = "Deletes an extension, its configurations, or both. To remove the named instance config, include the `--name` option. For example: `p4 extension --delete myCompany::fileSizeCheck --name=fileSizeCheck-instance --yes` To remove all revisions of an extension and its configurations, omit the `--name` option. For example: `p4 extension --delete myCompany::fileSizeCheck` To remove one specific revision and its instances, include the `--revision` option. For example: `p4 extension --delete myCompany::fileSizeCheck --name=fileSizeCheck-instance --revision=2 --yes` To delete file-based instances registered under the specified path, use the `--path` option. For example: `p4 extension --delete ExampleInc::extD1 --path=//depot/... --yes`",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "To actually perform the install or delete, the `--yes` option is required. Without `--yes`, the p4 extension command merely reports what it would do without actually performing the install or delete."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2022_2")),
        doc = "Deletes an extension, its configurations, or both. To remove the named instance config, include the `--name` option: `p4 extension --delete myCompany::fileSizeCheck --name fileSizeCheck-instance --yes` To remove all revisions of an extension and its configurations, omit the `--name` option: `p4 extension --delete myCompany::fileSizeCheck` To remove one specific revision and its instances, include the `--revision` option: `p4 extension --delete myCompany::fileSizeCheck --revision=2 --yes` To delete file-based instances registered under the specified path: `p4 extension --delete ExampleInc::extD1 --path //depot/... --yes`",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "To actually perform the install or delete, the `--yes` option is required. Without `--yes`, the p4 extension command merely reports what it would do without actually performing the install or delete.",
        doc = "",
        doc = "extFQN means fully-qualified name: the combination of the Extension's namespace, name, and optional depot file revision."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "Deletes an extension, its configurations, or both. To remove the named instance config, include the `--name` option: `p4 extension --delete myCompany::fileSizeCheck --name fileSizeCheck-instance --yes` To remove all revisions of an extension and its configurations, omit the `--name` option: `p4 extension --delete myCompany::fileSizeCheck` To remove one specific revision and its instances, include the `--revision` option: `p4 extension --delete myCompany::fileSizeCheck --revision=2 --yes` To delete file-based instances registered under the specified path: `p4 extension --delete ExampleInc::extD1 --path //depot/... --yes`",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "To perform the install or delete, the `--yes` option is required. Without `--yes`, the p4 extension command merely reports what it would do without actually performing the install or delete.",
        doc = "",
        doc = "extFQN means fully-qualified name: the combination of the Extension's namespace, name, and optional depot file revision."
    )]
    pub fn delete(self, fqn: impl Into<String>) -> Extension<Delete<Unselected>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Delete {
                fqn: fqn.into(),
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// --configure extFQN
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Supplies details about an Extension's configuration. To create a global configuration, use `--configure` with the fully-qualified name of the Extension; to create an instance configuration, combine it with the `--name` option."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "The `--configure` flag is used to supply details about an Extension's configuration.  There are two parts to configuring an Extension: the global configuration the instance configuration. To create a global configuration, use `--configure` with the FQN of the Extension. To create an instance configuration, use the `--configure` option with the `--name` option. The `--name` option takes the name of the configuration to create or modify.  More than one named instance is allowed for most event types that an Extension can be coded to work with, such as the change-submit event. However, the auth-check event allows only one named instance."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_2")),
        doc = "An Extension is not active until it has been configured. The default global configuration is: `p4 extension --configure ExampleInc::myExt` and your global configuration might be: `p4 extension --configure myCompany::fileSizeCheck` and the corresponding instance configuration might be: `p4 extension --configure myCompany::extName --name Release1.0-fileSizeCheckInst` An Extension can have multiple configurations, depending on the events it registers for.  Each configuration of an Extension can point to a different version of the Extension."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2022_2")),
        doc = "An Extension is not active until it has been configured. In this syntax: `p4 extension --configure extFQN [ --revision=n ] [ -o | -i ]` extFQN means fully-qualified name: the combination of the Extension's namespace, name, and optional depot file revision. The default global configuration is: `p4 extension --configure ExampleInc::myExt` and your global configuration might be: `p4 extension --configure myCompany::fileSizeCheck` and the corresponding instance configuration might be: `p4 extension --configure myCompany::extName --name Release1.0-fileSizeCheckInst` An Extension can have multiple configurations, depending on the events it registers for.  Each configuration of an Extension can point to a different version of the Extension."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "An Extension is not active until it has been configured. In this syntax: `p4 extension --configure extFQN [ --revision=n ] [ -o | -i ]` extFQN means fully-qualified name: the combination of the Extension's namespace, name, and optional depot file revision. The default global configuration is: `p4 extension --configure ExampleInc::myExt` and your global configuration might be: `p4 extension --configure myCompany::fileSizeCheck` and the corresponding instance configuration might be: `p4 extension --configure myCompany::extName --name Release1.0-fileSizeCheckInst` An Extension can have multiple configurations, depending on the events it registers for.  Each configuration of an Extension can point to a different version of the Extension.",
        doc = "",
        doc = "Since 2025.2, you can prevent users from creating multiple configurations of the same extension. To learn more,, see the limit_instance field under Server Extension JSON manifest fields in the P4 Server Administration Documentation."
    )]
    pub fn configure(self, fqn: impl Into<String>) -> Extension<Configure<Unselected>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Configure {
                fqn: fqn.into(),
                revision: None,
                name: None,
                mode: Unselected,
            },
        }
    }

    /// # Description
    ///
    /// --run instName
    ///
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "Executes the Extension-supplied command. An Extension can implement its own end-user commands that behave similarly to native Helix Core commands in that they can have tagged output, send RPC-level messages, and open forms client-side. However, Extension-supplied commands are restricted to reporting output and cannot access client-side content. The `--run` option takes an instance configuration as its argument, and any other arguments are left to the Extension to interpret."
    )]
    #[cfg_attr(
        not(feature = "lt2020_2"),
        doc = "Executes the specified Extension. An Extension can implement custom end-user commands, executed by passing the `--run` flag the name of an instance config and supplying any optional arguments the Extension might need."
    )]
    #[cfg(not(feature = "lt2020_1"))]
    pub fn run(self, inst: impl Into<String>) -> Extension<Run> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Run { inst: inst.into() },
        }
    }

    /// # Description
    ///
    /// --list
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Displays the installed extensions and their instances.  If this option is specified with the `--type=extensions`, `--list` option, the output shows information about which extensions are installed, and `--type=configs` shows information about the different configurations."
    )]
    #[cfg_attr(
        all(feature = "lt2020_2", not(feature = "lt2020_1")),
        doc = "Displays information about installed Extensions. The value of the `--type` flag determines which data is reported.  Values for `--type` are: extensions configs, further narrowed by global and instance Adding the `--path` option filters instance configuration output to instances whose events are path-based and match the specified filespec."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_2")),
        doc = "Displays the installed extensions and their instances. To list all Extensions: `p4 extension --list --type=extensions` To list all instance configurations: `p4 extension --list --type=configs` You can replace `--type=configs` with `--type=global` or `--type=instance` to filter results accordingly. Adding `--path=<path>` filters output to instance configurations that are path-based and their path argument matches the given `<path>`. The output of `p4 extension --list --type=extensions` indicates the status of extension configuration: No Global Config, No Instance Config, Incomplete Global Config, Incomplete Instance Config, or OK.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "For an extension to be active, an extension must have a global configuration and at least one instance configuration."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Displays the installed extensions and their instances. To list all Extensions: `p4 extension --list --type extensions` To list all instance configurations: `p4 extension --list --type configs` You can replace `--type configs` with `--type global` or `--type instance` to filter results accordingly. Adding `--path path` filters output to instance configurations that are path-based and their path argument matches the given path. The output of `p4 extension --list --type extensions` indicates the status of extension configuration: No Global Config, No Instance Config, Incomplete Global Config, Incomplete Instance Config, or OK.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "For an extension to be active, an extension must have a global configuration and at least one instance configuration."
    )]
    pub fn list(self) -> Extension<List> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: List {
                ty: None,

                #[cfg(not(feature = "lt2022_2"))]
                path: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Extension<T> — command carrier
// ---------------------------------------------------------------------------

/// A `p4 extension` command with a selected operation.
///
/// # Syntax
///
/// ```text
/// p4 [g-opts] extension --sample extName
/// p4 [g-opts] extension --package dir [--sign keyDir]
#[cfg_attr(
    feature = "lt2020_1",
    doc = "p4 [g-opts] extension --install extName.p4-extension"
)]
#[cfg_attr(
    not(feature = "lt2020_1"),
    doc = "p4 [g-opts] extension --install extName.p4-extension [--yes]"
)]
/// p4 [g-opts] extension --install certificateFile --cert [--comment]
/// p4 [g-opts] extension --delete extFQN [--name instName] [--revision=n] [--path FileSpec] [--yes]
/// p4 [g-opts] extension --delete fingerprint --cert
/// p4 [g-opts] extension --configure extFQN [--revision=n] [-o | -i]
/// p4 [g-opts] extension --configure extFQN [--revision=n] [-o | -i] --name instName
/// p4 [g-opts] extension --run instName [extArguments]
/// p4 [g-opts] extension --list --type type [--path FileSpec]
/// ```
///
/// Manages the installation, versioning, and configuration of extensions.
/// The type parameter selects the operation, such as [`Sample`], [`Package`],
/// [`Install`], [`Delete`], [`Configure`],
#[cfg_attr(feature = "lt2020_1", doc = "or [`List`].")]
#[cfg_attr(not(feature = "lt2020_1"), doc = "[`Run`], or [`List`].")]
pub struct Extension<T: ExclusiveOption> {
    bin: PathBuf,

    global_opts: GlobalOpts,

    op: T,
}

impl<T: ExclusiveOption> Extension<T> {
    /// Returns the path of the p4 binary.
    pub fn get_bin(&self) -> &Path {
        &self.bin
    }

    /// Sets the path of the p4 binary.
    pub fn set_bin(&mut self, v: impl Into<PathBuf>) -> &mut Self {
        self.bin = v.into();
        self
    }

    /// Sets the path of the p4 binary.
    pub fn bin(mut self, v: impl Into<PathBuf>) -> Self {
        self.bin = v.into();
        self
    }

    /// # Description
    ///
    /// g-opts
    ///
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
        all(feature = "lt2018_2", not(feature = "lt2017_1")),
        doc = "See [Global Options](GlobalOpts)."
    )]
    #[cfg_attr(not(feature = "lt2018_2"), doc = "See [Global options](GlobalOpts).")]
    pub fn global_opts(mut self, v: GlobalOpts) -> Self {
        self.global_opts = v;
        self
    }
}

impl<T: ExclusiveOption> SubCommand for Extension<T> {
    fn name(&self) -> &str {
        "extension"
    }

    fn inject_local_args(&self, command: &mut Command) {
        self.op.inject_args(command);
    }

    fn global_opts(&self) -> Option<&GlobalOpts> {
        Some(&self.global_opts)
    }
}

impl<T: ExclusiveOption> ParameterizedSpawn<()> for Extension<T> {
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns the selected `p4 extension` operation as a child process with
    /// piped standard output and error streams; use the returned [`Child`]
    /// handle to wait for it or interact with it.
    fn spawn_with(&mut self, (): ()) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

// ---------------------------------------------------------------------------
// sample
// ---------------------------------------------------------------------------

/// The `--sample` operation of `p4 extension`.
///
/// Creates a skeleton of a new extension in a directory named after it.
/// Selected with [`ExtensionEntry::sample`].
#[derive(Debug, Clone, Default)]
pub struct Sample {
    name: String,
}

impl ExclusiveOption for Sample {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--sample").arg(&self.name);
    }
}

impl Extension<Sample> {
    /// Returns the name of the extension skeleton to create.
    pub fn get_name(&self) -> &str {
        &self.op.name
    }

    /// Sets the name of the extension skeleton to create.
    pub fn set_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.name = v.into();
        self
    }

    /// Sets the name of the extension skeleton to create.
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.op.name = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// package
// ---------------------------------------------------------------------------

/// The `--package` operation of `p4 extension`.
///
/// Packages a directory of extension files into a `.p4-extension` file.
/// Selected with [`ExtensionEntry::package`].
#[derive(Debug, Clone, Default)]
pub struct Package {
    dir: String,

    sign: Option<String>,
}

impl ExclusiveOption for Package {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--package").arg(&self.dir);

        if let Some(key_dir) = &self.sign {
            command.arg("--sign").arg(key_dir);
        }
    }
}

impl Extension<Package> {
    /// Returns the directory of extension files to package.
    pub fn get_dir(&self) -> &str {
        &self.op.dir
    }

    /// Sets the directory of extension files to package.
    pub fn set_dir(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.dir = v.into();
        self
    }

    /// Sets the directory of extension files to package.
    pub fn dir(mut self, v: impl Into<String>) -> Self {
        self.op.dir = v.into();
        self
    }

    /// # Description
    ///
    /// --sign keyDir
    ///
    /// Can be supplied in `--package`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_sign(&self) -> Option<&String> {
        self.op.sign.as_ref()
    }

    /// # Description
    ///
    /// --sign keyDir
    ///
    /// Can be supplied in `--package`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_sign(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.sign = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --sign keyDir
    ///
    /// Can be supplied in `--package`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn sign(mut self, v: impl Into<String>) -> Self {
        self.op.sign = Some(v.into());
        self
    }
}

// ---------------------------------------------------------------------------
// install
// ---------------------------------------------------------------------------

/// The extension-file install state of `--install`.
///
/// Adds [`--yes`](Extension::yes) to perform the install for real; without
/// it the command only reports what it would do.
#[cfg(not(feature = "lt2020_1"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct InstallExt {
    yes: bool,

    #[cfg(not(feature = "lt2022_2"))]
    allow_unsigned: bool,
}

#[cfg(not(feature = "lt2020_1"))]
impl ExclusiveOption for InstallExt {
    fn inject_args(&self, command: &mut Command) {
        if self.yes {
            command.arg("--yes");
        }

        #[cfg(not(feature = "lt2022_2"))]
        if self.allow_unsigned {
            command.arg("--allow-unsigned");
        }
    }
}

/// The certificate install state of `--install`.
///
/// Installs a certificate file with `--cert`, optionally recording a
/// [`--comment`](Extension::comment).
#[derive(Debug, Clone, Default)]
#[cfg(not(feature = "lt2022_2"))]
pub struct InstallCert {
    comment: Option<String>,
}

#[cfg(not(feature = "lt2022_2"))]
impl ExclusiveOption for InstallCert {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--cert");

        if let Some(comment) = &self.comment {
            command.arg("--comment").arg(comment);
        }
    }
}

/// The `--install` operation of `p4 extension`.
///
#[cfg_attr(
    feature = "lt2020_1",
    doc = "The `M` type parameter only ever takes [`Unselected`]; `--yes` cannot be supplied to `--install`."
)]
#[cfg_attr(
    all(feature = "lt2022_2", not(feature = "lt2020_1")),
    doc = "The type parameter selects what is installed: [`Unselected`] for a plain install, or [`InstallExt`] for an extension package."
)]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "The type parameter selects what is installed: [`Unselected`] for a plain install, [`InstallExt`] for an extension package, or [`InstallCert`] for a certificate."
)]
/// Selected with [`ExtensionEntry::install`].
#[derive(Debug, Clone, Default)]
pub struct Install<M = Unselected> {
    file: String,

    mode: M,
}

impl<M: ExclusiveOption> ExclusiveOption for Install<M> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--install").arg(&self.file);
        self.mode.inject_args(command);
    }
}

impl Extension<Install<Unselected>> {
    /// Returns the packaged extension or certificate file to install.
    pub fn get_file(&self) -> &str {
        &self.op.file
    }

    /// Sets the packaged extension or certificate file to install.
    pub fn set_file(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.file = v.into();
        self
    }

    /// Sets the packaged extension or certificate file to install.
    pub fn file(mut self, v: impl Into<String>) -> Self {
        self.op.file = v.into();
        self
    }

    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn yes(self) -> Extension<Install<InstallExt>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Install {
                file: self.op.file,
                mode: InstallExt {
                    yes: true,

                    #[cfg(not(feature = "lt2022_2"))]
                    allow_unsigned: false,
                },
            },
        }
    }

    /// # Description
    ///
    /// --cert
    ///
    /// Can be supplied in `--install` or `--delete`. For example,
    /// `p4 extension --install path/certificate.txt --cert --comment "We trust ACTools"`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn cert(self) -> Extension<Install<InstallCert>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Install {
                file: self.op.file,
                mode: InstallCert { comment: None },
            },
        }
    }

    /// # Description
    ///
    /// --comment
    ///
    /// Can be supplied in `--install` or `--delete`. For example,
    /// `p4 extension --install path/certificate.txt --cert --comment "We trust ACTools"`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn comment(self, v: impl Into<String>) -> Extension<Install<InstallCert>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Install {
                file: self.op.file,
                mode: InstallCert {
                    comment: Some(v.into()),
                },
            },
        }
    }
}

#[cfg(not(feature = "lt2020_1"))]
impl Extension<Install<InstallExt>> {
    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn get_yes(&self) -> bool {
        self.op.mode.yes
    }

    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn set_yes(&mut self, v: bool) -> &mut Self {
        self.op.mode.yes = v;
        self
    }

    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn yes(mut self, v: bool) -> Self {
        self.op.mode.yes = v;
        self
    }

    /// # Description
    ///
    /// --allow-unsigned
    ///
    #[cfg_attr(
        all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        ),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the Configurables reference configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg_attr(
        not(all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        )),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the server.extensions.allow.unsigned configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_allow_unsigned(&self) -> bool {
        self.op.mode.allow_unsigned
    }

    /// # Description
    ///
    /// --allow-unsigned
    ///
    #[cfg_attr(
        all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        ),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the Configurables reference configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg_attr(
        not(all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        )),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the server.extensions.allow.unsigned configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_allow_unsigned(&mut self, v: bool) -> &mut Self {
        self.op.mode.allow_unsigned = v;
        self
    }

    /// # Description
    ///
    /// --allow-unsigned
    ///
    #[cfg_attr(
        all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        ),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the Configurables reference configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg_attr(
        not(all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        )),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the server.extensions.allow.unsigned configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg(not(feature = "lt2022_2"))]
    pub fn allow_unsigned(mut self, v: bool) -> Self {
        self.op.mode.allow_unsigned = v;
        self
    }
}

#[cfg(not(feature = "lt2022_2"))]
impl Extension<Install<InstallCert>> {
    /// # Description
    ///
    /// --comment
    ///
    /// Can be supplied in `--install` or `--delete`. For example,
    /// `p4 extension --install path/certificate.txt --cert --comment "We trust ACTools"`.
    pub fn get_comment(&self) -> Option<&String> {
        self.op.mode.comment.as_ref()
    }

    /// # Description
    ///
    /// --comment
    ///
    /// Can be supplied in `--install` or `--delete`. For example,
    /// `p4 extension --install path/certificate.txt --cert --comment "We trust ACTools"`.
    pub fn set_comment(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.mode.comment = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --comment
    ///
    /// Can be supplied in `--install` or `--delete`. For example,
    /// `p4 extension --install path/certificate.txt --cert --comment "We trust ACTools"`.
    pub fn comment(mut self, v: impl Into<String>) -> Self {
        self.op.mode.comment = Some(v.into());
        self
    }
}

// ---------------------------------------------------------------------------
// delete
// ---------------------------------------------------------------------------

/// The extension delete state of `--delete`.
///
/// Narrows the deletion with [`--name`](Extension::name),
/// [`--revision`](Extension::revision), or [`--path`](Extension::path), and
/// performs it for real with [`--yes`](Extension::yes).
#[derive(Debug, Clone, Default)]
pub struct DeleteExt {
    yes: bool,

    #[cfg(not(feature = "lt2020_1"))]
    name: Option<String>,

    revision: Option<String>,

    path: Option<String>,

    #[cfg(not(feature = "lt2022_2"))]
    allow_unsigned: bool,
}

impl ExclusiveOption for DeleteExt {
    fn inject_args(&self, command: &mut Command) {
        #[cfg(not(feature = "lt2020_1"))]
        if let Some(name) = &self.name {
            command.arg("--name").arg(name);
        }

        if let Some(revision) = &self.revision {
            command.arg(format!("--revision={revision}"));
        }

        if let Some(path) = &self.path {
            command.arg("--path").arg(path);
        }

        if self.yes {
            command.arg("--yes");
        }

        #[cfg(not(feature = "lt2022_2"))]
        if self.allow_unsigned {
            command.arg("--allow-unsigned");
        }
    }
}

/// The certificate delete state of `--delete`.
///
/// Deletes a certificate by fingerprint with `--cert`. Selected with
/// [`Extension::cert`].
#[derive(Debug, Clone, Copy, Default)]
#[cfg(not(feature = "lt2022_2"))]
pub struct DeleteCert;

#[cfg(not(feature = "lt2022_2"))]
impl ExclusiveOption for DeleteCert {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--cert");
    }
}

/// The `--delete` operation of `p4 extension`.
///
/// The type parameter selects what is deleted:
#[cfg_attr(
    feature = "lt2022_2",
    doc = "[`Unselected`] or [`DeleteExt`] for an extension."
)]
#[cfg_attr(
    not(feature = "lt2022_2"),
    doc = "[`Unselected`] or [`DeleteExt`] for an extension, [`DeleteCert`] for a certificate."
)]
/// Selected with [`ExtensionEntry::delete`].
#[derive(Debug, Clone, Default)]
pub struct Delete<M = Unselected> {
    fqn: String,

    mode: M,
}

impl<M: ExclusiveOption> ExclusiveOption for Delete<M> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--delete").arg(&self.fqn);
        self.mode.inject_args(command);
    }
}

impl Extension<Delete<Unselected>> {
    /// Returns the fully-qualified name (or certificate fingerprint) to
    /// delete.
    pub fn get_fqn(&self) -> &str {
        &self.op.fqn
    }

    /// Sets the fully-qualified name (or certificate fingerprint) to delete.
    pub fn set_fqn(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.fqn = v.into();
        self
    }

    /// Sets the fully-qualified name (or certificate fingerprint) to delete.
    pub fn fqn(mut self, v: impl Into<String>) -> Self {
        self.op.fqn = v.into();
        self
    }

    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn yes(self) -> Extension<Delete<DeleteExt>> {
        self.delete_ext(|mode| mode.yes = true)
    }

    /// # Description
    ///
    /// --name instName
    ///
    /// To remove the named instance config, include the `--name` option.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn name(self, v: impl Into<String>) -> Extension<Delete<DeleteExt>> {
        let name = v.into();
        self.delete_ext(|mode| mode.name = Some(name))
    }

    /// # Description
    ///
    /// --revision=n
    ///
    /// To remove one specific revision and its instances, include the
    /// `--revision` option.
    pub fn revision(self, v: impl Into<String>) -> Extension<Delete<DeleteExt>> {
        let revision = v.into();
        self.delete_ext(|mode| mode.revision = Some(revision))
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// To delete file-based instances registered under the specified path,
    /// use the `--path` option.
    pub fn path(self, v: impl Into<String>) -> Extension<Delete<DeleteExt>> {
        let path = v.into();
        self.delete_ext(|mode| mode.path = Some(path))
    }

    /// # Description
    ///
    /// --cert
    ///
    /// Can be supplied in `--install` or `--delete`. For example,
    /// `p4 extension --install path/certificate.txt --cert --comment "We trust ACTools"`.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn cert(self) -> Extension<Delete<DeleteCert>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Delete {
                fqn: self.op.fqn,
                mode: DeleteCert,
            },
        }
    }

    fn delete_ext(mut self, f: impl FnOnce(&mut DeleteExt)) -> Extension<Delete<DeleteExt>> {
        let mut mode = DeleteExt::default();
        f(&mut mode);
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Delete {
                fqn: std::mem::take(&mut self.op.fqn),
                mode,
            },
        }
    }
}

impl Extension<Delete<DeleteExt>> {
    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn get_yes(&self) -> bool {
        self.op.mode.yes
    }

    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn set_yes(&mut self, v: bool) -> &mut Self {
        self.op.mode.yes = v;
        self
    }

    /// # Description
    ///
    /// --yes
    ///
    /// To actually perform the install or delete, the `--yes` option is
    /// required. Without `--yes`, the p4 extension command merely reports
    /// what it would do without actually performing the install or delete.
    /// `-y` is equivalent to `--yes`.
    pub fn yes(mut self, v: bool) -> Self {
        self.op.mode.yes = v;
        self
    }

    /// # Description
    ///
    /// --name instName
    ///
    /// To remove the named instance config, include the `--name` option.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn get_name(&self) -> Option<&String> {
        self.op.mode.name.as_ref()
    }

    /// # Description
    ///
    /// --name instName
    ///
    /// To remove the named instance config, include the `--name` option.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn set_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.mode.name = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --name instName
    ///
    /// To remove the named instance config, include the `--name` option.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.op.mode.name = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --revision=n
    ///
    /// To remove one specific revision and its instances, include the
    /// `--revision` option.
    pub fn get_revision(&self) -> Option<&String> {
        self.op.mode.revision.as_ref()
    }

    /// # Description
    ///
    /// --revision=n
    ///
    /// To remove one specific revision and its instances, include the
    /// `--revision` option.
    pub fn set_revision(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.mode.revision = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --revision=n
    ///
    /// To remove one specific revision and its instances, include the
    /// `--revision` option.
    pub fn revision(mut self, v: impl Into<String>) -> Self {
        self.op.mode.revision = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// To delete file-based instances registered under the specified path,
    /// use the `--path` option.
    pub fn get_path(&self) -> Option<&String> {
        self.op.mode.path.as_ref()
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// To delete file-based instances registered under the specified path,
    /// use the `--path` option.
    pub fn set_path(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.mode.path = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// To delete file-based instances registered under the specified path,
    /// use the `--path` option.
    pub fn path(mut self, v: impl Into<String>) -> Self {
        self.op.mode.path = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --allow-unsigned
    ///
    #[cfg_attr(
        all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        ),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the Configurables reference configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg_attr(
        not(all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        )),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the server.extensions.allow.unsigned configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_allow_unsigned(&self) -> bool {
        self.op.mode.allow_unsigned
    }

    /// # Description
    ///
    /// --allow-unsigned
    ///
    #[cfg_attr(
        all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        ),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the Configurables reference configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg_attr(
        not(all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        )),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the server.extensions.allow.unsigned configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_allow_unsigned(&mut self, v: bool) -> &mut Self {
        self.op.mode.allow_unsigned = v;
        self
    }

    /// # Description
    ///
    /// --allow-unsigned
    ///
    #[cfg_attr(
        all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        ),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the Configurables reference configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg_attr(
        not(all(
            feature = "lt2025_1",
            not(feature = "lt2022_2"),
            feature = "lt2024_2",
            not(feature = "lt2024_1")
        )),
        doc = "(Optional) If included in the commands to install (or delete) an extension, it is the same as if the server.extensions.allow.unsigned configurable were set to 1 so that the extension does not need to be signed."
    )]
    #[cfg(not(feature = "lt2022_2"))]
    pub fn allow_unsigned(mut self, v: bool) -> Self {
        self.op.mode.allow_unsigned = v;
        self
    }
}

// ---------------------------------------------------------------------------
// configure
// ---------------------------------------------------------------------------

/// The `-i` state of `--configure`.
///
/// Reads the extension definition from standard input. Selected with
/// [`Extension::stdin`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Stdin;

impl ExclusiveOption for Stdin {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-i");
    }
}

/// The `-o` state of `--configure`.
///
/// Writes the extension definition to standard output. Selected with
/// [`Extension::stdout`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Stdout;

impl ExclusiveOption for Stdout {
    fn inject_args(&self, command: &mut Command) {
        command.arg("-o");
    }
}

/// The `--configure` operation of `p4 extension`.
///
/// The type parameter selects where the extension definition goes:
/// [`Unselected`] to edit it in a form, [`Stdin`] (`-i`) to read it from
/// standard input, or [`Stdout`] (`-o`) to write it to standard output.
/// Selected with [`ExtensionEntry::configure`].
#[derive(Debug, Clone, Default)]
pub struct Configure<M = Unselected> {
    fqn: String,

    revision: Option<String>,

    name: Option<String>,

    mode: M,
}

impl<M: ExclusiveOption> ExclusiveOption for Configure<M> {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--configure").arg(&self.fqn);

        if let Some(revision) = &self.revision {
            command.arg(format!("--revision={revision}"));
        }

        self.mode.inject_args(command);

        if let Some(name) = &self.name {
            command.arg("--name").arg(name);
        }
    }
}

impl Extension<Configure<Unselected>> {
    /// # Description
    ///
    /// -i
    ///
    #[cfg_attr(
        not(feature = "lt2022_1"),
        doc = "Read the extension definition from standard input."
    )]
    pub fn stdin(self) -> Extension<Configure<Stdin>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Configure {
                fqn: self.op.fqn,
                revision: self.op.revision,
                name: self.op.name,
                mode: Stdin,
            },
        }
    }

    /// # Description
    ///
    /// -o
    ///
    #[cfg_attr(
        not(feature = "lt2022_1"),
        doc = "Write the extension definition from to standard output."
    )]
    pub fn stdout(self) -> Extension<Configure<Stdout>> {
        Extension {
            bin: self.bin,
            global_opts: self.global_opts,
            op: Configure {
                fqn: self.op.fqn,
                revision: self.op.revision,
                name: self.op.name,
                mode: Stdout,
            },
        }
    }
}

impl<M: ExclusiveOption> Extension<Configure<M>> {
    /// Returns the fully-qualified name of the extension to configure.
    pub fn get_fqn(&self) -> &str {
        &self.op.fqn
    }

    /// Sets the fully-qualified name of the extension to configure.
    pub fn set_fqn(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.fqn = v.into();
        self
    }

    /// Sets the fully-qualified name of the extension to configure.
    pub fn fqn(mut self, v: impl Into<String>) -> Self {
        self.op.fqn = v.into();
        self
    }

    /// # Description
    ///
    /// --revision=n
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option."
    )]
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2020_1")),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2022_1")),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option. It is optional to include a # followed by a revision number, such as ExampleInc::extName#6."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option."
    )]
    pub fn get_revision(&self) -> Option<&String> {
        self.op.revision.as_ref()
    }

    /// # Description
    ///
    /// --revision=n
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option."
    )]
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2020_1")),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2022_1")),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option. It is optional to include a # followed by a revision number, such as ExampleInc::extName#6."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option."
    )]
    pub fn set_revision(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.revision = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --revision=n
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option."
    )]
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2020_1")),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2022_1")),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option. It is optional to include a # followed by a revision number, such as ExampleInc::extName#6."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Which depot version of the Extension to configure.  The default is the head revision.  This option applies to the `--configure` option. This flag applies to the `--configure` option when used with or without the `--name` option."
    )]
    pub fn revision(mut self, v: impl Into<String>) -> Self {
        self.op.revision = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --name instName
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "The name of the configuration to create or modify."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_1")),
        doc = "The name of the instance configuration to create or modify."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "The name of the instance configuration to create, modify, or delete."
    )]
    pub fn get_name(&self) -> Option<&String> {
        self.op.name.as_ref()
    }

    /// # Description
    ///
    /// --name instName
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "The name of the configuration to create or modify."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_1")),
        doc = "The name of the instance configuration to create or modify."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "The name of the instance configuration to create, modify, or delete."
    )]
    pub fn set_name(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.name = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --name instName
    ///
    #[cfg_attr(
        feature = "lt2020_1",
        doc = "The name of the configuration to create or modify."
    )]
    #[cfg_attr(
        all(feature = "lt2022_2", not(feature = "lt2020_1")),
        doc = "The name of the instance configuration to create or modify."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "The name of the instance configuration to create, modify, or delete."
    )]
    pub fn name(mut self, v: impl Into<String>) -> Self {
        self.op.name = Some(v.into());
        self
    }
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

/// The `--run` operation of `p4 extension`.
///
/// Executes an extension-supplied command on an instance configuration.
/// Selected with [`ExtensionEntry::run`].
///
/// Extra arguments the extension might need are passed at spawn time:
/// `ParameterizedSpawn<()>` spawns the command with no extra arguments, and
/// `ParameterizedSpawn<(S,)>` spawns it with an arbitrary number of them.
#[cfg(not(feature = "lt2020_1"))]
#[derive(Debug, Clone, Default)]
pub struct Run {
    inst: String,
}

#[cfg(not(feature = "lt2020_1"))]
impl ExclusiveOption for Run {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--run").arg(&self.inst);
    }
}

#[cfg(not(feature = "lt2020_1"))]
impl<S, I> ParameterizedSpawn<(S,)> for Extension<Run>
where
    S: IntoIterator<Item = I>,
    I: AsRef<OsStr>,
{
    type Output = Child;
    type Error = std::io::Error;

    /// Spawns `p4 extension --run instName` with the given extra arguments
    /// passed through to the extension, with piped standard output and error
    /// streams; use the returned [`Child`] handle to wait for it or interact
    /// with it.
    fn spawn_with(&mut self, (arguments,): (S,)) -> Result<Self::Output, Self::Error> {
        self.setup_command(&self.bin)
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[cfg(not(feature = "lt2020_1"))]
impl Extension<Run> {
    /// Returns the instance configuration whose extension is executed.
    pub fn get_inst(&self) -> &str {
        &self.op.inst
    }

    /// Sets the instance configuration whose extension is executed.
    pub fn set_inst(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.inst = v.into();
        self
    }

    /// Sets the instance configuration whose extension is executed.
    pub fn inst(mut self, v: impl Into<String>) -> Self {
        self.op.inst = v.into();
        self
    }
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

/// The `--type` value of `--list`.
///
/// Determines which data is reported: installed extensions, their global
/// configurations, or their instance configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// Reports the installed extensions.
    Extensions,

    /// Reports both global and instance configurations.
    Configs,

    /// Reports global configurations.
    #[cfg(not(feature = "lt2020_1"))]
    Global,

    /// Reports instance configurations.
    #[cfg(not(feature = "lt2020_1"))]
    Instance,

    /// Reports the installed certificates.
    #[cfg(not(feature = "lt2022_2"))]
    Certs,
}

impl Type {
    fn to_str(&self) -> &'static str {
        match self {
            Type::Extensions => "extensions",
            Type::Configs => "configs",

            #[cfg(not(feature = "lt2020_1"))]
            Type::Global => "global",

            #[cfg(not(feature = "lt2020_1"))]
            Type::Instance => "instance",

            #[cfg(not(feature = "lt2022_2"))]
            Type::Certs => "certs",
        }
    }
}

/// The `--list` operation of `p4 extension`.
///
/// Displays the installed extensions and their configurations, as selected
/// by [`--type`](Extension::ty). Selected with [`ExtensionEntry::list`].
#[derive(Debug, Clone, Default)]
pub struct List {
    ty: Option<Type>,

    #[cfg(not(feature = "lt2022_2"))]
    path: Option<String>,
}

impl ExclusiveOption for List {
    fn inject_args(&self, command: &mut Command) {
        command.arg("--list");

        if let Some(ty) = &self.ty {
            command.arg("--type").arg(ty.to_str());
        }

        #[cfg(not(feature = "lt2022_2"))]
        if let Some(path) = &self.path {
            command.arg("--path").arg(path);
        }
    }
}

impl Extension<List> {
    /// # Description
    ///
    /// --type type
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "Determines which data is reported: the installed extensions, their global configurations, or their instance configurations."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Must be supplied with `--list`. Possible type values: extensions, global, instance, certs, configs (both global and instance). For example, `p4 extension --list --type certs`."
    )]
    pub fn get_ty(&self) -> Option<Type> {
        self.op.ty
    }

    /// # Description
    ///
    /// --type type
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "Determines which data is reported: the installed extensions, their global configurations, or their instance configurations."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Must be supplied with `--list`. Possible type values: extensions, global, instance, certs, configs (both global and instance). For example, `p4 extension --list --type certs`."
    )]
    pub fn set_ty(&mut self, v: Type) -> &mut Self {
        self.op.ty = Some(v);
        self
    }

    /// # Description
    ///
    /// --type type
    ///
    #[cfg_attr(
        feature = "lt2022_2",
        doc = "Determines which data is reported: the installed extensions, their global configurations, or their instance configurations."
    )]
    #[cfg_attr(
        not(feature = "lt2022_2"),
        doc = "Must be supplied with `--list`. Possible type values: extensions, global, instance, certs, configs (both global and instance). For example, `p4 extension --list --type certs`."
    )]
    pub fn ty(mut self, v: Type) -> Self {
        self.op.ty = Some(v);
        self
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// Filters output to instance configurations that are path-based and
    /// whose path argument matches the given filespec.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_path(&self) -> Option<&String> {
        self.op.path.as_ref()
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// Filters output to instance configurations that are path-based and
    /// whose path argument matches the given filespec.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_path(&mut self, v: impl Into<String>) -> &mut Self {
        self.op.path = Some(v.into());
        self
    }

    /// # Description
    ///
    /// --path FileSpec
    ///
    /// Filters output to instance configurations that are path-based and
    /// whose path argument matches the given filespec.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn path(mut self, v: impl Into<String>) -> Self {
        self.op.path = Some(v.into());
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

    fn entry() -> ExtensionEntry {
        ExtensionEntry::new("p4".into(), GlobalOpts::new())
    }

    /// Dry-run checks of the assembled `p4 extension` command line; no
    /// process is spawned.
    #[test]
    fn sample_injects_name() {
        assert_eq!(
            args(&entry().sample("ExtName")),
            ["extension", "--sample", "ExtName"]
        );
    }

    #[test]
    fn package_without_sign() {
        assert_eq!(
            args(&entry().package("ExtName")),
            ["extension", "--package", "ExtName"]
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn package_with_sign() {
        assert_eq!(
            args(&entry().package("ExtName").sign("keys")),
            ["extension", "--package", "ExtName", "--sign", "keys"]
        );
    }

    #[test]
    fn install_dry_run_has_no_yes() {
        assert_eq!(
            args(&entry().install("ExtName.p4-extension")),
            ["extension", "--install", "ExtName.p4-extension"]
        );
    }

    #[cfg(not(feature = "lt2020_1"))]
    #[test]
    fn install_ext_yes() {
        assert_eq!(
            args(&entry().install("ExtName.p4-extension").yes()),
            ["extension", "--install", "ExtName.p4-extension", "--yes"]
        );
    }

    #[cfg(all(not(feature = "lt2020_1"), not(feature = "lt2022_2")))]
    #[test]
    fn install_ext_yes_and_allow_unsigned() {
        assert_eq!(
            args(
                &entry()
                    .install("ExtName.p4-extension")
                    .yes()
                    .allow_unsigned(true)
            ),
            [
                "extension",
                "--install",
                "ExtName.p4-extension",
                "--yes",
                "--allow-unsigned"
            ]
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn install_cert_with_comment() {
        assert_eq!(
            args(&entry().install("cert.txt").comment("We trust ACTools")),
            [
                "extension",
                "--install",
                "cert.txt",
                "--cert",
                "--comment",
                "We trust ACTools"
            ]
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn install_cert_without_comment() {
        assert_eq!(
            args(&entry().install("cert.txt").cert()),
            ["extension", "--install", "cert.txt", "--cert"]
        );
    }

    #[test]
    fn delete_dry_run_has_no_options() {
        assert_eq!(
            args(&entry().delete("myCompany::ext")),
            ["extension", "--delete", "myCompany::ext"]
        );
    }

    #[test]
    fn delete_ext_with_revision_path_yes() {
        assert_eq!(
            args(
                &entry()
                    .delete("myCompany::ext")
                    .revision("2")
                    .path("//depot/...")
                    .yes(true)
            ),
            [
                "extension",
                "--delete",
                "myCompany::ext",
                "--revision=2",
                "--path",
                "//depot/...",
                "--yes"
            ]
        );
    }

    #[cfg(not(feature = "lt2020_1"))]
    #[test]
    fn delete_ext_with_name() {
        assert_eq!(
            args(&entry().delete("myCompany::ext").name("inst").yes(true)),
            [
                "extension",
                "--delete",
                "myCompany::ext",
                "--name",
                "inst",
                "--yes"
            ]
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn delete_cert() {
        assert_eq!(
            args(&entry().delete("AA:BB:CC").cert()),
            ["extension", "--delete", "AA:BB:CC", "--cert"]
        );
    }

    #[test]
    fn configure_global_only() {
        assert_eq!(
            args(&entry().configure("myCompany::ext")),
            ["extension", "--configure", "myCompany::ext"]
        );
    }

    #[test]
    fn configure_instance_with_revision_and_stdin() {
        assert_eq!(
            args(
                &entry()
                    .configure("myCompany::ext")
                    .revision("2")
                    .name("inst")
                    .stdin()
            ),
            [
                "extension",
                "--configure",
                "myCompany::ext",
                "--revision=2",
                "-i",
                "--name",
                "inst"
            ]
        );
    }

    #[test]
    fn configure_stdout() {
        assert_eq!(
            args(&entry().configure("myCompany::ext").stdout()),
            ["extension", "--configure", "myCompany::ext", "-o"]
        );
    }

    #[cfg(not(feature = "lt2020_1"))]
    #[test]
    fn run_without_arguments() {
        assert_eq!(args(&entry().run("inst")), ["extension", "--run", "inst"]);
    }

    /// The extra-argument form is a separate `ParameterizedSpawn` impl that
    /// appends the argument list at spawn time.
    #[cfg(not(feature = "lt2020_1"))]
    #[test]
    fn run_accepts_arbitrary_arguments_at_spawn() {
        fn assert_arguments_spawn<S, I>()
        where
            S: IntoIterator<Item = I>,
            I: AsRef<OsStr>,
            Extension<Run>: ParameterizedSpawn<(S,)>,
        {
        }

        assert_arguments_spawn::<Vec<String>, String>();
        assert_arguments_spawn::<[&str; 2], &str>();
    }

    #[test]
    fn list_with_type() {
        assert_eq!(
            args(&entry().list().ty(Type::Configs)),
            ["extension", "--list", "--type", "configs"]
        );
    }

    #[cfg(not(feature = "lt2022_2"))]
    #[test]
    fn list_with_type_and_path() {
        assert_eq!(
            args(&entry().list().ty(Type::Instance).path("//depot/...")),
            [
                "extension",
                "--list",
                "--type",
                "instance",
                "--path",
                "//depot/..."
            ]
        );
    }

    #[test]
    fn global_opts_are_injected_once() {
        let cmd = ExtensionEntry::new(
            "p4".into(),
            GlobalOpts::new().port("localhost:1666").quiet_mode(true),
        )
        .configure("myCompany::ext");

        assert_eq!(
            args(&cmd),
            [
                "-p",
                "localhost:1666",
                "-q",
                "extension",
                "--configure",
                "myCompany::ext"
            ]
        );
    }

    #[test]
    fn bin_accessors_update_executable_path() {
        let mut cmd = entry().sample("ExtName");
        assert_eq!(cmd.get_bin(), Path::new("p4"));

        cmd.set_bin("/opt/perforce/bin/p4");
        assert_eq!(cmd.get_bin(), Path::new("/opt/perforce/bin/p4"));

        let cmd = cmd.bin("p4custom");
        assert_eq!(cmd.get_bin(), Path::new("p4custom"));
    }

    #[test]
    fn delete_ext_accessors_update_mode() {
        let mut cmd = entry().delete("myCompany::ext").yes();
        assert!(cmd.get_yes());

        cmd.set_yes(false);
        assert!(!cmd.get_yes());

        cmd.set_revision("3");
        assert_eq!(cmd.get_revision(), Some(&"3".to_string()));
    }
}
