pub struct Print {
    bin: PathBuf,

    global_opts: GlobalOpts,

    all_revisions: bool,

    archive_depots: bool,

    skip_rcs_keyword_expansion: bool,

    line_ending: Option<LineEnding>,

    charset: Option<String>,

    utf8bom: Option<String>,

    limit: Option<u64>,

    offset: Option<u64>,

    size: Option<u64>,

    localfile: Option<PathBuf>,

    skip_file_header: bool,

    unload_depot: bool,

    attribute: Option<String>,

    ignore_change_view: bool,
}
