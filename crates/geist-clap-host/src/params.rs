// =============================================================================
// File: crates/geist-clap-host/src/params.rs
// Layer: CLAP host
// Purpose: Read-side view of a plugin's clap.params extension
// Status: Implemented; discovery + value read. Compile-checked; FFI calls
//         validated against real plugins.
// Notes: Parameter setting in CLAP is event-based (param_value events through
//        in_events, in process() or flush()), so it lands with the note/param
//        event bridge. This slice covers count, per-parameter info, and current
//        value. The view borrows the instance so it cannot outlive the plugin.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::c_char;

use clap_sys::ext::params::{clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS};
use clap_sys::id::clap_id;

use crate::instance::ClapInstance;

// Static identity and range of one plugin parameter
#[derive(Clone, Debug, PartialEq)]
pub struct ParamInfo {
    pub id: clap_id,
    pub name: String,
    pub module: String,
    pub flags: u32,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

// Safe read-side view of a plugin's clap.params extension
// Borrows the instance so it cannot outlive the plugin it queries
pub struct ClapParams<'a> {
    instance: &'a ClapInstance,
    ext: *const clap_plugin_params,
}

impl<'a> ClapParams<'a> {
    // Bind to an instance's parameter extension, or None if unsupported
    pub fn new(instance: &'a ClapInstance) -> Option<Self> {
        let ext = instance.extension(CLAP_EXT_PARAMS) as *const clap_plugin_params;
        if ext.is_null() {
            return None;
        }
        Some(Self { instance, ext })
    }

    // Number of parameters the plugin exposes
    pub fn count(&self) -> u32 {
        // SAFETY: ext and the plugin are live for the borrowed instance's life
        unsafe {
            match (*self.ext).count {
                Some(count) => count(self.instance.plugin_ptr()),
                None => 0,
            }
        }
    }

    // Static info for the parameter at `index`, if present
    pub fn info(&self, index: u32) -> Option<ParamInfo> {
        // SAFETY: clap_param_info is repr(C) POD; all-zero is a valid initial
        // state and get_info overwrites every field it reports
        let mut raw: clap_param_info = unsafe { std::mem::zeroed() };
        // SAFETY: ext and plugin are live; raw is a valid out-parameter
        let ok = unsafe {
            match (*self.ext).get_info {
                Some(get_info) => get_info(self.instance.plugin_ptr(), index, &mut raw),
                None => false,
            }
        };
        if !ok {
            return None;
        }
        Some(ParamInfo {
            id: raw.id,
            name: c_array_to_string(&raw.name),
            module: c_array_to_string(&raw.module),
            flags: raw.flags,
            min_value: raw.min_value,
            max_value: raw.max_value,
            default_value: raw.default_value,
        })
    }

    // Every parameter's info in index order
    pub fn infos(&self) -> Vec<ParamInfo> {
        let count = self.count();
        let mut out = Vec::with_capacity(count as usize);
        for index in 0..count {
            if let Some(info) = self.info(index) {
                out.push(info);
            }
        }
        out
    }

    // Current value of parameter `id`, if the plugin reports one
    pub fn value(&self, id: clap_id) -> Option<f64> {
        let mut out = 0.0_f64;
        // SAFETY: ext and plugin are live; out is a valid out-parameter
        let ok = unsafe {
            match (*self.ext).get_value {
                Some(get_value) => get_value(self.instance.plugin_ptr(), id, &mut out),
                None => false,
            }
        };
        if ok {
            Some(out)
        } else {
            None
        }
    }
}

// Read a NUL-terminated fixed C char array into an owned String
fn c_array_to_string(buf: &[c_char]) -> String {
    let bytes: Vec<u8> = buf
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a fixed C char buffer from a string for the marshalling tests
    fn fill(text: &str, len: usize) -> Vec<c_char> {
        let mut buf = vec![0 as c_char; len];
        for (slot, byte) in buf.iter_mut().zip(text.bytes()) {
            *slot = byte as c_char;
        }
        buf
    }

    #[test]
    fn reads_up_to_the_nul_terminator() {
        let buf = fill("Cutoff", 16);
        assert_eq!(c_array_to_string(&buf), "Cutoff");
    }

    #[test]
    fn reads_a_fully_filled_buffer() {
        let buf = fill("ABCD", 4);
        assert_eq!(c_array_to_string(&buf), "ABCD");
    }

    #[test]
    fn empty_buffer_is_empty_string() {
        let buf = fill("", 8);
        assert_eq!(c_array_to_string(&buf), "");
    }
}
