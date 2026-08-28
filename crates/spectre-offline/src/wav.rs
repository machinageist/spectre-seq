// Author: Jeff
// Date: 2026-08-25
// Description: Minimal 32-bit IEEE float WAVE writer over std::io::Write
// Notes: The engine's buffers are f32 (decision 6), so writing f32 is a copy rather than a
//   conversion. The file therefore holds the exact bits the plan produced, which is what makes a
//   bounce's hash a claim about the file and not only about a buffer. Dither exists to shape the
//   error of a conversion to fixed point; there is no such conversion here, so there is nothing
//   to dither. 16- and 24-bit output, and with them dither, are a later milestone's.

use std::io::{Result, Write};

// Canonical WAVE header length for a format chunk with no extension
const HEADER_BYTES: u32 = 44;
// Bytes the RIFF size field does not count: "RIFF" plus the size field itself
const RIFF_PREAMBLE_BYTES: u32 = 8;
// WAVE format tag for IEEE 754 float samples
const FORMAT_IEEE_FLOAT: u16 = 3;
const BITS_PER_SAMPLE: u16 = 32;
const BYTES_PER_SAMPLE: u32 = BITS_PER_SAMPLE as u32 / 8;
// Byte count of the fmt chunk body for a non-extensible format
const FMT_CHUNK_BYTES: u32 = 16;

// Write a canonical 44-byte WAVE header for 32-bit IEEE float PCM.
// `frames` is the total the file will hold; the sizes are written up front, so a caller that
// writes a different number of frames produces a file whose header disagrees with its data
pub fn write_header<W: Write>(
    sink: &mut W,
    channels: u16,
    sample_rate: u32,
    frames: usize,
) -> Result<()> {
    let block_align = u32::from(channels) * BYTES_PER_SAMPLE;
    // Saturating: a frame count past u32 cannot be described by this header at all, and a
    // saturated size is a visibly wrong file rather than a silently wrapped one
    let data_bytes = (frames as u64)
        .saturating_mul(u64::from(block_align))
        .min(u64::from(u32::MAX)) as u32;

    sink.write_all(b"RIFF")?;
    sink.write_all(&(HEADER_BYTES - RIFF_PREAMBLE_BYTES + data_bytes).to_le_bytes())?;
    sink.write_all(b"WAVE")?;

    sink.write_all(b"fmt ")?;
    sink.write_all(&FMT_CHUNK_BYTES.to_le_bytes())?;
    sink.write_all(&FORMAT_IEEE_FLOAT.to_le_bytes())?;
    sink.write_all(&channels.to_le_bytes())?;
    sink.write_all(&sample_rate.to_le_bytes())?;
    sink.write_all(&(sample_rate * block_align).to_le_bytes())?;
    sink.write_all(&(block_align as u16).to_le_bytes())?;
    sink.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;

    sink.write_all(b"data")?;
    sink.write_all(&data_bytes.to_le_bytes())?;
    Ok(())
}

// Write one interleaved block as little-endian f32.
// Bits, not a numeric conversion: -0.0 and a NaN payload survive the write intact, which is what
// keeps the file bit-comparable against the render that produced it
pub fn write_block<W: Write>(sink: &mut W, samples: &[f32]) -> Result<()> {
    for sample in samples {
        sink.write_all(&sample.to_bits().to_le_bytes())?;
    }
    Ok(())
}
