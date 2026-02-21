//! tcfs: TummyCrypt filesystem CLI
//!
//! Phase 1 commands:
//!   status              - show daemon status (connects via gRPC Unix socket)
//!   config show         - display current configuration
//!   kdbx resolve <path> - resolve a credential from a KDBX database
//!
//! Phase 2 commands:
//!   push <local> [<prefix>]      - upload file or directory tree to SeaweedFS
//!   pull <manifest> [<local>]    - download file from manifest path
//!   sync-status [<path>]         - show local sync state for a file/dir

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use tcfs_core::proto::{
    tcfs_daemon_client::TcfsDaemonClient,
    Empty, StatusRequest,
};

// ── CLI structure ──────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "tcfs",
    version,
    about = "TummyCrypt filesystem client",
    long_about = "tcfs: manage TummyCrypt FUSE mounts, credentials, and sync operations"
)]
struct Cli {
    /// Path to tcfs.toml configuration file
    #[arg(long, short = 'c', env = "TCFS_CONFIG", default_value = "/etc/tcfs/config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show daemon and storage status
    Status,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// KDBX credential management (RemoteJuggler bridge)
    Kdbx {
        #[command(subcommand)]
        action: KdbxAction,
    },

    // ── Phase 2 commands ───────────────────────────────────────────────────────

    /// Upload a local file or directory tree to SeaweedFS
    ///
    /// Credentials are read from AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY
    /// environment variables (or set in the config credentials_file via SOPS).
    Push {
        /// Local path (file or directory)
        local: PathBuf,
        /// Remote prefix in the bucket (default: derived from local path name)
        #[arg(long, short = 'p')]
        prefix: Option<String>,
        /// Path to the sync state cache JSON file (overrides config)
        #[arg(long, env = "TCFS_STATE_PATH")]
        state: Option<PathBuf>,
    },

    /// Download a file from SeaweedFS using a manifest path
    ///
    /// The manifest path is in format: {prefix}/manifests/{hash}
    Pull {
        /// Remote manifest path (e.g. mydata/manifests/abc123...)
        manifest: String,
        /// Local destination path (default: current dir + hash basename)
        local: Option<PathBuf>,
        /// Remote prefix to look up chunks (default: derived from manifest path)
        #[arg(long, short = 'p')]
        prefix: Option<String>,
        /// Path to the sync state cache JSON file (overrides config)
        #[arg(long, env = "TCFS_STATE_PATH")]
        state: Option<PathBuf>,
    },

    /// Show local sync state for a file or directory
    #[command(name = "sync-status")]
    SyncStatus {
        /// Path to check (default: current directory)
        path: Option<PathBuf>,
        /// Path to the sync state cache JSON file (overrides config)
        #[arg(long, env = "TCFS_STATE_PATH")]
        state: Option<PathBuf>,
    },

    // ── Phase 3: FUSE mount + stub management ────────────────────────────────

    /// Mount a remote as a local directory (requires FUSE)
    #[cfg(feature = "fuse")]
    Mount {
        /// Remote spec (e.g. seaweedfs://host/bucket[/prefix])
        remote: String,
        /// Local mountpoint
        mountpoint: PathBuf,
        /// Mount read-only
        #[arg(long)]
        read_only: bool,
    },

    /// Unmount a tcfs mountpoint (requires FUSE)
    #[cfg(feature = "fuse")]
    Unmount {
        /// Local mountpoint to unmount
        mountpoint: PathBuf,
    },

    /// Convert hydrated file back to .tc stub, reclaiming disk space
    Unsync {
        /// Path to unsync
        path: PathBuf,
        /// Force unsync even if local changes exist
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Print the active configuration (merged defaults + config file)
    Show,
}

#[derive(Subcommand, Debug)]
enum KdbxAction {
    /// Resolve a credential entry by group/title path
    Resolve {
        /// Path in format group/subgroup/entry-title
        /// Example: tummycrypt/tcfs/seaweedfs/admin/access-key
        query: String,

        /// KDBX database file (overrides config kdbx_path)
        #[arg(long, env = "TCFS_KDBX_PATH")]
        kdbx_path: Option<PathBuf>,

        /// Master password for the KDBX database
        #[arg(long, env = "TCFS_KDBX_PASSWORD")]
        password: String,
    },

    /// Import credentials from KDBX into SOPS-encrypted credential files (Phase 5)
    Import {
        #[arg(long, env = "TCFS_KDBX_PATH")]
        kdbx_path: Option<PathBuf>,
        #[arg(long, env = "TCFS_KDBX_PASSWORD")]
        password: String,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_config(&cli.config).await?;

    match cli.command {
        Commands::Status => cmd_status(&config).await,
        Commands::Config { action: ConfigAction::Show } => cmd_config_show(&config, &cli.config),
        Commands::Kdbx { action: KdbxAction::Resolve { query, kdbx_path, password } } => {
            cmd_kdbx_resolve(&config, &query, kdbx_path.as_deref(), &password)
        }
        Commands::Kdbx { action: KdbxAction::Import { .. } } => {
            anyhow::bail!("kdbx import: not yet implemented (Phase 5)")
        }
        Commands::Push { local, prefix, state } => {
            cmd_push(&config, &local, prefix.as_deref(), state.as_deref()).await
        }
        Commands::Pull { manifest, local, prefix, state } => {
            cmd_pull(&config, &manifest, local.as_deref(), prefix.as_deref(), state.as_deref()).await
        }
        Commands::SyncStatus { path, state } => {
            cmd_sync_status(&config, path.as_deref(), state.as_deref())
        }
        #[cfg(feature = "fuse")]
        Commands::Mount { remote, mountpoint, read_only } => {
            cmd_mount(&config, &remote, &mountpoint, read_only).await
        }
        #[cfg(feature = "fuse")]
        Commands::Unmount { mountpoint } => cmd_unmount(&mountpoint),
        Commands::Unsync { path, force } => cmd_unsync(&config, &path, force).await,
    }
}

// ── Config loading ────────────────────────────────────────────────────────────

async fn load_config(path: &Path) -> Result<tcfs_core::config::TcfsConfig> {
    if path.exists() {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("reading config: {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("parsing config: {}", path.display()))
    } else {
        Ok(tcfs_core::config::TcfsConfig::default())
    }
}

// ── Storage operator from environment credentials ─────────────────────────────

/// Build an OpenDAL operator using credentials from environment variables.
///
/// Reads AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY (standard S3 env vars).
/// These override any config file credentials for direct CLI use.
fn build_operator_from_env(
    config: &tcfs_core::config::TcfsConfig,
) -> Result<opendal::Operator> {
    let access_key = std::env::var("AWS_ACCESS_KEY_ID")
        .or_else(|_| std::env::var("TCFS_ACCESS_KEY_ID"))
        .context(
            "S3 credentials not set\n\
             Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY environment variables.\n\
             Example:\n\
             \texport AWS_ACCESS_KEY_ID=your-key\n\
             \texport AWS_SECRET_ACCESS_KEY=your-secret"
        )?;
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
        .or_else(|_| std::env::var("TCFS_SECRET_ACCESS_KEY"))
        .context("AWS_SECRET_ACCESS_KEY environment variable not set")?;

    tcfs_storage::operator::build_from_core_config(&config.storage, &access_key, &secret_key)
        .context("building storage operator")
}

/// Expand `~` in path to the user's home directory
fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(format!("{}/{}", home, &s[2..]))
    } else {
        path.to_path_buf()
    }
}

/// Resolve the state cache path: CLI flag > config > default user data dir
fn resolve_state_path(config: &tcfs_core::config::TcfsConfig, override_path: Option<&Path>) -> PathBuf {
    if let Some(p) = override_path {
        return p.to_path_buf();
    }
    // Config uses state_db (designed for RocksDB in Phase 4); for JSON Phase 2
    // we derive a sibling .json file
    let db = expand_tilde(&config.sync.state_db);
    db.with_extension("json")
}

// ── Progress bar helpers ──────────────────────────────────────────────────────

fn make_progress_bar(total: u64, prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{prefix:.bold} [{bar:40.cyan/blue}] {pos}/{len} {msg}"
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb.set_prefix(prefix.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

fn make_spinner(prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{prefix:.bold} {spinner} {msg}")
            .unwrap()
    );
    pb.set_prefix(prefix.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

// ── `tcfs push` ───────────────────────────────────────────────────────────────

async fn cmd_push(
    config: &tcfs_core::config::TcfsConfig,
    local: &Path,
    prefix: Option<&str>,
    state_override: Option<&Path>,
) -> Result<()> {
    let op = build_operator_from_env(config)?;
    let state_path = resolve_state_path(config, state_override);
    let mut state = tcfs_sync::state::StateCache::open(&state_path)
        .with_context(|| format!("opening state cache: {}", state_path.display()))?;

    // Default prefix: local directory/file name
    let remote_prefix = prefix
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| {
            local
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "tcfs".to_string())
        });

    println!(
        "Pushing {} → {}:{} (endpoint: {})",
        local.display(),
        config.storage.bucket,
        remote_prefix,
        config.storage.endpoint,
    );

    if local.is_file() {
        // Single-file push
        let pb = make_progress_bar(0, "push");
        pb.set_message(format!("{}", local.display()));

        let pb_clone = pb.clone();
        let progress: tcfs_sync::engine::ProgressFn = Box::new(move |done, total, msg| {
            pb_clone.set_length(total);
            pb_clone.set_position(done);
            pb_clone.set_message(msg.to_string());
        });

        let result = tcfs_sync::engine::upload_file(
            &op, local, &remote_prefix, &mut state, Some(&progress),
        )
        .await
        .with_context(|| format!("uploading {}", local.display()))?;

        state.flush().context("flushing state cache")?;

        if result.skipped {
            pb.finish_with_message(format!("{} (unchanged)", local.file_name().unwrap_or_default().to_string_lossy()));
            println!("  skipped (unchanged since last sync)");
        } else {
            pb.finish_with_message("done".to_string());
            println!(
                "  hash:    {}",
                &result.hash[..16.min(result.hash.len())]
            );
            println!("  chunks:  {}", result.chunks);
            println!("  bytes:   {}", fmt_bytes(result.bytes));
            println!("  remote:  {}", result.remote_path);
        }
    } else if local.is_dir() {
        // Directory tree push
        let pb = make_spinner("push");
        pb.set_message("scanning files...");

        let pb_clone = pb.clone();
        let progress: tcfs_sync::engine::ProgressFn = Box::new(move |done, total, msg| {
            if total > 0 {
                pb_clone.set_style(
                    ProgressStyle::with_template(
                        "{prefix:.bold} [{bar:40.cyan/blue}] {pos}/{len} {msg}"
                    )
                    .unwrap()
                    .progress_chars("=>-"),
                );
                pb_clone.set_length(total);
            }
            pb_clone.set_position(done);
            pb_clone.set_message(msg.to_string());
        });

        let (uploaded, skipped, bytes) = tcfs_sync::engine::push_tree(
            &op, local, &remote_prefix, &mut state, Some(&progress),
        )
        .await
        .with_context(|| format!("pushing tree: {}", local.display()))?;

        pb.finish_with_message("done".to_string());
        println!();
        println!("Push complete:");
        println!("  uploaded: {} files ({})", uploaded, fmt_bytes(bytes));
        println!("  skipped:  {} files (unchanged)", skipped);
        println!("  total:    {} files", uploaded + skipped);
    } else {
        anyhow::bail!("path not found or not a file/directory: {}", local.display());
    }

    Ok(())
}

// ── `tcfs pull` ───────────────────────────────────────────────────────────────

async fn cmd_pull(
    config: &tcfs_core::config::TcfsConfig,
    manifest_path: &str,
    local: Option<&Path>,
    prefix: Option<&str>,
    _state_override: Option<&Path>,
) -> Result<()> {
    let op = build_operator_from_env(config)?;

    // Derive the remote prefix from the manifest path if not provided
    // e.g. "mydata/manifests/abc123" → prefix = "mydata"
    let remote_prefix = prefix
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|| {
            manifest_path
                .split('/')
                .next()
                .unwrap_or("tcfs")
                .to_string()
        });

    // Default local path: current dir + manifest hash (last path component)
    let hash_basename = manifest_path
        .split('/')
        .next_back()
        .unwrap_or("downloaded");
    let local_path = local
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(hash_basename));

    println!(
        "Pulling {} → {}",
        manifest_path,
        local_path.display(),
    );

    let pb = make_progress_bar(0, "pull");
    pb.set_message("fetching manifest...".to_string());

    let pb_clone = pb.clone();
    let progress: tcfs_sync::engine::ProgressFn = Box::new(move |done, total, msg| {
        pb_clone.set_length(total);
        pb_clone.set_position(done);
        pb_clone.set_message(msg.to_string());
    });

    let result = tcfs_sync::engine::download_file(
        &op, manifest_path, &local_path, &remote_prefix, Some(&progress),
    )
    .await
    .with_context(|| format!("downloading {}", manifest_path))?;

    pb.finish_with_message("done".to_string());
    println!();
    println!("Downloaded:");
    println!("  local:  {}", result.local_path.display());
    println!("  bytes:  {}", fmt_bytes(result.bytes));

    Ok(())
}

// ── `tcfs sync-status` ────────────────────────────────────────────────────────

fn cmd_sync_status(
    config: &tcfs_core::config::TcfsConfig,
    path: Option<&Path>,
    state_override: Option<&Path>,
) -> Result<()> {
    let state_path = resolve_state_path(config, state_override);
    let state = tcfs_sync::state::StateCache::open(&state_path)
        .with_context(|| format!("opening state cache: {}", state_path.display()))?;

    println!("State cache: {}", state_path.display());
    println!("Tracked files: {}", state.len());

    if let Some(p) = path {
        let canonical = std::fs::canonicalize(p)
            .with_context(|| format!("resolving path: {}", p.display()))?;

        match state.get(&canonical) {
            Some(entry) => {
                println!();
                println!("File: {}", canonical.display());
                println!("  hash:       {}", &entry.blake3[..16.min(entry.blake3.len())]);
                println!("  size:       {}", fmt_bytes(entry.size));
                println!("  chunks:     {}", entry.chunk_count);
                println!("  remote:     {}", entry.remote_path);
                println!("  last sync:  {} seconds ago", {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    now.saturating_sub(entry.last_synced)
                });

                // Check if it needs re-sync
                match state.needs_sync(&canonical)? {
                    None => println!("  status:     up to date"),
                    Some(reason) => println!("  status:     needs sync ({reason})"),
                }
            }
            None => {
                println!();
                println!("File: {} — not in sync state (never pushed)", canonical.display());
            }
        }
    }

    Ok(())
}

// ── `tcfs status` ─────────────────────────────────────────────────────────────

async fn cmd_status(config: &tcfs_core::config::TcfsConfig) -> Result<()> {
    let socket = &config.daemon.socket;

    if !socket.exists() {
        eprintln!("tcfsd: socket not found at {}", socket.display());
        eprintln!("       Is tcfsd running?  Try: tcfsd --config /etc/tcfs/config.toml");
        std::process::exit(1);
    }

    let mut client = connect_daemon(socket).await?;

    // Daemon status
    let status = client
        .status(tonic::Request::new(StatusRequest {}))
        .await
        .context("status RPC failed")?
        .into_inner();

    // Credential status
    let creds = client
        .credential_status(tonic::Request::new(Empty {}))
        .await
        .context("credential_status RPC failed")?
        .into_inner();

    let uptime = format_uptime(status.uptime_secs);

    println!("tcfsd v{}", status.version);
    println!("  uptime:        {uptime}");
    println!("  socket:        {}", socket.display());
    println!("  storage:       {} [{}]",
        status.storage_endpoint,
        if status.storage_ok { "ok" } else { "UNREACHABLE" }
    );
    println!("  nats:          {}", if status.nats_ok { "ok" } else { "not connected (Phase 2)" });
    println!("  active mounts: {}", status.active_mounts);
    println!("  credentials:   {} (source: {})",
        if creds.loaded { "loaded" } else { "NOT LOADED" },
        creds.source
    );
    if creds.needs_reload {
        println!("  WARNING: credentials need reload");
    }

    Ok(())
}

// ── gRPC connection ───────────────────────────────────────────────────────────

async fn connect_daemon(socket_path: &Path) -> Result<TcfsDaemonClient<Channel>> {
    let path = socket_path.to_path_buf();

    // tonic over Unix domain socket: use a tower service_fn connector
    let channel = Endpoint::from_static("http://[::]:0")
        .connect_with_connector(service_fn(move |_: Uri| {
            let path = path.clone();
            async move {
                let stream = tokio::net::UnixStream::connect(&path).await?;
                Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(stream))
            }
        }))
        .await
        .with_context(|| format!("connecting to tcfsd at {}", socket_path.display()))?;

    Ok(TcfsDaemonClient::new(channel))
}

// ── `tcfs config show` ────────────────────────────────────────────────────────

fn cmd_config_show(config: &tcfs_core::config::TcfsConfig, config_path: &Path) -> Result<()> {
    if config_path.exists() {
        println!("# Configuration from: {}", config_path.display());
    } else {
        println!("# Configuration: defaults (no file at {})", config_path.display());
    }
    println!();
    let rendered = toml::to_string_pretty(config)
        .context("serializing config to TOML")?;
    print!("{rendered}");
    Ok(())
}

// ── `tcfs kdbx resolve` ───────────────────────────────────────────────────────

fn cmd_kdbx_resolve(
    config: &tcfs_core::config::TcfsConfig,
    query: &str,
    kdbx_path_override: Option<&Path>,
    password: &str,
) -> Result<()> {
    // Resolve the KDBX path: CLI flag > config > error
    let kdbx_path = kdbx_path_override
        .map(|p| p.to_path_buf())
        .or_else(|| config.secrets.kdbx_path.clone())
        .with_context(|| {
            "no KDBX path provided; use --kdbx-path or set secrets.kdbx_path in config"
        })?;

    if !kdbx_path.exists() {
        anyhow::bail!("KDBX file not found: {}", kdbx_path.display());
    }

    let store = tcfs_secrets::KdbxStore::open(&kdbx_path);
    let cred = store
        .resolve(query, password)
        .with_context(|| format!("resolving '{query}' in {}", kdbx_path.display()))?;

    println!("title:    {}", cred.title);
    if let Some(ref u) = cred.username {
        println!("username: {u}");
    }
    println!("password: {}", cred.password);
    if let Some(ref url) = cred.url {
        println!("url:      {url}");
    }

    Ok(())
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn format_uptime(secs: i64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

// ── `tcfs mount` (requires fuse feature) ────────────────────────────────────

#[cfg(feature = "fuse")]
/// Parse a remote spec like `seaweedfs://host:port/bucket[/prefix]`
fn parse_remote_spec(spec: &str) -> anyhow::Result<(String, String, String)> {
    let rest = spec
        .strip_prefix("seaweedfs://")
        .with_context(|| format!("remote spec must start with seaweedfs:// — got: {}", spec))?;

    // Split host:port from /bucket[/prefix]
    let slash = rest
        .find('/')
        .with_context(|| format!("remote spec must include /bucket — got: {}", spec))?;

    let host = &rest[..slash];   // e.g. "dees-appu-bearts:8333"
    let path = &rest[slash + 1..]; // e.g. "tcfs-test" or "tcfs-test/subdir"

    // First path component = bucket, remainder = prefix
    let (bucket, prefix) = path.split_once('/').unwrap_or((path, ""));

    Ok((
        format!("http://{}", host),
        bucket.to_string(),
        prefix.trim_end_matches('/').to_string(),
    ))
}

#[cfg(feature = "fuse")]
async fn cmd_mount(
    config: &tcfs_core::config::TcfsConfig,
    remote: &str,
    mountpoint: &std::path::Path,
    read_only: bool,
) -> Result<()> {
    let (endpoint, bucket, prefix) = parse_remote_spec(remote)?;

    // Credentials
    let access_key = std::env::var("AWS_ACCESS_KEY_ID")
        .or_else(|_| std::env::var("TCFS_ACCESS_KEY_ID"))
        .context(
            "S3 credentials not set — export AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY"
        )?;
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
        .or_else(|_| std::env::var("TCFS_SECRET_ACCESS_KEY"))
        .context("AWS_SECRET_ACCESS_KEY not set")?;

    let storage_cfg = tcfs_storage::operator::StorageConfig {
        endpoint: endpoint.clone(),
        region: config.storage.region.clone(),
        bucket: bucket.clone(),
        access_key_id: access_key,
        secret_access_key: secret_key,
    };
    let op = tcfs_storage::build_operator(&storage_cfg)
        .context("building storage operator")?;

    // Ensure mountpoint exists
    tokio::fs::create_dir_all(mountpoint)
        .await
        .with_context(|| format!("creating mountpoint: {}", mountpoint.display()))?;

    let cache_dir = expand_tilde(&config.fuse.cache_dir);
    let neg_ttl = config.fuse.negative_cache_ttl_secs;
    let cache_max = config.fuse.cache_max_mb * 1024 * 1024;

    println!(
        "Mounting {} (prefix: {}) → {}",
        format!("{}:{}", endpoint, bucket),
        if prefix.is_empty() { "(root)" } else { &prefix },
        mountpoint.display()
    );
    println!("Press Ctrl-C or run `tcfs unmount {}` to stop.", mountpoint.display());

    tcfs_fuse::mount(tcfs_fuse::MountConfig {
        op,
        prefix,
        mountpoint: mountpoint.to_path_buf(),
        cache_dir,
        cache_max_bytes: cache_max,
        negative_ttl_secs: neg_ttl,
        read_only,
        allow_other: false,
    })
    .await
    .context("FUSE mount failed")
}

// ── `tcfs unmount` (requires fuse feature) ──────────────────────────────────

#[cfg(feature = "fuse")]
fn cmd_unmount(mountpoint: &std::path::Path) -> Result<()> {
    // macOS: use umount directly (works with FUSE-T and macFUSE)
    // Linux: use fusermount3 first, fall back to umount
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("umount")
            .arg(mountpoint)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("Unmounted: {}", mountpoint.display());
                return Ok(());
            }
            Ok(s) => anyhow::bail!(
                "umount exited {}: try `diskutil unmount {}`",
                s,
                mountpoint.display()
            ),
            Err(e) => anyhow::bail!("failed to run umount: {e}"),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let status = std::process::Command::new("fusermount3")
            .args(["-u", &mountpoint.to_string_lossy()])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("Unmounted: {}", mountpoint.display());
                Ok(())
            }
            Ok(s) => {
                // Fallback: try plain umount (works as root or with FUSE-T)
                let fallback = std::process::Command::new("umount")
                    .arg(mountpoint)
                    .status();
                match fallback {
                    Ok(f) if f.success() => {
                        println!("Unmounted: {}", mountpoint.display());
                        Ok(())
                    }
                    _ => anyhow::bail!(
                        "fusermount3 exited {}: use `fusermount3 -u {}` or `umount {}` manually",
                        s,
                        mountpoint.display(),
                        mountpoint.display()
                    ),
                }
            }
            Err(e) => anyhow::bail!("failed to run fusermount3: {e}"),
        }
    }
}

// ── `tcfs unsync` ─────────────────────────────────────────────────────────────

/// Convert a hydrated file back to a `.tc` stub, reclaiming disk space.
///
/// Reads the file, computes its BLAKE3 hash, looks up the matching index entry,
/// and replaces the file content with a stub. The actual remote data is NOT deleted.
async fn cmd_unsync(
    config: &tcfs_core::config::TcfsConfig,
    path: &std::path::Path,
    force: bool,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("path not found: {}", path.display());
    }
    if tcfs_fuse::is_stub_path(path) {
        println!("{} is already a stub — nothing to do.", path.display());
        return Ok(());
    }

    // Read file content and compute hash
    let data = tokio::fs::read(path)
        .await
        .with_context(|| format!("reading: {}", path.display()))?;

    let hash = tcfs_chunks::hash_bytes(&data);
    let hash_hex = tcfs_chunks::hash_to_hex(&hash);
    let size = data.len() as u64;

    if !force {
        let state_path = resolve_state_path(config, None);
        let state = tcfs_sync::state::StateCache::open(&state_path)
            .with_context(|| format!("opening state cache: {}", state_path.display()))?;

        match state.get(path) {
            None => anyhow::bail!(
                "{} is not tracked (never pushed). Use --force to unsync anyway.",
                path.display()
            ),
            Some(entry) if entry.blake3 != hash_hex => anyhow::bail!(
                "{} has local changes (hash mismatch). Use --force to unsync anyway.",
                path.display()
            ),
            _ => {}
        }
    }

    // Build stub at path.tc
    let stub_path = tcfs_fuse::real_to_stub_name(
        path.file_name().context("path has no filename")?
    );
    let stub_full = path.parent().unwrap_or(std::path::Path::new(".")).join(stub_path);

    let stub = tcfs_fuse::StubMeta {
        chunks: 0,  // unknown without state — leave as 0
        compressed: false,
        fetched: false,
        oid: format!("blake3:{}", hash_hex),
        origin: format!("seaweedfs://{}/{}", config.storage.endpoint, hash_hex),
        size,
    };

    // Write stub then remove original
    tokio::fs::write(&stub_full, stub.to_bytes())
        .await
        .with_context(|| format!("writing stub: {}", stub_full.display()))?;
    tokio::fs::remove_file(path)
        .await
        .with_context(|| format!("removing hydrated file: {}", path.display()))?;

    println!("Unsynced: {} → {}", path.display(), stub_full.display());
    println!("  hash: {}", &hash_hex[..16]);
    println!("  size: {} freed", fmt_bytes(size));

    Ok(())
}

fn fmt_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
