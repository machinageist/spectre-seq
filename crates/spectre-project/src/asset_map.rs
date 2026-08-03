// =============================================================================
// File: crates/spectre-project/src/asset_map.rs
// Layer: project persistence
// Purpose: Content-addressed registry of external audio assets
// Status: Implemented; blake3 hashing with relative-path references and dedup.
// Notes: Audio files are referenced, never embedded. Each reference carries a
//        relative path plus a blake3 content hash so duplicates collapse and
//        moved or altered files are detectable on load.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::HashMap;

use crate::schema::AssetRef;

// Hash a byte buffer to a lowercase-hex blake3 digest
pub fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

// Registry of asset references with content-addressed deduplication
#[derive(Clone, Debug, Default)]
pub struct AssetMap {
    refs: Vec<AssetRef>,
    by_hash: HashMap<String, usize>,
}

impl AssetMap {
    // Build an empty registry
    pub fn new() -> Self {
        Self::default()
    }

    // Rebuild a registry from a project's stored asset list
    pub fn from_refs(refs: Vec<AssetRef>) -> Self {
        let mut by_hash = HashMap::with_capacity(refs.len());
        for (i, r) in refs.iter().enumerate() {
            by_hash.entry(r.content_hash.clone()).or_insert(i);
        }
        Self { refs, by_hash }
    }

    // Register a file's bytes under a relative path, returning its index
    // Identical content collapses onto the existing reference
    pub fn register(&mut self, relative_path: impl Into<String>, bytes: &[u8]) -> usize {
        let content_hash = hash_bytes(bytes);
        if let Some(&i) = self.by_hash.get(&content_hash) {
            return i;
        }
        let i = self.refs.len();
        self.refs.push(AssetRef {
            relative_path: relative_path.into(),
            content_hash: content_hash.clone(),
            size_bytes: bytes.len() as u64,
        });
        self.by_hash.insert(content_hash, i);
        i
    }

    // Borrow a reference by index
    pub fn get(&self, index: usize) -> Option<&AssetRef> {
        self.refs.get(index)
    }

    // Find the index of an already-registered content hash
    pub fn index_of_hash(&self, hash: &str) -> Option<usize> {
        self.by_hash.get(hash).copied()
    }

    // Confirm a buffer still matches the stored hash at an index
    pub fn verify(&self, index: usize, bytes: &[u8]) -> bool {
        self.get(index)
            .is_some_and(|r| r.content_hash == hash_bytes(bytes))
    }

    // Borrow the references for embedding into a project file
    pub fn as_refs(&self) -> &[AssetRef] {
        &self.refs
    }

    // Consume the registry into its reference list
    pub fn into_refs(self) -> Vec<AssetRef> {
        self.refs
    }

    pub fn len(&self) -> usize {
        self.refs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.refs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_known_blake3_empty_vector() {
        // Official blake3 digest of the empty input
        assert_eq!(
            hash_bytes(b""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn hash_is_deterministic_and_content_sensitive() {
        assert_eq!(hash_bytes(b"kick.wav data"), hash_bytes(b"kick.wav data"));
        assert_ne!(hash_bytes(b"kick"), hash_bytes(b"snare"));
    }

    #[test]
    fn register_dedups_identical_content() {
        let mut map = AssetMap::new();
        let a = map.register("samples/kick.wav", b"PCM-DATA");
        // Same content under a different path collapses to the first index
        let b = map.register("other/kick_copy.wav", b"PCM-DATA");
        assert_eq!(a, b);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(a).unwrap().size_bytes, 8);
    }

    #[test]
    fn register_distinct_content_grows_map() {
        let mut map = AssetMap::new();
        let a = map.register("a.wav", b"one");
        let b = map.register("b.wav", b"two");
        assert_ne!(a, b);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn verify_detects_tampering() {
        let mut map = AssetMap::new();
        let i = map.register("loop.wav", b"original");
        assert!(map.verify(i, b"original"));
        assert!(!map.verify(i, b"corrupted"));
    }

    #[test]
    fn round_trips_through_ref_list() {
        let mut map = AssetMap::new();
        map.register("a.wav", b"one");
        map.register("b.wav", b"two");
        let refs = map.clone().into_refs();
        let rebuilt = AssetMap::from_refs(refs);
        assert_eq!(rebuilt.len(), 2);
        let hash_two = hash_bytes(b"two");
        assert_eq!(rebuilt.index_of_hash(&hash_two), Some(1));
    }
}
