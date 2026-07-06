// =============================================================================
// File: crates/geist-synth/src/engine/mod_matrix.rs
// Layer: internal synth device
// Purpose: internal mod matrix (identity-addressed sources → synth mod targets)
// Status: Implemented; typed routes summed into destination accumulators.
// Notes: ModSource/ModTarget discriminants are append-only — persisted routes
//        rely on stable indices. Resolved per block, never allocating.
//        Executes AGENTS/changes/sound-design-depth PLAN M1.1 (serum spec §6.2).
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Stable modulation source identity; append variants only, never reorder
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModSource {
    Lfo1,
}

impl ModSource {
    // Number of source slots in a resolve input array
    pub const COUNT: usize = 1;

    // Map the source onto its slot in the resolve input array
    pub const fn index(self) -> usize {
        match self {
            ModSource::Lfo1 => 0,
        }
    }
}

// Stable modulation target identity; append variants only, never reorder
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModTarget {
    Cutoff,
    Pitch,
    Fm,
}

impl ModTarget {
    // Number of target slots in a resolve output array
    pub const COUNT: usize = 3;

    // Map the target onto its slot in the resolve output array
    pub const fn index(self) -> usize {
        match self {
            ModTarget::Cutoff => 0,
            ModTarget::Pitch => 1,
            ModTarget::Fm => 2,
        }
    }
}

// One modulation connection: scale a source into a target accumulator
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModRoute {
    // Source identity resolved against the input array
    pub source: ModSource,
    // Target identity resolved against the output array
    pub target: ModTarget,
    // Signed amount applied to the source
    pub depth: f32,
    // True passes the source through; false rectifies it to its positive half
    pub bipolar: bool,
    // False skips the route without removing it
    pub enabled: bool,
}

impl ModRoute {
    // Build an enabled bipolar route (full-range source)
    pub fn bipolar(source: ModSource, target: ModTarget, depth: f32) -> Self {
        Self {
            source,
            target,
            depth,
            bipolar: true,
            enabled: true,
        }
    }

    // Build an enabled unipolar route (only the positive part contributes)
    pub fn unipolar(source: ModSource, target: ModTarget, depth: f32) -> Self {
        Self {
            source,
            target,
            depth,
            bipolar: false,
            enabled: true,
        }
    }
}

// Small fixed route count for realtime-safe per-block resolution
const MAX_MOD_ROUTES: usize = 8;

// Resolves a list of routes from source values into target accumulators
#[derive(Clone, Debug)]
pub struct ModMatrix {
    routes: [Option<ModRoute>; MAX_MOD_ROUTES],
}

impl Default for ModMatrix {
    fn default() -> Self {
        Self {
            routes: [None; MAX_MOD_ROUTES],
        }
    }
}

impl ModMatrix {
    // Build an empty matrix
    pub fn new() -> Self {
        Self::default()
    }

    // Append a route if a fixed slot is available
    pub fn add_route(&mut self, route: ModRoute) {
        if let Some(slot) = self.routes.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(route);
        }
    }

    // Number of routes
    pub fn route_count(&self) -> usize {
        self.routes.iter().filter(|route| route.is_some()).count()
    }

    // Remove every route
    pub fn clear(&mut self) {
        self.routes = [None; MAX_MOD_ROUTES];
    }

    // Sum all enabled routes from `sources` into `dests`, overwriting `dests`
    // Typed array lengths make indices infallible; never allocates
    pub fn resolve(&self, sources: &[f32; ModSource::COUNT], dests: &mut [f32; ModTarget::COUNT]) {
        for d in dests.iter_mut() {
            *d = 0.0;
        }
        for route in self.routes.iter().flatten() {
            if !route.enabled {
                continue;
            }
            let raw = sources[route.source.index()];
            let value = if route.bipolar {
                raw
            } else {
                raw.clamp(0.0, 1.0)
            };
            dests[route.target.index()] += value * route.depth;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_route_scales_source_into_target() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(ModSource::Lfo1, ModTarget::Pitch, 0.5));
        let sources = [1.0];
        let mut dests = [0.0; ModTarget::COUNT];
        matrix.resolve(&sources, &mut dests);
        assert_eq!(dests[ModTarget::Cutoff.index()], 0.0);
        assert_eq!(dests[ModTarget::Pitch.index()], 0.5);
    }

    #[test]
    fn routes_to_same_target_sum() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(ModSource::Lfo1, ModTarget::Cutoff, 1.0));
        matrix.add_route(ModRoute::bipolar(ModSource::Lfo1, ModTarget::Cutoff, 0.5));
        let mut dests = [0.0; ModTarget::COUNT];
        matrix.resolve(&[0.4], &mut dests);
        assert!((dests[ModTarget::Cutoff.index()] - (0.4 + 0.2)).abs() < 1e-6);
    }

    #[test]
    fn unipolar_rectifies_negative_sources() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::unipolar(ModSource::Lfo1, ModTarget::Cutoff, 1.0));
        let mut dests = [0.0; ModTarget::COUNT];

        matrix.resolve(&[-0.7], &mut dests);
        assert_eq!(dests[ModTarget::Cutoff.index()], 0.0); // rectified to zero

        matrix.resolve(&[0.6], &mut dests);
        assert!((dests[ModTarget::Cutoff.index()] - 0.6).abs() < 1e-6);
    }

    #[test]
    fn bipolar_passes_negative_sources() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(ModSource::Lfo1, ModTarget::Cutoff, 1.0));
        let mut dests = [0.0; ModTarget::COUNT];
        matrix.resolve(&[-0.7], &mut dests);
        assert!((dests[ModTarget::Cutoff.index()] + 0.7).abs() < 1e-6);
    }

    #[test]
    fn resolve_overwrites_previous_values() {
        let matrix = ModMatrix::new(); // no routes
        let mut dests = [3.0, 4.0, 5.0];
        matrix.resolve(&[1.0], &mut dests);
        assert_eq!(dests, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn disabled_route_is_skipped_without_removal() {
        let mut matrix = ModMatrix::new();
        let mut route = ModRoute::bipolar(ModSource::Lfo1, ModTarget::Fm, 1.0);
        route.enabled = false;
        matrix.add_route(route);
        let mut dests = [0.0; ModTarget::COUNT];
        matrix.resolve(&[0.9], &mut dests);
        assert_eq!(dests[ModTarget::Fm.index()], 0.0);
        assert_eq!(matrix.route_count(), 1);
    }

    #[test]
    fn identity_indices_are_stable_and_dense() {
        // Persistence relies on these exact values; appending is the only
        // legal way to grow either enum
        assert_eq!(ModSource::Lfo1.index(), 0);
        assert_eq!(ModTarget::Cutoff.index(), 0);
        assert_eq!(ModTarget::Pitch.index(), 1);
        assert_eq!(ModTarget::Fm.index(), 2);
    }
}
