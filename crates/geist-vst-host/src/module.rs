// =============================================================================
// File: crates/geist-vst-host/src/module.rs
// Layer: plugin host
// Purpose: Load a .vst3 binary and enumerate its plugin factory
// Status: Implemented; library load + factory enumeration. Unsafe FFI confined here.
// Notes: This is the first FFI layer. Loading and the COM calls are unsafe and
//        cannot be validated headless (no real .vst3 in CI); they are compile-
//        checked here and validated against real plugins on a dev machine. The
//        platform module-entry lifecycle (bundleEntry/InitDll/ModuleEntry) is
//        required before createInstance and lands with the instance layer; pure
//        factory enumeration works without it for typical plugins.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::{c_char, c_void};
use std::fmt;

use libloading::{Library, Symbol};
use vst3::Steinberg::Vst::IComponent;
use vst3::Steinberg::{kResultOk, IPluginFactory, IPluginFactoryTrait, PClassInfo, PFactoryInfo};
use vst3::{ComPtr, Interface};

use crate::bundle::Vst3Bundle;
use crate::descriptor::{c_array_to_string, tuid_to_hex, Vst3ClassInfo, Vst3FactoryInfo};

// Exported entry point every VST3 binary provides
type GetFactoryFn = unsafe extern "C" fn() -> *mut IPluginFactory;

// Failure modes of opening a VST3 module
#[derive(Debug)]
pub enum ModuleError {
    // The shared library could not be opened
    Load(libloading::Error),
    // The GetPluginFactory symbol was missing
    Symbol(libloading::Error),
    // The factory entry returned a null pointer
    NullFactory,
    // A class could not be enumerated or instantiated as an IComponent
    Instantiate,
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::Load(e) => write!(f, "failed to load VST3 binary: {e}"),
            ModuleError::Symbol(e) => write!(f, "missing GetPluginFactory entry: {e}"),
            ModuleError::NullFactory => write!(f, "GetPluginFactory returned null"),
            ModuleError::Instantiate => write!(f, "failed to instantiate component class"),
        }
    }
}

impl std::error::Error for ModuleError {}

// An opened VST3 module: its loaded binary and its plugin factory
// Field order matters: the factory (a COM reference) must release before the
// library unloads, so it is declared first and therefore dropped first
pub struct Vst3Module {
    factory: ComPtr<IPluginFactory>,
    _library: Library,
}

impl Vst3Module {
    // Open a bundle's binary and acquire its plugin factory
    pub fn open(bundle: &Vst3Bundle) -> Result<Self, ModuleError> {
        let binary = bundle.binary_path();

        // SAFETY: loading an external library is inherently unsafe. The path is
        // a scanned .vst3 binary and the resolved symbol matches the VST3 ABI.
        let library = unsafe { Library::new(&binary) }.map_err(ModuleError::Load)?;

        let factory_ptr = unsafe {
            let entry: Symbol<GetFactoryFn> = library
                .get(b"GetPluginFactory\0")
                .map_err(ModuleError::Symbol)?;
            entry()
        };

        // SAFETY: GetPluginFactory returns an owned IPluginFactory reference per
        // the VST3 contract; ComPtr::from_raw takes ownership and releases on drop.
        let factory = unsafe { ComPtr::from_raw(factory_ptr) }.ok_or(ModuleError::NullFactory)?;

        Ok(Self {
            factory,
            _library: library,
        })
    }

    // Vendor identity reported by the factory
    pub fn factory_info(&self) -> Vst3FactoryInfo {
        let mut info = PFactoryInfo {
            vendor: [0; 64],
            url: [0; 256],
            email: [0; 128],
            flags: 0,
        };
        // SAFETY: factory is a live COM pointer; info is a valid out-parameter
        let result = unsafe { self.factory.getFactoryInfo(&mut info) };
        if result != kResultOk {
            return Vst3FactoryInfo::default();
        }
        Vst3FactoryInfo {
            vendor: c_array_to_string(&info.vendor),
            url: c_array_to_string(&info.url),
            email: c_array_to_string(&info.email),
        }
    }

    // Instantiate the class at `class_index` as an IComponent
    // The raw 16-byte class id is read fresh so callers never handle TUIDs
    pub fn create_component(&self, class_index: i32) -> Result<ComPtr<IComponent>, ModuleError> {
        let mut info = PClassInfo {
            cid: [0; 16],
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
        };
        // SAFETY: factory is a live COM pointer; info is a valid out-parameter
        let result = unsafe { self.factory.getClassInfo(class_index, &mut info) };
        if result != kResultOk {
            return Err(ModuleError::Instantiate);
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        // SAFETY: cid/iid point to valid 16-byte TUIDs; obj is a valid out-pointer
        // IID bytes are u8; the factory wants an FIDString (*const c_char)
        let iid = IComponent::IID.as_ptr().cast::<c_char>();
        // SAFETY: cid/iid point to valid 16-byte ids; obj is a valid out-pointer
        let result = unsafe {
            self.factory
                .createInstance(info.cid.as_ptr(), iid, &mut obj)
        };
        if result != kResultOk || obj.is_null() {
            return Err(ModuleError::Instantiate);
        }
        // SAFETY: createInstance returns an owned IComponent reference in obj
        unsafe { ComPtr::from_raw(obj as *mut IComponent) }.ok_or(ModuleError::Instantiate)
    }

    // Every class the factory can instantiate, in declaration order
    pub fn classes(&self) -> Vec<Vst3ClassInfo> {
        // SAFETY: factory is a live COM pointer
        let count = unsafe { self.factory.countClasses() };
        let mut out = Vec::with_capacity(count.max(0) as usize);
        for index in 0..count {
            let mut info = PClassInfo {
                cid: [0; 16],
                cardinality: 0,
                category: [0; 32],
                name: [0; 64],
            };
            // SAFETY: index is in range; info is a valid out-parameter
            let result = unsafe { self.factory.getClassInfo(index, &mut info) };
            if result == kResultOk {
                out.push(Vst3ClassInfo {
                    cid: tuid_to_hex(&info.cid),
                    category: c_array_to_string(&info.category),
                    name: c_array_to_string(&info.name),
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_nonexistent_binary_is_load_error() {
        // Exercises the libloading path without a real plugin
        let bundle = Vst3Bundle::new("/no/such/Phantom.vst3");
        let result = Vst3Module::open(&bundle);
        assert!(matches!(result, Err(ModuleError::Load(_))));
    }
}
