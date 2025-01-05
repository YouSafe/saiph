use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Context;
use bulletformat::{BulletFormat, ChessBoard};
use chess_core::{board::Board, color::Color, piece::Piece, square::Square};
use clap::Parser;
use engine::evaluation::Evaluation;
use rand::{thread_rng, Rng};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long)]
    threads: Option<u8>,
    #[arg(short, long)]
    seed: Option<i32>,

    output_path: PathBuf,
}

fn generate_data(rng: &mut impl Rng) -> Vec<ChessBoard> {
    // generate random starting position
    // TODO: make random plies configurable
    let board = generate_random_position(10, rng);
    let data = vec![];

    // do a shallow search (limited by number of nodes)

    // if shallow search determined the position to be horrible don't bother playing the game out
    // if the position is okay-ish (max down by 4 points) then start playing out the game

    // track moves with their evaluation
    // determine game result

    data
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let stop_flag = Arc::new(AtomicBool::new(false));
    {
        // handle CTRL+C
        let stop_flag = stop_flag.clone();
        ctrlc::set_handler(move || stop_flag.store(true, Ordering::SeqCst))
            .context("Failed to set Ctrl-C handler")?;
    }

    fs::create_dir_all(&cli.output_path).context("Failed to create output dir")?;

    // spawn scoped threads
    std::thread::scope(|scope| {
        // each thread generates data
        for id in 0..cli.threads.unwrap_or(1) {
            let output_path = cli.output_path.clone();
            let stop_flag = stop_flag.clone();
            scope.spawn(move || -> anyhow::Result<()> {
                // TODO: Add hash to file name
                let path = output_path.join(format!("{id}.bin"));
                let mut output =
                    BufWriter::new(File::create(path).context("Failed to create output file")?);

                let mut rng = thread_rng();
                let mut data = vec![];

                while !stop_flag.load(Ordering::Relaxed) {
                    let entry = generate_data(&mut rng);
                    data.extend(entry);

                    if data.len() > 10000 {
                        BulletFormat::write_to_bin(&mut output, &data)
                            .context("Failed to write data to file")?;
                    }
                }
                Ok(())
            });
        }
    });

    // Questions to answer:
    // - What if a position is already in the output?
    // -> No problem, I think it's rather rare to have end up in the same exact position after playing a random opening

    // - Do I want to have a separate TT for each search or just per thread?
    // -> each game has their own TT
    Ok(())
}

fn generate_random_position(random_plies: usize, random: &mut impl Rng) -> Board {
    let mut board = Board::default();
    loop {
        for _ in 0..random_plies {
            let moves = board.generate_moves();
            if moves.is_empty() {
                // out of moves! try again
                board = Board::default();
                break;
            }

            let index = random.gen_range(0..moves.len());
            board.apply_move(moves[index])
        }

        if !board.generate_moves().is_empty() {
            return board;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use super::*;

    #[test]
    fn generate_pos() {
        let mut rng = thread_rng();
        let board = generate_random_position(10, &mut rng);
        println!("{}", board);
    }
}
