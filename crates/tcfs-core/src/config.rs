use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level daemon configuration (loaded from tcfs.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TcfsConfig {
    pub daemon: DaemonConfig,
    pub storage: StorageConfig,
    pub secrets: SecretsConfig,
    pub sync: SyncConfig,
    pub fuse: FuseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    /// Unix socket path for gRPC (default: /run/tcfsd/tcfsd.sock)
    pub socket: PathBuf,
    /// TCP listen address for remote gRPC (optional)
    pub listen: Option<String>,
    /// Prometheus metrics endpoint (default: 127.0.0.1:9100)
    pub metrics_addr: Option<String>,
    /// Log level (default: info)
    pub log_level: String,
    /// Log format: "json" or "text"
    pub log_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// SeaweedFS S3 endpoint
    pub endpoint: String,
    /// S3 region (default: us-east-1)
    pub region: String,
    /// Default bucket name
    pub bucket: String,
    /// SOPS credential file path
    pub credentials_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecretsConfig {
    /// Age identity file (default: ~/.config/sops/age/keys.txt)
    pub age_identity: Option<PathBuf>,
    /// KDBX database file path
    pub kdbx_path: Option<PathBuf>,
    /// SOPS credentials directory
    pub sops_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SyncConfig {
    /// NATS JetStream endpoint
    pub nats_url: String,
    /// RocksDB state cache path
    pub state_db: PathBuf,
    /// Worker thread count (0 = cpu_count)
    pub workers: usize,
    /// Retry limit for failed tasks
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FuseConfig {
    /// Negative dentry cache TTL in seconds (default: 30)
    pub negative_cache_ttl_secs: u64,
    /// Disk cache directory for partial downloads
    pub cache_dir: PathBuf,
    /// Maximum disk cache size in MB
    pub cache_max_mb: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket: PathBuf::from("/run/tcfsd/tcfsd.sock"),
            listen: None,
            metrics_addr: Some("127.0.0.1:9100".into()),
            log_level: "info".into(),
            log_format: "json".into(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8333".into(),
            region: "us-east-1".into(),
            bucket: "tcfs".into(),
            credentials_file: None,
        }
    }
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            age_identity: None,
            kdbx_path: None,
            sops_dir: None,
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            nats_url: "nats://localhost:4222".into(),
            state_db: PathBuf::from("~/.local/share/tcfsd/state.db"),
            workers: 0,
            max_retries: 3,
        }
    }
}

impl Default for FuseConfig {
    fn default() -> Self {
        Self {
            negative_cache_ttl_secs: 30,
            cache_dir: PathBuf::from("~/.cache/tcfs"),
            cache_max_mb: 10240,
        }
    }
}

impl Default for TcfsConfig {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig::default(),
            storage: StorageConfig::default(),
            secrets: SecretsConfig::default(),
            sync: SyncConfig::default(),
            fuse: FuseConfig::default(),
        }
    }
}
