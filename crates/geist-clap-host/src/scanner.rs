// =============================================================================
// File: crates/geist-clap-host/src/scanner.rs
// Layer: CLAP host
// Purpose: Discover .clap bundles in the platform's standard plugin paths
// Status: Implemented; standard paths plus a bounded, bundle-aware walk.
// Notes: A .clap entry is a single shared library on Linux/Windows but a bundle
//        directory on macOS. The extension check matches both. The walk records
//        entries but never descends into one, and does not follow symlinks, so
//        it cannot loop.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::{Path, PathBuf};

// Marker extension of a CLAP plugin on every platform
const CLAP_EXTENSION: &str = "clap";
// Bound on recursion depth so a pathological tree can never hang a scan
const MAX_SCAN_DEPTH: usize = 8;

// Standard CLAP install locations for the current platform
pub fn standard_clap_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
        if let Some(home) = home_dir() {
            paths.push(home.join("Library/Audio/Plug-Ins/CLAP"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from(r"C:\Program Files\Common Files\CLAP"));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            paths.push(Path::new(&local).join(r"Programs\Common\CLAP"));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = home_dir() {
            paths.push(home.join(".clap"));
        }
        paths.push(PathBuf::from("/usr/lib/clap"));
        paths.push(PathBuf::from("/usr/local/lib/clap"));
    }

    paths
}

// Home directory from the environment; unix-only, used by the path table
#[cfg(unix)]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

// Discover every .clap entry reachable under the given roots
// Results are sorted and de-duplicated for deterministic ordering
pub fn discover_bundles(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        collect(root, 0, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

// Scan the platform's standard CLAP paths
pub fn scan_standard() -> Vec<PathBuf> {
    discover_bundles(&standard_clap_paths())
}

// Recurse one directory, recording entries and descending into plain folders
fn collect(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_clap_bundle(&path) {
            // A .clap entry is a leaf; never look inside it for more plugins
            out.push(path);
            continue;
        }
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() && !file_type.is_symlink() {
                collect(&path, depth + 1, out);
            }
        }
    }
}

// Whether a path names a CLAP plugin by its extension
fn is_clap_bundle(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some(CLAP_EXTENSION)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("geist_clap_{tag}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn discovers_file_and_bundle_entries() {
        let root = temp_dir("scan");
        // Single-file plugin (Linux/Windows shape)
        let alpha = root.join("Alpha.clap");
        std::fs::write(&alpha, b"so").unwrap();
        // Bundle-directory plugin (macOS shape) nested under a vendor folder
        let beta = root.join("vendor").join("Beta.clap");
        std::fs::create_dir_all(&beta).unwrap();
        // A plugin inside a bundle must never be reported
        std::fs::write(beta.join("Nested.clap"), b"so").unwrap();
        // A non-plugin file is ignored
        std::fs::write(root.join("readme.txt"), b"x").unwrap();

        let found = discover_bundles(std::slice::from_ref(&root));
        std::fs::remove_dir_all(&root).ok();

        assert!(found.contains(&alpha), "single-file plugin missing");
        assert!(found.contains(&beta), "nested bundle plugin missing");
        assert!(
            !found.iter().any(|p| p.ends_with("Nested.clap")),
            "must not descend into a bundle"
        );
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn ignores_directories_without_plugins() {
        let root = temp_dir("empty");
        std::fs::create_dir_all(root.join("plain")).unwrap();
        std::fs::write(root.join("song.wav"), b"data").unwrap();

        let found = discover_bundles(std::slice::from_ref(&root));
        std::fs::remove_dir_all(&root).ok();
        assert!(found.is_empty());
    }

    #[test]
    fn missing_root_yields_no_results_without_error() {
        let found = discover_bundles(&[PathBuf::from("/no/such/clap/root")]);
        assert!(found.is_empty());
    }

    #[test]
    fn results_are_sorted_and_deduplicated() {
        let root = temp_dir("order");
        for name in ["Zeta.clap", "Alpha.clap", "Mu.clap"] {
            std::fs::write(root.join(name), b"so").unwrap();
        }
        // Same root twice must not double-count
        let found = discover_bundles(&[root.clone(), root.clone()]);
        std::fs::remove_dir_all(&root).ok();

        assert_eq!(found.len(), 3);
        let mut sorted = found.clone();
        sorted.sort();
        assert_eq!(found, sorted, "results must be sorted");
    }

    #[test]
    fn standard_paths_are_platform_appropriate() {
        let paths = standard_clap_paths();
        assert!(!paths.is_empty());
        #[cfg(target_os = "macos")]
        assert!(paths.iter().any(|p| p.ends_with("Plug-Ins/CLAP")));
        #[cfg(all(unix, not(target_os = "macos")))]
        assert!(paths.iter().any(|p| p.ends_with("/usr/lib/clap")));
    }
}
