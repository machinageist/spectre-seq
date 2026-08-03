// =============================================================================
// File: crates/geist-vst-host/src/plugin_node.rs
// Layer: plugin host
// Purpose: Wrap a VstInstance as a geist-graph AudioNode
// Status: Implemented; compile-checked. Audio bridging validated on real plugins.
// Notes: Bridges the channel-major ProcessContext to VST3 ProcessData. The
//        per-channel pointer arrays are sized in prepare(), so process() builds
//        the bus buffers without allocating. VST3 input buses are read-only to
//        the plugin, so the input channel pointers cast away const for the ABI.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use geist_graph::node::AudioNode;

use vst3::Steinberg::int32;
use vst3::Steinberg::Vst::{
    AudioBusBuffers, AudioBusBuffers__type0, ProcessData, ProcessModes_, Sample32,
    SymbolicSampleSizes_,
};

use crate::instance::VstInstance;

// A hosted VST3 plugin presented to the graph as one node
pub struct VstPluginNode {
    instance: VstInstance,
    // Whether setup + activation succeeded; process is a no-op otherwise
    active: bool,
    // Reused channel-pointer scratch so process() never allocates
    in_ptrs: Vec<*mut Sample32>,
    out_ptrs: Vec<*mut Sample32>,
}

// SAFETY: the node is owned by one graph slot. The raw channel pointers are
// written and dereferenced only inside process() on the owning audio thread;
// between blocks they are inert and never read from another thread. The wrapped
// VstInstance carries the same single-owner-thread guarantee.
unsafe impl Send for VstPluginNode {}

impl VstPluginNode {
    // Wrap an instance; it is configured and activated in prepare()
    pub fn new(instance: VstInstance) -> Self {
        Self {
            instance,
            active: false,
            in_ptrs: Vec::new(),
            out_ptrs: Vec::new(),
        }
    }
}

impl AudioNode for VstPluginNode {
    // Configure the plugin for the stream and pre-size the pointer scratch
    fn prepare(&mut self, config: &AudioConfig) {
        let max_block = config.block_size_frames as i32;
        self.active = self
            .instance
            .setup_processing(config.sample_rate_hz as f64, max_block)
            .and_then(|_| self.instance.set_active(true))
            .is_ok();
        self.in_ptrs = vec![std::ptr::null_mut(); config.input_channels as usize];
        self.out_ptrs = vec![std::ptr::null_mut(); config.output_channels as usize];
    }

    // Point VST3 bus buffers at the context channels and run one block
    fn process(&mut self, ctx: &mut ProcessContext) {
        if !self.active {
            return;
        }
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        let (input, output) = ctx.io();

        // Fill channel pointer arrays into the flat channel-major buffers
        let in_n = self.in_ptrs.len().min(in_ch);
        for (ch, slot) in self.in_ptrs.iter_mut().enumerate().take(in_n) {
            // SAFETY: VST3 input buses are read-only; the *mut is an ABI artifact
            *slot = input[ch * frames..].as_ptr() as *mut Sample32;
        }
        let out_n = self.out_ptrs.len().min(out_ch);
        for (ch, slot) in self.out_ptrs.iter_mut().enumerate().take(out_n) {
            *slot = output[ch * frames..].as_mut_ptr();
        }

        let mut in_bus = AudioBusBuffers {
            numChannels: in_n as int32,
            silenceFlags: 0,
            __field0: AudioBusBuffers__type0 {
                channelBuffers32: self.in_ptrs.as_mut_ptr(),
            },
        };
        let mut out_bus = AudioBusBuffers {
            numChannels: out_n as int32,
            silenceFlags: 0,
            __field0: AudioBusBuffers__type0 {
                channelBuffers32: self.out_ptrs.as_mut_ptr(),
            },
        };

        let mut data = ProcessData {
            processMode: ProcessModes_::kRealtime as int32,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as int32,
            numSamples: frames as int32,
            numInputs: if in_n > 0 { 1 } else { 0 },
            numOutputs: if out_n > 0 { 1 } else { 0 },
            inputs: &mut in_bus as *mut AudioBusBuffers,
            outputs: &mut out_bus as *mut AudioBusBuffers,
            inputParameterChanges: std::ptr::null_mut(),
            outputParameterChanges: std::ptr::null_mut(),
            inputEvents: std::ptr::null_mut(),
            outputEvents: std::ptr::null_mut(),
            processContext: std::ptr::null_mut(),
        };

        self.instance.process(&mut data);
    }

    // Cycle activation to clear plugin state back to silence
    fn reset(&mut self) {
        if self.active {
            let _ = self.instance.set_active(false);
            self.active = self.instance.set_active(true).is_ok();
        }
    }
}
