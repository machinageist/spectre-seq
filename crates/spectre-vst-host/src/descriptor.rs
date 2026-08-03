// =============================================================================
// File: crates/spectre-vst-host/src/descriptor.rs
// Layer: plugin host
// Purpose: Safe owned descriptors built from VST3 factory C structs
// Status: Implemented; fixed-array C string conversion is unit-tested.
// Notes: VST3 returns fixed char8 arrays and 16-byte TUIDs. These helpers turn
//        them into owned Rust strings so the rest of the host never touches the
//        raw C representation. The conversions are pure and fully testable.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::c_char;

// Read a null-terminated, fixed-size C char array into an owned String
// Stops at the first NUL; lossy on invalid UTF-8 so it never panics
pub(crate) fn c_array_to_string(buf: &[c_char]) -> String {
    let bytes: Vec<u8> = buf
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

// Format a TUID (16 raw bytes) as an uppercase hex class id
pub(crate) fn tuid_to_hex(cid: &[c_char]) -> String {
    cid.iter().map(|&b| format!("{:02X}", b as u8)).collect()
}

// Vendor identity reported by a plugin factory
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vst3FactoryInfo {
    pub vendor: String,
    pub url: String,
    pub email: String,
}

// One class a factory can instantiate: a plugin within the bundle
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ClassInfo {
    // Hex of the 16-byte class id used to instantiate the plugin
    pub cid: String,
    // Category string, e.g. "Audio Module Class" or "Component Controller Class"
    pub category: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_up_to_the_null_terminator() {
        let buf: [c_char; 8] = [
            b'G' as c_char,
            b'a' as c_char,
            b'i' as c_char,
            b'n' as c_char,
            0,
            b'X' as c_char, // garbage past the terminator must be ignored
            0,
            0,
        ];
        assert_eq!(c_array_to_string(&buf), "Gain");
    }

    #[test]
    fn reads_full_buffer_when_unterminated() {
        let buf: [c_char; 3] = [b'A' as c_char, b'B' as c_char, b'C' as c_char];
        assert_eq!(c_array_to_string(&buf), "ABC");
    }

    #[test]
    fn empty_buffer_is_empty_string() {
        let buf: [c_char; 4] = [0, 0, 0, 0];
        assert_eq!(c_array_to_string(&buf), "");
    }

    #[test]
    fn tuid_formats_as_uppercase_hex() {
        let cid: [c_char; 4] = [0x00, 0x0F, 0x7F, -1i8 as c_char];
        // -1 as u8 is 0xFF
        assert_eq!(tuid_to_hex(&cid), "000F7FFF");
    }
}
