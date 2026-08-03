// =============================================================================
// Author: Jeff
// Date: 2026-07-31
// Description: Stable canonical identity across every durable domain.
// Notes: IDs are never zero or reused; allocation is app-thread-owned and non-cloneable.
//
// File: crates/spectre-document/src/identity.rs
// Layer: document
// Purpose: Nonzero stable IDs and checked app-thread allocation
// Status: Implemented; ten independent checked sequences with atomic batch reservation.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

pub use spectre_core::ids::{
    AssetId, AutomationTargetId, ClipId, DeviceId, MappingId, NoteId, ParamKey, RouteId, SceneId,
    TrackId,
};
use spectre_core::transaction::{IdentityDomain, TransactionError};

// One monotonic nonzero sequence; zero marks permanent exhaustion
#[derive(Debug)]
struct IdSequence {
    next: u64,
}

impl IdSequence {
    const fn new() -> Self {
        Self { next: 1 }
    }

    fn allocate(&mut self) -> Option<u64> {
        if self.next == 0 {
            return None;
        }
        let id = self.next;
        self.next = self.next.checked_add(1).unwrap_or(0);
        Some(id)
    }

    fn observe(&mut self, id: u64) {
        if self.next != 0 && id >= self.next {
            self.next = id.checked_add(1).unwrap_or(0);
        }
    }

    // Can this sequence yield `count` more IDs without exhausting
    // Checked before a batch starts so a partial batch is never handed out
    fn can_supply(&self, count: u64) -> bool {
        if count == 0 {
            return true;
        }
        if self.next == 0 {
            return false;
        }
        // The last ID the batch would consume, if it fits at all
        self.next.checked_add(count - 1).is_some()
    }
}

impl Default for IdSequence {
    fn default() -> Self {
        Self::new()
    }
}

// App-thread allocator owning one independent sequence per durable domain
//
// Deliberately not Clone: two copies would hand out the same IDs, and the
// document is the single owner. Cloning it is the bug this absence prevents.
#[derive(Debug)]
pub struct IdentityAllocator {
    sequences: [IdSequence; IdentityDomain::ALL.len()],
}

impl Default for IdentityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityAllocator {
    // Build independent sequences beginning at one
    pub fn new() -> Self {
        Self {
            sequences: std::array::from_fn(|_| IdSequence::new()),
        }
    }

    fn sequence(&mut self, domain: IdentityDomain) -> &mut IdSequence {
        &mut self.sequences[domain as usize]
    }

    // Allocate the next raw ID in a domain, or report which domain is exhausted
    pub fn allocate_raw(&mut self, domain: IdentityDomain) -> Result<u64, TransactionError> {
        self.sequence(domain)
            .allocate()
            .ok_or(TransactionError::IdentityExhausted(domain))
    }

    // Advance a domain past an ID restored from durable state
    pub fn observe_raw(&mut self, domain: IdentityDomain, raw: u64) {
        self.sequence(domain).observe(raw);
    }

    // Reserve `count` IDs in one domain, all-or-nothing
    //
    // The pre-check is the whole point: a batch that does not fully fit must
    // fail before the allocator moves, so no caller observes half a batch
    pub fn reserve_raw(
        &mut self,
        domain: IdentityDomain,
        count: usize,
    ) -> Result<Vec<u64>, TransactionError> {
        if !self.sequence(domain).can_supply(count as u64) {
            return Err(TransactionError::IdentityExhausted(domain));
        }
        let sequence = self.sequence(domain);
        let mut reserved = Vec::with_capacity(count);
        for _ in 0..count {
            // can_supply already proved every one of these succeeds
            let raw = sequence
                .allocate()
                .expect("can_supply guaranteed the batch fits");
            reserved.push(raw);
        }
        Ok(reserved)
    }

    // Next value a domain would hand out; zero once exhausted
    // Tests assert on this to prove a rejection advanced nothing
    pub fn peek_raw(&self, domain: IdentityDomain) -> u64 {
        self.sequences[domain as usize].next
    }
}

// Typed allocate/observe/reserve triples, one per durable domain
// A macro so a new domain cannot be added with a mismatched pairing
macro_rules! typed_identity_accessors {
    ($( $domain:ident => $ty:ty, $allocate:ident, $observe:ident, $reserve:ident; )*) => {
        impl IdentityAllocator {
            $(
                // Allocate the next ID in this domain or report exhaustion
                pub fn $allocate(&mut self) -> Result<$ty, TransactionError> {
                    let raw = self.allocate_raw(IdentityDomain::$domain)?;
                    <$ty>::new(raw)
                        .ok_or(TransactionError::IdentityExhausted(IdentityDomain::$domain))
                }

                // Advance this domain past an ID restored from durable state
                pub fn $observe(&mut self, id: $ty) {
                    self.observe_raw(IdentityDomain::$domain, id.raw());
                }

                // Reserve a batch in this domain, all-or-nothing
                pub fn $reserve(&mut self, count: usize) -> Result<Vec<$ty>, TransactionError> {
                    let raws = self.reserve_raw(IdentityDomain::$domain, count)?;
                    Ok(raws.into_iter().filter_map(<$ty>::new).collect())
                }
            )*
        }
    };
}

typed_identity_accessors! {
    Track => TrackId, allocate_track, observe_track, reserve_tracks;
    Clip => ClipId, allocate_clip, observe_clip, reserve_clips;
    Asset => AssetId, allocate_asset, observe_asset, reserve_assets;
    Scene => SceneId, allocate_scene, observe_scene, reserve_scenes;
    Device => DeviceId, allocate_device, observe_device, reserve_devices;
    Param => ParamKey, allocate_param, observe_param, reserve_params;
    Route => RouteId, allocate_route, observe_route, reserve_routes;
    Note => NoteId, allocate_note, observe_note, reserve_notes;
    Mapping => MappingId, allocate_mapping, observe_mapping, reserve_mappings;
    AutomationTarget => AutomationTargetId, allocate_automation_target,
        observe_automation_target, reserve_automation_targets;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_not_a_canonical_id() {
        assert_eq!(TrackId::new(0), None);
        assert_eq!(ClipId::new(0), None);
        assert_eq!(NoteId::new(0), None);
        assert_eq!(AssetId::new(0), None);
    }

    #[test]
    fn valid_raw_ids_round_trip() {
        assert_eq!(TrackId::new(41).unwrap().raw(), 41);
        assert_eq!(ClipId::new(73).unwrap().raw(), 73);
    }

    #[test]
    fn every_domain_starts_at_one() {
        let mut ids = IdentityAllocator::new();
        for domain in IdentityDomain::ALL {
            assert_eq!(ids.allocate_raw(domain).unwrap(), 1, "{domain:?}");
        }
    }

    #[test]
    fn domains_advance_independently() {
        let mut ids = IdentityAllocator::new();
        assert_eq!(ids.allocate_track().unwrap().raw(), 1);
        assert_eq!(ids.allocate_track().unwrap().raw(), 2);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 1);
        assert_eq!(ids.allocate_note().unwrap().raw(), 1);
        assert_eq!(ids.allocate_track().unwrap().raw(), 3);
    }

    #[test]
    fn observed_ids_advance_without_moving_backward() {
        let mut ids = IdentityAllocator::new();
        ids.observe_track(TrackId::new(80).unwrap());
        ids.observe_track(TrackId::new(12).unwrap());
        ids.observe_clip(ClipId::new(120).unwrap());
        ids.observe_clip(ClipId::new(20).unwrap());

        assert_eq!(ids.allocate_track().unwrap().raw(), 81);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 121);
    }

    #[test]
    fn allocated_ids_are_not_reused() {
        let mut ids = IdentityAllocator::new();
        let deleted = ids.allocate_clip().unwrap();
        let next = ids.allocate_clip().unwrap();
        assert_ne!(deleted, next);
        assert_eq!(ids.allocate_clip().unwrap().raw(), 3);
    }

    #[test]
    fn exhaustion_names_its_domain_and_does_not_wrap_to_zero() {
        let mut ids = IdentityAllocator::new();
        ids.sequences[IdentityDomain::Clip as usize].next = u64::MAX;

        assert_eq!(ids.allocate_clip().unwrap().raw(), u64::MAX);
        assert_eq!(
            ids.allocate_clip(),
            Err(TransactionError::IdentityExhausted(IdentityDomain::Clip))
        );
        assert_eq!(
            ids.allocate_clip(),
            Err(TransactionError::IdentityExhausted(IdentityDomain::Clip))
        );
    }

    #[test]
    fn exhausting_one_domain_leaves_the_others_alone() {
        let mut ids = IdentityAllocator::new();
        ids.observe_clip(ClipId::new(u64::MAX).unwrap());

        assert!(ids.allocate_clip().is_err());
        for domain in IdentityDomain::ALL {
            if domain == IdentityDomain::Clip {
                continue;
            }
            assert_eq!(ids.allocate_raw(domain).unwrap(), 1, "{domain:?}");
        }
    }

    #[test]
    fn observing_an_older_id_does_not_revive_an_exhausted_sequence() {
        let mut ids = IdentityAllocator::new();
        ids.sequences[IdentityDomain::Clip as usize].next = u64::MAX;
        assert_eq!(ids.allocate_clip().unwrap().raw(), u64::MAX);

        ids.observe_clip(ClipId::new(12).unwrap());
        assert!(ids.allocate_clip().is_err());
    }

    #[test]
    fn a_batch_reserves_contiguous_ids() {
        let mut ids = IdentityAllocator::new();
        let notes = ids.reserve_notes(4).unwrap();
        let raws: Vec<u64> = notes.iter().map(|n| n.raw()).collect();
        assert_eq!(raws, vec![1, 2, 3, 4]);
        assert_eq!(ids.allocate_note().unwrap().raw(), 5);
    }

    #[test]
    fn an_empty_batch_succeeds_and_advances_nothing() {
        let mut ids = IdentityAllocator::new();
        assert!(ids.reserve_notes(0).unwrap().is_empty());
        assert_eq!(ids.peek_raw(IdentityDomain::Note), 1);
    }

    // The invariant the SPEC names directly: a batch that does not fully fit
    // fails before the allocator moves at all
    #[test]
    fn an_unavailable_batch_fails_atomically_without_advancing() {
        let mut ids = IdentityAllocator::new();
        // Room for exactly two more IDs: u64::MAX - 1 and u64::MAX
        ids.sequences[IdentityDomain::Note as usize].next = u64::MAX - 1;
        let before = ids.peek_raw(IdentityDomain::Note);

        assert_eq!(
            ids.reserve_notes(3),
            Err(TransactionError::IdentityExhausted(IdentityDomain::Note))
        );
        assert_eq!(
            ids.peek_raw(IdentityDomain::Note),
            before,
            "a failed batch advanced the allocator"
        );

        // The batch that does fit still works afterwards
        assert_eq!(ids.reserve_notes(2).unwrap().len(), 2);
    }

    #[test]
    fn a_batch_against_an_exhausted_domain_fails_without_panicking() {
        let mut ids = IdentityAllocator::new();
        ids.sequences[IdentityDomain::Scene as usize].next = 0;
        assert_eq!(
            ids.reserve_scenes(1),
            Err(TransactionError::IdentityExhausted(IdentityDomain::Scene))
        );
    }

    #[test]
    fn peek_reports_the_next_value_without_consuming_it() {
        let mut ids = IdentityAllocator::new();
        assert_eq!(ids.peek_raw(IdentityDomain::Device), 1);
        assert_eq!(ids.peek_raw(IdentityDomain::Device), 1);
        ids.allocate_device().unwrap();
        assert_eq!(ids.peek_raw(IdentityDomain::Device), 2);
    }

    #[test]
    fn every_typed_domain_allocates_reserves_and_observes() {
        let mut ids = IdentityAllocator::new();
        assert_eq!(ids.allocate_asset().unwrap().raw(), 1);
        assert_eq!(ids.allocate_scene().unwrap().raw(), 1);
        assert_eq!(ids.allocate_device().unwrap().raw(), 1);
        assert_eq!(ids.allocate_param().unwrap().raw(), 1);
        assert_eq!(ids.allocate_route().unwrap().raw(), 1);
        assert_eq!(ids.allocate_mapping().unwrap().raw(), 1);
        assert_eq!(ids.allocate_automation_target().unwrap().raw(), 1);

        assert_eq!(ids.reserve_assets(2).unwrap().len(), 2);
        assert_eq!(ids.reserve_routes(3).unwrap().len(), 3);

        ids.observe_automation_target(AutomationTargetId::new(500).unwrap());
        assert_eq!(ids.allocate_automation_target().unwrap().raw(), 501);
    }
}
