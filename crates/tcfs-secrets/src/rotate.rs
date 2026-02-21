//! Credential rotation: atomic file replacement after re-encryption

use anyhow::Result;
use std::path::Path;

/// Atomically replace a SOPS-encrypted file with updated content.
/// Writes to a temp file then renames to ensure no partial reads.
pub async fn atomic_replace(path: &Path, new_content: &str) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let tmp_path = parent.join(format!(".{}.tmp", path.file_name().unwrap_or_default().to_string_lossy()));

    tokio::fs::write(&tmp_path, new_content.as_bytes()).await?;
    tokio::fs::rename(&tmp_path, path).await?;

    tracing::info!("credential file rotated: {}", path.display());
    Ok(())
}
