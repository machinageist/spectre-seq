// Author: Jeff
// Date: 2026-08-25
// Description: Bounce panel state and the pure request checks the shell draws from
// Notes: The panel owns its own fields rather than living on AppModel, for the same reason
//   LiveEngine does: AppModel stays renderer-neutral. Everything decidable without a window is
//   a free function here, so the button-disabled rules are a regression gate rather than a
//   manual check — this repository has no GUI-driving harness and R4-8 does not add one.

use spectre_dsp::{DeviceParameterSnapshot, NoteEvent};
use spectre_offline::bounce::{
    bounce_into, max_frames, BounceConfig, BounceError, BounceProgress, BounceReport,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Why a bounce request cannot be started. One variant per §3.6 row that is decidable before
// anything is rendered; the rest are outcomes of a render and live in BounceError
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BounceRequestError {
    // E1 — no destination typed
    DestinationEmpty,
    // E1 — the path names a directory that does not exist, so creating the file must fail
    DestinationParentMissing(String),
    // E3 — a zero-length render
    LengthZero,
    // E3 — past the ceiling max_frames sets for this rate
    LengthAboveCeiling { frames: usize, maximum: usize },
}

impl std::fmt::Display for BounceRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DestinationEmpty => write!(formatter, "Choose a destination file"),
            Self::DestinationParentMissing(parent) => {
                write!(formatter, "Directory does not exist: {parent}")
            }
            Self::LengthZero => write!(formatter, "Length must be at least 1 sample"),
            Self::LengthAboveCeiling { frames, maximum } => write!(
                formatter,
                "Length must be between 1 and {maximum} samples; {frames} is longer"
            ),
        }
    }
}

// Everything the shell can decide before it renders one sample. `max_frames` is a parameter
// rather than a call so the caller states which sample rate the ceiling belongs to; a bounce
// reads its rate off the running engine, not off this function
pub fn validate_request(
    destination: &str,
    frames: usize,
    max_frames: usize,
) -> Result<(), BounceRequestError> {
    let trimmed = destination.trim();
    if trimmed.is_empty() {
        return Err(BounceRequestError::DestinationEmpty);
    }
    // A path with no parent component is a bare filename in the working directory, which exists
    if let Some(parent) = Path::new(trimmed).parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return Err(BounceRequestError::DestinationParentMissing(
                parent.to_string_lossy().into_owned(),
            ));
        }
    }
    if frames == 0 {
        return Err(BounceRequestError::LengthZero);
    }
    if frames > max_frames {
        return Err(BounceRequestError::LengthAboveCeiling {
            frames,
            maximum: max_frames,
        });
    }
    Ok(())
}

// Samples to mm:ss.mmm, the readout beside the length field
pub fn duration_label(frames: usize, sample_rate: f64) -> String {
    if sample_rate <= 0.0 {
        return "--:--.---".into();
    }
    let total_ms = (frames as f64 / sample_rate * 1_000.0).round() as u64;
    let minutes = total_ms / 60_000;
    let seconds = (total_ms % 60_000) / 1_000;
    let milliseconds = total_ms % 1_000;
    format!("{minutes:02}:{seconds:02}.{milliseconds:03}")
}

// What the panel is doing right now. The shell's smoke path reports this verbatim, so a headless
// launch that started a render would say so rather than print a literal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BounceState {
    Idle,
    Running,
    Finished,
    Failed,
}

impl BounceState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Finished => "finished",
            Self::Failed => "failed",
        }
    }
}

// Panel-owned state. The worker handle carries the render's result until it is joined
pub struct BouncePanel {
    pub destination: String,
    pub frames: usize,
    pub compare_against_live: bool,
    pub message: String,
    state: BounceState,
    cancel: Arc<AtomicBool>,
    progress: Arc<BounceProgress>,
    worker: Option<std::thread::JoinHandle<Result<BounceReport, BounceError>>>,
    report: Option<BounceReport>,
}

impl Default for BouncePanel {
    fn default() -> Self {
        Self {
            destination: String::new(),
            // Two seconds at the corpus rate. A default length is a starting point in a text
            // field the user overwrites, not a limit, so it needs no ledger row
            frames: 96_000,
            compare_against_live: false,
            message: String::new(),
            state: BounceState::Idle,
            cancel: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(BounceProgress::default()),
            worker: None,
            report: None,
        }
    }
}

impl BouncePanel {
    pub fn state(&self) -> BounceState {
        self.state
    }

    pub fn report(&self) -> Option<&BounceReport> {
        self.report.as_ref()
    }

    // Blocks done and total, for the progress line. Reads the shared counters rather than the
    // worker, so it is correct whether or not the worker has finished
    pub fn progress(&self) -> (u64, u64) {
        (self.progress.blocks_done(), self.progress.blocks_total())
    }

    // Start a render on a worker thread. Rendering never happens on the UI thread and never on
    // an audio callback: the bounce touches no RenderBridge at all
    pub fn start(
        &mut self,
        config: BounceConfig,
        values: Vec<DeviceParameterSnapshot>,
        events: Vec<NoteEvent>,
    ) -> Result<(), BounceRequestError> {
        validate_request(
            &self.destination,
            config.frames,
            max_frames(config.sample_rate),
        )?;
        if self.state == BounceState::Running {
            return Ok(());
        }

        let destination = self.destination.trim().to_string();
        self.cancel = Arc::new(AtomicBool::new(false));
        self.progress = Arc::new(BounceProgress::default());
        let cancel = Arc::clone(&self.cancel);
        let progress = Arc::clone(&self.progress);

        self.report = None;
        self.message.clear();
        self.state = BounceState::Running;
        self.worker = Some(std::thread::spawn(move || {
            let file = std::fs::File::create(&destination)
                .map_err(|error| BounceError::Write { block: 0, error })?;
            let mut sink = std::io::BufWriter::new(file);
            let outcome = bounce_into(config, &values, &events, &mut sink, &cancel, &progress);
            // Every failure but E7 leaves no partial file. A truncated render is
            // indistinguishable from a finished short one once the app is closed
            if outcome.is_err() {
                let _ = std::fs::remove_file(&destination);
            }
            outcome
        }));
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    // Collect the worker's result once it has finished. Called from the shell's repaint; never
    // blocks, so a long render does not freeze the window
    pub fn poll(&mut self) {
        if self
            .worker
            .as_ref()
            .is_none_or(|worker| !worker.is_finished())
        {
            return;
        }
        let Some(worker) = self.worker.take() else {
            return;
        };
        match worker.join() {
            Ok(Ok(report)) => {
                self.message = format!(
                    "Wrote {} frames in {} blocks; peak {:.4}",
                    report.frames, report.blocks, report.peak
                );
                if report.contaminated_nodes > 0 {
                    // E9 — the warning accompanies the report rather than replacing it
                    self.message.push_str(&format!(
                        " — containment silenced {} node-quanta; this render contains silence \
                         the devices did not intend",
                        report.contaminated_nodes
                    ));
                }
                self.report = Some(report);
                self.state = BounceState::Finished;
            }
            Ok(Err(error)) => {
                self.message = error.to_string();
                self.state = BounceState::Failed;
            }
            Err(_) => {
                self.message =
                    "The bounce worker panicked; this is a defect, please report it".into();
                self.state = BounceState::Failed;
            }
        }
    }
}
