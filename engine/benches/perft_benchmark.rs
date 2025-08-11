use std::str::FromStr;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

use engine::board::Board;
use engine::movegen::perf_driver;

fn perf_test(depth: u8, fen: &str, expected_total_nodes: u64) {
    let mut board = Board::from_str(fen).unwrap();

    let mut actual_total_nodes = 0;
    perf_driver(&mut board, depth, &mut actual_total_nodes);
    assert_eq!(expected_total_nodes, actual_total_nodes);
}

fn perft_kiwipete() {
    perf_test(
        5,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        193690690,
    );
}

fn perft_startpos() {
    perf_test(4, Board::STARTING_POS_FEN, 197281);
}

fn perft_king_vs_pawn() {
    perf_test(6, "8/P1k5/K7/8/8/8/8/8 w - - 0 1", 92683);
}

fn perft_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft_benchmark");
    group.measurement_time(Duration::new(10, 0));
    group.sample_size(10);

    group.throughput(criterion::Throughput::Elements(193690690));
    group.bench_function("kiwipete", |b| b.iter(perft_kiwipete));

    group.throughput(criterion::Throughput::Elements(197281));
    group.bench_function("startpos", |b| b.iter(perft_startpos));

    group.throughput(criterion::Throughput::Elements(92683));
    group.bench_function("king vs pawn", |b| b.iter(perft_king_vs_pawn));
}

criterion_group!(benches, perft_benchmark);
criterion_main!(benches);
