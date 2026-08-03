// =============================================================================
// Author: Jeff
// Date: 2026-08-03
// Description: Typed durable commands; validation is pure and application cannot fail.
// Notes: The two-phase split is what makes "rejected leaves the document unchanged" structural.
//
// File: crates/spectre-document/src/command.rs
// Layer: document
// Purpose: Command vocabulary and its validate-then-apply contract
// Status: Implemented; arrangement commands only until slice D3 reshapes them.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::ids::{ClipId, TrackId};
use spectre_core::time::MusicalTime;
use spectre_core::transaction::{IdentityDomain, TransactionError};

use crate::arrangement::Arrangement;
use crate::revision::{Aggregate, EffectSet};

// One durable edit, addressed by stable identity
//
// The command set is deliberately small. Slice D3 reshapes arrangement
// placement and slice CC1 splits content out, so anything richer written here
// would be rewritten before it had a second caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    // Add an arrangement track; allocates one TrackId
    CreateTrack,
    // Add a clip to an existing track; allocates one ClipId
    CreateClip {
        owner: TrackId,
        start: MusicalTime,
        duration: MusicalTime,
    },
    // Remove a clip; its identity is retired, never reused
    RemoveClip {
        clip: ClipId,
    },
}

impl Command {
    // Human-readable label for history UIs
    pub const fn label(&self) -> &'static str {
        match self {
            Command::CreateTrack => "create track",
            Command::CreateClip { .. } => "create clip",
            Command::RemoveClip { .. } => "remove clip",
        }
    }

    // Which aggregates this command would change if accepted
    // History uses this to scope its before-image before anything mutates
    pub fn effects(&self) -> EffectSet {
        match self {
            Command::CreateTrack | Command::CreateClip { .. } | Command::RemoveClip { .. } => {
                EffectSet::of(Aggregate::Arrangement)
            }
        }
    }

    // How many identities this command needs, by domain
    // Reserved as one checked batch after validation and before application
    pub const fn identity_need(&self) -> Option<(IdentityDomain, usize)> {
        match self {
            Command::CreateTrack => Some((IdentityDomain::Track, 1)),
            Command::CreateClip { .. } => Some((IdentityDomain::Clip, 1)),
            Command::RemoveClip { .. } => None,
        }
    }

    // Check every precondition against current state, mutating nothing
    //
    // This pass must not allocate identity, must not touch the allocator, and
    // must not leave a partial edit behind, because a rejection returns from
    // here and the document has to be byte-identical to before the call
    pub fn validate(&self, arrangement: &Arrangement) -> Result<(), TransactionError> {
        match self {
            Command::CreateTrack => Ok(()),
            Command::CreateClip {
                owner, duration, ..
            } => {
                if arrangement.track(*owner).is_none() {
                    return Err(TransactionError::missing_track(*owner));
                }
                if *duration == MusicalTime::ZERO {
                    return Err(TransactionError::InvalidDuration(
                        "clip duration must contain at least one tick",
                    ));
                }
                Ok(())
            }
            Command::RemoveClip { clip } => {
                if arrangement.clip(*clip).is_none() {
                    return Err(TransactionError::stale_clip(*clip));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track_one() -> TrackId {
        TrackId::new(1).unwrap()
    }

    fn clip_one() -> ClipId {
        ClipId::new(1).unwrap()
    }

    fn beat() -> MusicalTime {
        MusicalTime::from_ticks(960)
    }

    #[test]
    fn every_command_has_a_label() {
        let commands = [
            Command::CreateTrack,
            Command::CreateClip {
                owner: track_one(),
                start: MusicalTime::ZERO,
                duration: beat(),
            },
            Command::RemoveClip { clip: clip_one() },
        ];
        for command in commands {
            assert!(!command.label().is_empty());
        }
    }

    #[test]
    fn creating_a_clip_on_a_missing_track_reports_a_missing_owner() {
        let arrangement = Arrangement::new();
        let err = Command::CreateClip {
            owner: track_one(),
            start: MusicalTime::ZERO,
            duration: beat(),
        }
        .validate(&arrangement)
        .unwrap_err();
        assert_eq!(err, TransactionError::missing_track(track_one()));
    }

    #[test]
    fn a_zero_duration_clip_reports_an_invalid_duration() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track_one()).unwrap();
        let err = Command::CreateClip {
            owner: track_one(),
            start: MusicalTime::ZERO,
            duration: MusicalTime::ZERO,
        }
        .validate(&arrangement)
        .unwrap_err();
        assert!(matches!(err, TransactionError::InvalidDuration(_)));
    }

    #[test]
    fn removing_an_absent_clip_reports_stale_identity() {
        let arrangement = Arrangement::new();
        let err = Command::RemoveClip { clip: clip_one() }
            .validate(&arrangement)
            .unwrap_err();
        assert_eq!(err, TransactionError::stale_clip(clip_one()));
    }

    #[test]
    fn missing_owner_and_stale_identity_are_distinguishable() {
        let arrangement = Arrangement::new();
        let missing = Command::CreateClip {
            owner: track_one(),
            start: MusicalTime::ZERO,
            duration: beat(),
        }
        .validate(&arrangement)
        .unwrap_err();
        let stale = Command::RemoveClip { clip: clip_one() }
            .validate(&arrangement)
            .unwrap_err();
        assert_ne!(missing, stale);
    }

    #[test]
    fn identity_need_matches_what_each_command_creates() {
        assert_eq!(
            Command::CreateTrack.identity_need(),
            Some((IdentityDomain::Track, 1))
        );
        assert_eq!(
            Command::CreateClip {
                owner: track_one(),
                start: MusicalTime::ZERO,
                duration: beat(),
            }
            .identity_need(),
            Some((IdentityDomain::Clip, 1))
        );
        assert_eq!(
            Command::RemoveClip { clip: clip_one() }.identity_need(),
            None
        );
    }

    #[test]
    fn every_command_names_the_arrangement_aggregate() {
        assert!(Command::CreateTrack
            .effects()
            .contains(Aggregate::Arrangement));
        assert!(Command::RemoveClip { clip: clip_one() }
            .effects()
            .contains(Aggregate::Arrangement));
    }

    #[test]
    fn validation_does_not_mutate_the_arrangement() {
        let mut arrangement = Arrangement::new();
        arrangement.insert_track(track_one()).unwrap();
        let before = arrangement.clone();

        let _ = Command::CreateClip {
            owner: track_one(),
            start: MusicalTime::ZERO,
            duration: beat(),
        }
        .validate(&arrangement);
        let _ = Command::RemoveClip { clip: clip_one() }.validate(&arrangement);

        assert_eq!(format!("{before:?}"), format!("{arrangement:?}"));
    }
}
