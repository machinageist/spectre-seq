// Author: Jeff
// Date: 2026-08-03
// Description: Monotonic document revision and the set of aggregates a transaction touched.
// Notes: EffectSet is load-bearing twice: projections rebuild by it, and undo snapshots scope to it.

use std::fmt;

// One aggregate of the project document
// The table in docs/changes/project-document/SPEC.md is the source of this list;
// `clips` and the re-scoped `arrangement` come from the accepted clip-content amendment
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Aggregate {
    Identity,
    Meta,
    Tracks,
    Clips,
    Arrangement,
    Launcher,
    Graph,
    Assets,
    Conductor,
    Automation,
    Mappings,
}

impl Aggregate {
    // Every aggregate, in a stable order
    pub const ALL: [Aggregate; 11] = [
        Aggregate::Identity,
        Aggregate::Meta,
        Aggregate::Tracks,
        Aggregate::Clips,
        Aggregate::Arrangement,
        Aggregate::Launcher,
        Aggregate::Graph,
        Aggregate::Assets,
        Aggregate::Conductor,
        Aggregate::Automation,
        Aggregate::Mappings,
    ];

    // Bit position in an EffectSet mask
    const fn bit(self) -> u16 {
        1 << (self as u16)
    }

    // Lowercase name for diagnostics
    pub const fn as_str(self) -> &'static str {
        match self {
            Aggregate::Identity => "identity",
            Aggregate::Meta => "meta",
            Aggregate::Tracks => "tracks",
            Aggregate::Clips => "clips",
            Aggregate::Arrangement => "arrangement",
            Aggregate::Launcher => "launcher",
            Aggregate::Graph => "graph",
            Aggregate::Assets => "assets",
            Aggregate::Conductor => "conductor",
            Aggregate::Automation => "automation",
            Aggregate::Mappings => "mappings",
        }
    }
}

// Which aggregates one accepted transaction changed
// A bitmask rather than a Vec so building one allocates nothing
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct EffectSet {
    mask: u16,
}

impl EffectSet {
    // No aggregate changed; the identity value for accumulation
    pub const fn empty() -> Self {
        Self { mask: 0 }
    }

    // Record that one aggregate changed
    pub fn insert(&mut self, aggregate: Aggregate) {
        self.mask |= aggregate.bit();
    }

    // Build a set from one aggregate
    pub fn of(aggregate: Aggregate) -> Self {
        let mut set = Self::empty();
        set.insert(aggregate);
        set
    }

    // Did this aggregate change
    pub const fn contains(self, aggregate: Aggregate) -> bool {
        self.mask & aggregate.bit() != 0
    }

    // Nothing changed at all
    pub const fn is_empty(self) -> bool {
        self.mask == 0
    }

    // Every aggregate named by either set
    pub const fn union(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    // Aggregates named by this set, in stable order
    pub fn iter(self) -> impl Iterator<Item = Aggregate> {
        Aggregate::ALL
            .into_iter()
            .filter(move |a| self.contains(*a))
    }
}

impl fmt::Display for EffectSet {
    // Render the named aggregates for logs
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "(none)");
        }
        let mut first = true;
        for aggregate in self.iter() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", aggregate.as_str())?;
            first = false;
        }
        Ok(())
    }
}

// Monotonic count of accepted transactions applied to a document
// An accepted transaction advances it by exactly one; a rejected one does not move it
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug)]
pub struct DocumentRevision(u64);

impl DocumentRevision {
    // A document that has accepted no transaction
    pub const fn initial() -> Self {
        Self(0)
    }

    // The revision after one accepted transaction
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    // Underlying count, for persistence and dirty-state comparison
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for DocumentRevision {
    // Render as a bare count
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_document_starts_at_revision_zero() {
        assert_eq!(DocumentRevision::initial().raw(), 0);
    }

    #[test]
    fn revision_advances_by_exactly_one() {
        let start = DocumentRevision::initial();
        assert_eq!(start.next().raw(), 1);
        assert_eq!(start.next().next().raw(), 2);
    }

    #[test]
    fn revisions_order_by_age() {
        let old = DocumentRevision::initial();
        let new = old.next();
        assert!(new > old);
    }

    #[test]
    fn an_empty_effect_set_names_nothing() {
        let set = EffectSet::empty();
        assert!(set.is_empty());
        assert_eq!(set.iter().count(), 0);
        for aggregate in Aggregate::ALL {
            assert!(!set.contains(aggregate));
        }
    }

    #[test]
    fn each_aggregate_occupies_its_own_bit() {
        for aggregate in Aggregate::ALL {
            let set = EffectSet::of(aggregate);
            for other in Aggregate::ALL {
                assert_eq!(
                    aggregate == other,
                    set.contains(other),
                    "{aggregate:?} leaked into {other:?}"
                );
            }
        }
    }

    #[test]
    fn insert_accumulates_without_clearing() {
        let mut set = EffectSet::of(Aggregate::Clips);
        set.insert(Aggregate::Arrangement);
        assert!(set.contains(Aggregate::Clips));
        assert!(set.contains(Aggregate::Arrangement));
        assert!(!set.contains(Aggregate::Graph));
    }

    #[test]
    fn inserting_twice_is_idempotent() {
        let mut set = EffectSet::of(Aggregate::Tracks);
        set.insert(Aggregate::Tracks);
        assert_eq!(set.iter().count(), 1);
    }

    #[test]
    fn union_names_every_aggregate_from_both_sides() {
        let a = EffectSet::of(Aggregate::Meta);
        let b = EffectSet::of(Aggregate::Assets);
        let joined = a.union(b);
        assert!(joined.contains(Aggregate::Meta));
        assert!(joined.contains(Aggregate::Assets));
        assert_eq!(joined.iter().count(), 2);
    }

    #[test]
    fn iteration_order_is_stable_regardless_of_insertion_order() {
        let mut forward = EffectSet::empty();
        forward.insert(Aggregate::Mappings);
        forward.insert(Aggregate::Tracks);

        let mut backward = EffectSet::empty();
        backward.insert(Aggregate::Tracks);
        backward.insert(Aggregate::Mappings);

        let a: Vec<_> = forward.iter().collect();
        let b: Vec<_> = backward.iter().collect();
        assert_eq!(a, b);
        assert_eq!(a, vec![Aggregate::Tracks, Aggregate::Mappings]);
    }

    #[test]
    fn the_mask_has_room_for_every_aggregate() {
        // A u16 mask must cover every variant; adding a twelfth is fine, a
        // seventeenth silently wraps and this test is what catches it
        assert!(Aggregate::ALL.len() <= 16);
        let mut all = EffectSet::empty();
        for aggregate in Aggregate::ALL {
            all.insert(aggregate);
        }
        assert_eq!(all.iter().count(), Aggregate::ALL.len());
    }

    #[test]
    fn display_lists_names_and_marks_the_empty_set() {
        assert_eq!(EffectSet::empty().to_string(), "(none)");
        let mut set = EffectSet::of(Aggregate::Clips);
        set.insert(Aggregate::Arrangement);
        assert_eq!(set.to_string(), "clips, arrangement");
    }

    #[test]
    fn every_aggregate_has_a_distinct_name() {
        let mut names: Vec<&str> = Aggregate::ALL.iter().map(|a| a.as_str()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
