//! OpenDAL Operator factory for tcfs storage backends

use anyhow::{Context, Result};
use opendal::Operator;

/// Minimal config needed to build an operator
/// (full config lives in tcfs-core's StorageConfig)
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Build an OpenDAL Operator for SeaweedFS S3 (or any S3-compatible endpoint)
///
/// Uses path-style addressing (default in opendal 0.55), which is required by
/// SeaweedFS and MinIO. Do NOT call enable_virtual_host_style() for these.
pub fn build_operator(cfg: &StorageConfig) -> Result<Operator> {
    // opendal 0.55: S3 builder uses consuming pattern (methods take `self`, return `Self`)
    let builder = opendal::services::S3::default()
        .endpoint(&cfg.endpoint)
        .region(&cfg.region)
        .bucket(&cfg.bucket)
        .access_key_id(&cfg.access_key_id)
        .secret_access_key(&cfg.secret_access_key);
    // Note: path-style addressing is the default — no enable_virtual_host_style() needed

    let op = Operator::new(builder)
        .context("creating OpenDAL S3 operator")?
        .layer(opendal::layers::RetryLayer::new().with_max_times(3))
        .finish();

    Ok(op)
}

/// Build an operator from tcfs-core config + loaded credentials
pub fn build_from_core_config(
    storage: &tcfs_core::config::StorageConfig,
    access_key_id: &str,
    secret_access_key: &str,
) -> Result<Operator> {
    build_operator(&StorageConfig {
        endpoint: storage.endpoint.clone(),
        region: storage.region.clone(),
        bucket: storage.bucket.clone(),
        access_key_id: access_key_id.to_string(),
        secret_access_key: secret_access_key.to_string(),
    })
}
