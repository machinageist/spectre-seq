// =============================================================================
// File: crates/spectre-lv2-host/src/scanner.rs
// Layer: LV2 host
// Purpose: Discover LV2 bundles in LV2_PATH or the platform's standard paths
// Status: Implemented; LV2_PATH override plus a bounded, manifest-aware walk.
// Notes: An LV2 bundle is a directory containing a manifest.ttl, conventionally
//        named with a .lv2 suffix; the manifest is the authoritative marker, so
//        the walk keys on it rather than the suffix. LV2_PATH, when set, replaces
//        the defaults per the LV2 spec and is split with the OS list separator.
//        The walk records bundles but never descends into one, and skips symlinks,
//        so it cannot loop. Parsing each manifest is the next slice's job.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

// Environment variable that, when set, replaces the default search paths
const LV2_PATH_VAR: &str = "LV2_PATH";
// File that marks a directory as an LV2 bundle on every platform
const LV2_MANIFEST: &str = "manifest.ttl";
// Bound on recursion depth so a pathological tree can never hang a scan
const MAX_SCAN_DEPTH: usize = 8;

// Search paths to scan: LV2_PATH if set, otherwise the platform defaults
pub fn standard_lv2_paths() -> Vec<PathBuf> {
    match std::env::var_os(LV2_PATH_VAR) {
        Some(value) => parse_lv2_path(&value),
        None => default_paths(),
    }
}

// Split an LV2_PATH value on the OS list separator, dropping empty segments
fn parse_lv2_path(value: &OsStr) -> Vec<PathBuf> {
    std::env::split_paths(value)
        .filter(|p| !p.as_os_str().is_empty())
        .collect()
}

// Standard LV2 install locations for the current platform
fn default_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/LV2"));
        if let Some(home) = home_dir() {
            paths.push(home.join("Library/Audio/Plug-Ins/LV2"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(common) = std::env::var_os("COMMONPROGRAMFILES") {
            paths.push(Path::new(&common).join("LV2"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            paths.push(Path::new(&appdata).join("LV2"));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = home_dir() {
            paths.push(home.join(".lv2"));
        }
        paths.push(PathBuf::from("/usr/lib/lv2"));
        paths.push(PathBuf::from("/usr/local/lib/lv2"));
    }

    paths
}

// Home directory from the environment; unix-only, used by the path table
#[cfg(unix)]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

// Discover every LV2 bundle reachable under the given roots
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

// Scan the platform's standard LV2 paths
pub fn scan_standard() -> Vec<PathBuf> {
    discover_bundles(&standard_lv2_paths())
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
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if is_lv2_bundle(&path) {
            // A bundle is a leaf; never look inside it for more bundles
            out.push(path);
            continue;
        }
        collect(&path, depth + 1, out);
    }
}

// Whether a directory is an LV2 bundle, i.e. directly contains a manifest.ttl
fn is_lv2_bundle(path: &Path) -> bool {
    path.join(LV2_MANIFEST).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Unique temp directory per test
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("spectre_lv2_{tag}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // Create an LV2 bundle directory with a manifest under `parent`
    fn make_bundle(parent: &Path, name: &str) -> PathBuf {
        let bundle = parent.join(name);
        std::fs::create_dir_all(&bundle).unwrap();
        std::fs::write(bundle.join(LV2_MANIFEST), b"# manifest").unwrap();
        bundle
    }

    #[test]
    fn discovers_bundles_by_their_manifest() {
        let root = temp_dir("scan");
        // A bundle directly under the root
        let foo = make_bundle(&root, "Foo.lv2");
        // A bundle nested under a vendor folder
        let bar = make_bundle(&root.join("vendor"), "Bar.lv2");
        // A manifest inside a found bundle must never be reported
        make_bundle(&foo, "Inner.lv2");
        // A directory without a manifest is not a bundle
        std::fs::create_dir_all(root.join("Empty.lv2")).unwrap();
        // A stray file is ignored
        std::fs::write(root.join("readme.txt"), b"x").unwrap();

        let found = discover_bundles(std::slice::from_ref(&root));
        std::fs::remove_dir_all(&root).ok();

        assert!(found.contains(&foo), "top-level bundle missing");
        assert!(found.contains(&bar), "nested bundle missing");
        assert!(
            !found.iter().any(|p| p.ends_with("Inner.lv2")),
            "must not descend into a bundle"
        );
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn ignores_directories_without_a_manifest() {
        let root = temp_dir("empty");
        std::fs::create_dir_all(root.join("plain")).unwrap();
        std::fs::write(root.join("song.wav"), b"data").unwrap();

        let found = discover_bundles(std::slice::from_ref(&root));
        std::fs::remove_dir_all(&root).ok();
        assert!(found.is_empty());
    }

    #[test]
    fn missing_root_yields_no_results_without_error() {
        let found = discover_bundles(&[PathBuf::from("/no/such/lv2/root")]);
        assert!(found.is_empty());
    }

    #[test]
    fn results_are_sorted_and_deduplicated() {
        let root = temp_dir("order");
        for name in ["Zeta.lv2", "Alpha.lv2", "Mu.lv2"] {
            make_bundle(&root, name);
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
    fn lv2_path_splits_on_the_os_separator() {
        let joined: OsString =
            std::env::join_paths([PathBuf::from("/a/lv2"), PathBuf::from("/b/lv2")]).unwrap();
        let parsed = parse_lv2_path(&joined);
        assert_eq!(
            parsed,
            vec![PathBuf::from("/a/lv2"), PathBuf::from("/b/lv2")]
        );
    }

    #[test]
    fn default_paths_are_platform_appropriate() {
        let paths = default_paths();
        assert!(!paths.is_empty());
        #[cfg(target_os = "macos")]
        assert!(paths.iter().any(|p| p.ends_with("Plug-Ins/LV2")));
        #[cfg(all(unix, not(target_os = "macos")))]
        assert!(paths.iter().any(|p| p == Path::new("/usr/lib/lv2")));
    }
}
