//! Caching for previously-computed prompt prefixes.

use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::Mutex;

/// Trait implemented by KV-cache storage backends.
pub trait Cache: Send + Sync {
    /// Look up a stored prefix by exact match of `tokens`.
    fn lookup(&self, tokens: &[crate::token::LlamaToken]) -> Option<CacheEntry>;
    /// Store `tokens` → `entry`.
    fn store(&self, tokens: &[crate::token::LlamaToken], entry: CacheEntry);
}

/// A single cached entry. The raw bytes are opaque to the cache.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Serialized KV state produced by `llama_state_get_data`.
    pub state: Vec<u8>,
    /// Position in the sequence at which the cache is valid.
    pub n_past: i32,
}

/// In-memory cache with prefix matching via longest suffix.
#[derive(Debug, Default, Clone)]
pub struct RamCache {
    inner: Arc<Mutex<BTreeMap<Vec<crate::token::LlamaToken>, CacheEntry>>>,
}

impl RamCache {
    /// Construct a new empty RAM cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Cache for RamCache {
    fn lookup(&self, tokens: &[crate::token::LlamaToken]) -> Option<CacheEntry> {
        let mut best_len = 0_usize;
        let mut best_entry: Option<CacheEntry> = None;
        let g = self.inner.lock();
        for (key, val) in g.iter() {
            if key.len() > tokens.len() || key.len() <= best_len {
                continue;
            }
            if tokens.starts_with(key) {
                best_len = key.len();
                best_entry = Some(val.clone());
            }
        }
        best_entry
    }

    fn store(&self, tokens: &[crate::token::LlamaToken], entry: CacheEntry) {
        self.inner.lock().insert(tokens.to_vec(), entry);
    }
}
