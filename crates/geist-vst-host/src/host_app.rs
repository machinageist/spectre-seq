// =============================================================================
// File: crates/geist-vst-host/src/host_app.rs
// Layer: plugin host
// Purpose: Host-side IHostApplication the plugin receives as its init context
// Status: Implemented; minimal getName, createInstance returns notImplemented.
// Notes: A VST3 component's initialize() is handed an IHostApplication. This is
//        the smallest viable host: it reports a name and declines to vend host
//        objects. Implemented from Rust via ComWrapper. Unsafe is the COM ABI.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::c_void;

use vst3::Class;
use vst3::Steinberg::Vst::{IHostApplication, IHostApplicationTrait, String128, TChar};
use vst3::Steinberg::{kInvalidArgument, kNotImplemented, kResultOk, tresult, TUID};

// Name reported to plugins that query the host
const HOST_NAME: &str = "Geist";

// Minimal IHostApplication implementation handed to each plugin at init
pub struct HostApplication {
    // Host name pre-encoded as UTF-16 for the String128 out-parameter
    name_utf16: Vec<u16>,
}

impl HostApplication {
    pub fn new() -> Self {
        Self {
            name_utf16: HOST_NAME.encode_utf16().collect(),
        }
    }
}

impl Default for HostApplication {
    fn default() -> Self {
        Self::new()
    }
}

// Declare which COM interfaces this Rust object exposes
impl Class for HostApplication {
    type Interfaces = (IHostApplication,);
}

impl IHostApplicationTrait for HostApplication {
    // Copy the host name into the caller's UTF-16 String128, NUL-terminated
    unsafe fn getName(&self, name: *mut String128) -> tresult {
        if name.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: name is non-null and points to a String128 owned by the caller
        let dst = unsafe { &mut *name };
        let count = self.name_utf16.len().min(dst.len() - 1);
        dst[..count].copy_from_slice(&self.name_utf16[..count]);
        dst[count] = 0 as TChar;
        kResultOk
    }

    // Geist does not vend host objects (IMessage / IAttributeList) yet
    unsafe fn createInstance(
        &self,
        _cid: *mut TUID,
        _iid: *mut TUID,
        _obj: *mut *mut c_void,
    ) -> tresult {
        kNotImplemented
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_name_encodes_to_utf16() {
        let host = HostApplication::new();
        assert_eq!(
            host.name_utf16,
            "Geist".encode_utf16().collect::<Vec<u16>>()
        );
    }
}
