//! [`HfDownloader`] trait and [`MockHfDownloader`] test double.
//!
//! The trait is `Send + Sync` so it can be shared across the multi-threaded
//! server worker. [`MockHfDownloader`] is a thread-safe in-memory stub used
//! by the resolver tests (Task 6+) and the load-time integration tests
//! (Task 11) without requiring a network round-trip.
//!
//! The `HfDownloader` impl for `MockHfDownloader` is `#[cfg(test)]`-gated
//! because the mock creates on-disk `.gguf` blobs via the `tempfile` crate,
//! which is currently a `dev-dependency` (kept out of the production lib to
//! avoid pulling in a binary-format helper that is never used at runtime).

use crate::error::LlamaError;
use std::path::PathBuf;

use super::repo::HfRepo;

/// Abstraction for fetching a single file from a Hugging Face repo.
///
/// Implementations must be safe to share across threads (`Send + Sync`) —
/// the HTTP server worker may resolve several model lookups concurrently.
pub trait HfDownloader: Send + Sync {
    /// Download (or return from cache) `filename` from `repo`.
    ///
    /// # Errors
    /// Returns [`LlamaError::ModelDownload`] (or another variant) on network
    /// failure, missing file, or any other I/O problem.
    fn get(&self, repo: &HfRepo, filename: &str) -> Result<PathBuf, LlamaError>;

    /// Enumerate filenames available in `repo`.
    ///
    /// The default impl returns `Ok(vec![])` so simple stubs / stubs that
    /// only need `get` do not have to implement listing. The real downloader
    /// (Task 8) will use `Api::model(...).info()` to surface `.gguf` files
    /// for the auto-pick path in the resolver.
    fn list_repo_files(&self, _repo: &HfRepo) -> Result<Vec<String>, LlamaError> {
        Ok(vec![])
    }
}

/// In-memory [`HfDownloader`] double used by unit tests.
///
/// Holds:
/// - a `(repo_id, filename) -> path` cache (serves `get` cache hits),
/// - a single-shot error injection slot (one pre-armed error is consumed
///   on the next `get` call),
/// - a per-repo `list_repo_files` override map (default: empty).
///
/// The fields are `std::sync::Mutex<...>` to keep the type `Send + Sync`
/// without pulling in `parking_lot` for this single use site.
#[derive(Debug)]
pub struct MockHfDownloader {
    cache: std::sync::Mutex<std::collections::HashMap<(String, String), PathBuf>>,
    next_error: std::sync::Mutex<Option<LlamaError>>,
    list_files: std::sync::Mutex<std::collections::HashMap<String, Vec<String>>>,
}

impl Default for MockHfDownloader {
    fn default() -> Self {
        Self {
            cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_error: std::sync::Mutex::new(None),
            list_files: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(test)]
impl MockHfDownloader {
    /// Pre-populate the cache: subsequent `get(repo, filename)` returns `path`
    /// without going through the on-disk blob-creation path.
    #[must_use]
    pub fn with_paths(self, repo: &str, filename: &str, path: PathBuf) -> Self {
        self.cache
            .lock()
            .expect("mock cache mutex poisoned")
            .insert((repo.to_string(), filename.to_string()), path);
        self
    }

    /// Configure `list_repo_files(repo)` to return `files`. Replaces any
    /// previously-configured list for the same repo.
    #[must_use]
    pub fn with_files(self, repo: &str, files: Vec<String>) -> Self {
        self.list_files
            .lock()
            .expect("mock list_files mutex poisoned")
            .insert(repo.to_string(), files);
        self
    }

    /// Arm a one-shot error: the next call to `get` will return `err` and
    /// clear the slot. Subsequent `get` calls behave normally.
    #[must_use]
    pub fn with_next_error(self, err: LlamaError) -> Self {
        *self
            .next_error
            .lock()
            .expect("mock next_error mutex poisoned") = Some(err);
        self
    }
}

#[cfg(test)]
impl HfDownloader for MockHfDownloader {
    fn get(&self, repo: &HfRepo, filename: &str) -> Result<PathBuf, LlamaError> {
        // 1. If an error is armed, take it and clear the slot in a single
        //    critical section so a concurrent caller does not see it twice.
        let armed = std::mem::replace(
            &mut *self
                .next_error
                .lock()
                .expect("mock next_error mutex poisoned"),
            None,
        );
        if let Some(err) = armed {
            return Err(err);
        }

        // 2. Cache hit short-circuits before any I/O.
        //    Uses `as_str()` because the more specific `repo_id()` accessor
        //    is added by Task 7; both return the same inner `&str`.
        let key = (repo.as_str().to_string(), filename.to_string());
        if let Some(cached) = self
            .cache
            .lock()
            .expect("mock cache mutex poisoned")
            .get(&key)
            .cloned()
        {
            return Ok(cached);
        }

        // 3. Cold path: create a temp .gguf file with a valid GGUF v3 magic
        //    header so downstream code that does a defensive magic-check
        //    (and tests that do `std::fs::read`) observe the expected bytes.
        //    `keep()` returns `(File, PathBuf)`; the File is bound to a name
        //    we never close so the path survives.
        use std::io::Write as _;
        use tempfile::NamedTempFile;

        let mut tmp = NamedTempFile::new()
            .map_err(|e| LlamaError::ModelDownload(format!("tempfile create: {e}")))?;
        tmp.write_all(b"GGUF\x00\x00\x00\x03")
            .map_err(|e| LlamaError::ModelDownload(format!("tempfile write: {e}")))?;
        let (_file, path) = tmp
            .keep()
            .map_err(|e| LlamaError::ModelDownload(format!("tempfile keep: {e}")))?;

        // 4. Store and return.
        self.cache
            .lock()
            .expect("mock cache mutex poisoned")
            .insert(key, path.clone());
        Ok(path)
    }

    fn list_repo_files(&self, repo: &HfRepo) -> Result<Vec<String>, LlamaError> {
        let configured = self
            .list_files
            .lock()
            .expect("mock list_files mutex poisoned")
            .get(repo.as_str())
            .cloned();
        Ok(configured.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LlamaError;
    use std::path::PathBuf;

    /// Second call with the same `(repo, filename)` returns the SAME path.
    /// (The first call still has to produce a usable on-disk file so the
    /// resolver can hand it to `LlamaModel::load_from_file`.)
    #[test]
    fn mock_caches_path_per_repo_filename() {
        let mock = MockHfDownloader::default();
        let repo = HfRepo::new("TheBloke/Foo").expect("valid repo id");

        let first = mock.get(&repo, "model.Q4_K_M.gguf").expect("first get");
        let second = mock.get(&repo, "model.Q4_K_M.gguf").expect("second get");

        assert_eq!(first, second, "same (repo, filename) must return same path");
        assert!(
            first.exists(),
            "cached path must still exist on disk: {}",
            first.display()
        );
        assert_eq!(
            std::fs::read(&first).expect("read back blob"),
            b"GGUF\x00\x00\x00\x03",
            "blob must contain valid GGUF v3 magic"
        );
    }

    /// When `next_error` is armed, the next `get` returns that error and
    /// clears the slot (so the call after that succeeds).
    #[test]
    fn mock_returns_injected_error() {
        let mock = MockHfDownloader::default().with_next_error(LlamaError::ModelDownload(
            "injected 404".into(),
        ));
        let repo = HfRepo::new("TheBloke/Foo").expect("valid repo id");

        let err = mock
            .get(&repo, "model.gguf")
            .expect_err("injected error must propagate");

        match err {
            LlamaError::ModelDownload(msg) => {
                assert_eq!(msg, "injected 404", "error message preserved");
            }
            other => panic!("expected ModelDownload, got {other:?}"),
        }

        // After consuming, the mock is healthy again.
        let recovered = mock
            .get(&repo, "model.gguf")
            .expect("post-injection get must succeed");
        assert!(recovered.exists(), "recovered path must exist on disk");
    }

    /// `list_repo_files` returns whatever was configured via `with_files`,
    /// falling back to the default empty `Vec` for repos that were not
    /// configured.
    #[test]
    fn mock_list_files_returns_configured() {
        let mock = MockHfDownloader::default()
            .with_files("TheBloke/Foo", vec!["a.gguf".into(), "b.gguf".into()])
            // A second repo is configured as empty to cover the "explicit
            // empty" branch separately from "never configured".
            .with_files("TheBloke/Empty", vec![]);

        let configured = mock
            .list_repo_files(&HfRepo::new("TheBloke/Foo").expect("valid"))
            .expect("list configured");
        assert_eq!(configured, vec!["a.gguf".to_string(), "b.gguf".to_string()]);

        let empty = mock
            .list_repo_files(&HfRepo::new("TheBloke/Empty").expect("valid"))
            .expect("list empty");
        assert!(empty.is_empty(), "explicit empty must be empty");

        let never = mock
            .list_repo_files(&HfRepo::new("TheBloke/Other").expect("valid"))
            .expect("list never-configured");
        assert!(
            never.is_empty(),
            "never-configured repo must fall through to default empty"
        );

        // Suppress the unused-import warning for PathBuf when nothing else
        // references it on a given toolchain.
        let _ = PathBuf::new();
    }
}
