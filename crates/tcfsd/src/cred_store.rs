//! Credential store: shared state + mtime-watching reload

use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared reference to a credential store instance
pub type SharedCredStore = Arc<RwLock<Option<tcfs_secrets::CredStore>>>;

/// Create a new empty shared credential store
pub fn new_shared() -> SharedCredStore {
    Arc::new(RwLock::new(None))
}

