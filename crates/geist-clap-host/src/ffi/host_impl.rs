// =============================================================================
// File: crates/geist-clap-host/src/ffi/host_impl.rs
// Layer: CLAP host
// Purpose: The clap_host vtable this host presents to plugins
// Status: Implemented. Advertises identity and the log, params, and gui host
//         extensions.
// Notes: One shared 'static host backs every instance: its callbacks are
//        stateless, so host_data is null and a single immutable host is sound to
//        share. The params callbacks (rescan/clear/request_flush) are accepted
//        no-ops until the host keeps a param cache; making them actionable needs
//        per-instance host_data and lands with param automation. The gui callbacks
//        refuse resize/show/hide (return false) because honoring them needs the
//        app's window and per-instance host_data, which land with app embedding.
//        host_log is not realtime-safe, which is fine: plugins must not log from
//        the audio thread.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::{c_char, c_void, CStr};

use clap_sys::ext::gui::{clap_host_gui, CLAP_EXT_GUI};
use clap_sys::ext::log::{
    clap_host_log, clap_log_severity, CLAP_EXT_LOG, CLAP_LOG_DEBUG, CLAP_LOG_ERROR, CLAP_LOG_FATAL,
    CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_INFO, CLAP_LOG_PLUGIN_MISBEHAVING, CLAP_LOG_WARNING,
};
use clap_sys::ext::params::{
    clap_host_params, clap_param_clear_flags, clap_param_rescan_flags, CLAP_EXT_PARAMS,
};
use clap_sys::host::clap_host;
use clap_sys::id::clap_id;
use clap_sys::version::CLAP_VERSION;

// Host identity strings, NUL-terminated for the C ABI
const HOST_NAME: &[u8] = b"Geist DAW\0";
const HOST_VENDOR: &[u8] = b"Geist\0";
const HOST_URL: &[u8] = b"https://geist.audio\0";
const HOST_VERSION: &[u8] = b"0.1.0\0";

// Shared stub host: identifies the host and exposes the log + params extensions
static GEIST_HOST: clap_host = clap_host {
    clap_version: CLAP_VERSION,
    host_data: std::ptr::null_mut(),
    name: HOST_NAME.as_ptr() as *const c_char,
    vendor: HOST_VENDOR.as_ptr() as *const c_char,
    url: HOST_URL.as_ptr() as *const c_char,
    version: HOST_VERSION.as_ptr() as *const c_char,
    get_extension: Some(get_extension),
    request_restart: Some(request_restart),
    request_process: Some(request_process),
    request_callback: Some(request_callback),
};

// Log extension: forwards plugin messages to stderr
static HOST_LOG: clap_host_log = clap_host_log {
    log: Some(host_log),
};

// Params extension: change notifications are accepted but not yet acted on
static HOST_PARAMS: clap_host_params = clap_host_params {
    rescan: Some(params_rescan),
    clear: Some(params_clear),
    request_flush: Some(params_request_flush),
};

// Gui extension: requests are accepted but refused until the app embeds windows
static HOST_GUI: clap_host_gui = clap_host_gui {
    resize_hints_changed: Some(gui_resize_hints_changed),
    request_resize: Some(gui_request_resize),
    request_show: Some(gui_request_show),
    request_hide: Some(gui_request_hide),
    closed: Some(gui_closed),
};

// The shared stub host; its 'static reference is valid for the whole program
pub fn host() -> &'static clap_host {
    &GEIST_HOST
}

// Resolve a host extension the plugin asks for; null when unsupported
unsafe extern "C" fn get_extension(
    _host: *const clap_host,
    extension_id: *const c_char,
) -> *const c_void {
    if extension_id.is_null() {
        return std::ptr::null();
    }
    // SAFETY: extension_id is a NUL-terminated id supplied by the plugin
    let id = unsafe { CStr::from_ptr(extension_id) };
    if id == CLAP_EXT_LOG {
        &HOST_LOG as *const clap_host_log as *const c_void
    } else if id == CLAP_EXT_PARAMS {
        &HOST_PARAMS as *const clap_host_params as *const c_void
    } else if id == CLAP_EXT_GUI {
        &HOST_GUI as *const clap_host_gui as *const c_void
    } else {
        std::ptr::null()
    }
}

// Restart/process/callback requests are accepted and ignored by the stub
unsafe extern "C" fn request_restart(_host: *const clap_host) {}
unsafe extern "C" fn request_process(_host: *const clap_host) {}
unsafe extern "C" fn request_callback(_host: *const clap_host) {}

// Forward a plugin log line to stderr; not realtime-safe by design
unsafe extern "C" fn host_log(
    _host: *const clap_host,
    severity: clap_log_severity,
    msg: *const c_char,
) {
    if msg.is_null() {
        return;
    }
    // SAFETY: msg is a NUL-terminated message from the plugin
    let text = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
    eprintln!("[clap {}] {text}", severity_label(severity));
}

// Param rescan/clear/flush requests; no-ops until the host caches param state
unsafe extern "C" fn params_rescan(_host: *const clap_host, _flags: clap_param_rescan_flags) {}
unsafe extern "C" fn params_clear(
    _host: *const clap_host,
    _param_id: clap_id,
    _flags: clap_param_clear_flags,
) {
}
unsafe extern "C" fn params_request_flush(_host: *const clap_host) {}

// Gui requests; the stub host owns no window, so resize/show/hide are refused and
// the notifications are no-ops until app-side embedding wires per-instance state
unsafe extern "C" fn gui_resize_hints_changed(_host: *const clap_host) {}
unsafe extern "C" fn gui_request_resize(_host: *const clap_host, _width: u32, _height: u32) -> bool {
    false
}
unsafe extern "C" fn gui_request_show(_host: *const clap_host) -> bool {
    false
}
unsafe extern "C" fn gui_request_hide(_host: *const clap_host) -> bool {
    false
}
unsafe extern "C" fn gui_closed(_host: *const clap_host, _was_destroyed: bool) {}

// Human label for a CLAP log severity
fn severity_label(severity: clap_log_severity) -> &'static str {
    match severity {
        CLAP_LOG_DEBUG => "debug",
        CLAP_LOG_INFO => "info",
        CLAP_LOG_WARNING => "warning",
        CLAP_LOG_ERROR => "error",
        CLAP_LOG_FATAL => "fatal",
        CLAP_LOG_HOST_MISBEHAVING => "host-misbehaving",
        CLAP_LOG_PLUGIN_MISBEHAVING => "plugin-misbehaving",
        _ => "log",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_is_named() {
        let host = host();
        // SAFETY: host.name is the static NUL-terminated identity string
        let name = unsafe { CStr::from_ptr(host.name) };
        assert_eq!(name.to_str().unwrap(), "Geist DAW");
    }

    #[test]
    fn offers_log_params_and_gui_extensions() {
        let host = host();
        // SAFETY: get_extension is set; the ids are valid NUL-terminated strings
        let log = unsafe { host.get_extension.unwrap()(host, CLAP_EXT_LOG.as_ptr()) };
        let params = unsafe { host.get_extension.unwrap()(host, CLAP_EXT_PARAMS.as_ptr()) };
        let gui = unsafe { host.get_extension.unwrap()(host, CLAP_EXT_GUI.as_ptr()) };
        assert!(!log.is_null());
        assert!(!params.is_null());
        assert!(!gui.is_null());
    }

    #[test]
    fn unknown_extension_is_null() {
        let host = host();
        let id = c"geist.not-an-extension";
        // SAFETY: get_extension is set; id is a valid NUL-terminated string
        let ext = unsafe { host.get_extension.unwrap()(host, id.as_ptr()) };
        assert!(ext.is_null());
    }

    #[test]
    fn gui_requests_are_refused_by_the_stub_host() {
        // The stub host owns no window, so the plugin's resize/show/hide are denied
        let host = host();
        // SAFETY: callbacks are set; host pointer is the live shared host
        unsafe {
            assert!(!HOST_GUI.request_resize.unwrap()(host, 640, 480));
            assert!(!HOST_GUI.request_show.unwrap()(host));
            assert!(!HOST_GUI.request_hide.unwrap()(host));
        }
    }

    #[test]
    fn severity_label_maps_levels() {
        assert_eq!(severity_label(CLAP_LOG_ERROR), "error");
        assert_eq!(severity_label(CLAP_LOG_INFO), "info");
        assert_eq!(severity_label(999), "log");
    }
}
