use crate::board::Board;
use crate::search_limits::{SearchLimits, TimeLimit};
use crate::types::color::Color;
use crate::uci_move::UCIMove;
use crate::{Printer, SearcherPool};
use std::str::{FromStr, Split};
use std::time::Duration;

pub struct EngineUCI<S: SearcherPool, P: Printer> {
    board: Board,
    workers: S,
    printer: P,
}

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Vec<UCIMove>),
    Go(SearchLimits),
    Debug,
    Eval,
    Stop,
}

#[derive(Debug)]
enum ParseCommandError {
    MissingParts,
    UnknownCommand,
    InvalidStartingPos,
    InvalidMove,
    InvalidNumber,
}

#[derive(Debug, PartialEq)]
enum StartingPosition {
    Standard,
    Custom(Board),
}

impl<S: SearcherPool, P: Printer> EngineUCI<S, P> {
    pub fn new(workers: S, printer: P) -> Self {
        EngineUCI {
            workers,
            board: Default::default(),
            printer,
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
            "position" => parse_position(&mut parts)?,
            "go" => parse_go(parts)?,
            "debug" => Command::Debug,
            "eval" => Command::Eval,
            "stop" => Command::Stop,
            _ => return Err(ParseCommandError::UnknownCommand),
        };

        Ok(command)
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Uci => {
                self.printer.print("uciok");
            }
            Command::IsReady => {
                self.printer.print("readyok");
            }
            Command::NewGame => {
                self.workers.clear_tables();
            }
            Command::Position(start_pos, moves) => {
                let mut board = match start_pos {
                    StartingPosition::Standard => Board::default(),
                    StartingPosition::Custom(board) => board,
                };

                for uci_move in moves {
                    let chess_move = board
                        .generate_moves()
                        .into_iter()
                        .find(|m| uci_move == m)
                        .unwrap();
                    board.apply_move(chess_move);
                }

                self.board = board;
            }
            Command::Go(limits) => {
                self.workers.initiate_search(self.board.clone(), limits);
            }
            Command::Eval => {
                // self.printer
                //     .print(format!("Eval: {}", self.board.evaluate()).as_str());
            }
            Command::Debug => {
                self.printer.print(self.board.to_string().as_str());
            }
            Command::Stop => {
                self.workers.stop_search();
            }
        }
    }
}

fn parse_go(mut parts: Split<'_, char>) -> Result<Command, ParseCommandError> {
    let mut depth: Option<u8> = None;
    let mut mate: Option<u8> = None;
    let mut time_left: [Duration; 2] = Default::default();
    let mut move_time: Option<Duration> = None;
    let mut increment: [Duration; 2] = Default::default();
    let mut moves_to_go: Option<u8> = None;
    let mut nodes: Option<u64> = None;
    let mut infinite = false;
    while let Some(token) = parts.next() {
        match token {
            "infinite" => {
                infinite = true;
            }
            "wtime" | "btime" | "winc" | "binc" | "movetime" => {
                let param = parts
                    .next()
                    .ok_or(ParseCommandError::MissingParts)?
                    .parse()
                    .map_err(|_| ParseCommandError::InvalidNumber)?;

                let param = Duration::from_millis(param);

                match token {
                    "wtime" => time_left[Color::White as usize] = param,
                    "btime" => time_left[Color::Black as usize] = param,
                    "winc" => increment[Color::White as usize] = param,
                    "binc" => increment[Color::Black as usize] = param,
                    "movetime" => move_time = Some(param),
                    _ => unreachable!(),
                }
            }
            "depth" | "mate" | "moves_to_go" => {
                let param = parts
                    .next()
                    .ok_or(ParseCommandError::MissingParts)?
                    .parse()
                    .map_err(|_| ParseCommandError::InvalidNumber)?;

                match token {
                    "depth" => depth = Some(param),
                    "mate" => mate = Some(param),
                    "moves_to_go" => moves_to_go = Some(param),
                    _ => unreachable!(),
                }
            }
            "nodes" => {
                let param = parts
                    .next()
                    .ok_or(ParseCommandError::MissingParts)?
                    .parse()
                    .map_err(|_| ParseCommandError::InvalidNumber)?;

                nodes = Some(param);
            }
            _ => {}
        }
    }
    let time = if infinite {
        TimeLimit::Infinite
    } else if let Some(move_time) = move_time {
        TimeLimit::Fixed { move_time }
    } else if !time_left.contains(&Duration::default()) {
        TimeLimit::Dynamic {
            time_left,
            increment,
        }
    } else {
        TimeLimit::Infinite
    };
    let limits = SearchLimits {
        time,
        depth,
        mate,
        nodes,
        moves_to_go,
    };
    Ok(Command::Go(limits))
}

fn parse_position(parts: &mut Split<'_, char>) -> Result<Command, ParseCommandError> {
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
                Board::from_str(fen.as_str()).map_err(|_| ParseCommandError::InvalidStartingPos)?,
            );
        }
        _ => return Err(ParseCommandError::MissingParts),
    }
    let moves = parts
        .map(UCIMove::from_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ParseCommandError::InvalidMove)?;
    Ok(Command::Position(starting_pos, moves))
}
