use crate::searcher::Searcher;
use crate::timer::Timer;
use chess_core::board::Board;
use chess_core::uci_move::UCIMove;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Vec<UCIMove>),
    Go,
    Stop,
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
    searcher: Searcher,
    timer: Timer,
}

impl Default for EngineUCI {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineUCI {
    pub fn new() -> Self {
        EngineUCI {
            searcher: Searcher::new(),
            board: Default::default(),
            timer: Timer::new(),
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
                let token = parts.next();
                let starting_pos;
                match token {
                    Some("startpos") => {
                        starting_pos = StartingPosition::Standard;
                        parts.next(); // consume "moves" token
                    }
                    Some("fen") => {
                        // this also consumes the "moves" token
                        let fen = parts
                            .by_ref()
                            .take_while(|s| *s != "moves")
                            .collect::<Vec<_>>()
                            .join(" ");

                        starting_pos = StartingPosition::Custom(
                            Board::from_str(fen.as_str())
                                .map_err(|_| ParseCommandError::InvalidStartingPos)?,
                        );
                    }
                    _ => return Err(ParseCommandError::MissingParts),
                }

                let moves = parts
                    .map(|move_str| UCIMove::from_str(move_str))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| ParseCommandError::InvalidMove)?;

                Command::Position(starting_pos, moves)
            }
            "go" => Command::Go,
            "stop" => Command::Stop,
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
            Command::NewGame => {
                self.searcher.clear_tables();
            }
            Command::Position(start_pos, moves) => {
                let mut board = match start_pos {
                    StartingPosition::Standard => Board::default(),
                    StartingPosition::Custom(board) => board,
                };

                for chess_move in moves {
                    board.apply_uci_move(chess_move);
                }

                self.board = board;
            }
            Command::Go => {
                self.timer.set_timer(Duration::from_secs(2));
                self.searcher
                    .initiate_search(self.board.clone(), self.timer.clone());
            }
            Command::Stop => {
                self.searcher.stop_search();
            }
        }
    }
}
