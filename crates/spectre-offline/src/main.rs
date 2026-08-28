// Author: Jeff
// Date: 2026-07-12
// Description: Command-line entrypoint for deterministic offline inspection and bounce
// Notes: Use --self-test for the built-in fixture, --bounce to render it, or pass one project
//   JSON path. The bounce renders the built-in device fixture; tracks and clips are not yet a
//   bounce source, so --bounce takes a length rather than reading one off a project

use spectre_core::ObjectId;
use spectre_dsp::{
    DeviceParameterSnapshot, GAIN_PARAMETERS, PULSE_PARAMETERS, SATURATOR_PARAMETERS,
};
use spectre_offline::bounce::{
    bounce_into, bounce_report, max_frames, BounceConfig, BounceProgress,
    BOUNCE_FALLBACK_BLOCK_FRAMES, BOUNCE_FALLBACK_SAMPLE_RATE,
};
use spectre_offline::fixture;
use spectre_offline::{default_project, fixture_events, inspect_project};
use spectre_project::to_bytes;
use std::sync::atomic::AtomicBool;
use std::{env, fs, process::ExitCode};

// Instance identities for the fixture's three devices. Fixed rather than generated: the CLI's
// output is compared byte for byte against the equivalence test's, and a generated identity
// would make two runs of the same command describe two different device sets
const PULSE_DEVICE: u64 = 10;
const PULSE_LEVEL: u64 = 11;
const GAIN_DEVICE: u64 = 20;
const GAIN_VALUE: u64 = 21;
const SATURATOR_DEVICE: u64 = 30;
const SATURATOR_DRIVE: u64 = 31;
const SATURATOR_MIX: u64 = 32;

// The fixture's four parameters in the identity shape DeviceValues::from_snapshot accepts
fn fixture_snapshot() -> Result<Vec<DeviceParameterSnapshot>, String> {
    let id = |raw: u64| ObjectId::from_raw(raw).ok_or_else(|| format!("bad object id {raw}"));
    Ok(vec![
        DeviceParameterSnapshot::new(
            id(PULSE_DEVICE)?,
            "pulse",
            id(PULSE_LEVEL)?,
            PULSE_PARAMETERS[0],
            fixture::FIXTURE_PULSE_LEVEL,
        ),
        DeviceParameterSnapshot::new(
            id(GAIN_DEVICE)?,
            "gain",
            id(GAIN_VALUE)?,
            GAIN_PARAMETERS[0],
            fixture::FIXTURE_GAIN,
        ),
        DeviceParameterSnapshot::new(
            id(SATURATOR_DEVICE)?,
            "saturator",
            id(SATURATOR_DRIVE)?,
            SATURATOR_PARAMETERS[0],
            fixture::FIXTURE_SATURATOR_DRIVE,
        ),
        DeviceParameterSnapshot::new(
            id(SATURATOR_DEVICE)?,
            "saturator",
            id(SATURATOR_MIX)?,
            SATURATOR_PARAMETERS[1],
            fixture::FIXTURE_SATURATOR_MIX,
        ),
    ])
}

// Read one `--flag value` pair, failing on a flag with nothing after it
fn value_of(arguments: &[String], flag: &str) -> Result<Option<String>, String> {
    match arguments.iter().position(|argument| argument == flag) {
        None => Ok(None),
        Some(index) => arguments
            .get(index + 1)
            .cloned()
            .map(Some)
            .ok_or_else(|| format!("{flag} needs a value")),
    }
}

fn parsed<T: std::str::FromStr>(arguments: &[String], flag: &str) -> Result<Option<T>, String> {
    match value_of(arguments, flag)? {
        None => Ok(None),
        Some(raw) => raw
            .parse::<T>()
            .map(Some)
            .map_err(|_| format!("{flag} value {raw} is not valid")),
    }
}

// Render the built-in fixture and print the report; write a WAV when --out names a path
fn run_bounce(arguments: &[String]) -> Result<(), String> {
    let sample_rate = parsed::<f64>(arguments, "--rate")?.unwrap_or(BOUNCE_FALLBACK_SAMPLE_RATE);
    let block_frames =
        parsed::<usize>(arguments, "--block")?.unwrap_or(BOUNCE_FALLBACK_BLOCK_FRAMES);
    let frames = parsed::<usize>(arguments, "--frames")?.ok_or("--bounce needs --frames <n>")?;
    if frames > max_frames(sample_rate) {
        return Err(format!(
            "--frames {frames} exceeds the {} frame ceiling at {sample_rate} Hz",
            max_frames(sample_rate)
        ));
    }

    let config = BounceConfig {
        sample_rate,
        frames,
        block_frames,
        // Off by default: the per-block log exists for live/offline comparison, and a CLI run
        // that nobody is comparing would pay for a vector it does not read
        log_block_hashes: arguments
            .iter()
            .any(|argument| argument == "--block-hashes"),
    };
    let values = fixture_snapshot()?;
    let events = fixture_events(frames);

    let report = match value_of(arguments, "--out")? {
        None => bounce_report(config, &values, &events).map_err(|error| error.to_string())?,
        Some(path) => {
            let mut file = std::io::BufWriter::new(
                fs::File::create(&path)
                    .map_err(|error| format!("failed to create {path}: {error}"))?,
            );
            bounce_into(
                config,
                &values,
                &events,
                &mut file,
                &AtomicBool::new(false),
                &BounceProgress::default(),
            )
            .map_err(|error| error.to_string())?
        }
    };

    let output = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.iter().any(|argument| argument == "--bounce") {
        return run_bounce(&arguments);
    }

    let bytes = match arguments.first().map(String::as_str) {
        None | Some("--self-test") => {
            to_bytes(&default_project()).map_err(|error| error.to_string())?
        }
        Some(path) => fs::read(path).map_err(|error| format!("failed to read {path}: {error}"))?,
    };
    let report = inspect_project(&bytes)?;
    let output = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("spectre-offline: {error}");
            ExitCode::FAILURE
        }
    }
}
