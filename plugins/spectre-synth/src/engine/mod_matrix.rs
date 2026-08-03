// =============================================================================
// File: plugins/spectre-synth/src/engine/mod_matrix.rs
// Layer: synth plugin
// Purpose: internal mod matrix (env/lfo → any param)
// Status: Implemented; index-based routes summed into destination accumulators.
// Notes: Sources and destinations are opaque indices, so the matrix is reusable;
//        the synth assigns meanings. Resolved per block, never allocating.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// One modulation connection: scale a source into a destination
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModRoute {
    // Index into the source value slice
    pub source: usize,
    // Index into the destination accumulator slice
    pub dest: usize,
    // Signed amount applied to the source
    pub depth: f32,
    // True passes the source through; false rectifies it to its positive half
    pub bipolar: bool,
}

impl ModRoute {
    // Build a bipolar route (full-range source)
    pub fn bipolar(source: usize, dest: usize, depth: f32) -> Self {
        Self {
            source,
            dest,
            depth,
            bipolar: true,
        }
    }

    // Build a unipolar route (only the positive part of the source contributes)
    pub fn unipolar(source: usize, dest: usize, depth: f32) -> Self {
        Self {
            source,
            dest,
            depth,
            bipolar: false,
        }
    }
}

// Resolves a list of routes from source values into destination accumulators
#[derive(Clone, Debug, Default)]
pub struct ModMatrix {
    routes: Vec<ModRoute>,
}

impl ModMatrix {
    // Build an empty matrix
    pub fn new() -> Self {
        Self::default()
    }

    // Append a route
    pub fn add_route(&mut self, route: ModRoute) {
        self.routes.push(route);
    }

    // Number of routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    // Remove every route
    pub fn clear(&mut self) {
        self.routes.clear();
    }

    // Sum all routes from `sources` into `dests`, overwriting `dests`
    // Out-of-range indices are skipped; never allocates
    pub fn resolve(&self, sources: &[f32], dests: &mut [f32]) {
        for d in dests.iter_mut() {
            *d = 0.0;
        }
        for route in &self.routes {
            if route.source >= sources.len() || route.dest >= dests.len() {
                continue;
            }
            let value = if route.bipolar {
                sources[route.source]
            } else {
                sources[route.source].clamp(0.0, 1.0)
            };
            dests[route.dest] += value * route.depth;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_route_scales_source_into_dest() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(0, 1, 0.5));
        let sources = [1.0, 0.0];
        let mut dests = [0.0; 2];
        matrix.resolve(&sources, &mut dests);
        assert_eq!(dests[0], 0.0);
        assert_eq!(dests[1], 0.5);
    }

    #[test]
    fn routes_to_same_dest_sum() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(0, 0, 1.0));
        matrix.add_route(ModRoute::bipolar(1, 0, 0.5));
        let sources = [0.4, 0.8];
        let mut dests = [0.0; 1];
        matrix.resolve(&sources, &mut dests);
        assert!((dests[0] - (0.4 + 0.4)).abs() < 1e-6);
    }

    #[test]
    fn unipolar_rectifies_negative_sources() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::unipolar(0, 0, 1.0));
        let mut dests = [0.0; 1];

        matrix.resolve(&[-0.7], &mut dests);
        assert_eq!(dests[0], 0.0); // negative source rectified to zero

        matrix.resolve(&[0.6], &mut dests);
        assert!((dests[0] - 0.6).abs() < 1e-6);
    }

    #[test]
    fn bipolar_passes_negative_sources() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(0, 0, 1.0));
        let mut dests = [0.0; 1];
        matrix.resolve(&[-0.7], &mut dests);
        assert!((dests[0] + 0.7).abs() < 1e-6);
    }

    #[test]
    fn resolve_overwrites_previous_values() {
        let matrix = ModMatrix::new(); // no routes
        let mut dests = [3.0, 4.0];
        matrix.resolve(&[1.0], &mut dests);
        assert_eq!(dests, [0.0, 0.0]);
    }

    #[test]
    fn out_of_range_indices_are_skipped() {
        let mut matrix = ModMatrix::new();
        matrix.add_route(ModRoute::bipolar(5, 0, 1.0)); // bad source
        matrix.add_route(ModRoute::bipolar(0, 9, 1.0)); // bad dest
        matrix.add_route(ModRoute::bipolar(0, 0, 1.0)); // valid
        let mut dests = [0.0; 1];
        matrix.resolve(&[0.25], &mut dests);
        assert_eq!(dests[0], 0.25);
    }
}
