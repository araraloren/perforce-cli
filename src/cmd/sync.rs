pub struct Sync {
    bin: PathBuf,

    global_opts: GlobalOpts,

    sync_mode: Option<SyncMode>,

    scrpit_list_mode: bool,

    preview: Option<PreviewMode>,

    suppress_keyword_expansion: bool,

    quiet_mode: bool,

    limit: Option<u64>,

    verify_edge_replication: bool,

    parallel: Option<ParallelConfig>,

    stream_spec_version: Option<StreamSpecVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncMode {
    Standard {
        force: bool,

        metadata_only_flush: bool,

        reopen_moved_files: bool,
    },

    Safe {
        safe_sync: bool,
    },

    Populate {
        populate_only: bool,
    },

    HistoricalSnapshot {
        sync_time: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ParallelConfig {
    pub threads: u64,

    pub batch_files: Option<u64>,

    pub batch_size_bytes: Option<u64>,

    pub min_files: Option<u64>,

    pub min_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum StreamSpecVersion {
    AutoFromMaxFilelists,
    CurrentVersion,
    SpecificChangelist(u32),
}
