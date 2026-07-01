// Author: Jeff
// Date: 2026-05-27
// Description: Stable newtype identifiers shared by graph, timeline, automation, and UI layers.
// Notes: IDs are opaque handles; allocation policy belongs to owning collections.

// Generate one opaque u64 identifier newtype with constructor and raw accessor
// Derives give deterministic maps, ordering, and equality for tests
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        pub struct $name(u64);

        impl $name {
            // Wrap a raw value as a typed identifier
            #[inline]
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            // Return the underlying raw value
            #[inline]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

// Identifies a node in the audio process graph
define_id!(NodeId);
// Identifies a single port on a node
define_id!(PortId);
// Identifies an automatable/modulatable parameter
define_id!(ParamId);
// Identifies an internal or hosted device instance
define_id!(DeviceId);
// Identifies a clip in the timeline arena
define_id!(ClipId);
// Identifies a track in the arrangement
define_id!(TrackId);
// Identifies a top-level project
define_id!(ProjectId);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn raw_round_trips() {
        assert_eq!(NodeId::new(7).raw(), 7);
        assert_eq!(PortId::new(u64::MAX).raw(), u64::MAX);
        assert_eq!(ParamId::new(0).raw(), 0);
    }

    #[test]
    fn equality_and_ordering_follow_raw_value() {
        assert_eq!(NodeId::new(3), NodeId::new(3));
        assert_ne!(NodeId::new(3), NodeId::new(4));
        assert!(NodeId::new(1) < NodeId::new(2));
    }

    #[test]
    fn hashes_as_map_key() {
        let mut map = HashMap::new();
        map.insert(TrackId::new(42), "lead");
        assert_eq!(map.get(&TrackId::new(42)), Some(&"lead"));
        assert_eq!(map.get(&TrackId::new(43)), None);
    }
}
