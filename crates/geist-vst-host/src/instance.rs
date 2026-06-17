// =============================================================================
// File: crates/geist-vst-host/src/instance.rs
// Layer: plugin host
// Purpose: VST3 component lifecycle: create, initialize, activate, process, end
// Status: Implemented; compile-checked. Behavior validated against real plugins.
// Notes: Drives IComponent + IAudioProcessor through the VST3 state machine. The
//        instance owns the host context that the plugin borrows for its lifetime.
//        It carries an unsafe Send: a plugin is owned by one graph node and only
//        touched on that node's thread after activation (see the impl comment).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::fmt;

use vst3::Steinberg::Vst::{
    IAudioProcessor, IAudioProcessorTrait, IComponent, IComponentTrait, IHostApplication,
    ProcessData, ProcessModes_, ProcessSetup, SymbolicSampleSizes_,
};
use vst3::Steinberg::{int32, kResultOk, FUnknown, IPluginBaseTrait, TBool};
use vst3::{ComPtr, ComWrapper};

use crate::host_app::HostApplication;
use crate::module::{ModuleError, Vst3Module};

// On/off truth values for the VST3 TBool out-parameters
const VST_TRUE: TBool = 1;
const VST_FALSE: TBool = 0;

// Failure modes of bringing a plugin instance to life
#[derive(Debug)]
pub enum InstanceError {
    Module(ModuleError),
    NoProcessor,
    InitFailed,
    SetupFailed,
    ActivateFailed,
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstanceError::Module(e) => write!(f, "{e}"),
            InstanceError::NoProcessor => write!(f, "component is not an audio processor"),
            InstanceError::InitFailed => write!(f, "component initialize() failed"),
            InstanceError::SetupFailed => write!(f, "setupProcessing() failed"),
            InstanceError::ActivateFailed => write!(f, "setActive() failed"),
        }
    }
}

impl std::error::Error for InstanceError {}

impl From<ModuleError> for InstanceError {
    fn from(e: ModuleError) -> Self {
        InstanceError::Module(e)
    }
}

// A live VST3 plugin: its component, processor view, and host context
// Field order is the safe teardown order; the host outlives neither yet drops last
pub struct VstInstance {
    component: ComPtr<IComponent>,
    processor: ComPtr<IAudioProcessor>,
    // Kept alive because the plugin borrows it from initialize() to terminate()
    _host: ComPtr<IHostApplication>,
    active: bool,
}

// SAFETY: a VstInstance is owned by exactly one graph node. Creation, setup, and
// teardown run on the app thread; after activation the node moves to the audio
// thread and only that thread calls process(). No COM pointer is ever shared
// across threads, which is what Send requires.
unsafe impl Send for VstInstance {}

impl VstInstance {
    // Instantiate class `class_index` from a module and initialize it
    pub fn create(module: &Vst3Module, class_index: i32) -> Result<Self, InstanceError> {
        let component = module.create_component(class_index)?;

        let host = ComWrapper::new(HostApplication::new());
        let host_ptr = host
            .to_com_ptr::<IHostApplication>()
            .ok_or(InstanceError::InitFailed)?;
        let host_context = host
            .to_com_ptr::<FUnknown>()
            .ok_or(InstanceError::InitFailed)?;

        // SAFETY: component is freshly created; host_context is a valid FUnknown
        // that stays alive via host_ptr for the whole instance lifetime
        let result = unsafe { component.initialize(host_context.as_ptr()) };
        if result != kResultOk {
            return Err(InstanceError::InitFailed);
        }

        let processor = component
            .cast::<IAudioProcessor>()
            .ok_or(InstanceError::NoProcessor)?;

        Ok(Self {
            component,
            processor,
            _host: host_ptr,
            active: false,
        })
    }

    // Configure the realtime 32-bit float processing setup
    pub fn setup_processing(
        &mut self,
        sample_rate: f64,
        max_block: i32,
    ) -> Result<(), InstanceError> {
        let mut setup = ProcessSetup {
            processMode: ProcessModes_::kRealtime as int32,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as int32,
            maxSamplesPerBlock: max_block,
            sampleRate: sample_rate,
        };
        // SAFETY: processor is live; setup is a valid out/in pointer
        let result = unsafe { self.processor.setupProcessing(&mut setup) };
        if result != kResultOk {
            return Err(InstanceError::SetupFailed);
        }
        Ok(())
    }

    // Activate or deactivate the component and toggle processing together
    pub fn set_active(&mut self, active: bool) -> Result<(), InstanceError> {
        let flag = if active { VST_TRUE } else { VST_FALSE };
        // SAFETY: component is live
        let result = unsafe { self.component.setActive(flag) };
        if result != kResultOk {
            return Err(InstanceError::ActivateFailed);
        }
        // SAFETY: processor is live; mirrors the active state
        unsafe {
            self.processor.setProcessing(flag);
        }
        self.active = active;
        Ok(())
    }

    // Run one processing block; the caller owns and fills the ProcessData
    pub fn process(&mut self, data: &mut ProcessData) {
        // SAFETY: data is a valid &mut for this block; processor is live
        unsafe {
            self.processor.process(data as *mut ProcessData);
        }
    }
}

impl Drop for VstInstance {
    fn drop(&mut self) {
        // SAFETY: component/processor remain live until their ComPtrs release,
        // which happens as the fields drop after this body
        unsafe {
            if self.active {
                self.processor.setProcessing(VST_FALSE);
                self.component.setActive(VST_FALSE);
            }
            self.component.terminate();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_wraps_module_error() {
        let err: InstanceError = ModuleError::NullFactory.into();
        assert!(matches!(err, InstanceError::Module(_)));
        // Display delegates to the wrapped error
        assert!(format!("{err}").contains("null"));
    }
}
