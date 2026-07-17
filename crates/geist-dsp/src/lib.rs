// Author: Jeff
// Date: 2026-07-12
// Description: Public native DSP device and process surface
// Notes: UI consumes backend parameter metadata after process contracts stabilize

mod effect;
mod io;
mod parameter;
mod source;

pub use effect::{Gain, Saturator, GAIN_PARAMETERS, SATURATOR_PARAMETERS};
pub use io::{
    AudioProcessor, DeviceClass, DeviceIo, NoteEvent, NoteEventKind, ProcessContext, ProcessError,
    MAX_NOTE_EVENTS_PER_BLOCK,
};
pub use parameter::{DeviceParameterKey, DspParameter};
pub use source::{PulseInstrument, ToneSource, Waveform, PULSE_PARAMETERS, TONE_PARAMETERS};
