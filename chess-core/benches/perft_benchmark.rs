use std::str::FromStr;

use criterion::{Criterion, criterion_group, criterion_main};

use chess_core::board::Board;
use chess_core::movgen::perf_driver;

fn perf_test(depth: u8, fen: &str, expected_total_nodes: u64) {
    let board = Board::from_str(fen).unwrap();

    let mut actual_total_nodes = 0;
    perf_driver(&board, depth, &mut actual_total_nodes);
    assert_eq!(expected_total_nodes, actual_total_nodes);
}

fn perft_kiwipete() {
    perf_test(
        5,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        193690690,
    );
}

fn perft_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft_benchmark");
    group.sample_size(10);

    group.bench_function("kiwipete", |b| b.iter(|| perft_kiwipete()));
}

criterion_group!(benches, perft_benchmark);
criterion_main!(benches);
