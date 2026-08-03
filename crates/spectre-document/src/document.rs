// =============================================================================
// Author: Jeff
// Date: 2026-08-03
// Description: The canonical app-thread ProjectDocument, its transactions, and its history.
// Notes: Single owner of durable truth; UI, persistence, and realtime state are projections.
//
// File: crates/spectre-document/src/document.rs
// Layer: document
// Purpose: Aggregate ownership, transaction execution, undo/redo, dirty state
// Status: Implemented; aggregates beyond arrangement are empty until their sub-specs land.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::transaction::TransactionError;

use crate::arrangement::{Arrangement, ClipEntity};
use crate::command::Command;
use crate::identity::IdentityAllocator;
use crate::revision::{Aggregate, DocumentRevision, EffectSet};

// Outcome of submitting one command
// Never a silent no-op: an unchanged document always carries a reason
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionResult {
    Accepted {
        revision: DocumentRevision,
        effects: EffectSet,
    },
    Rejected {
        reason: TransactionError,
    },
}

impl TransactionResult {
    // Did the document change
    pub const fn is_accepted(&self) -> bool {
        matches!(self, TransactionResult::Accepted { .. })
    }

    // Why the document did not change
    pub const fn rejection(&self) -> Option<TransactionError> {
        match self {
            TransactionResult::Rejected { reason } => Some(*reason),
            TransactionResult::Accepted { .. } => None,
        }
    }
}

// Copy of the aggregates one transaction was about to change
//
// Scoped by EffectSet rather than snapshotting the whole document, so cost
// tracks what actually moved. The identity allocator is deliberately absent:
// undo restores the original IDs and redo reuses them, so rolling the
// allocator back would let a retired ID be handed out twice.
#[derive(Clone, Debug, Default)]
struct BeforeImage {
    arrangement: Option<Arrangement>,
}

// One recorded durable edit
#[derive(Clone, Debug)]
struct HistoryEntry {
    command: Command,
    effects: EffectSet,
    before: BeforeImage,
}

// The single app-thread owner of durable project truth
//
// Aggregates whose content sub-spec has not landed are absent rather than
// stubbed. An empty aggregate is still the named authority for its domain.
#[derive(Debug)]
pub struct ProjectDocument {
    revision: DocumentRevision,
    // Revision at the last successful save; dirty state derives from it
    saved_revision: DocumentRevision,
    identity: IdentityAllocator,
    arrangement: Arrangement,
    done: Vec<HistoryEntry>,
    undone: Vec<HistoryEntry>,
}

impl Default for ProjectDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectDocument {
    // Build an empty document at revision zero
    pub fn new() -> Self {
        Self {
            revision: DocumentRevision::initial(),
            saved_revision: DocumentRevision::initial(),
            identity: IdentityAllocator::new(),
            arrangement: Arrangement::new(),
            done: Vec::new(),
            undone: Vec::new(),
        }
    }

    // Current revision; advances by exactly one per accepted transaction
    pub const fn revision(&self) -> DocumentRevision {
        self.revision
    }

    // Read-only arrangement aggregate
    pub const fn arrangement(&self) -> &Arrangement {
        &self.arrangement
    }

    // Read-only identity allocator, for tests and diagnostics
    pub const fn identity(&self) -> &IdentityAllocator {
        &self.identity
    }

    // Has the document changed since the last save
    //
    // Derived from the revision, never tracked independently by a pane. Undo
    // and redo advance the revision rather than rewinding it, because the SPEC
    // requires a monotonic counter and projections rely on that to tell whether
    // they are stale. The consequence is a conservative false positive: saving,
    // then undoing and redoing back to the same content still reads as dirty.
    // Erring toward offering a save is safe; the reverse is not.
    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    // Record that the current revision reached durable storage
    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    // Number of edits that can be undone
    pub fn undo_depth(&self) -> usize {
        self.done.len()
    }

    // Number of edits that can be redone
    pub fn redo_depth(&self) -> usize {
        self.undone.len()
    }

    // Submit one command: validate everything, then mutate or reject
    //
    // The order below is the contract. Validation and identity reservation both
    // run before a single byte of the document moves, so a rejection returns
    // with document, history, revision, and every allocator untouched.
    pub fn execute(&mut self, command: Command) -> TransactionResult {
        if let Err(reason) = command.validate(&self.arrangement) {
            return TransactionResult::Rejected { reason };
        }

        // Reserve identity as one checked batch; a partial batch fails here
        // with the allocator unmoved rather than leaking half of itself
        let reserved = match command.identity_need() {
            Some((domain, count)) => match self.identity.reserve_raw(domain, count) {
                Ok(raws) => raws,
                Err(reason) => return TransactionResult::Rejected { reason },
            },
            None => Vec::new(),
        };

        let effects = command.effects();
        let before = self.capture(effects);

        self.apply(&command, &reserved);

        self.revision = self.revision.next();
        self.done.push(HistoryEntry {
            command,
            effects,
            before,
        });
        // A new edit invalidates the redo branch
        self.undone.clear();

        TransactionResult::Accepted {
            revision: self.revision,
            effects,
        }
    }

    // Undo the most recent edit; a no-op when there is nothing to undo
    pub fn undo(&mut self) -> Option<DocumentRevision> {
        let entry = self.done.pop()?;
        let after = self.capture(entry.effects);
        self.restore(&entry.before);
        self.revision = self.revision.next();
        self.undone.push(HistoryEntry {
            command: entry.command,
            effects: entry.effects,
            before: after,
        });
        Some(self.revision)
    }

    // Redo the most recently undone edit
    //
    // Restores the state that edit produced rather than re-running it, so the
    // original identities come back instead of fresh ones being allocated
    pub fn redo(&mut self) -> Option<DocumentRevision> {
        let entry = self.undone.pop()?;
        let before = self.capture(entry.effects);
        self.restore(&entry.before);
        self.revision = self.revision.next();
        self.done.push(HistoryEntry {
            command: entry.command,
            effects: entry.effects,
            before,
        });
        Some(self.revision)
    }

    // Copy the aggregates an EffectSet names
    fn capture(&self, effects: EffectSet) -> BeforeImage {
        BeforeImage {
            arrangement: effects
                .contains(Aggregate::Arrangement)
                .then(|| self.arrangement.clone()),
        }
    }

    // Put captured aggregates back exactly as they were
    fn restore(&mut self, image: &BeforeImage) {
        if let Some(arrangement) = &image.arrangement {
            self.arrangement = arrangement.clone();
        }
    }

    // Mutate the document; cannot fail, because validate() already ran
    //
    // Every arrangement call here was proven to succeed by validation and by
    // the identity reservation above. An Err would mean validation and this
    // function disagree, which is a bug rather than a user-visible rejection.
    fn apply(&mut self, command: &Command, reserved: &[u64]) {
        match command {
            Command::CreateTrack => {
                let id = spectre_core::ids::TrackId::new(reserved[0])
                    .expect("reserved identities are nonzero");
                self.arrangement
                    .insert_track(id)
                    .expect("validated track insert cannot fail");
            }
            Command::CreateClip {
                owner,
                start,
                duration,
            } => {
                let id = spectre_core::ids::ClipId::new(reserved[0])
                    .expect("reserved identities are nonzero");
                let entity = ClipEntity::new(id, *owner, *start, *duration)
                    .expect("validated clip fields cannot fail");
                self.arrangement
                    .insert_clip(entity)
                    .expect("validated clip insert cannot fail");
            }
            Command::RemoveClip { clip } => {
                self.arrangement
                    .remove_clip(*clip)
                    .expect("validated clip removal cannot fail");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::ids::{ClipId, TrackId};
    use spectre_core::time::MusicalTime;
    use spectre_core::transaction::IdentityDomain;

    fn beat() -> MusicalTime {
        MusicalTime::from_ticks(960)
    }

    // A document with one track and one clip on it
    fn seeded() -> (ProjectDocument, TrackId, ClipId) {
        let mut doc = ProjectDocument::new();
        assert!(doc.execute(Command::CreateTrack).is_accepted());
        let track = doc.arrangement().tracks()[0].id();
        assert!(doc
            .execute(Command::CreateClip {
                owner: track,
                start: MusicalTime::ZERO,
                duration: beat(),
            })
            .is_accepted());
        let clip = doc.arrangement().track(track).unwrap().clip_ids()[0];
        (doc, track, clip)
    }

    #[test]
    fn a_new_document_is_empty_clean_and_at_revision_zero() {
        let doc = ProjectDocument::new();
        assert_eq!(doc.revision(), DocumentRevision::initial());
        assert!(doc.arrangement().tracks().is_empty());
        assert!(!doc.is_dirty());
        assert_eq!(doc.undo_depth(), 0);
        assert_eq!(doc.redo_depth(), 0);
    }

    #[test]
    fn an_accepted_transaction_advances_the_revision_by_exactly_one() {
        let mut doc = ProjectDocument::new();
        let result = doc.execute(Command::CreateTrack);
        match result {
            TransactionResult::Accepted { revision, effects } => {
                assert_eq!(revision.raw(), 1);
                assert!(effects.contains(Aggregate::Arrangement));
            }
            TransactionResult::Rejected { reason } => panic!("unexpected rejection: {reason}"),
        }
        assert!(doc.execute(Command::CreateTrack).is_accepted());
        assert_eq!(doc.revision().raw(), 2);
    }

    // The invariant the plan names first: a rejection changes nothing at all
    #[test]
    fn a_rejected_transaction_leaves_document_history_revision_and_allocators_unchanged() {
        let (mut doc, _track, _clip) = seeded();

        let revision_before = doc.revision();
        let arrangement_before = format!("{:?}", doc.arrangement());
        let undo_before = doc.undo_depth();
        let redo_before = doc.redo_depth();
        let allocators_before: Vec<u64> = IdentityDomain::ALL
            .iter()
            .map(|d| doc.identity().peek_raw(*d))
            .collect();

        // Missing owner: the track id is one past anything allocated
        let ghost = TrackId::new(9_999).unwrap();
        let result = doc.execute(Command::CreateClip {
            owner: ghost,
            start: MusicalTime::ZERO,
            duration: beat(),
        });

        assert_eq!(
            result.rejection(),
            Some(TransactionError::missing_track(ghost))
        );
        assert_eq!(doc.revision(), revision_before);
        assert_eq!(format!("{:?}", doc.arrangement()), arrangement_before);
        assert_eq!(doc.undo_depth(), undo_before);
        assert_eq!(doc.redo_depth(), redo_before);

        let allocators_after: Vec<u64> = IdentityDomain::ALL
            .iter()
            .map(|d| doc.identity().peek_raw(*d))
            .collect();
        assert_eq!(
            allocators_after, allocators_before,
            "a rejected transaction advanced an allocator"
        );
    }

    #[test]
    fn a_rejected_transaction_does_not_clear_redo_history() {
        let (mut doc, _track, _clip) = seeded();
        doc.undo().unwrap();
        assert_eq!(doc.redo_depth(), 1);

        let result = doc.execute(Command::RemoveClip {
            clip: ClipId::new(4_242).unwrap(),
        });
        assert!(!result.is_accepted());
        assert_eq!(doc.redo_depth(), 1, "rejection cleared the redo branch");
    }

    #[test]
    fn a_new_accepted_transaction_clears_redo_history() {
        let (mut doc, _track, _clip) = seeded();
        doc.undo().unwrap();
        assert_eq!(doc.redo_depth(), 1);

        assert!(doc.execute(Command::CreateTrack).is_accepted());
        assert_eq!(doc.redo_depth(), 0);
    }

    #[test]
    fn undo_restores_the_exact_prior_arrangement() {
        let (mut doc, track, _clip) = seeded();
        let before_third_edit = format!("{:?}", doc.arrangement());

        assert!(doc
            .execute(Command::CreateClip {
                owner: track,
                start: beat(),
                duration: beat(),
            })
            .is_accepted());
        assert_ne!(format!("{:?}", doc.arrangement()), before_third_edit);

        doc.undo().unwrap();
        assert_eq!(format!("{:?}", doc.arrangement()), before_third_edit);
    }

    #[test]
    fn redo_restores_the_original_identity_rather_than_allocating() {
        let (mut doc, _track, clip) = seeded();
        let allocator_after_create = doc.identity().peek_raw(IdentityDomain::Clip);

        doc.undo().unwrap();
        assert!(doc.arrangement().clip(clip).is_none());

        doc.redo().unwrap();
        assert!(
            doc.arrangement().clip(clip).is_some(),
            "redo did not restore the original ClipId"
        );
        assert_eq!(
            doc.identity().peek_raw(IdentityDomain::Clip),
            allocator_after_create,
            "redo allocated a replacement identity"
        );
    }

    #[test]
    fn undo_does_not_rewind_the_allocator_so_ids_are_never_reused() {
        let (mut doc, track, _clip) = seeded();
        let retired = doc.arrangement().track(track).unwrap().clip_ids()[0];

        doc.undo().unwrap();
        assert!(doc
            .execute(Command::CreateClip {
                owner: track,
                start: MusicalTime::ZERO,
                duration: beat(),
            })
            .is_accepted());

        let fresh = doc.arrangement().track(track).unwrap().clip_ids()[0];
        assert_ne!(fresh, retired, "an undone identity was handed out again");
    }

    #[test]
    fn undo_and_redo_both_advance_the_revision() {
        let (mut doc, _track, _clip) = seeded();
        let after_edits = doc.revision();

        let undone = doc.undo().unwrap();
        assert_eq!(undone.raw(), after_edits.raw() + 1);

        let redone = doc.redo().unwrap();
        assert_eq!(redone.raw(), after_edits.raw() + 2);
    }

    #[test]
    fn undo_and_redo_are_no_ops_at_the_ends_of_history() {
        let mut doc = ProjectDocument::new();
        assert_eq!(doc.undo(), None);
        assert_eq!(doc.redo(), None);
        assert_eq!(doc.revision(), DocumentRevision::initial());
    }

    #[test]
    fn removing_a_clip_and_undoing_restores_it_with_the_same_identity() {
        let (mut doc, _track, clip) = seeded();
        assert!(doc.execute(Command::RemoveClip { clip }).is_accepted());
        assert!(doc.arrangement().clip(clip).is_none());

        doc.undo().unwrap();
        let restored = doc.arrangement().clip(clip).expect("clip was not restored");
        assert_eq!(restored.id(), clip);
    }

    #[test]
    fn dirty_state_derives_from_the_revision_last_saved() {
        let mut doc = ProjectDocument::new();
        assert!(!doc.is_dirty());

        assert!(doc.execute(Command::CreateTrack).is_accepted());
        assert!(doc.is_dirty());

        doc.mark_saved();
        assert!(!doc.is_dirty());

        // Undo moves the revision forward, so a saved document becomes dirty
        // again rather than appearing clean at a state it never saved
        doc.undo().unwrap();
        assert!(doc.is_dirty());
    }

    #[test]
    fn a_rejected_transaction_does_not_make_a_clean_document_dirty() {
        let mut doc = ProjectDocument::new();
        doc.mark_saved();
        let result = doc.execute(Command::RemoveClip {
            clip: ClipId::new(1).unwrap(),
        });
        assert!(!result.is_accepted());
        assert!(!doc.is_dirty());
    }

    #[test]
    fn identity_exhaustion_rejects_without_mutating_the_document() {
        let (mut doc, track, _clip) = seeded();
        let arrangement_before = format!("{:?}", doc.arrangement());
        let revision_before = doc.revision();

        // Drive the clip domain to exhaustion through the public surface
        doc.identity.observe_clip(ClipId::new(u64::MAX).unwrap());

        let result = doc.execute(Command::CreateClip {
            owner: track,
            start: MusicalTime::ZERO,
            duration: beat(),
        });

        assert_eq!(
            result.rejection(),
            Some(TransactionError::IdentityExhausted(IdentityDomain::Clip))
        );
        assert_eq!(format!("{:?}", doc.arrangement()), arrangement_before);
        assert_eq!(doc.revision(), revision_before);
        assert_eq!(doc.undo_depth(), 2);
    }

    #[test]
    fn exhausting_one_domain_does_not_block_another() {
        let mut doc = ProjectDocument::new();
        doc.identity.observe_clip(ClipId::new(u64::MAX).unwrap());
        assert!(doc.execute(Command::CreateTrack).is_accepted());
    }

    #[test]
    fn history_is_a_single_project_level_stack() {
        let (mut doc, track, _clip) = seeded();
        assert!(doc
            .execute(Command::CreateClip {
                owner: track,
                start: beat(),
                duration: beat(),
            })
            .is_accepted());
        assert_eq!(doc.undo_depth(), 3);

        doc.undo().unwrap();
        doc.undo().unwrap();
        doc.undo().unwrap();
        assert_eq!(doc.undo_depth(), 0);
        assert_eq!(doc.redo_depth(), 3);
        assert!(doc.arrangement().tracks().is_empty());
    }
}
