use std::{path::PathBuf, process::Command};

///
/// ## Synopsis
///
/// Global options for Perforce commands; these options can be supplied on the command line before any Perforce command.
///
#[derive(Debug, Clone, Default)]
pub struct GlobalOpts {
    batch_size: Option<u32>,

    client: Option<String>,

    dir: Option<PathBuf>,

    progress_indicators: bool,

    #[cfg(not(feature = "lt2019_1"))]
    standard_in: bool,

    marshalled_formatted: bool,

    #[cfg(not(feature = "lt2020_1"))]
    json_formatted: bool,

    host: Option<String>,

    #[cfg(not(feature = "lt2019_1"))]
    standard_out: bool,

    port: Option<String>,

    pass: Option<String>,

    retries: Option<u32>,

    prepend_descriptive: bool,

    user: Option<String>,

    argfile: Option<PathBuf>,

    charset: Option<String>,

    command_charset: Option<String>,

    language: Option<String>,

    tagged_format: bool,

    quiet_mode: bool,

    #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
    debug: bool,

    #[cfg(not(feature = "lt2022_2"))]
    debug_variable: Option<Vec<String>>,

    display_version: bool,

    display_help: bool,
}

impl GlobalOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn setup_args<'a>(&self, command: &'a mut Command) -> &'a mut Command {
        if let Some(batch_size) = self.batch_size {
            command.arg("-b").arg(batch_size.to_string());
        }
        if let Some(client) = self.client.as_ref() {
            command.arg("-c").arg(client);
        }
        if let Some(dir) = self.dir.as_ref() {
            command.arg("-d").arg(dir);
        }
        if self.progress_indicators {
            command.arg("-I");
        }
        #[cfg(not(feature = "lt2019_1"))]
        if self.standard_in {
            command.arg("-i");
        }
        if self.marshalled_formatted {
            command.arg("-G");
        }
        #[cfg(not(feature = "lt2020_1"))]
        if self.json_formatted {
            command.arg("-Mj");
        }
        if let Some(host) = self.host.as_ref() {
            command.arg("-H").arg(host);
        }
        #[cfg(not(feature = "lt2019_1"))]
        if self.standard_out {
            command.arg("-o");
        }
        if let Some(port) = self.port.as_ref() {
            command.arg("-p").arg(port);
        }
        if let Some(pass) = self.pass.as_ref() {
            command.arg("-P").arg(pass);
        }
        if let Some(retries) = self.retries {
            command.arg("-r").arg(retries.to_string());
        }
        if self.prepend_descriptive {
            command.arg("-s");
        }
        if let Some(user) = self.user.as_ref() {
            command.arg("-u").arg(user);
        }
        if let Some(argfile) = self.argfile.as_ref() {
            command.arg("-x").arg(argfile);
        }
        if let Some(charset) = self.charset.as_ref() {
            command.arg("-C").arg(charset);
        }
        if let Some(command_charset) = self.command_charset.as_ref() {
            command.arg("-Q").arg(command_charset);
        }
        if let Some(language) = self.language.as_ref() {
            command.arg("-L").arg(language);
        }
        if self.tagged_format {
            command.arg("-z").arg("tag");
        }
        if self.quiet_mode {
            command.arg("-q");
        }
        #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
        if self.debug {
            command.arg("-v");
        }
        #[cfg(not(feature = "lt2022_2"))]
        if let Some(debug_variables) = self.debug_variable.as_ref() {
            for debug_variable in debug_variables {
                command.arg("-v").arg(debug_variable);
            }
        }
        if self.display_version {
            command.arg("-V");
        }
        if self.display_help {
            command.arg("-h");
        }
        command
    }

    // pub fn setup_command_with<F>(&self, cmd: &str, handler: F) -> std::process::Command
    // where
    //     F: FnOnce(&mut std::process::Command),
    // {
    //     let mut command = std::process::Command::new(cmd);

    //     handler(&mut command);
    //     command
    // }
}

impl GlobalOpts {
    /// # Description
    ///
    /// -b batchsize
    ///
    /// Specifies a batch size (number of arguments) to use when processing a command
    /// from a file with the -x argfile option. By default, the batch size is 128.
    pub fn get_batch_size(&self) -> Option<&u32> {
        self.batch_size.as_ref()
    }
    /// # Description
    ///
    /// -b batchsize
    ///
    /// Specifies a batch size (number of arguments) to use when processing a command
    /// from a file with the -x argfile option. By default, the batch size is 128.
    pub fn set_batch_size(&mut self, v: u32) -> &mut Self {
        self.batch_size = Some(v);
        self
    }

    /// # Description
    ///
    /// -b batchsize
    ///
    /// Specifies a batch size (number of arguments) to use when processing a command
    /// from a file with the -x argfile option. By default, the batch size is 128.
    pub fn batch_size(mut self, v: u32) -> Self {
        self.batch_size = Some(v);
        self
    }

    /// # Description
    ///
    /// -c client
    ///
    /// Overrides any P4CLIENT setting with the specified client name.
    pub fn get_client(&self) -> Option<&String> {
        self.client.as_ref()
    }
    /// # Description
    ///
    /// -c client
    ///
    /// Overrides any P4CLIENT setting with the specified client name.
    pub fn set_client(&mut self, v: impl Into<String>) -> &mut Self {
        self.client = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -c client
    ///
    /// Overrides any P4CLIENT setting with the specified client name.
    pub fn client(mut self, v: impl Into<String>) -> Self {
        self.client = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -d dir
    ///
    /// Overrides any PWD setting (current working directory) and replaces it with
    /// the specified directory.
    pub fn get_dir(&self) -> Option<&PathBuf> {
        self.dir.as_ref()
    }
    /// # Description
    ///
    /// -d dir
    ///
    /// Overrides any PWD setting (current working directory) and replaces it with
    /// the specified directory.
    pub fn set_dir(&mut self, v: PathBuf) -> &mut Self {
        self.dir = Some(v);
        self
    }
    /// # Description
    ///
    /// -d dir
    ///
    /// Overrides any PWD setting (current working directory) and replaces it with
    /// the specified directory.
    pub fn dir(mut self, v: PathBuf) -> Self {
        self.dir = Some(v);
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Specify that progress indicators, if available, are desired.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "This flag is not compatible with the -s and -G options."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2014_2")),
        doc = "This option is not compatible with the -s and -G options."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "This option is not compatible with the -s and -G options.",
        doc = "",
        doc = "The progress indicator with the -I option is available for p4 -I move,",
        doc = "p4 -I reconcile, p4 -I status, p4 -I submit, and p4 -I sync -q. For",
        doc = "p4 -I reconcile and p4 -I status, the progress indicator shows four",
        doc = "phases: reconcile add, reconcile edit, matching digest, and matching",
        doc = "content. The add phase displays the number of directories scanned. All",
        doc = "other phases show a percentage of files processed against total of",
        doc = "candidate (deleted) files. For p4 -I move, the progress indicator shows",
        doc = "two phases: matching digest, and matching content."
    )]
    pub fn get_progress_indicators(&self) -> bool {
        self.progress_indicators
    }

    /// # Description
    ///
    /// -I
    ///
    /// Specify that progress indicators, if available, are desired.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "This flag is not compatible with the -s and -G options."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2014_2")),
        doc = "This option is not compatible with the -s and -G options."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "This option is not compatible with the -s and -G options.",
        doc = "",
        doc = "The progress indicator with the -I option is available for p4 -I move,",
        doc = "p4 -I reconcile, p4 -I status, p4 -I submit, and p4 -I sync -q. For",
        doc = "p4 -I reconcile and p4 -I status, the progress indicator shows four",
        doc = "phases: reconcile add, reconcile edit, matching digest, and matching",
        doc = "content. The add phase displays the number of directories scanned. All",
        doc = "other phases show a percentage of files processed against total of",
        doc = "candidate (deleted) files. For p4 -I move, the progress indicator shows",
        doc = "two phases: matching digest, and matching content."
    )]
    pub fn set_progress_indicators(&mut self, v: bool) -> &mut Self {
        self.progress_indicators = v;
        self
    }

    /// # Description
    ///
    /// -I
    ///
    /// Specify that progress indicators, if available, are desired.
    #[cfg_attr(
        feature = "lt2014_2",
        doc = "This flag is not compatible with the -s and -G options."
    )]
    #[cfg_attr(
        all(feature = "lt2025_1", not(feature = "lt2014_2")),
        doc = "This option is not compatible with the -s and -G options."
    )]
    #[cfg_attr(
        not(feature = "lt2025_1"),
        doc = "This option is not compatible with the -s and -G options.",
        doc = "",
        doc = "The progress indicator with the -I option is available for p4 -I move,",
        doc = "p4 -I reconcile, p4 -I status, p4 -I submit, and p4 -I sync -q. For",
        doc = "p4 -I reconcile and p4 -I status, the progress indicator shows four",
        doc = "phases: reconcile add, reconcile edit, matching digest, and matching",
        doc = "content. The add phase displays the number of directories scanned. All",
        doc = "other phases show a percentage of files processed against total of",
        doc = "candidate (deleted) files. For p4 -I move, the progress indicator shows",
        doc = "two phases: matching digest, and matching content."
    )]
    pub fn progress_indicators(mut self, v: bool) -> Self {
        self.progress_indicators = v;
        self
    }

    /// # Description
    ///
    /// -i
    ///
    /// Although not global, the -i and -o options work with forms and represent
    /// standard in and standard out.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn get_standard_in(&self) -> bool {
        self.standard_in
    }
    /// # Description
    ///
    /// -i
    ///
    /// Although not global, the -i and -o options work with forms and represent
    /// standard in and standard out.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn set_standard_in(&mut self, v: bool) -> &mut Self {
        self.standard_in = v;
        self
    }
    /// # Description
    ///
    /// -i
    ///
    /// Although not global, the -i and -o options work with forms and represent
    /// standard in and standard out.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn standard_in(mut self, v: bool) -> Self {
        self.standard_in = v;
        self
    }

    /// # Description
    ///
    /// -G
    ///
    /// Causes all output (and batch input for form commands with -i) to be
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "formatted as marshalled Python dictionary objects. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2016_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2020_1", not(feature = "lt2018_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting. See Usage Notes."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2020_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use the -G option with the -z tag option. Otherwise the marshaled",
        doc = "output might be invalid.",
        doc = "",
        doc = "See also the Usage Notes."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use the -G option with the -z tag option. Otherwise the marshaled",
        doc = "output might be invalid.",
        doc = "",
        doc = "See also the Usage notes."
    )]
    pub fn get_marshalled_formatted(&self) -> bool {
        self.marshalled_formatted
    }

    /// # Description
    ///
    /// -G
    ///
    /// Causes all output (and batch input for form commands with -i) to be
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "formatted as marshalled Python dictionary objects. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2016_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2020_1", not(feature = "lt2018_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting. See Usage Notes."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2020_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use the -G option with the -z tag option. Otherwise the marshaled",
        doc = "output might be invalid.",
        doc = "",
        doc = "See also the Usage Notes."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use the -G option with the -z tag option. Otherwise the marshaled",
        doc = "output might be invalid.",
        doc = "",
        doc = "See also the Usage notes."
    )]
    pub fn set_marshalled_formatted(&mut self, v: bool) -> &mut Self {
        self.marshalled_formatted = v;
        self
    }

    /// # Description
    ///
    /// -G
    ///
    /// Causes all output (and batch input for form commands with -i) to be
    #[cfg_attr(
        feature = "lt2016_1",
        doc = "formatted as marshalled Python dictionary objects. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2016_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2020_1", not(feature = "lt2018_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting. See Usage Notes."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2020_1")),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use the -G option with the -z tag option. Otherwise the marshaled",
        doc = "output might be invalid.",
        doc = "",
        doc = "See also the Usage Notes."
    )]
    #[cfg_attr(
        not(feature = "lt2024_1"),
        doc = "formatted as marshaled Python dictionary objects. This is most often",
        doc = "used when scripting.",
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use the -G option with the -z tag option. Otherwise the marshaled",
        doc = "output might be invalid.",
        doc = "",
        doc = "See also the Usage notes."
    )]
    pub fn marshalled_formatted(mut self, v: bool) -> Self {
        self.marshalled_formatted = v;
        self
    }

    /// # Description
    ///
    /// -Mj
    ///
    /// Formats output as line-delimited JSON objects, with non-UTF8 characters
    /// replaced with U+FFFD
    ///
    /// # Note
    ///
    /// Use the -Mj option with the -z tag option. Otherwise the marshaled
    /// output might be invalid.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn get_json_formatted(&self) -> bool {
        self.json_formatted
    }
    /// # Description
    ///
    /// -Mj
    ///
    /// Formats output as line-delimited JSON objects, with non-UTF8 characters
    /// replaced with U+FFFD
    ///
    /// # Note
    ///
    /// Use the -Mj option with the -z tag option. Otherwise the marshaled
    /// output might be invalid.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn set_json_formatted(&mut self, v: bool) -> &mut Self {
        self.json_formatted = v;
        self
    }
    /// # Description
    ///
    /// -Mj
    ///
    /// Formats output as line-delimited JSON objects, with non-UTF8 characters
    /// replaced with U+FFFD
    ///
    /// # Note
    ///
    /// Use the -Mj option with the -z tag option. Otherwise the marshaled
    /// output might be invalid.
    #[cfg(not(feature = "lt2020_1"))]
    pub fn json_formatted(mut self, v: bool) -> Self {
        self.json_formatted = v;
        self
    }

    /// # Description
    ///
    /// -H host
    ///
    /// Overrides any P4HOST setting and replaces it with the specified hostname.
    pub fn get_host(&self) -> Option<&String> {
        self.host.as_ref()
    }
    /// # Description
    ///
    /// -H host
    ///
    /// Overrides any P4HOST setting and replaces it with the specified hostname.
    pub fn set_host(&mut self, v: impl Into<String>) -> &mut Self {
        self.host = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -H host
    ///
    /// Overrides any P4HOST setting and replaces it with the specified hostname.
    pub fn host(mut self, v: impl Into<String>) -> Self {
        self.host = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -o
    ///
    /// Although not global, the -i and -o options work with forms and represent
    /// standard in and standard out.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn get_standard_out(&self) -> bool {
        self.standard_out
    }
    /// # Description
    ///
    /// -o
    ///
    /// Although not global, the -i and -o options work with forms and represent
    /// standard in and standard out.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn set_standard_out(&mut self, v: bool) -> &mut Self {
        self.standard_out = v;
        self
    }
    /// # Description
    ///
    /// -o
    ///
    /// Although not global, the -i and -o options work with forms and represent
    /// standard in and standard out.
    #[cfg(not(feature = "lt2019_1"))]
    pub fn standard_out(mut self, v: bool) -> Self {
        self.standard_out = v;
        self
    }

    /// # Description
    ///
    /// -p port
    ///
    /// Overrides any P4PORT setting with the specified protocol:host:port.
    pub fn get_port(&self) -> Option<&String> {
        self.port.as_ref()
    }
    /// # Description
    ///
    /// -p port
    ///
    /// Overrides any P4PORT setting with the specified protocol:host:port.
    pub fn set_port(&mut self, v: impl Into<String>) -> &mut Self {
        self.port = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -p port
    ///
    /// Overrides any P4PORT setting with the specified protocol:host:port.
    pub fn port(mut self, v: impl Into<String>) -> Self {
        self.port = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -P pass
    ///
    #[cfg_attr(
        feature = "lt2021_2",
        doc = "Overrides any P4PASSWD setting with the specified password."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Enables a password (or ticket) to be passed on the command line, thus",
        doc = "bypassing the password associated with P4PASSWD. If a valid ticket exists,",
        doc = "using this option with an invalid pass causes access to be granted by the",
        doc = "valid ticket."
    )]
    pub fn get_pass(&self) -> Option<&String> {
        self.pass.as_ref()
    }

    /// # Description
    ///
    /// -P pass
    ///
    #[cfg_attr(
        feature = "lt2021_2",
        doc = "Overrides any P4PASSWD setting with the specified password."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Enables a password (or ticket) to be passed on the command line, thus",
        doc = "bypassing the password associated with P4PASSWD. If a valid ticket exists,",
        doc = "using this option with an invalid pass causes access to be granted by the",
        doc = "valid ticket."
    )]
    pub fn set_pass(&mut self, v: impl Into<String>) -> &mut Self {
        self.pass = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -P pass
    ///
    #[cfg_attr(
        feature = "lt2021_2",
        doc = "Overrides any P4PASSWD setting with the specified password."
    )]
    #[cfg_attr(
        not(feature = "lt2021_2"),
        doc = "Enables a password (or ticket) to be passed on the command line, thus",
        doc = "bypassing the password associated with P4PASSWD. If a valid ticket exists,",
        doc = "using this option with an invalid pass causes access to be granted by the",
        doc = "valid ticket."
    )]
    pub fn pass(mut self, v: impl Into<String>) -> Self {
        self.pass = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -r retries
    ///
    /// Specifies the number of times to retry a command (notably, p4 sync) if
    /// the network times out.
    pub fn get_retries(&self) -> Option<&u32> {
        self.retries.as_ref()
    }
    /// # Description
    ///
    /// -r retries
    ///
    /// Specifies the number of times to retry a command (notably, p4 sync) if
    /// the network times out.
    pub fn set_retries(&mut self, v: u32) -> &mut Self {
        self.retries = Some(v);
        self
    }
    /// # Description
    ///
    /// -r retries
    ///
    /// Specifies the number of times to retry a command (notably, p4 sync) if
    /// the network times out.
    pub fn retries(mut self, v: u32) -> Self {
        self.retries = Some(v);
        self
    }

    /// # Description
    ///
    /// -s
    ///
    /// Prepends a descriptive field (for example, text:, info:, error:, exit:)
    /// to each line of output produced by a
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce command. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2017_2")),
        doc = "Helix Server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server command. This is",
        doc = "most often used when scripting."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server command. This is most",
        doc = "often used when scripting."
    )]
    pub fn get_prepend_descriptive(&self) -> bool {
        self.prepend_descriptive
    }

    /// # Description
    ///
    /// -s
    ///
    /// Prepends a descriptive field (for example, text:, info:, error:, exit:)
    /// to each line of output produced by a
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce command. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2017_2")),
        doc = "Helix Server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server command. This is",
        doc = "most often used when scripting."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server command. This is most",
        doc = "often used when scripting."
    )]
    pub fn set_prepend_descriptive(&mut self, v: bool) -> &mut Self {
        self.prepend_descriptive = v;
        self
    }

    /// # Description
    ///
    /// -s
    ///
    /// Prepends a descriptive field (for example, text:, info:, error:, exit:)
    /// to each line of output produced by a
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce command. This is most often",
        doc = "used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2017_2")),
        doc = "Helix Server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server command. This is most",
        doc = "often used when scripting."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server command. This is",
        doc = "most often used when scripting."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server command. This is most",
        doc = "often used when scripting."
    )]
    pub fn prepend_descriptive(mut self, v: bool) -> Self {
        self.prepend_descriptive = v;
        self
    }

    /// # Description
    ///
    /// -u user
    ///
    /// Overrides any P4USER, USER, or USERNAME setting with the specified user name.
    pub fn get_user(&self) -> Option<&String> {
        self.user.as_ref()
    }
    /// # Description
    ///
    /// -u user
    ///
    /// Overrides any P4USER, USER, or USERNAME setting with the specified user name.
    pub fn set_user(&mut self, v: impl Into<String>) -> &mut Self {
        self.user = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -u user
    ///
    /// Overrides any P4USER, USER, or USERNAME setting with the specified user name.
    pub fn user(mut self, v: impl Into<String>) -> Self {
        self.user = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -x argfile
    ///
    /// Instructs
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce to read arguments, one per line, from the specified",
        doc = "file. If file is a single hyphen (-), then standard input is read."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2017_2")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If file is a single hyphen (-), then standard input is",
        doc = "read."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix Core",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server to read arguments, one per line, from the specified",
        doc = "file. If argfile is a single hyphen (-), instructs P4 Server to read from",
        doc = "standard input instead of from a file."
    )]
    pub fn get_argfile(&self) -> Option<&PathBuf> {
        self.argfile.as_ref()
    }

    /// # Description
    ///
    /// -x argfile
    ///
    /// Instructs
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce to read arguments, one per line, from the specified",
        doc = "file. If file is a single hyphen (-), then standard input is read."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2017_2")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If file is a single hyphen (-), then standard input is",
        doc = "read."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix Core",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server to read arguments, one per line, from the specified",
        doc = "file. If argfile is a single hyphen (-), instructs P4 Server to read from",
        doc = "standard input instead of from a file."
    )]
    pub fn set_argfile(&mut self, v: PathBuf) -> &mut Self {
        self.argfile = Some(v);
        self
    }

    /// # Description
    ///
    /// -x argfile
    ///
    /// Instructs
    #[cfg_attr(
        feature = "lt2017_2",
        doc = "Perforce to read arguments, one per line, from the specified",
        doc = "file. If file is a single hyphen (-), then standard input is read."
    )]
    #[cfg_attr(
        all(feature = "lt2018_1", not(feature = "lt2017_2")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If file is a single hyphen (-), then standard input is",
        doc = "read."
    )]
    #[cfg_attr(
        all(feature = "lt2019_1", not(feature = "lt2018_1")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2021_1", not(feature = "lt2019_1")),
        doc = "Helix server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2024_1", not(feature = "lt2021_1")),
        doc = "Helix Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        all(feature = "lt2024_2", not(feature = "lt2024_1")),
        doc = "Helix Core Server to read arguments, one per line, from the",
        doc = "specified file. If argfile is a single hyphen (-), instructs Helix Core",
        doc = "Server to read from standard input instead of from a file."
    )]
    #[cfg_attr(
        not(feature = "lt2024_2"),
        doc = "P4 Server to read arguments, one per line, from the specified",
        doc = "file. If argfile is a single hyphen (-), instructs P4 Server to read from",
        doc = "standard input instead of from a file."
    )]
    pub fn argfile(mut self, v: PathBuf) -> Self {
        self.argfile = Some(v);
        self
    }

    /// # Description
    ///
    /// -C charset
    ///
    /// Overrides any P4CHARSET setting with the specified character set.
    pub fn get_charset(&self) -> Option<&String> {
        self.charset.as_ref()
    }
    /// # Description
    ///
    /// -C charset
    ///
    /// Overrides any P4CHARSET setting with the specified character set.
    pub fn set_charset(&mut self, v: impl Into<String>) -> &mut Self {
        self.charset = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -C charset
    ///
    /// Overrides any P4CHARSET setting with the specified character set.
    pub fn charset(mut self, v: impl Into<String>) -> Self {
        self.charset = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -Q charset
    ///
    /// Overrides any P4COMMANDCHARSET setting with the specified character set.
    pub fn get_command_charset(&self) -> Option<&String> {
        self.command_charset.as_ref()
    }
    /// # Description
    ///
    /// -Q charset
    ///
    /// Overrides any P4COMMANDCHARSET setting with the specified character set.
    pub fn set_command_charset(&mut self, v: impl Into<String>) -> &mut Self {
        self.command_charset = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -Q charset
    ///
    /// Overrides any P4COMMANDCHARSET setting with the specified character set.
    pub fn command_charset(mut self, v: impl Into<String>) -> Self {
        self.command_charset = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -L language
    ///
    /// This feature is reserved for system integrators.
    pub fn get_language(&self) -> Option<&String> {
        self.language.as_ref()
    }
    /// # Description
    ///
    /// -L language
    ///
    /// This feature is reserved for system integrators.
    pub fn set_language(&mut self, v: impl Into<String>) -> &mut Self {
        self.language = Some(v.into());
        self
    }
    /// # Description
    ///
    /// -L language
    ///
    /// This feature is reserved for system integrators.
    pub fn language(mut self, v: impl Into<String>) -> Self {
        self.language = Some(v.into());
        self
    }

    /// # Description
    ///
    /// -z tag
    ///
    /// Causes output of many reporting commands to be in the same tagged format
    /// as that generated by p4 fstat.
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2020_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp information."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2023_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp and Unix epoch time information."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp and Unix epoch information."
    )]
    pub fn get_tagged_format(&self) -> bool {
        self.tagged_format
    }

    /// # Description
    ///
    /// -z tag
    ///
    /// Causes output of many reporting commands to be in the same tagged format
    /// as that generated by p4 fstat.
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2020_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp information."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2023_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp and Unix epoch time information."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp and Unix epoch information."
    )]
    pub fn set_tagged_format(&mut self, v: bool) -> &mut Self {
        self.tagged_format = v;
        self
    }

    /// # Description
    ///
    /// -z tag
    ///
    /// Causes output of many reporting commands to be in the same tagged format
    /// as that generated by p4 fstat.
    #[cfg_attr(
        all(feature = "lt2022_1", not(feature = "lt2020_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid."
    )]
    #[cfg_attr(
        all(feature = "lt2023_1", not(feature = "lt2022_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp information."
    )]
    #[cfg_attr(
        all(feature = "lt2025_2", not(feature = "lt2023_1")),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp and Unix epoch time information."
    )]
    #[cfg_attr(
        not(feature = "lt2025_2"),
        doc = "",
        doc = "# Note",
        doc = "",
        doc = "Use this option for all marshaled output, such as that of -G and -Mj.",
        doc = "Otherwise the marshaled output might be invalid.",
        doc = "",
        doc = "See also the Timestamp and Unix epoch information."
    )]
    pub fn tagged_format(mut self, v: bool) -> Self {
        self.tagged_format = v;
        self
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress all informational message and report only warnings",
        doc = "or errors."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses informational messages and reports only",
        doc = "warnings or errors."
    )]
    pub fn get_quiet_mode(&self) -> bool {
        self.quiet_mode
    }

    /// # Description
    ///
    /// -q
    ///
    #[cfg_attr(
        feature = "lt2018_1",
        doc = "Quiet mode; suppress all informational message and report only warnings",
        doc = "or errors."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses informational messages and reports only",
        doc = "warnings or errors."
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
        feature = "lt2018_1",
        doc = "Quiet mode; suppress all informational message and report only warnings",
        doc = "or errors."
    )]
    #[cfg_attr(
        not(feature = "lt2018_1"),
        doc = "Quiet mode, which suppresses informational messages and reports only",
        doc = "warnings or errors."
    )]
    pub fn quiet_mode(mut self, v: bool) -> Self {
        self.quiet_mode = v;
        self
    }

    /// # Description
    ///
    /// -v
    ///
    /// Debug modes
    #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
    pub fn get_debug(&self) -> bool {
        self.debug
    }
    /// # Description
    ///
    /// -v var
    ///
    /// Debug level or configurable represented by var, such as: p4 -v 1 info or
    /// p4 -v net.autotune=0 info. This option can be specified more than once;
    /// each call appends another var.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn get_debug_variable(&self) -> Option<&[String]> {
        self.debug_variable.as_deref()
    }
    /// # Description
    ///
    /// -v
    ///
    /// Debug modes
    #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
    pub fn set_debug(&mut self, v: bool) -> &mut Self {
        self.debug = v;
        self
    }
    /// # Description
    ///
    /// -v var
    ///
    /// Debug level or configurable represented by var, such as: p4 -v 1 info or
    /// p4 -v net.autotune=0 info. This option can be specified more than once;
    /// each call appends another var.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn set_debug_variable(&mut self, v: impl Into<String>) -> &mut Self {
        self.debug_variable
            .get_or_insert_with(Vec::new)
            .push(v.into());
        self
    }
    /// # Description
    ///
    /// -v
    ///
    /// Debug modes
    #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
    pub fn debug(mut self, v: bool) -> Self {
        self.debug = v;
        self
    }
    /// # Description
    ///
    /// -v var
    ///
    /// Debug level or configurable represented by var, such as: p4 -v 1 info or
    /// p4 -v net.autotune=0 info. This option can be specified more than once;
    /// each call appends another var.
    #[cfg(not(feature = "lt2022_2"))]
    pub fn debug_variable(mut self, v: impl Into<String>) -> Self {
        self.debug_variable
            .get_or_insert_with(Vec::new)
            .push(v.into());
        self
    }

    /// # Description
    ///
    /// -V
    ///
    /// Displays the version of the p4 application and exits.
    pub fn get_display_version(&self) -> bool {
        self.display_version
    }
    /// # Description
    ///
    /// -V
    ///
    /// Displays the version of the p4 application and exits.
    pub fn set_display_version(&mut self, v: bool) -> &mut Self {
        self.display_version = v;
        self
    }
    /// # Description
    ///
    /// -V
    ///
    /// Displays the version of the p4 application and exits.
    pub fn display_version(mut self, v: bool) -> Self {
        self.display_version = v;
        self
    }

    /// # Description
    ///
    /// -h
    ///
    /// Displays basic usage information and exits.
    pub fn get_display_help(&self) -> bool {
        self.display_help
    }
    /// # Description
    ///
    /// -h
    ///
    /// Displays basic usage information and exits.
    pub fn set_display_help(&mut self, v: bool) -> &mut Self {
        self.display_help = v;
        self
    }
    /// # Description
    ///
    /// -h
    ///
    /// Displays basic usage information and exits.
    pub fn display_help(mut self, v: bool) -> Self {
        self.display_help = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::args_of;
    use std::path::PathBuf;
    use std::process::Command;

    /// Every global option is injected exactly once in the order documented in
    /// [`GlobalOpts::setup_args`].
    #[test]
    fn options_are_injected_in_documented_order() {
        let mut global_opts = GlobalOpts::new();
        global_opts
            .set_batch_size(256)
            .set_client("ws")
            .set_dir(PathBuf::from("C:\\tmp"))
            .set_progress_indicators(true)
            .set_marshalled_formatted(true)
            .set_host("helix.example")
            .set_port("localhost:1666")
            .set_pass("secret")
            .set_retries(1)
            .set_prepend_descriptive(true)
            .set_user("bruno")
            .set_argfile(PathBuf::from("args.txt"))
            .set_charset("utf8")
            .set_command_charset("utf8")
            .set_language("en")
            .set_tagged_format(true)
            .set_quiet_mode(true)
            .set_display_version(true)
            .set_display_help(true);
        #[cfg(not(feature = "lt2019_1"))]
        global_opts.set_standard_in(true).set_standard_out(true);
        #[cfg(not(feature = "lt2020_1"))]
        global_opts.set_json_formatted(true);
        #[cfg(not(feature = "lt2022_2"))]
        global_opts.set_debug_variable("net.autotune=0");
        #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
        global_opts.set_debug(true);

        let mut expected: Vec<&str> = vec!["-b", "256", "-c", "ws", "-d", "C:\\tmp", "-I"];
        #[cfg(not(feature = "lt2019_1"))]
        expected.push("-i");
        expected.push("-G");
        #[cfg(not(feature = "lt2020_1"))]
        expected.push("-Mj");
        expected.extend(["-H", "helix.example"]);
        #[cfg(not(feature = "lt2019_1"))]
        expected.push("-o");
        expected.extend([
            "-p",
            "localhost:1666",
            "-P",
            "secret",
            "-r",
            "1",
            "-s",
            "-u",
            "bruno",
            "-x",
            "args.txt",
            "-C",
            "utf8",
            "-Q",
            "utf8",
            "-L",
            "en",
            "-z",
            "tag",
            "-q",
        ]);
        #[cfg(not(feature = "lt2022_2"))]
        expected.extend(["-v", "net.autotune=0"]);
        #[cfg(all(feature = "lt2022_2", not(feature = "lt2021_2")))]
        expected.push("-v");
        expected.extend(["-V", "-h"]);

        let mut command = Command::new("p4");
        global_opts.setup_args(&mut command);

        assert_eq!(args_of(&command), expected);
    }
}
