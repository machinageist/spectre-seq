// =============================================================================
// File: crates/spectre-clap-host/src/db.rs
// Layer: CLAP host
// Purpose: Persistent plugin metadata cache so scans skip unchanged bundles
// Status: Implemented; flat CBOR file keyed by bundle path. Compile-checked;
//         freshness logic unit-tested via a loader closure.
// Notes: Opening a .clap (dlopen + entry init + factory enumeration) is the costly
//        part of a scan; this cache stores each bundle's descriptors against a
//        filesystem fingerprint (modified time + length) and reloads only when the
//        fingerprint changes. The plan suggested a sled/sqlite DB, but the index
//        is at most a few hundred rows, so a single serialized file is right-sized
//        and reuses the project's CBOR stack (ADR-003) with no new dependency; the
//        module boundary keeps the backend swappable. A corrupt or stale-version
//        cache loads as empty, which simply forces a full rescan. App-thread only.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::bundle::{ClapBundle, ClapPluginDescriptor};

// Bump when the cache layout changes; an older version loads as empty
const CACHE_VERSION: u32 = 1;
// Where the cache file lives within the platform cache directory
const CACHE_DIR_NAME: &str = "spectre-seq";
const CACHE_FILE_NAME: &str = "clap-metadata.cbor";

// Filesystem fingerprint used to detect that a bundle changed on disk
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    // Modified time as nanoseconds since the epoch; 0 if unavailable
    pub modified_nanos: u64,
    // Length in bytes of the bundle path's metadata
    pub len: u64,
}

impl Fingerprint {
    // Read the current fingerprint of a bundle path; None if it cannot be stat'd
    pub fn of(path: &Path) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        let modified_nanos = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Some(Self {
            modified_nanos,
            len: meta.len(),
        })
    }
}

// One cached bundle: how to tell it changed, and the plugins it exposes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub fingerprint: Fingerprint,
    pub descriptors: Vec<ClapPluginDescriptor>,
}

// Plugin metadata cache keyed by bundle path
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginCache {
    version: u32,
    // BTreeMap so serialization and descriptor order are deterministic
    entries: BTreeMap<PathBuf, CacheEntry>,
}

impl Default for PluginCache {
    fn default() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: BTreeMap::new(),
        }
    }
}

impl PluginCache {
    // Read a cache from disk; any missing/corrupt/old-version file yields an empty
    // cache, which forces a full rescan rather than failing
    pub fn load(path: &Path) -> Self {
        let Ok(bytes) = std::fs::read(path) else {
            return Self::default();
        };
        match ciborium::from_reader::<PluginCache, _>(bytes.as_slice()) {
            Ok(cache) if cache.version == CACHE_VERSION => cache,
            _ => Self::default(),
        }
    }

    // Write the cache to disk, creating the parent directory if needed
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
        std::fs::write(path, bytes)
    }

    // Reconcile the cache against freshly scanned paths, calling `load` only for
    // bundles that are new or whose fingerprint changed. Paths no longer scanned
    // are dropped. Returns how many bundles were loaded (a cache-miss count).
    pub fn refresh<F>(&mut self, paths: &[PathBuf], mut load: F) -> usize
    where
        F: FnMut(&Path) -> Vec<ClapPluginDescriptor>,
    {
        let mut fresh: BTreeMap<PathBuf, CacheEntry> = BTreeMap::new();
        let mut loaded = 0;
        for path in paths {
            let current = Fingerprint::of(path);
            // Reuse a cached entry only when its fingerprint still matches
            if let (Some(fp), Some(entry)) = (&current, self.entries.get(path)) {
                if &entry.fingerprint == fp {
                    fresh.insert(path.clone(), entry.clone());
                    continue;
                }
            }
            // Miss: load descriptors under the current fingerprint. A path that
            // cannot be stat'd gets a zero fingerprint so it re-scans next time.
            let fingerprint = current.unwrap_or(Fingerprint {
                modified_nanos: 0,
                len: 0,
            });
            let descriptors = load(path);
            fresh.insert(
                path.clone(),
                CacheEntry {
                    fingerprint,
                    descriptors,
                },
            );
            loaded += 1;
        }
        self.entries = fresh;
        loaded
    }

    // Refresh by opening bundles from disk; convenience over refresh()
    pub fn refresh_from_disk(&mut self, paths: &[PathBuf]) -> usize {
        self.refresh(paths, load_bundle_descriptors)
    }

    // Every cached descriptor across all bundles, in bundle-path order
    pub fn descriptors(&self) -> Vec<ClapPluginDescriptor> {
        self.entries
            .values()
            .flat_map(|e| e.descriptors.iter().cloned())
            .collect()
    }

    // The cached entry for one bundle path
    pub fn get(&self, path: &Path) -> Option<&CacheEntry> {
        self.entries.get(path)
    }

    // Number of cached bundles
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    // Whether the cache holds no bundles
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// Default loader: open a bundle and read its descriptors; empty on any error
pub fn load_bundle_descriptors(path: &Path) -> Vec<ClapPluginDescriptor> {
    match ClapBundle::open(path) {
        Ok(bundle) => bundle.descriptors(),
        Err(_) => Vec::new(),
    }
}

// Platform-standard location for the plugin metadata cache file
pub fn default_cache_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")?;
        Some(
            Path::new(&home)
                .join("Library/Caches")
                .join(CACHE_DIR_NAME)
                .join(CACHE_FILE_NAME),
        )
    }
    #[cfg(target_os = "windows")]
    {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(Path::new(&local).join(CACHE_DIR_NAME).join(CACHE_FILE_NAME))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".cache")))?;
        Some(base.join(CACHE_DIR_NAME).join(CACHE_FILE_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Unique temp directory per test, mirroring the scanner's helper
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("geist_clapdb_{tag}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // A descriptor carrying just an id, enough to identify it in assertions
    fn descriptor(id: &str) -> ClapPluginDescriptor {
        ClapPluginDescriptor {
            id: id.to_string(),
            name: id.to_string(),
            ..Default::default()
        }
    }

    // Create a bundle-shaped file with the given byte length
    fn write_bundle(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn refresh_loads_every_new_path() {
        let dir = temp_dir("new");
        let a = write_bundle(&dir, "A.clap", b"aa");
        let b = write_bundle(&dir, "B.clap", b"bb");
        let mut cache = PluginCache::default();

        let loaded = cache.refresh(&[a.clone(), b.clone()], |p| {
            vec![descriptor(p.file_name().unwrap().to_str().unwrap())]
        });
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(loaded, 2);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.descriptors().len(), 2);
    }

    #[test]
    fn refresh_reuses_unchanged_entries() {
        let dir = temp_dir("reuse");
        let a = write_bundle(&dir, "A.clap", b"aa");
        let mut cache = PluginCache::default();
        cache.refresh(std::slice::from_ref(&a), |_| vec![descriptor("A")]);

        // A second pass over the unchanged file must not call the loader
        let mut calls = 0;
        let loaded = cache.refresh(std::slice::from_ref(&a), |_| {
            calls += 1;
            vec![descriptor("A")]
        });
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(loaded, 0);
        assert_eq!(calls, 0);
        assert_eq!(cache.descriptors(), vec![descriptor("A")]);
    }

    #[test]
    fn refresh_reloads_when_fingerprint_changes() {
        let dir = temp_dir("changed");
        let a = write_bundle(&dir, "A.clap", b"aa");
        let b = write_bundle(&dir, "B.clap", b"bb");
        let mut cache = PluginCache::default();
        cache.refresh(&[a.clone(), b.clone()], |p| {
            vec![descriptor(p.file_name().unwrap().to_str().unwrap())]
        });

        // Grow A.clap so its length differs from the cached fingerprint
        write_bundle(&dir, "A.clap", b"aaaa");
        let mut reloaded = Vec::new();
        let loaded = cache.refresh(&[a.clone(), b.clone()], |p| {
            reloaded.push(p.to_path_buf());
            vec![descriptor(p.file_name().unwrap().to_str().unwrap())]
        });
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(loaded, 1, "only the changed bundle reloads");
        assert_eq!(reloaded, vec![a]);
    }

    #[test]
    fn refresh_drops_paths_no_longer_scanned() {
        let dir = temp_dir("drop");
        let a = write_bundle(&dir, "A.clap", b"aa");
        let b = write_bundle(&dir, "B.clap", b"bb");
        let mut cache = PluginCache::default();
        cache.refresh(&[a.clone(), b.clone()], |_| vec![descriptor("x")]);
        assert_eq!(cache.len(), 2);

        // B is gone from the scan; its entry must be evicted
        cache.refresh(std::slice::from_ref(&a), |_| vec![descriptor("x")]);
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(cache.len(), 1);
        assert!(cache.get(&a).is_some());
        assert!(cache.get(&b).is_none());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = temp_dir("roundtrip");
        let a = write_bundle(&dir, "A.clap", b"aa");
        let mut cache = PluginCache::default();
        cache.refresh(std::slice::from_ref(&a), |_| {
            vec![descriptor("alpha"), descriptor("beta")]
        });

        let file = dir.join("cache.cbor");
        cache.save(&file).unwrap();
        let loaded = PluginCache::load(&file);
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(loaded, cache);
        assert_eq!(loaded.descriptors().len(), 2);
    }

    #[test]
    fn load_missing_file_is_empty() {
        let cache = PluginCache::load(Path::new("/no/such/clap-metadata.cbor"));
        assert!(cache.is_empty());
    }

    #[test]
    fn load_rejects_corrupt_file() {
        let dir = temp_dir("corrupt");
        let file = dir.join("cache.cbor");
        std::fs::write(&file, b"not cbor at all").unwrap();
        let cache = PluginCache::load(&file);
        std::fs::remove_dir_all(&dir).ok();
        assert!(cache.is_empty());
    }

    #[test]
    fn default_cache_path_names_the_cache_file() {
        if let Some(path) = default_cache_path() {
            assert!(path.ends_with(CACHE_FILE_NAME));
            assert!(path.to_string_lossy().contains(CACHE_DIR_NAME));
        }
    }

    #[test]
    fn fingerprint_of_missing_path_is_none() {
        assert!(Fingerprint::of(Path::new("/no/such/Phantom.clap")).is_none());
    }
}
