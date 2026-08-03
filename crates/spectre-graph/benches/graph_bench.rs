// =============================================================================
// File: crates/spectre-graph/benches/graph_bench.rs
// Layer: audio graph
// Purpose: app-thread compile and swap timing for a 128-node graph
// Status: Implemented; validates the under-1ms compile + swap target.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use spectre_core::config::AudioConfig;
use spectre_core::port::PortType;
use spectre_graph::prelude::*;

// Number of nodes the plan targets sorting and swapping within one millisecond
const CHAIN_LEN: usize = 128;

// Build a chain of stereo passthrough nodes connected end to end
fn build_chain(length: usize) -> Graph {
    let mut graph = Graph::new();
    let mut previous_out = None;
    for _ in 0..length {
        let node = graph.add_node(
            Box::new(PassthroughNode::new()),
            &[PortSpec::new(PortType::Audio, 2, "in")],
            &[PortSpec::new(PortType::Audio, 2, "out")],
        );
        let in_port = graph.input_ports(node).unwrap()[0];
        let out_port = graph.output_ports(node).unwrap()[0];
        if let Some(source) = previous_out {
            graph.connect(source, in_port).unwrap();
        }
        previous_out = Some(out_port);
    }
    graph
}

// Time the schedule plus buffer-routing compilation on the app thread
fn bench_compile(c: &mut Criterion) {
    let config = AudioConfig::new(48_000, 512, 2, 2).unwrap();
    let graph = build_chain(CHAIN_LEN);
    c.bench_function("compile_128_node_chain", |b| {
        b.iter(|| {
            let plan = compile(black_box(&graph), &config).unwrap();
            black_box(plan);
        });
    });
}

// Time publishing a freshly compiled executor and adopting it on the audio side
fn bench_swap(c: &mut Criterion) {
    let config = AudioConfig::new(48_000, 512, 2, 2).unwrap();
    c.bench_function("publish_and_swap_128", |b| {
        b.iter_batched(
            || {
                let graph = build_chain(CHAIN_LEN);
                let plan = compile(&graph, &config).unwrap();
                let executor = Executor::new(graph, plan, &config).unwrap();
                let (publisher, active) = graph_swap(None);
                (publisher, active, executor)
            },
            |(mut publisher, mut active, executor)| {
                publisher.publish(executor);
                active.poll_swap();
                black_box(active.has_graph());
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_compile, bench_swap);
criterion_main!(benches);
