// =============================================================================
// File: crates/geist-clap-host/src/bundle.rs
// Layer: CLAP host
// Purpose: Load a .clap binary, resolve its entry point, enumerate its factory
// Status: Implemented; library load + factory enumeration. Unsafe FFI confined here.
// Notes: The exported `clap_entry` symbol is the entry struct itself (a data
//        symbol, not a function). Per the CLAP contract init() must precede
//        get_factory(), and each init() pairs with one deinit() on drop. All of
//        this is main-thread work; ClapBundle is intentionally !Send. Real
//        descriptor enumeration needs a real .clap and is validated on a dev
//        machine; the load-error and string-marshalling paths are tested here.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::{c_char, CStr, CString};
use std::fmt;
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use clap_sys::entry::clap_plugin_entry;
use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::host::clap_host;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::version::clap_version_is_compatible;

// Exported entry-point symbol every .clap binary provides
const CLAP_ENTRY_SYMBOL: &[u8] = b"clap_entry\0";

// Failure modes of opening a CLAP bundle
#[derive(Debug)]
pub enum BundleError {
    // The shared library could not be opened
    Load(libloading::Error),
    // The clap_entry symbol was missing
    Symbol(libloading::Error),
    // The clap_entry symbol resolved to null
    NoEntry,
    // The plugin reports an ABI version this host cannot speak
    IncompatibleVersion,
    // The entry's init() returned false
    InitFailed,
    // The standard plugin factory was unavailable
    NoFactory,
    // The bundle path contained an interior NUL and could not cross FFI
    BadPath,
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BundleError::Load(e) => write!(f, "failed to load .clap binary: {e}"),
            BundleError::Symbol(e) => write!(f, "missing clap_entry symbol: {e}"),
            BundleError::NoEntry => write!(f, "clap_entry resolved to null"),
            BundleError::IncompatibleVersion => write!(f, "incompatible CLAP ABI version"),
            BundleError::InitFailed => write!(f, "clap_entry init() returned false"),
            BundleError::NoFactory => write!(f, "plugin factory unavailable"),
            BundleError::BadPath => write!(f, "bundle path is not a valid C string"),
        }
    }
}

impl std::error::Error for BundleError {}

// One plugin's static identity as reported by the factory
// Serde-serializable so the scanner metadata cache (db.rs) can persist it
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClapPluginDescriptor {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
}

impl ClapPluginDescriptor {
    // Copy a raw descriptor into owned strings
    // Caller guarantees `desc` is a live descriptor pointer from the factory
    unsafe fn from_raw(desc: *const clap_plugin_descriptor) -> Self {
        let d = unsafe { &*desc };
        Self {
            id: unsafe { cstr_to_string(d.id) },
            name: unsafe { cstr_to_string(d.name) },
            vendor: unsafe { cstr_to_string(d.vendor) },
            version: unsafe { cstr_to_string(d.version) },
            description: unsafe { cstr_to_string(d.description) },
            features: unsafe { features_to_vec(d.features) },
        }
    }
}

// An opened CLAP bundle: its loaded binary, entry, and plugin factory
// The library must unload last, so manual Drop runs deinit() before the
// `_library` field drops in declaration order.
pub struct ClapBundle {
    entry: *const clap_plugin_entry,
    factory: *const clap_plugin_factory,
    path: PathBuf,
    _library: Library,
}

impl ClapBundle {
    // Open a .clap, run its entry init(), and acquire the plugin factory
    pub fn open(path: impl AsRef<Path>) -> Result<Self, BundleError> {
        let path = path.as_ref().to_path_buf();
        let binary = binary_path(&path);

        // SAFETY: loading an external library is inherently unsafe. The path is
        // a scanned .clap binary and the resolved symbol matches the CLAP ABI.
        let library = unsafe { Library::new(&binary) }.map_err(BundleError::Load)?;

        // The clap_entry symbol's address IS the entry struct; read it as such
        let entry: *const clap_plugin_entry = unsafe {
            let sym: Symbol<*const clap_plugin_entry> = library
                .get(CLAP_ENTRY_SYMBOL)
                .map_err(BundleError::Symbol)?;
            *sym
        };
        if entry.is_null() {
            return Err(BundleError::NoEntry);
        }

        // SAFETY: entry points at a live clap_plugin_entry in the loaded library
        let version = unsafe { (*entry).clap_version };
        if !clap_version_is_compatible(version) {
            return Err(BundleError::IncompatibleVersion);
        }

        // Init the entry with the bundle path before any factory access
        let path_c = path_to_cstring(&path)?;
        // SAFETY: entry is live; path_c outlives the call
        let init_ok = unsafe {
            match (*entry).init {
                Some(init) => init(path_c.as_ptr()),
                // An entry without init is unusual but permitted by the ABI
                None => true,
            }
        };
        if !init_ok {
            return Err(BundleError::InitFailed);
        }

        // Acquire the standard plugin factory by its well-known id
        // SAFETY: entry is live; the id is a static NUL-terminated C string
        let factory = unsafe {
            match (*entry).get_factory {
                Some(get) => get(CLAP_PLUGIN_FACTORY_ID.as_ptr()) as *const clap_plugin_factory,
                None => std::ptr::null(),
            }
        };
        if factory.is_null() {
            // Keep init()/deinit() balanced before bailing out
            // SAFETY: entry is live and init() succeeded above
            unsafe {
                if let Some(deinit) = (*entry).deinit {
                    deinit();
                }
            }
            return Err(BundleError::NoFactory);
        }

        Ok(Self {
            entry,
            factory,
            path,
            _library: library,
        })
    }

    // The .clap path this bundle was opened from
    pub fn path(&self) -> &Path {
        &self.path
    }

    // Instantiate a plugin by id using the given host context
    // Returns a raw, owned clap_plugin the caller must init() then destroy();
    // None if the factory cannot create or the id is unknown. The returned
    // plugin borrows this bundle's library and the host, so it must outlive
    // neither.
    pub fn create_plugin(&self, plugin_id: &CStr, host: &clap_host) -> Option<*const clap_plugin> {
        // SAFETY: factory is live; id and host stay valid across the call
        unsafe {
            let create = (*self.factory).create_plugin?;
            let plugin = create(self.factory, host, plugin_id.as_ptr());
            if plugin.is_null() {
                None
            } else {
                Some(plugin)
            }
        }
    }

    // Every plugin the factory can instantiate, in declaration order
    pub fn descriptors(&self) -> Vec<ClapPluginDescriptor> {
        let mut out = Vec::new();
        // SAFETY: factory is a live pointer owned by the loaded entry
        let count = unsafe {
            match (*self.factory).get_plugin_count {
                Some(get_count) => get_count(self.factory),
                None => return out,
            }
        };
        out.reserve(count as usize);
        for index in 0..count {
            // SAFETY: index < count; factory is live
            let desc = unsafe {
                match (*self.factory).get_plugin_descriptor {
                    Some(get_desc) => get_desc(self.factory, index),
                    None => continue,
                }
            };
            if desc.is_null() {
                continue;
            }
            // SAFETY: desc is a valid descriptor pointer from the factory
            out.push(unsafe { ClapPluginDescriptor::from_raw(desc) });
        }
        out
    }
}

impl Drop for ClapBundle {
    fn drop(&mut self) {
        // SAFETY: entry came from the still-loaded library; this deinit() pairs
        // with the init() run in open(). The library unloads after this returns.
        unsafe {
            if let Some(deinit) = (*self.entry).deinit {
                deinit();
            }
        }
    }
}

// Path to the loadable binary inside a .clap
// A .clap is a single shared library on Linux/Windows but a bundle directory on
// macOS, where the binary lives under Contents/MacOS.
fn binary_path(bundle: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if bundle.is_dir() {
            let stem = bundle
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("plugin");
            return bundle.join("Contents").join("MacOS").join(stem);
        }
        bundle.to_path_buf()
    }
    #[cfg(not(target_os = "macos"))]
    {
        bundle.to_path_buf()
    }
}

// Encode a path for the CLAP entry's init(), rejecting interior NULs
fn path_to_cstring(path: &Path) -> Result<CString, BundleError> {
    CString::new(path.to_string_lossy().as_bytes()).map_err(|_| BundleError::BadPath)
}

// Read a possibly-null C string into an owned String; null yields empty
unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

// Read a null-terminated array of C strings into an owned Vec
unsafe fn features_to_vec(mut array: *const *const c_char) -> Vec<String> {
    let mut out = Vec::new();
    if array.is_null() {
        return out;
    }
    loop {
        // SAFETY: array walks a null-terminated pointer table from the descriptor
        let entry = unsafe { *array };
        if entry.is_null() {
            break;
        }
        out.push(unsafe { cstr_to_string(entry) });
        array = unsafe { array.add(1) };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_nonexistent_binary_is_load_error() {
        // Exercises the libloading path without a real plugin
        let result = ClapBundle::open("/no/such/Phantom.clap");
        assert!(matches!(result, Err(BundleError::Load(_))));
    }

    #[test]
    fn cstr_marshalling_handles_value_and_null() {
        let c = CString::new("reverb").unwrap();
        assert_eq!(unsafe { cstr_to_string(c.as_ptr()) }, "reverb");
        assert_eq!(unsafe { cstr_to_string(std::ptr::null()) }, "");
    }

    #[test]
    fn features_array_reads_until_null_terminator() {
        let a = CString::new("audio-effect").unwrap();
        let b = CString::new("stereo").unwrap();
        let table = [a.as_ptr(), b.as_ptr(), std::ptr::null()];
        let got = unsafe { features_to_vec(table.as_ptr()) };
        assert_eq!(got, vec!["audio-effect".to_string(), "stereo".to_string()]);
    }

    #[test]
    fn empty_features_array_is_empty_vec() {
        let table = [std::ptr::null::<c_char>()];
        assert!(unsafe { features_to_vec(table.as_ptr()) }.is_empty());
        assert!(unsafe { features_to_vec(std::ptr::null()) }.is_empty());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn binary_path_is_the_file_itself_off_macos() {
        let p = PathBuf::from("/plugins/Foo.clap");
        assert_eq!(binary_path(&p), p);
    }
}
