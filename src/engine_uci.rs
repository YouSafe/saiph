use crate::search::Search;
use chess::{Board, ChessMove};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Vec<ChessMove>),
    Go,
    Stop,
    Quit,
}

#[derive(Debug)]
enum ParseCommandError {
    MissingParts,
    UnknownCommand(String),
    InvalidStartingPos,
    InvalidMove,
}

#[derive(Debug, PartialEq)]
enum StartingPosition {
    Standard,
    Custom(Board),
}

pub struct EngineUCI {
    board: Board,
    search: Search,
}

impl EngineUCI {
    pub fn new() -> Self {
        EngineUCI {
            search: Search::new(),
            board: Default::default(),
        }
    }

    pub fn receive_command(&mut self, message: &str) {
        let command = self.parse_command(message);
        match command {
            Ok(command) => self.process_command(command),
            Err(err) => eprintln!("Parsing error: {:?}", err),
        }
    }

    fn parse_command(&self, message: &str) -> Result<Command, ParseCommandError> {
        let mut parts = message.split(' ');
        let cmd = parts.next().ok_or(ParseCommandError::MissingParts)?;

        let command = match cmd {
            "uci" => Command::Uci,
            "isready" => Command::IsReady,
            "ucinewgame" => Command::NewGame,
            "position" => {
                let args_str = parts.collect::<Vec<_>>().join(" ");

                let args: Vec<_> = args_str.split("moves").map(|s| s.trim()).collect();

                if args.len() != 2 {
                    return Err(ParseCommandError::MissingParts);
                }

                let starting_pos_part = args[0];
                let moves_part = args[1];

                let mut starting_pos_split = starting_pos_part.split(' ');

                let starting_pos = match starting_pos_split.next() {
                    Some("startpos") => StartingPosition::Standard,
                    Some("fen") => {
                        let fen = starting_pos_split.collect::<Vec<_>>().join(" ");
                        StartingPosition::Custom(
                            Board::from_str(fen.as_str())
                                .map_err(|_| ParseCommandError::InvalidStartingPos)?,
                        )
                    }
                    Some(_) => return Err(ParseCommandError::InvalidStartingPos),
                    None => return Err(ParseCommandError::MissingParts),
                };

                // Note: the filter allows having two spaces between the moves, which is not intended
                let moves = moves_part
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();

                // Also see: https://doc.rust-lang.org/rust-by-example/error/iter_result.html
                let moves = moves
                    .iter()
                    .map(|move_str| ChessMove::from_str(move_str))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| ParseCommandError::InvalidMove)?;

                Command::Position(starting_pos, moves)
            }
            "go" => Command::Go,
            "stop" => Command::Stop,
            "quit" => Command::Quit,
            _ => return Err(ParseCommandError::UnknownCommand(message.to_owned())),
        };

        Ok(command)
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Uci => {
                println!("uciok");
            }
            Command::IsReady => {
                println!("readyok");
            }
            Command::NewGame => {}
            Command::Position(start_pos, moves) => {
                let mut board = match start_pos {
                    StartingPosition::Standard => Board::default(),
                    StartingPosition::Custom(board) => board,
                };

                for chess_move in moves {
                    board = board.make_move_new(chess_move);
                }

                self.board = board;
            }
            Command::Go => {
                let pick = self.search.find_best_move(&self.board, 7).unwrap();

                println!("bestmove {}", pick.chess_move.unwrap());
            }
            Command::Stop => {}
            Command::Quit => {
                // do nothing
            }
        }
    }
}
