use super::DiffOptions;

pub struct WorkspaceFileMode {
    force: bool,

    limit: Option<u64>,

    differing_only: bool,

    display_opts: Option<String>,

    diff_nontext: bool,
}

pub struct StreamSpecMode {
    arbitrary_compare: bool,
}

pub struct Diff {
    bin: PathBuf,

    global_opts: GlobalOpts,

    diff_opts: Option<DiffOptions>,
}
