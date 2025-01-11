use std::env;
use std::str::FromStr;

use engine::board::Board;
use engine::move_generation::perf_test;
use engine::uci_move::UCIMove;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 && args.len() != 4 {
        println!("Usage: {} <depth> <fen> <moves>", args[0]);
        return;
    }

    let (depth, fen, moves) = (&args[1], &args[2], args.get(3));

    let mut board = Board::from_str(fen).unwrap();

    if let Some(moves) = moves {
        for mov in moves.split_whitespace() {
            let uci_move = UCIMove::from_str(mov).unwrap();

            let chess_move = board
                .generate_moves()
                .into_iter()
                .find(|m| uci_move == m)
                .unwrap();
            board.apply_move(chess_move);
        }
    }

    perf_test(&mut board, depth.parse().unwrap());
}
