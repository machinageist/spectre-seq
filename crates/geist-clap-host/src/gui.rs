// =============================================================================
// File: crates/geist-clap-host/src/gui.rs
// Layer: CLAP host
// Purpose: Safe wrapper over a plugin's clap.gui extension for embedded GUIs
// Status: Implemented; embed lifecycle + window bridge. Compile-checked. FFI
//         calls validated against real plugins; app-side reparenting lands when
//         the app hosts CLAP nodes.
// Notes: Every clap.gui call here is [main-thread] per the CLAP contract, so this
//        view borrows the instance immutably and must never be touched from the
//        audio thread. ClapWindow holds a 'static api string and a borrowed
//        native handle; set_parent/set_transient copy what they need synchronously
//        so the window may be a temporary. The host advertises clap.gui (see
//        ffi/host_impl.rs) but refuses resize until app wiring exists.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::{c_char, c_ulong, c_void, CStr, CString};

use clap_sys::ext::gui::{
    clap_gui_resize_hints, clap_plugin_gui, clap_window, clap_window_handle, CLAP_EXT_GUI,
    CLAP_WINDOW_API_COCOA, CLAP_WINDOW_API_WAYLAND, CLAP_WINDOW_API_WIN32, CLAP_WINDOW_API_X11,
};
use raw_window_handle::RawWindowHandle;

use crate::instance::ClapInstance;

// The platform windowing API a plugin GUI is embedded with
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WindowApi {
    // macOS NSView
    Cocoa,
    // Windows HWND
    Win32,
    // X11 window id
    X11,
    // Wayland wl_surface
    Wayland,
}

impl WindowApi {
    // The CLAP api id string for this windowing API
    pub fn as_clap(self) -> &'static CStr {
        match self {
            WindowApi::Cocoa => CLAP_WINDOW_API_COCOA,
            WindowApi::Win32 => CLAP_WINDOW_API_WIN32,
            WindowApi::X11 => CLAP_WINDOW_API_X11,
            WindowApi::Wayland => CLAP_WINDOW_API_WAYLAND,
        }
    }

    // Map a CLAP api id string back to a WindowApi, if recognized
    pub fn from_cstr(id: &CStr) -> Option<Self> {
        if id == CLAP_WINDOW_API_COCOA {
            Some(WindowApi::Cocoa)
        } else if id == CLAP_WINDOW_API_WIN32 {
            Some(WindowApi::Win32)
        } else if id == CLAP_WINDOW_API_X11 {
            Some(WindowApi::X11)
        } else if id == CLAP_WINDOW_API_WAYLAND {
            Some(WindowApi::Wayland)
        } else {
            None
        }
    }

    // The embedding API for the compile target; what the app should request
    pub fn native() -> Self {
        #[cfg(target_os = "macos")]
        {
            WindowApi::Cocoa
        }
        #[cfg(target_os = "windows")]
        {
            WindowApi::Win32
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            WindowApi::X11
        }
    }
}

// Whether and how a plugin GUI may be resized; mirror of clap_gui_resize_hints
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResizeHints {
    pub can_resize_horizontally: bool,
    pub can_resize_vertically: bool,
    pub preserve_aspect_ratio: bool,
    pub aspect_ratio_width: u32,
    pub aspect_ratio_height: u32,
}

impl ResizeHints {
    // Copy a raw clap_gui_resize_hints into the owned form
    fn from_raw(raw: &clap_gui_resize_hints) -> Self {
        Self {
            can_resize_horizontally: raw.can_resize_horizontally,
            can_resize_vertically: raw.can_resize_vertically,
            preserve_aspect_ratio: raw.preserve_aspect_ratio,
            aspect_ratio_width: raw.aspect_ratio_width,
            aspect_ratio_height: raw.aspect_ratio_height,
        }
    }
}

// A parent window handed to a plugin for embedding
// Owns a clap_window pointing at a 'static api id and a borrowed native handle
pub struct ClapWindow {
    window: clap_window,
}

impl ClapWindow {
    // Embed under a macOS NSView
    pub fn cocoa(ns_view: *mut c_void) -> Self {
        Self::from_parts(WindowApi::Cocoa, clap_window_handle { cocoa: ns_view })
    }

    // Embed under a Windows HWND
    pub fn win32(hwnd: *mut c_void) -> Self {
        Self::from_parts(WindowApi::Win32, clap_window_handle { win32: hwnd })
    }

    // Embed under an X11 window id
    pub fn x11(window: c_ulong) -> Self {
        Self::from_parts(WindowApi::X11, clap_window_handle { x11: window })
    }

    // Embed under a Wayland wl_surface
    pub fn wayland(surface: *mut c_void) -> Self {
        Self::from_parts(WindowApi::Wayland, clap_window_handle { ptr: surface })
    }

    // Bridge a winit/eframe RawWindowHandle into a CLAP parent window
    // Returns None for handle kinds CLAP cannot embed under
    pub fn from_raw_window_handle(handle: RawWindowHandle) -> Option<Self> {
        match handle {
            RawWindowHandle::AppKit(h) => Some(Self::cocoa(h.ns_view.as_ptr())),
            RawWindowHandle::Win32(h) => Some(Self::win32(h.hwnd.get() as *mut c_void)),
            RawWindowHandle::Xlib(h) => Some(Self::x11(h.window)),
            RawWindowHandle::Wayland(h) => Some(Self::wayland(h.surface.as_ptr())),
            _ => None,
        }
    }

    // Assemble a clap_window from an api and its matching handle union
    fn from_parts(api: WindowApi, specific: clap_window_handle) -> Self {
        Self {
            window: clap_window {
                api: api.as_clap().as_ptr(),
                specific,
            },
        }
    }

    // The raw clap_window for the set_parent/set_transient ABI calls
    fn as_raw(&self) -> *const clap_window {
        &self.window
    }
}

// Safe view of a plugin's clap.gui extension
// Borrows the instance so it cannot outlive the plugin it drives; all calls are
// main-thread only
pub struct ClapGui<'a> {
    instance: &'a ClapInstance,
    ext: *const clap_plugin_gui,
}

impl<'a> ClapGui<'a> {
    // Bind to an instance's gui extension, or None if the plugin has no GUI
    pub fn new(instance: &'a ClapInstance) -> Option<Self> {
        let ext = instance.extension(CLAP_EXT_GUI) as *const clap_plugin_gui;
        if ext.is_null() {
            return None;
        }
        Some(Self { instance, ext })
    }

    // Whether the plugin supports embedding (or floating) under `api`
    pub fn is_api_supported(&self, api: WindowApi, floating: bool) -> bool {
        // SAFETY: ext and plugin are live; api id is a 'static NUL-terminated str
        unsafe {
            match (*self.ext).is_api_supported {
                Some(f) => f(self.instance.plugin_ptr(), api.as_clap().as_ptr(), floating),
                None => false,
            }
        }
    }

    // The plugin's preferred api and whether it prefers a floating window
    pub fn preferred_api(&self) -> Option<(WindowApi, bool)> {
        let mut api: *const c_char = std::ptr::null();
        let mut floating = false;
        // SAFETY: ext and plugin are live; api/floating are valid out-parameters
        let ok = unsafe {
            match (*self.ext).get_preferred_api {
                Some(f) => f(self.instance.plugin_ptr(), &mut api, &mut floating),
                None => false,
            }
        };
        if !ok || api.is_null() {
            return None;
        }
        // SAFETY: a true return gives a NUL-terminated 'static api id from the plugin
        let id = unsafe { CStr::from_ptr(api) };
        WindowApi::from_cstr(id).map(|a| (a, floating))
    }

    // Create the GUI for `api`; floating windows are owned by the plugin, embedded
    // windows are reparented with set_parent afterward
    pub fn create(&self, api: WindowApi, floating: bool) -> bool {
        // SAFETY: ext and plugin are live; api id is a 'static NUL-terminated str
        unsafe {
            match (*self.ext).create {
                Some(f) => f(self.instance.plugin_ptr(), api.as_clap().as_ptr(), floating),
                None => false,
            }
        }
    }

    // Destroy the GUI; safe to call even if create() failed or was never called
    pub fn destroy(&self) {
        // SAFETY: ext and plugin are live; destroy tolerates an absent GUI
        unsafe {
            if let Some(f) = (*self.ext).destroy {
                f(self.instance.plugin_ptr());
            }
        }
    }

    // Set the GUI scale (e.g. 1.0, 2.0 for HiDPI); false if scale is unsupported
    pub fn set_scale(&self, scale: f64) -> bool {
        // SAFETY: ext and plugin are live
        unsafe {
            match (*self.ext).set_scale {
                Some(f) => f(self.instance.plugin_ptr(), scale),
                None => false,
            }
        }
    }

    // The GUI's current size in logical pixels, if it reports one
    pub fn size(&self) -> Option<(u32, u32)> {
        let mut width = 0u32;
        let mut height = 0u32;
        // SAFETY: ext and plugin are live; width/height are valid out-parameters
        let ok = unsafe {
            match (*self.ext).get_size {
                Some(f) => f(self.instance.plugin_ptr(), &mut width, &mut height),
                None => false,
            }
        };
        if ok {
            Some((width, height))
        } else {
            None
        }
    }

    // Whether the embedded GUI can be resized by the host
    pub fn can_resize(&self) -> bool {
        // SAFETY: ext and plugin are live
        unsafe {
            match (*self.ext).can_resize {
                Some(f) => f(self.instance.plugin_ptr()),
                None => false,
            }
        }
    }

    // Resize constraints for the GUI, if the plugin provides them
    pub fn resize_hints(&self) -> Option<ResizeHints> {
        // SAFETY: clap_gui_resize_hints is repr(C) POD; all-zero is a valid
        // initial state and get_resize_hints overwrites the fields it reports
        let mut raw: clap_gui_resize_hints = unsafe { std::mem::zeroed() };
        // SAFETY: ext and plugin are live; raw is a valid out-parameter
        let ok = unsafe {
            match (*self.ext).get_resize_hints {
                Some(f) => f(self.instance.plugin_ptr(), &mut raw),
                None => false,
            }
        };
        if ok {
            Some(ResizeHints::from_raw(&raw))
        } else {
            None
        }
    }

    // Round a requested size to the nearest the plugin will accept
    pub fn adjust_size(&self, width: u32, height: u32) -> Option<(u32, u32)> {
        let mut w = width;
        let mut h = height;
        // SAFETY: ext and plugin are live; w/h are valid in/out parameters
        let ok = unsafe {
            match (*self.ext).adjust_size {
                Some(f) => f(self.instance.plugin_ptr(), &mut w, &mut h),
                None => false,
            }
        };
        if ok {
            Some((w, h))
        } else {
            None
        }
    }

    // Force the GUI to a size; only valid sizes (see adjust_size) are accepted
    pub fn set_size(&self, width: u32, height: u32) -> bool {
        // SAFETY: ext and plugin are live
        unsafe {
            match (*self.ext).set_size {
                Some(f) => f(self.instance.plugin_ptr(), width, height),
                None => false,
            }
        }
    }

    // Embed the GUI under a host parent window; for non-floating GUIs only
    pub fn set_parent(&self, window: &ClapWindow) -> bool {
        // SAFETY: ext and plugin are live; window outlives this synchronous call
        unsafe {
            match (*self.ext).set_parent {
                Some(f) => f(self.instance.plugin_ptr(), window.as_raw()),
                None => false,
            }
        }
    }

    // Mark a floating GUI transient for a host window so it stacks correctly
    pub fn set_transient(&self, window: &ClapWindow) -> bool {
        // SAFETY: ext and plugin are live; window outlives this synchronous call
        unsafe {
            match (*self.ext).set_transient {
                Some(f) => f(self.instance.plugin_ptr(), window.as_raw()),
                None => false,
            }
        }
    }

    // Suggest a title for a floating GUI window
    pub fn suggest_title(&self, title: &str) {
        // A title with an interior NUL is dropped rather than truncated silently
        let Ok(c_title) = CString::new(title) else {
            return;
        };
        // SAFETY: ext and plugin are live; c_title is NUL-terminated and outlives the call
        unsafe {
            if let Some(f) = (*self.ext).suggest_title {
                f(self.instance.plugin_ptr(), c_title.as_ptr());
            }
        }
    }

    // Show the GUI
    pub fn show(&self) -> bool {
        // SAFETY: ext and plugin are live
        unsafe {
            match (*self.ext).show {
                Some(f) => f(self.instance.plugin_ptr()),
                None => false,
            }
        }
    }

    // Hide the GUI without destroying it
    pub fn hide(&self) -> bool {
        // SAFETY: ext and plugin are live
        unsafe {
            match (*self.ext).hide {
                Some(f) => f(self.instance.plugin_ptr()),
                None => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::NonNull;

    #[test]
    fn window_api_round_trips_through_clap_ids() {
        for api in [
            WindowApi::Cocoa,
            WindowApi::Win32,
            WindowApi::X11,
            WindowApi::Wayland,
        ] {
            assert_eq!(WindowApi::from_cstr(api.as_clap()), Some(api));
        }
    }

    #[test]
    fn unknown_api_id_is_none() {
        let id = c"vulkan";
        assert_eq!(WindowApi::from_cstr(id), None);
    }

    #[test]
    fn native_api_matches_target() {
        let native = WindowApi::native();
        #[cfg(target_os = "macos")]
        assert_eq!(native, WindowApi::Cocoa);
        #[cfg(target_os = "windows")]
        assert_eq!(native, WindowApi::Win32);
        #[cfg(all(unix, not(target_os = "macos")))]
        assert_eq!(native, WindowApi::X11);
    }

    #[test]
    fn cocoa_window_carries_its_api_and_handle() {
        let view = NonNull::<c_void>::dangling().as_ptr();
        let window = ClapWindow::cocoa(view);
        // SAFETY: as_raw points at the owned clap_window; api is the cocoa 'static id
        let api = unsafe { CStr::from_ptr((*window.as_raw()).api) };
        assert_eq!(WindowApi::from_cstr(api), Some(WindowApi::Cocoa));
        // SAFETY: cocoa and ptr alias the same bytes in the union
        let stored = unsafe { (*window.as_raw()).specific.ptr };
        assert_eq!(stored, view);
    }

    #[test]
    fn appkit_raw_handle_bridges_to_a_cocoa_window() {
        let view = NonNull::<c_void>::dangling();
        let raw = RawWindowHandle::AppKit(raw_window_handle::AppKitWindowHandle::new(view));
        let window = ClapWindow::from_raw_window_handle(raw).expect("appkit is embeddable");
        // SAFETY: as_raw points at the owned clap_window
        let api = unsafe { CStr::from_ptr((*window.as_raw()).api) };
        assert_eq!(WindowApi::from_cstr(api), Some(WindowApi::Cocoa));
        // SAFETY: cocoa and ptr alias the same bytes in the union
        let stored = unsafe { (*window.as_raw()).specific.ptr };
        assert_eq!(stored, view.as_ptr());
    }

    #[test]
    fn resize_hints_copy_every_field() {
        let raw = clap_gui_resize_hints {
            can_resize_horizontally: true,
            can_resize_vertically: false,
            preserve_aspect_ratio: true,
            aspect_ratio_width: 16,
            aspect_ratio_height: 9,
        };
        let hints = ResizeHints::from_raw(&raw);
        assert!(hints.can_resize_horizontally);
        assert!(!hints.can_resize_vertically);
        assert!(hints.preserve_aspect_ratio);
        assert_eq!(hints.aspect_ratio_width, 16);
        assert_eq!(hints.aspect_ratio_height, 9);
    }
}
