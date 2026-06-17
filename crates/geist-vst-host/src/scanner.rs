// =============================================================================
// File: crates/geist-vst-host/src/scanner.rs
// Layer: plugin host
// Purpose: Discover .vst3 bundles in the platform's standard plugin paths
// Status: Implemented; standard paths plus a bounded, bundle-aware walk.
// Notes: A .vst3 entry is a bundle (a directory on macOS/Linux, a file or
//        directory on Windows). The walk records bundles but never descends
//        into one, and does not follow symlinks, so it cannot loop.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::{Path, PathBuf};

// Marker extension of a VST3 bundle on every platform
const VST3_EXTENSION: &str = "vst3";
// Bound on recursion depth so a pathological tree can never hang a scan
const MAX_SCAN_DEPTH: usize = 8;

// Standard VST3 install locations for the current platform
pub fn standard_vst3_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/VST3"));
        if let Some(home) = home_dir() {
            paths.push(home.join("Library/Audio/Plug-Ins/VST3"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from(r"C:\Program Files\Common Files\VST3"));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            paths.push(Path::new(&local).join(r"Programs\Common\VST3"));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = home_dir() {
            paths.push(home.join(".vst3"));
        }
        paths.push(PathBuf::from("/usr/lib/vst3"));
        paths.push(PathBuf::from("/usr/local/lib/vst3"));
    }

    paths
}

// Home directory from the environment; unix-only, used by the path table
#[cfg(unix)]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

// Discover every .vst3 bundle reachable under the given roots
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

// Scan the platform's standard VST3 paths
pub fn scan_standard() -> Vec<PathBuf> {
    discover_bundles(&standard_vst3_paths())
}

// Recurse one directory, recording bundles and descending into plain folders
fn collect(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_vst3_bundle(&path) {
            // A bundle is a leaf; never look inside it for more bundles
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

// Whether a path names a VST3 bundle by its extension
fn is_vst3_bundle(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some(VST3_EXTENSION)
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
        let dir = std::env::temp_dir().join(format!("geist_vst_{tag}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn discovers_bundles_and_does_not_descend_into_them() {
        let root = temp_dir("scan");
        // Top-level bundle directory
        let alpha = root.join("Alpha.vst3");
        std::fs::create_dir_all(&alpha).unwrap();
        // Bundle nested under a plain vendor folder
        let beta = root.join("vendor").join("Beta.vst3");
        std::fs::create_dir_all(&beta).unwrap();
        // A bundle inside a bundle must never be reported
        std::fs::create_dir_all(alpha.join("Nested.vst3")).unwrap();
        // A non-bundle file is ignored
        std::fs::write(root.join("readme.txt"), b"x").unwrap();

        let found = discover_bundles(std::slice::from_ref(&root));
        std::fs::remove_dir_all(&root).ok();

        assert!(found.contains(&alpha), "top-level bundle missing");
        assert!(found.contains(&beta), "nested bundle missing");
        assert!(
            !found.iter().any(|p| p.ends_with("Nested.vst3")),
            "must not descend into a bundle"
        );
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn ignores_directories_without_bundles() {
        let root = temp_dir("empty");
        std::fs::create_dir_all(root.join("plain")).unwrap();
        std::fs::write(root.join("song.wav"), b"data").unwrap();

        let found = discover_bundles(std::slice::from_ref(&root));
        std::fs::remove_dir_all(&root).ok();
        assert!(found.is_empty());
    }

    #[test]
    fn missing_root_yields_no_results_without_error() {
        let found = discover_bundles(&[PathBuf::from("/no/such/vst3/root")]);
        assert!(found.is_empty());
    }

    #[test]
    fn standard_paths_are_platform_appropriate() {
        let paths = standard_vst3_paths();
        assert!(!paths.is_empty());
        #[cfg(target_os = "macos")]
        assert!(paths.iter().any(|p| p.ends_with("Plug-Ins/VST3")));
    }
}
