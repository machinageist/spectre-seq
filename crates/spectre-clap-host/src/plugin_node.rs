// =============================================================================
// File: crates/spectre-clap-host/src/plugin_node.rs
// Layer: CLAP host
// Purpose: Wrap a hosted CLAP plugin as a spectre-graph AudioNode
// Status: Implemented; compile-checked. Audio bridging validated on real plugins.
// Notes: Bridges the channel-major ProcessContext to clap_process. The per-channel
//        pointer arrays are sized in prepare(), so process() builds the audio
//        buffers without allocating. CLAP input buffers are read-only in this
//        bridge, so input channel pointers cast away const for the ABI. The node
//        owns its ClapBundle and ClapInstance; field order makes the instance
//        (destroy) drop before the bundle (deinit + library unload).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::config::AudioConfig;
use spectre_core::context::ProcessContext;
use spectre_graph::node::AudioNode;

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::process::clap_process;

use crate::bundle::ClapBundle;
use crate::ffi::events::{empty_input_events, empty_output_events};
use crate::instance::{ClapInstance, InstanceError};

// A hosted CLAP plugin presented to the graph as one node
pub struct ClapPluginNode {
    // Field order is the teardown order: the instance must destroy() before the
    // bundle drops and unloads the library it borrows
    instance: ClapInstance,
    bundle: ClapBundle,
    // Whether prepare() activated and started processing; process is a no-op otherwise
    processing: bool,
    // Reused channel-pointer scratch so process() never allocates
    in_ptrs: Vec<*mut f32>,
    out_ptrs: Vec<*mut f32>,
    // Running sample position reported to the plugin as steady_time
    steady_time: i64,
}

// SAFETY: the node is owned by one graph slot. The raw channel pointers are
// written and dereferenced only inside process() on the owning audio thread;
// between blocks they are inert and never read from another thread. The bundle's
// own pointers are touched only during construction on the app thread; afterward
// it merely keeps the library mapped. The wrapped ClapInstance carries the same
// single-owner-thread guarantee.
unsafe impl Send for ClapPluginNode {}

impl ClapPluginNode {
    // Instantiate plugin `plugin_id` from a bundle and take ownership of both
    // The plugin is configured and activated in prepare()
    pub fn new(bundle: ClapBundle, plugin_id: &str) -> Result<Self, InstanceError> {
        let instance = ClapInstance::create(&bundle, plugin_id)?;
        Ok(Self {
            instance,
            bundle,
            processing: false,
            in_ptrs: Vec::new(),
            out_ptrs: Vec::new(),
            steady_time: 0,
        })
    }

    // The .clap this node was loaded from
    pub fn bundle(&self) -> &ClapBundle {
        &self.bundle
    }
}

impl AudioNode for ClapPluginNode {
    // Activate the plugin for the stream and pre-size the pointer scratch
    fn prepare(&mut self, config: &AudioConfig) {
        let sample_rate = config.sample_rate_hz as f64;
        let max_frames = config.block_size_frames;
        // Both activate and start_processing must succeed to run blocks
        self.processing = self
            .instance
            .activate(sample_rate, 1, max_frames)
            .and_then(|_| self.instance.start_processing())
            .is_ok();
        self.in_ptrs = vec![std::ptr::null_mut(); config.input_channels as usize];
        self.out_ptrs = vec![std::ptr::null_mut(); config.output_channels as usize];
        self.steady_time = 0;
    }

    // Point CLAP audio buffers at the context channels and run one block
    fn process(&mut self, ctx: &mut ProcessContext) {
        if !self.processing {
            return;
        }
        let frames = ctx.frames();
        let in_ch = ctx.input_channels();
        let out_ch = ctx.output_channels();
        let (input, output) = ctx.io();

        // Fill channel pointer arrays from the flat channel-major buffers
        let in_n = self.in_ptrs.len().min(in_ch);
        for (ch, slot) in self.in_ptrs.iter_mut().enumerate().take(in_n) {
            // SAFETY: CLAP input buffers are read-only in this bridge; the *mut
            // is an ABI artifact. The region is channel ch's contiguous frames.
            *slot = input[ch * frames..].as_ptr() as *mut f32;
        }
        let out_n = self.out_ptrs.len().min(out_ch);
        for (ch, slot) in self.out_ptrs.iter_mut().enumerate().take(out_n) {
            *slot = output[ch * frames..].as_mut_ptr();
        }

        let in_bus = clap_audio_buffer {
            data32: self.in_ptrs.as_mut_ptr(),
            data64: std::ptr::null_mut(),
            channel_count: in_n as u32,
            latency: 0,
            constant_mask: 0,
        };
        let mut out_bus = clap_audio_buffer {
            data32: self.out_ptrs.as_mut_ptr(),
            data64: std::ptr::null_mut(),
            channel_count: out_n as u32,
            latency: 0,
            constant_mask: 0,
        };

        let process = clap_process {
            steady_time: self.steady_time,
            frames_count: frames as u32,
            // Transport and event routing are not bridged yet; CLAP allows a
            // null transport but requires non-null event lists
            transport: std::ptr::null(),
            audio_inputs: &in_bus,
            audio_outputs: &mut out_bus,
            audio_inputs_count: if in_n > 0 { 1 } else { 0 },
            audio_outputs_count: if out_n > 0 { 1 } else { 0 },
            in_events: empty_input_events(),
            out_events: empty_output_events(),
        };

        // The status is advisory; the graph drives blocks unconditionally for now
        let _status = self.instance.process(&process);
        self.steady_time = self.steady_time.wrapping_add(frames as i64);
    }

    // Clear plugin processing state back to silence
    fn reset(&mut self) {
        self.instance.reset();
        self.steady_time = 0;
    }
}
