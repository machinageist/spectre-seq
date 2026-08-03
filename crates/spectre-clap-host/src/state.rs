// =============================================================================
// File: crates/spectre-clap-host/src/state.rs
// Layer: CLAP host
// Purpose: Save and restore a plugin's opaque state via clap.state
// Status: Implemented; in-memory stream bridge. Compile-checked; stream plumbing
//         unit-tested, save/load FFI validated against real plugins.
// Notes: CLAP state crosses host-provided byte streams, not a returned blob: the
//        plugin writes into a clap_ostream on save and reads from a clap_istream
//        on load. This module backs those streams with a Vec<u8> (save) and a
//        slice cursor (load), so callers exchange plain &[u8]/Vec<u8>. Both calls
//        are [main-thread]; this view borrows the instance immutably and never
//        runs on the audio thread. State bytes are opaque and only meaningful to
//        the same plugin identity, so persistence pairs them with the plugin id.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::c_void;

use clap_sys::ext::state::{clap_plugin_state, CLAP_EXT_STATE};
use clap_sys::stream::{clap_istream, clap_ostream};

use crate::instance::ClapInstance;

// Safe view of a plugin's clap.state extension
// Borrows the instance so it cannot outlive the plugin; both calls are main-thread
pub struct ClapState<'a> {
    instance: &'a ClapInstance,
    ext: *const clap_plugin_state,
}

impl<'a> ClapState<'a> {
    // Bind to an instance's state extension, or None if the plugin has no state
    pub fn new(instance: &'a ClapInstance) -> Option<Self> {
        let ext = instance.extension(CLAP_EXT_STATE) as *const clap_plugin_state;
        if ext.is_null() {
            return None;
        }
        Some(Self { instance, ext })
    }

    // Capture the plugin's state as opaque bytes, or None if it declines to save
    pub fn save(&self) -> Option<Vec<u8>> {
        // SAFETY: ext is live for the borrowed instance's life
        let save = unsafe { (*self.ext).save }?;
        let mut buf: Vec<u8> = Vec::new();
        let stream = make_ostream(&mut buf as *mut Vec<u8> as *mut c_void);
        // SAFETY: plugin is live; stream is valid and its ctx points at buf for
        // the whole synchronous call
        let ok = unsafe { save(self.instance.plugin_ptr(), &stream) };
        if ok {
            Some(buf)
        } else {
            None
        }
    }

    // Restore previously saved bytes into the plugin; false if it rejects them
    pub fn load(&self, bytes: &[u8]) -> bool {
        // SAFETY: ext is live for the borrowed instance's life
        let Some(load) = (unsafe { (*self.ext).load }) else {
            return false;
        };
        let mut reader = SliceReader {
            data: bytes,
            pos: 0,
        };
        let stream = make_istream(&mut reader as *mut SliceReader as *mut c_void);
        // SAFETY: plugin is live; stream is valid and its ctx points at reader for
        // the whole synchronous call
        unsafe { load(self.instance.plugin_ptr(), &stream) }
    }
}

// Read cursor over borrowed bytes, handed to the istream via its ctx pointer
struct SliceReader<'a> {
    data: &'a [u8],
    pos: usize,
}

// Build an ostream whose ctx is a *mut Vec<u8> the plugin appends to
fn make_ostream(ctx: *mut c_void) -> clap_ostream {
    clap_ostream {
        ctx,
        write: Some(ostream_write),
    }
}

// Build an istream whose ctx is a *mut SliceReader the plugin reads from
fn make_istream(ctx: *mut c_void) -> clap_istream {
    clap_istream {
        ctx,
        read: Some(istream_read),
    }
}

// Append `size` bytes from the plugin into the backing Vec; returns bytes written
unsafe extern "C" fn ostream_write(
    stream: *const clap_ostream,
    buffer: *const c_void,
    size: u64,
) -> i64 {
    if stream.is_null() {
        return -1;
    }
    // SAFETY: stream is non-null and points at a clap_ostream we built
    let ctx = unsafe { (*stream).ctx } as *mut Vec<u8>;
    if ctx.is_null() {
        return -1;
    }
    let n = size as usize;
    if n == 0 {
        return 0;
    }
    if buffer.is_null() {
        return -1;
    }
    // SAFETY: the plugin guarantees buffer points at >= size readable bytes; ctx
    // is the Vec we passed and is valid for this synchronous call
    unsafe {
        let src = std::slice::from_raw_parts(buffer as *const u8, n);
        (*ctx).extend_from_slice(src);
    }
    size as i64
}

// Copy up to `size` bytes into the plugin's buffer; returns bytes read, 0 at EOF
unsafe extern "C" fn istream_read(
    stream: *const clap_istream,
    buffer: *mut c_void,
    size: u64,
) -> i64 {
    if stream.is_null() {
        return -1;
    }
    // SAFETY: stream is non-null and points at a clap_istream we built
    let ctx = unsafe { (*stream).ctx } as *mut SliceReader;
    if ctx.is_null() {
        return -1;
    }
    // SAFETY: ctx is the SliceReader we passed and is valid for this call
    let reader = unsafe { &mut *ctx };
    let remaining = reader.data.len() - reader.pos;
    let n = remaining.min(size as usize);
    if n == 0 {
        return 0;
    }
    if buffer.is_null() {
        return -1;
    }
    // SAFETY: the plugin guarantees buffer points at >= size writable bytes; the
    // source range is in bounds of reader.data
    unsafe {
        let dst = std::slice::from_raw_parts_mut(buffer as *mut u8, n);
        dst.copy_from_slice(&reader.data[reader.pos..reader.pos + n]);
    }
    reader.pos += n;
    n as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    // Invoke an ostream's write callback the way a plugin would
    fn write(stream: &clap_ostream, bytes: &[u8]) -> i64 {
        // SAFETY: stream.write is set; bytes is a valid readable region
        unsafe {
            stream.write.unwrap()(stream, bytes.as_ptr() as *const c_void, bytes.len() as u64)
        }
    }

    // Invoke an istream's read callback into a buffer the way a plugin would
    fn read(stream: &clap_istream, buf: &mut [u8]) -> i64 {
        // SAFETY: stream.read is set; buf is a valid writable region
        unsafe { stream.read.unwrap()(stream, buf.as_mut_ptr() as *mut c_void, buf.len() as u64) }
    }

    #[test]
    fn ostream_accumulates_successive_writes() {
        let mut buf: Vec<u8> = Vec::new();
        let stream = make_ostream(&mut buf as *mut Vec<u8> as *mut c_void);
        assert_eq!(write(&stream, &[1, 2, 3]), 3);
        assert_eq!(write(&stream, &[4, 5]), 2);
        assert_eq!(buf, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn ostream_zero_size_writes_nothing() {
        let mut buf: Vec<u8> = Vec::new();
        let stream = make_ostream(&mut buf as *mut Vec<u8> as *mut c_void);
        // A zero-size write is a no-op even with a null buffer
        // SAFETY: stream.write is set; size 0 means buffer is never read
        let n = unsafe { stream.write.unwrap()(&stream, std::ptr::null(), 0) };
        assert_eq!(n, 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn istream_reads_in_chunks_then_signals_eof() {
        let data = [10u8, 20, 30, 40];
        let mut reader = SliceReader {
            data: &data,
            pos: 0,
        };
        let stream = make_istream(&mut reader as *mut SliceReader as *mut c_void);

        let mut first = [0u8; 3];
        assert_eq!(read(&stream, &mut first), 3);
        assert_eq!(first, [10, 20, 30]);

        // A larger request returns only what remains
        let mut rest = [0u8; 8];
        assert_eq!(read(&stream, &mut rest), 1);
        assert_eq!(rest[0], 40);

        // Past the end the read reports EOF with zero bytes
        assert_eq!(read(&stream, &mut rest), 0);
    }

    #[test]
    fn ostream_then_istream_round_trips_bytes() {
        let original: Vec<u8> = (0u8..32).collect();

        let mut saved: Vec<u8> = Vec::new();
        let out = make_ostream(&mut saved as *mut Vec<u8> as *mut c_void);
        assert_eq!(write(&out, &original), original.len() as i64);

        let mut reader = SliceReader {
            data: &saved,
            pos: 0,
        };
        let inp = make_istream(&mut reader as *mut SliceReader as *mut c_void);
        let mut restored = vec![0u8; original.len()];
        assert_eq!(read(&inp, &mut restored), original.len() as i64);

        assert_eq!(restored, original);
    }
}
