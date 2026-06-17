// =============================================================================
// File: crates/geist-graph/src/topology.rs
// Layer: audio graph
// Purpose: topological sort, cycle detection, and feedback-aware scheduling
// Status: Implemented; feedback edges become a one-block delay during compilation.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::collections::{BTreeMap, BTreeSet};

use geist_core::ids::NodeId;

use crate::graph::Graph;

// Deterministic execution order for the graph's nodes
// Err carries the nodes that remain inside one or more feedback cycles
// Caller inserts a one-block delay to break each cycle during compilation
pub fn topological_order(graph: &Graph) -> Result<Vec<NodeId>, Vec<NodeId>> {
    let mut successors: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    let mut in_degree: BTreeMap<NodeId, usize> = BTreeMap::new();

    // Seed every node so isolated nodes still appear in the order
    for node in graph.node_ids() {
        successors.entry(node).or_default();
        in_degree.entry(node).or_insert(0);
    }

    // Collapse port edges to deduplicated node-level edges, self-loops included
    let mut counted: BTreeSet<(NodeId, NodeId)> = BTreeSet::new();
    for edge in graph.edges() {
        let (Some(src), Some(dst)) = (graph.port_node(edge.src), graph.port_node(edge.dst)) else {
            continue;
        };
        if counted.insert((src, dst)) {
            successors.entry(src).or_default().insert(dst);
            *in_degree.entry(dst).or_insert(0) += 1;
        }
    }

    // Kahn's algorithm; the ready set is sorted so ties resolve by node id
    let mut ready: BTreeSet<NodeId> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&n, _)| n)
        .collect();
    let mut order = Vec::with_capacity(graph.node_count());

    while let Some(&node) = ready.iter().next() {
        ready.remove(&node);
        order.push(node);
        for &next in &successors[&node] {
            let degree = in_degree
                .get_mut(&next)
                .expect("successor has an in-degree entry");
            *degree -= 1;
            if *degree == 0 {
                ready.insert(next);
            }
        }
    }

    if order.len() == graph.node_count() {
        Ok(order)
    } else {
        let cycle = in_degree
            .iter()
            .filter(|(_, &d)| d > 0)
            .map(|(&n, _)| n)
            .collect();
        Err(cycle)
    }
}

// Execution order plus the feedback edges that close cycles
// A feedback edge reads its producer's previous-block output, a one-block delay
pub struct Schedule {
    pub order: Vec<NodeId>,
    // Producer -> consumer node pairs whose edge was deferred to break a cycle
    pub feedback: BTreeSet<(NodeId, NodeId)>,
}

// DFS visit state
#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Gray,
    Black,
}

// Build a feedback-aware schedule that never fails on cycles
// Back-edges become feedback: the consumer is ordered ahead of the producer,
// so it reads the producer's buffer holding the previous block (a one-block delay)
pub fn schedule(graph: &Graph) -> Schedule {
    let mut successors: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for node in graph.node_ids() {
        successors.entry(node).or_default();
    }
    for edge in graph.edges() {
        if let (Some(src), Some(dst)) = (graph.port_node(edge.src), graph.port_node(edge.dst)) {
            successors.entry(src).or_default().insert(dst);
        }
    }

    let mut color: BTreeMap<NodeId, Color> =
        graph.node_ids().map(|node| (node, Color::White)).collect();
    let mut postorder = Vec::with_capacity(graph.node_count());
    let mut feedback = BTreeSet::new();

    for node in graph.node_ids() {
        if color[&node] == Color::White {
            visit(node, &successors, &mut color, &mut postorder, &mut feedback);
        }
    }
    // Reverse postorder is the topological order; back-edges were skipped
    postorder.reverse();
    Schedule {
        order: postorder,
        feedback,
    }
}

// Depth-first visit recording postorder and back-edges
// Recursion depth equals the longest signal path; deep graphs are atypical
fn visit(
    node: NodeId,
    successors: &BTreeMap<NodeId, BTreeSet<NodeId>>,
    color: &mut BTreeMap<NodeId, Color>,
    postorder: &mut Vec<NodeId>,
    feedback: &mut BTreeSet<(NodeId, NodeId)>,
) {
    color.insert(node, Color::Gray);
    for &next in &successors[&node] {
        match color[&next] {
            Color::White => visit(next, successors, color, postorder, feedback),
            // next is still on the stack: node -> next closes a cycle
            Color::Gray => {
                feedback.insert((node, next));
            }
            Color::Black => {}
        }
    }
    color.insert(node, Color::Black);
    postorder.push(node);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::PortSpec;
    use crate::graph::tests::{add_stereo_node, DummyNode};
    use geist_core::port::PortType;

    #[test]
    fn linear_chain_orders_source_to_sink() {
        let mut g = Graph::new();
        let (a, _a_in, a_out) = add_stereo_node(&mut g);
        let (b, b_in, b_out) = add_stereo_node(&mut g);
        let (c, c_in, _c_out) = add_stereo_node(&mut g);
        g.connect(a_out, b_in).unwrap();
        g.connect(b_out, c_in).unwrap();
        assert_eq!(topological_order(&g).unwrap(), vec![a, b, c]);
    }

    #[test]
    fn diamond_keeps_source_first_and_sink_last() {
        let mut g = Graph::new();
        let (a, _a_in, a_out) = add_stereo_node(&mut g);
        let (_b, b_in, b_out) = add_stereo_node(&mut g);
        let (_c, c_in, c_out) = add_stereo_node(&mut g);
        // Sink with two inputs so b and c can both feed it
        let d = g.add_node(
            Box::new(DummyNode),
            &[
                PortSpec::new(PortType::Audio, 2, "in0"),
                PortSpec::new(PortType::Audio, 2, "in1"),
            ],
            &[],
        );
        let d_in0 = g.input_ports(d).unwrap()[0];
        let d_in1 = g.input_ports(d).unwrap()[1];
        g.connect(a_out, b_in).unwrap();
        g.connect(a_out, c_in).unwrap();
        g.connect(b_out, d_in0).unwrap();
        g.connect(c_out, d_in1).unwrap();
        let order = topological_order(&g).unwrap();
        assert_eq!(order.first(), Some(&a));
        assert_eq!(order.last(), Some(&d));
    }

    #[test]
    fn two_node_cycle_is_detected() {
        let mut g = Graph::new();
        let (a, a_in, a_out) = add_stereo_node(&mut g);
        let (b, b_in, b_out) = add_stereo_node(&mut g);
        g.connect(a_out, b_in).unwrap();
        g.connect(b_out, a_in).unwrap();
        let cycle = topological_order(&g).unwrap_err();
        assert!(cycle.contains(&a) && cycle.contains(&b));
    }

    #[test]
    fn self_loop_is_a_cycle() {
        let mut g = Graph::new();
        let (a, a_in, a_out) = add_stereo_node(&mut g);
        g.connect(a_out, a_in).unwrap();
        assert_eq!(topological_order(&g).unwrap_err(), vec![a]);
    }

    #[test]
    fn schedule_orders_acyclic_chain_with_no_feedback() {
        let mut g = Graph::new();
        let (a, _a_in, a_out) = add_stereo_node(&mut g);
        let (b, b_in, b_out) = add_stereo_node(&mut g);
        let (c, c_in, _c_out) = add_stereo_node(&mut g);
        g.connect(a_out, b_in).unwrap();
        g.connect(b_out, c_in).unwrap();
        let result = schedule(&g);
        assert_eq!(result.order, vec![a, b, c]);
        assert!(result.feedback.is_empty());
    }

    #[test]
    fn schedule_marks_two_node_cycle_as_feedback() {
        let mut g = Graph::new();
        let (a, a_in, a_out) = add_stereo_node(&mut g);
        let (b, b_in, b_out) = add_stereo_node(&mut g);
        g.connect(a_out, b_in).unwrap();
        g.connect(b_out, a_in).unwrap();
        let result = schedule(&g);
        // Every node still appears exactly once; the back-edge is recorded
        assert_eq!(result.order.len(), 2);
        assert!(result.order.contains(&a) && result.order.contains(&b));
        assert_eq!(result.feedback, [(b, a)].into_iter().collect());
    }

    #[test]
    fn schedule_marks_self_loop_as_feedback() {
        let mut g = Graph::new();
        let (a, a_in, a_out) = add_stereo_node(&mut g);
        g.connect(a_out, a_in).unwrap();
        let result = schedule(&g);
        assert_eq!(result.order, vec![a]);
        assert_eq!(result.feedback, [(a, a)].into_iter().collect());
    }
}
