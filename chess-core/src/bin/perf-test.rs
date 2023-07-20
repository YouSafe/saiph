use chess_core::board::Board;
use chess_core::movgen::{generate_moves, perf_test};
use std::env;
use std::str::FromStr;

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
            let chess_move = generate_moves(&board)
                .into_iter()
                .find(|m| m.to_string() == mov)
                .unwrap();
            board = board.make_move(chess_move);
        }
    }

    perf_test(&board, depth.parse().unwrap());
}
