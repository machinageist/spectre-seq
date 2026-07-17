// Author: Jeff
// Date: 2026-07-12
// Description: Command-line entrypoint for deterministic offline inspection
// Notes: Use --self-test for the built-in fixture or pass one project JSON path

use geist_offline::{default_project, inspect_project};
use geist_project::to_bytes;
use std::{env, fs, process::ExitCode};

fn run() -> Result<(), String> {
    let argument = env::args().nth(1);
    let bytes = match argument.as_deref() {
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
            eprintln!("geist-offline: {error}");
            ExitCode::FAILURE
        }
    }
}
