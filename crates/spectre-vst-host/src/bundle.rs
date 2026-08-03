// =============================================================================
// File: crates/spectre-vst-host/src/bundle.rs
// Layer: plugin host
// Purpose: Resolve the loadable binary and metadata inside a .vst3 bundle
// Status: Implemented; platform/arch-aware path resolution, no FFI yet.
// Notes: VST3 bundles place the shared library under Contents in a platform and
//        architecture specific subfolder. This computes those paths; opening the
//        library is the FFI layer's job. Pure path logic, no filesystem reads.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::{Path, PathBuf};

// Subfolder under Contents/ holding the per-architecture binary
// macOS keeps a single universal binary under Contents/MacOS
fn platform_arch_dir() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "MacOS"
    }
    #[cfg(all(unix, not(target_os = "macos"), target_arch = "x86_64"))]
    {
        "x86_64-linux"
    }
    #[cfg(all(unix, not(target_os = "macos"), target_arch = "aarch64"))]
    {
        "aarch64-linux"
    }
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        "x86_64-win"
    }
    #[cfg(all(windows, target_arch = "x86"))]
    {
        "x86-win"
    }
}

// File extension of the loadable VST3 binary; macOS binaries have none
#[cfg(not(target_os = "macos"))]
fn binary_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "vst3"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "so"
    }
}

// A located VST3 bundle on disk; the entry a scan produced
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3Bundle {
    path: PathBuf,
}

impl Vst3Bundle {
    // Wrap a discovered bundle path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    // The bundle root (the .vst3 directory or file)
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Display name from the bundle stem, e.g. "Alpha" for "Alpha.vst3"
    pub fn name(&self) -> Option<&str> {
        self.path.file_stem().and_then(|s| s.to_str())
    }

    // Path to the shared library to load for the current platform/arch
    // On Windows a single-file .vst3 is its own binary
    pub fn binary_path(&self) -> PathBuf {
        let stem = self.name().unwrap_or("plugin");

        #[cfg(target_os = "macos")]
        {
            self.path
                .join("Contents")
                .join(platform_arch_dir())
                .join(stem)
        }

        #[cfg(target_os = "windows")]
        {
            if self.path.is_file() {
                return self.path.clone();
            }
            self.path
                .join("Contents")
                .join(platform_arch_dir())
                .join(format!("{stem}.{}", binary_extension()))
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            self.path
                .join("Contents")
                .join(platform_arch_dir())
                .join(format!("{stem}.{}", binary_extension()))
        }
    }

    // Path to the optional moduleinfo.json (VST3 3.7+ static metadata)
    pub fn moduleinfo_path(&self) -> PathBuf {
        self.path
            .join("Contents")
            .join("Resources")
            .join("moduleinfo.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_the_bundle_stem() {
        let b = Vst3Bundle::new("/plugins/Alpha.vst3");
        assert_eq!(b.name(), Some("Alpha"));
    }

    #[test]
    fn arch_dir_is_non_empty() {
        assert!(!platform_arch_dir().is_empty());
    }

    #[test]
    fn moduleinfo_path_is_under_contents_resources() {
        let b = Vst3Bundle::new("/plugins/Alpha.vst3");
        assert!(b
            .moduleinfo_path()
            .ends_with("Contents/Resources/moduleinfo.json"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_binary_is_under_contents_macos() {
        let b = Vst3Bundle::new("/plugins/Alpha.vst3");
        assert_eq!(
            b.binary_path(),
            PathBuf::from("/plugins/Alpha.vst3/Contents/MacOS/Alpha")
        );
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn linux_binary_is_arch_specific_so() {
        let b = Vst3Bundle::new("/plugins/Alpha.vst3");
        let bin = b.binary_path();
        assert!(bin.starts_with("/plugins/Alpha.vst3/Contents/"));
        assert_eq!(bin.extension().and_then(|e| e.to_str()), Some("so"));
    }
}
