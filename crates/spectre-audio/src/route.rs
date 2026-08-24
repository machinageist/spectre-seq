// Author: Jeff
// Date: 2026-08-24
// Description: Render-side map from an RT-002 parameter target to a live plan node and key
// Notes: Built once on the app thread and never mutated. Lookup is a binary search over a
//   preallocated boxed slice: no allocation, no locks, no panic, bounded by log2 of the entry
//   count. This module is callback-reachable and is scanned by rt_guard's RT_MODULES.

use crate::control::ParameterTarget;
use spectre_dsp::DeviceParameterKey;
use spectre_graph::NodeId;

// One resolved route from lane identity to plan addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParameterRoute {
    pub target: ParameterTarget,
    pub node: NodeId,
    pub key: DeviceParameterKey,
}

// App-thread route-construction failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    DuplicateTarget(ParameterTarget),
}

impl std::fmt::Display for RouteError {
    // App-thread diagnostic; never formatted on the render path
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateTarget(target) => {
                write!(formatter, "duplicate parameter route for {target:?}")
            }
        }
    }
}

impl std::error::Error for RouteError {}

// Immutable sorted routing table; construction is app-thread and may allocate
#[derive(Debug, Default)]
pub struct ParameterRoutes {
    // Sorted by `target`, which derives Ord, so lookup is a binary search
    entries: Box<[ParameterRoute]>,
}

impl ParameterRoutes {
    // Sort and freeze a route set; duplicate targets are refused rather than shadowed, because a
    // shadowed route would silently send every edit of one parameter to the wrong device
    pub fn new(mut entries: Vec<ParameterRoute>) -> Result<Self, RouteError> {
        entries.sort_unstable_by_key(|entry| entry.target);
        if let Some(pair) = entries
            .windows(2)
            .find(|pair| pair[0].target == pair[1].target)
        {
            return Err(RouteError::DuplicateTarget(pair[0].target));
        }
        Ok(Self {
            entries: entries.into_boxed_slice(),
        })
    }

    // An empty table: every incoming target is unrouted and counted
    pub fn empty() -> Self {
        Self::default()
    }

    // Resolve one target; callback-safe, allocation-free, bounded by log2 of the entry count
    pub fn resolve(&self, target: ParameterTarget) -> Option<(NodeId, DeviceParameterKey)> {
        self.entries
            .binary_search_by_key(&target, |entry| entry.target)
            .ok()
            .map(|index| (self.entries[index].node, self.entries[index].key))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
