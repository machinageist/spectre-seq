// =============================================================================
// File: app/geist-daw/src/main.rs
// Layer: application binary
// Purpose: Entry point; launch the GUI, or a headless demo with --headless
// Status: Implemented; playable egui window over the audio engine.
// Notes: Audio runs on cpal's own thread. The GUI runs the native event loop on
//        the main thread and drives the synth over the lock-free control plane.
//        --headless skips the window and auto-plays the demo (no display needed).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

#[cfg(test)]
mod alloc_guard;
mod control;
mod engine;
mod fx;
mod graph_view;
mod gui;
mod history;
mod init;
mod project;
mod recorder;
mod session;
mod studio;
mod startup;

use std::time::Duration;

// Test builds route every heap op through the counting allocator so the
// audio-callback no-alloc contract is enforced, not just described
#[cfg(test)]
#[global_allocator]
static TEST_ALLOC: alloc_guard::CountingAlloc = alloc_guard::CountingAlloc;

use crate::control::EngineControl;
use crate::engine::Engine;

// Seconds between headless status reports
const STATUS_INTERVAL_SECS: u64 = 2;
// Window size on launch; sized for the full studio shell
const WINDOW_SIZE: [f32; 2] = [1180.0, 760.0];

fn main() {
    let options = startup::parse_args(std::env::args());
    let (ui_state, workflow_diagnostics) = startup::resolve_ui_state(&options);
    for diagnostic in &workflow_diagnostics {
        eprintln!("geist-daw: workflow config warning: {diagnostic}");
    }

    // Headless seeds the demo arpeggio; the GUI starts quiet and is played live.
    // An input device, when present, yields a recorder for capturing audio clips.
    let (engine, control, recorder) = match init::start(options.headless) {
        Ok(triple) => triple,
        Err(err) => {
            eprintln!("geist-daw: failed to start audio: {err}");
            std::process::exit(1);
        }
    };

    if options.headless {
        run_headless(engine, control);
    } else if let Err(err) = run_gui(engine, control, recorder, options.classic, ui_state) {
        eprintln!("geist-daw: GUI error: {err}");
        std::process::exit(1);
    }
}

// Open the playable window and run the native event loop
fn run_gui(
    engine: Engine,
    control: EngineControl,
    recorder: Option<recorder::AudioRecorder>,
    classic: bool,
    ui_state: geist_ui::state::UIState,
) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size(WINDOW_SIZE),
        ..Default::default()
    };
    eframe::run_native(
        "Geist DAW",
        options,
        Box::new(move |cc| {
            // Install the tactile-dark theme once before the first frame
            geist_ui::theme::GeistTheme::apply(&cc.egui_ctx);
            let app: Box<dyn eframe::App> = if classic {
                Box::new(gui::GeistApp::new(engine, control))
            } else {
                Box::new(studio::StudioApp::with_ui_state(engine, control, recorder, ui_state))
            };
            Ok(app)
        }),
    )
}

// Auto-play the demo and report level/xruns until interrupted
fn run_headless(engine: Engine, control: EngineControl) {
    println!(
        "geist-daw: streaming {} channel(s) @ {} Hz — press Ctrl-C to stop",
        engine.channels(),
        engine.sample_rate_hz()
    );
    let mut last_xruns = 0u64;
    loop {
        std::thread::sleep(Duration::from_secs(STATUS_INTERVAL_SECS));
        let xruns = engine.xruns();
        if xruns != last_xruns {
            eprintln!("geist-daw: xruns={xruns}");
            last_xruns = xruns;
        }
        println!("geist-daw: output level {:.3}", control.level());
    }
}
