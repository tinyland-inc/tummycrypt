//! tcfs-secrets: SOPS/age/KDBX credential management
//!
//! Identity discovery chain (in order of precedence):
//!   1. $CREDENTIALS_DIRECTORY/age-identity  (systemd LoadCredentialEncrypted)
//!   2. $SOPS_AGE_KEY_FILE env var (path to key file)
//!   3. $SOPS_AGE_KEY env var (literal key content)
//!   4. ~/.config/sops/age/keys.txt (default fallback)

pub mod age;
pub mod identity;
pub mod kdbx;
pub mod rotate;
pub mod sops;

pub use identity::{IdentityProvider, find_age_identity};
pub use sops::{SopsFile, SopsCredentials, decrypt_sops_file};
pub use kdbx::{KdbxStore, KdbxCredential};

use std::path::PathBuf;
use anyhow::Result;

/// Loaded S3 credentials, sourced from SOPS-encrypted file or environment
#[derive(Debug, Clone)]
pub struct S3Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
    pub region: String,
}

/// Credential store: loads and caches credentials, watches for file changes
pub struct CredStore {
    pub s3: Option<S3Credentials>,
    pub source: String,
}

impl CredStore {
    /// Load credentials using the full discovery chain:
    /// 1. SOPS-encrypted file (decrypted with age identity)
    /// 2. Environment variables (AWS_ACCESS_KEY_ID etc.)
    pub async fn load(config: &tcfs_core::config::SecretsConfig, storage: &tcfs_core::config::StorageConfig) -> Result<Self> {
        // Try SOPS credential file first
        if let Some(cred_file) = &storage.credentials_file {
            if cred_file.exists() {
                match Self::load_from_sops(cred_file, config).await {
                    Ok(store) => return Ok(store),
                    Err(e) => tracing::warn!("SOPS credential load failed: {e}, falling back to env"),
                }
            }
        }

        // Fall back to environment variables
        Self::load_from_env(storage)
    }

    async fn load_from_sops(cred_file: &PathBuf, secrets_config: &tcfs_core::config::SecretsConfig) -> Result<Self> {
        let identity = identity::find_age_identity(secrets_config).await?;
        let creds = sops::decrypt_sops_file(cred_file, &identity).await?;

        Ok(CredStore {
            s3: Some(S3Credentials {
                access_key_id: creds.access_key_id,
                secret_access_key: creds.secret_access_key,
                endpoint: creds.endpoint.unwrap_or_default(),
                region: creds.region.unwrap_or_else(|| "us-east-1".into()),
            }),
            source: format!("sops:{}", cred_file.display()),
        })
    }

    fn load_from_env(storage: &tcfs_core::config::StorageConfig) -> Result<Self> {
        let access_key = std::env::var("AWS_ACCESS_KEY_ID")
            .or_else(|_| std::env::var("SEAWEED_ACCESS_KEY"))
            .unwrap_or_default();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .or_else(|_| std::env::var("SEAWEED_SECRET_KEY"))
            .unwrap_or_default();

        Ok(CredStore {
            s3: if !access_key.is_empty() {
                Some(S3Credentials {
                    access_key_id: access_key,
                    secret_access_key: secret_key,
                    endpoint: storage.endpoint.clone(),
                    region: storage.region.clone(),
                })
            } else {
                None
            },
            source: "env".into(),
        })
    }
}
