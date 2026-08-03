// =============================================================================
// File: crates/spectre-clap-host/src/instance.rs
// Layer: CLAP host
// Purpose: Drive one plugin through the CLAP lifecycle state machine
// Status: Implemented; compile-checked. Behavior validated against real plugins.
// Notes: create()->init() and activate()/deactivate()/destroy() are main-thread;
//        start/stop/process/reset are audio-thread. The instance tracks state so
//        teardown is always balanced. It holds only a raw plugin pointer borrowed
//        from a ClapBundle, so the owning node MUST keep that bundle loaded for
//        the instance's whole life (see plugin_node field order).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::ffi::{c_void, CStr, CString};
use std::fmt;

use clap_sys::plugin::clap_plugin;
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_ERROR};

use crate::bundle::ClapBundle;
use crate::ffi::host_impl;

// Failure modes of bringing a plugin instance to life
#[derive(Debug)]
pub enum InstanceError {
    // The plugin id contained an interior NUL and could not cross FFI
    BadId,
    // The factory returned null for this id
    CreateFailed,
    // The plugin's init() returned false
    InitFailed,
    // activate() returned false
    ActivateFailed,
    // start_processing() returned false
    StartFailed,
    // A transition was requested from the wrong lifecycle state
    BadState,
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstanceError::BadId => write!(f, "plugin id is not a valid C string"),
            InstanceError::CreateFailed => write!(f, "factory could not create plugin id"),
            InstanceError::InitFailed => write!(f, "plugin init() returned false"),
            InstanceError::ActivateFailed => write!(f, "plugin activate() returned false"),
            InstanceError::StartFailed => write!(f, "plugin start_processing() returned false"),
            InstanceError::BadState => write!(f, "lifecycle transition from wrong state"),
        }
    }
}

impl std::error::Error for InstanceError {}

// Where a plugin sits in the CLAP lifecycle
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum LifecycleState {
    // Initialized but not activated
    Inactive,
    // Activated for a stream, not yet processing
    Active,
    // Processing audio blocks
    Processing,
}

// A live CLAP plugin and its current lifecycle state
pub struct ClapInstance {
    plugin: *const clap_plugin,
    state: LifecycleState,
}

// SAFETY: a ClapInstance is owned by exactly one graph node. create/init/
// activate/deactivate/destroy run on the app thread; after activation the node
// moves to the audio thread and only that thread calls start/stop/process/reset.
// The raw plugin pointer is never shared across threads, which is what Send
// requires; the audio-thread contract enforces the rest.
unsafe impl Send for ClapInstance {}

impl ClapInstance {
    // Create plugin `plugin_id` from a bundle and run its init()
    pub fn create(bundle: &ClapBundle, plugin_id: &str) -> Result<Self, InstanceError> {
        let id = CString::new(plugin_id).map_err(|_| InstanceError::BadId)?;
        let plugin = bundle
            .create_plugin(&id, host_impl::host())
            .ok_or(InstanceError::CreateFailed)?;

        // SAFETY: plugin is a fresh non-null clap_plugin from the factory
        let init_ok = unsafe {
            match (*plugin).init {
                Some(init) => init(plugin),
                // A plugin without init is unusual but permitted by the ABI
                None => true,
            }
        };
        if !init_ok {
            // The CLAP contract requires destroy() to free a plugin whose
            // init() failed; keep create/destroy balanced before bailing out
            // SAFETY: plugin is live; destroy releases it
            unsafe {
                if let Some(destroy) = (*plugin).destroy {
                    destroy(plugin);
                }
            }
            return Err(InstanceError::InitFailed);
        }

        Ok(Self {
            plugin,
            state: LifecycleState::Inactive,
        })
    }

    // Activate for a stream; main thread, only from the inactive state
    pub fn activate(
        &mut self,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<(), InstanceError> {
        if self.state != LifecycleState::Inactive {
            return Err(InstanceError::BadState);
        }
        // SAFETY: plugin is live; called on the app thread while inactive
        let ok = unsafe {
            match (*self.plugin).activate {
                Some(activate) => activate(self.plugin, sample_rate, min_frames, max_frames),
                None => true,
            }
        };
        if !ok {
            return Err(InstanceError::ActivateFailed);
        }
        self.state = LifecycleState::Active;
        Ok(())
    }

    // Begin processing; audio thread, only from the active state
    pub fn start_processing(&mut self) -> Result<(), InstanceError> {
        if self.state != LifecycleState::Active {
            return Err(InstanceError::BadState);
        }
        // SAFETY: plugin is live and active; called on the audio thread
        let ok = unsafe {
            match (*self.plugin).start_processing {
                Some(start) => start(self.plugin),
                None => true,
            }
        };
        if !ok {
            return Err(InstanceError::StartFailed);
        }
        self.state = LifecycleState::Processing;
        Ok(())
    }

    // Run one block; audio thread. The caller owns and fills the clap_process.
    // Returns CLAP_PROCESS_ERROR if not processing or the plugin omits process.
    pub fn process(&mut self, process: &clap_process) -> clap_process_status {
        if self.state != LifecycleState::Processing {
            return CLAP_PROCESS_ERROR;
        }
        // SAFETY: plugin is live and processing; `process` is a valid block
        unsafe {
            match (*self.plugin).process {
                Some(run) => run(self.plugin, process as *const clap_process),
                None => CLAP_PROCESS_ERROR,
            }
        }
    }

    // End processing; audio thread. A no-op unless currently processing.
    pub fn stop_processing(&mut self) {
        if self.state != LifecycleState::Processing {
            return;
        }
        // SAFETY: plugin is live and processing; called on the audio thread
        unsafe {
            if let Some(stop) = (*self.plugin).stop_processing {
                stop(self.plugin);
            }
        }
        self.state = LifecycleState::Active;
    }

    // Fetch a plugin extension by id; null if the plugin does not support it
    // The caller reinterprets the result per that extension's ABI
    pub fn extension(&self, id: &CStr) -> *const c_void {
        // SAFETY: plugin is live; id is a NUL-terminated extension id
        unsafe {
            match (*self.plugin).get_extension {
                Some(get) => get(self.plugin, id.as_ptr()),
                None => std::ptr::null(),
            }
        }
    }

    // The raw plugin pointer, for extension calls that take it as their receiver
    pub(crate) fn plugin_ptr(&self) -> *const clap_plugin {
        self.plugin
    }

    // Clear processing state to silence; audio thread
    pub fn reset(&mut self) {
        // SAFETY: plugin is live; reset is valid in any activated state
        unsafe {
            if let Some(reset) = (*self.plugin).reset {
                reset(self.plugin);
            }
        }
    }

    // Deactivate; main thread. Stops processing first to honor the state machine.
    pub fn deactivate(&mut self) {
        self.stop_processing();
        if self.state != LifecycleState::Active {
            return;
        }
        // SAFETY: plugin is live and active; called on the app thread
        unsafe {
            if let Some(deactivate) = (*self.plugin).deactivate {
                deactivate(self.plugin);
            }
        }
        self.state = LifecycleState::Inactive;
    }
}

impl Drop for ClapInstance {
    fn drop(&mut self) {
        // Unwind the state machine, then release the plugin
        self.deactivate();
        // SAFETY: plugin stays live until destroy releases it here; the owning
        // node keeps the bundle (and its library) loaded until after this drop
        unsafe {
            if let Some(destroy) = (*self.plugin).destroy {
                destroy(self.plugin);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_is_descriptive() {
        assert!(format!("{}", InstanceError::CreateFailed).contains("create"));
        assert!(format!("{}", InstanceError::BadState).contains("state"));
    }

    #[test]
    fn process_status_error_is_zero() {
        // The graph treats a zero status as "do not continue"; confirm the const
        assert_eq!(CLAP_PROCESS_ERROR, 0);
    }
}
