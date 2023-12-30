use crate::evaluation::Evaluation;
use crate::search_limits::SearchLimits;
use crate::searcher::Searcher;
use chess_core::board::Board;
use chess_core::color::Color;
use chess_core::uci_move::UCIMove;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Vec<UCIMove>),
    Go(SearchLimits),
    Stop,
}

enum InfoAttribute {
    Depth(u8),
    Time(Duration),
    Nodes(u64),
    Pv(Vec<UCIMove>),
    Score(Evaluation),
}

enum Response {
    UciOk,
    ReadyOk,
    BestMove(UCIMove),
    Info(Vec<InfoAttribute>),
}

#[derive(Debug)]
enum ParseCommandError {
    MissingParts,
    UnknownCommand(String),
    InvalidStartingPos,
    InvalidMove,
    InvalidNumber,
}

pub trait Printer {
    fn print(s: &str);
}

#[derive(Debug, PartialEq)]
enum StartingPosition {
    Standard,
    Custom(Board),
}

pub struct EngineUCI<S: Searcher, P: Printer> {
    board: Board,
    searcher: S,
    _marker: PhantomData<P>,
}

impl<S: Searcher, P: Printer> Default for EngineUCI<S, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Searcher, P: Printer> EngineUCI<S, P> {
    pub fn new() -> Self {
        EngineUCI {
            searcher: S::new(),
            board: Default::default(),
            _marker: PhantomData,
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
                    .map(UCIMove::from_str)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| ParseCommandError::InvalidMove)?;

                Command::Position(starting_pos, moves)
            }
            "go" => {
                let mut limits = SearchLimits {
                    infinite: false,
                    time_left: [Duration::default(); 2],
                    increment: [Duration::default(); 2],
                    move_time: Duration::default(),
                    depth: 0,
                    mate: 0,
                };

                while let Some(token) = parts.next() {
                    match token {
                        "infinite" => {
                            limits.infinite = true;
                        }
                        "wtime" | "btime" | "winc" | "binc" | "movetime" => {
                            let param = parts
                                .next()
                                .ok_or(ParseCommandError::MissingParts)?
                                .parse()
                                .map_err(|_| ParseCommandError::InvalidNumber)?;

                            match token {
                                "wtime" => {
                                    limits.time_left[Color::White as usize] =
                                        Duration::from_millis(param)
                                }
                                "btime" => {
                                    limits.time_left[Color::Black as usize] =
                                        Duration::from_millis(param)
                                }
                                "winc" => {
                                    limits.increment[Color::White as usize] =
                                        Duration::from_millis(param)
                                }
                                "binc" => {
                                    limits.increment[Color::Black as usize] =
                                        Duration::from_millis(param)
                                }
                                "movetime" => limits.move_time = Duration::from_millis(param),
                                _ => (),
                            }
                        }
                        "depth" => {
                            let param = parts
                                .next()
                                .ok_or(ParseCommandError::MissingParts)?
                                .parse()
                                .map_err(|_| ParseCommandError::InvalidNumber)?;
                            limits.depth = param
                        }
                        "mate" => {
                            let param = parts
                                .next()
                                .ok_or(ParseCommandError::MissingParts)?
                                .parse()
                                .map_err(|_| ParseCommandError::InvalidNumber)?;
                            limits.mate = param
                        }
                        _ => {}
                    }
                }

                Command::Go(limits)
            }
            "stop" => Command::Stop,
            _ => return Err(ParseCommandError::UnknownCommand(message.to_owned())),
        };

        Ok(command)
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Uci => {
                P::print("uciok");
            }
            Command::IsReady => {
                P::print("readyok");
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
            Command::Go(limits) => {
                self.searcher.initiate_search(self.board.clone(), limits);
            }
            Command::Stop => {
                self.searcher.stop_search();
            }
        }
    }
}
