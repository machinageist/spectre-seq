// Author: Jeff
// Date: 2026-08-03
// Description: Rejection vocabulary for durable document transactions.
// Notes: Lives in core so persistence, UI, and document all name one set of reasons.

use crate::ids::{ClipId, TrackId};
use std::fmt;

// Name of one durable identity domain, for exhaustion and duplicate reporting
// A domain may be named before its aggregate content exists
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum IdentityDomain {
    Track,
    Clip,
    Asset,
    Scene,
    Device,
    Param,
    Route,
    Note,
    Mapping,
    AutomationTarget,
}

impl IdentityDomain {
    // Every domain the document allocates; iteration order is stable
    pub const ALL: [IdentityDomain; 10] = [
        IdentityDomain::Track,
        IdentityDomain::Clip,
        IdentityDomain::Asset,
        IdentityDomain::Scene,
        IdentityDomain::Device,
        IdentityDomain::Param,
        IdentityDomain::Route,
        IdentityDomain::Note,
        IdentityDomain::Mapping,
        IdentityDomain::AutomationTarget,
    ];

    // Lowercase name for diagnostics and user-facing messages
    pub const fn as_str(self) -> &'static str {
        match self {
            IdentityDomain::Track => "track",
            IdentityDomain::Clip => "clip",
            IdentityDomain::Asset => "asset",
            IdentityDomain::Scene => "scene",
            IdentityDomain::Device => "device",
            IdentityDomain::Param => "param",
            IdentityDomain::Route => "route",
            IdentityDomain::Note => "note",
            IdentityDomain::Mapping => "mapping",
            IdentityDomain::AutomationTarget => "automation target",
        }
    }
}

// Why a transaction was rejected
// The SPEC requires these seven to be distinguishable for tests and for
// user-facing messaging; a caller must never have to parse a string to tell
// "you referenced something deleted" from "you referenced something missing"
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TransactionError {
    // Target existed once but no longer does
    StaleIdentity { domain: IdentityDomain, raw: u64 },
    // Named owner is absent, so the child has nowhere to live
    MissingOwner { domain: IdentityDomain, raw: u64 },
    // Identity is already present where it must be unique
    DuplicateIdentity { domain: IdentityDomain, raw: u64 },
    // Musical or sample position failed range or ordering validation
    InvalidCoordinate(&'static str),
    // Duration is zero, negative, or otherwise unrepresentable
    InvalidDuration(&'static str),
    // Reference is preserved but its subject is absent; not an invalid document
    UnresolvedReference { domain: IdentityDomain, raw: u64 },
    // Domain cannot allocate further; other domains are unaffected
    IdentityExhausted(IdentityDomain),
}

impl TransactionError {
    // Which identity domain the rejection concerns, when it concerns one
    pub const fn domain(self) -> Option<IdentityDomain> {
        match self {
            TransactionError::StaleIdentity { domain, .. }
            | TransactionError::MissingOwner { domain, .. }
            | TransactionError::DuplicateIdentity { domain, .. }
            | TransactionError::UnresolvedReference { domain, .. }
            | TransactionError::IdentityExhausted(domain) => Some(domain),
            TransactionError::InvalidCoordinate(_) | TransactionError::InvalidDuration(_) => None,
        }
    }

    // Build a stale-identity rejection for a track
    pub fn stale_track(id: TrackId) -> Self {
        TransactionError::StaleIdentity {
            domain: IdentityDomain::Track,
            raw: id.raw(),
        }
    }

    // Build a stale-identity rejection for a clip
    pub fn stale_clip(id: ClipId) -> Self {
        TransactionError::StaleIdentity {
            domain: IdentityDomain::Clip,
            raw: id.raw(),
        }
    }

    // Build a missing-owner rejection naming the absent track
    pub fn missing_track(id: TrackId) -> Self {
        TransactionError::MissingOwner {
            domain: IdentityDomain::Track,
            raw: id.raw(),
        }
    }
}

impl fmt::Display for TransactionError {
    // Render a human-readable rejection for logs and UI
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::StaleIdentity { domain, raw } => {
                write!(f, "stale {} identity: {raw}", domain.as_str())
            }
            TransactionError::MissingOwner { domain, raw } => {
                write!(f, "missing {} owner: {raw}", domain.as_str())
            }
            TransactionError::DuplicateIdentity { domain, raw } => {
                write!(f, "duplicate {} identity: {raw}", domain.as_str())
            }
            TransactionError::InvalidCoordinate(why) => write!(f, "invalid coordinate: {why}"),
            TransactionError::InvalidDuration(why) => write!(f, "invalid duration: {why}"),
            TransactionError::UnresolvedReference { domain, raw } => {
                write!(f, "unresolved {} reference: {raw}", domain.as_str())
            }
            TransactionError::IdentityExhausted(domain) => {
                write!(f, "{} identity exhausted", domain.as_str())
            }
        }
    }
}

impl std::error::Error for TransactionError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Every variant the SPEC requires to be distinguishable
    fn one_of_each() -> [TransactionError; 7] {
        [
            TransactionError::StaleIdentity {
                domain: IdentityDomain::Clip,
                raw: 3,
            },
            TransactionError::MissingOwner {
                domain: IdentityDomain::Track,
                raw: 4,
            },
            TransactionError::DuplicateIdentity {
                domain: IdentityDomain::Note,
                raw: 5,
            },
            TransactionError::InvalidCoordinate("start before zero"),
            TransactionError::InvalidDuration("zero ticks"),
            TransactionError::UnresolvedReference {
                domain: IdentityDomain::Asset,
                raw: 6,
            },
            TransactionError::IdentityExhausted(IdentityDomain::Scene),
        ]
    }

    #[test]
    fn seven_rejection_reasons_are_mutually_distinguishable() {
        let cases = one_of_each();
        for (i, a) in cases.iter().enumerate() {
            for (j, b) in cases.iter().enumerate() {
                assert_eq!(i == j, a == b, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn display_is_non_empty_for_each_variant() {
        for err in one_of_each() {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn same_reason_in_different_domains_does_not_compare_equal() {
        let clip = TransactionError::IdentityExhausted(IdentityDomain::Clip);
        let track = TransactionError::IdentityExhausted(IdentityDomain::Track);
        assert_ne!(clip, track);
    }

    #[test]
    fn domain_is_reported_only_where_one_applies() {
        assert_eq!(
            TransactionError::stale_clip(ClipId::new(9).unwrap()).domain(),
            Some(IdentityDomain::Clip)
        );
        assert_eq!(
            TransactionError::missing_track(TrackId::new(2).unwrap()).domain(),
            Some(IdentityDomain::Track)
        );
        assert_eq!(TransactionError::InvalidDuration("zero").domain(), None);
    }

    #[test]
    fn every_domain_has_a_distinct_name() {
        let mut names: Vec<&str> = IdentityDomain::ALL.iter().map(|d| d.as_str()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn constructors_carry_the_raw_value_through() {
        let err = TransactionError::stale_track(TrackId::new(77).unwrap());
        assert_eq!(
            err,
            TransactionError::StaleIdentity {
                domain: IdentityDomain::Track,
                raw: 77
            }
        );
    }
}
