use crate::board::Board;
use crate::clock::Clock;
use crate::movegen::perf_test;
use crate::threadpool::ThreadPool;
use crate::transposition::TranspositionTable;
use crate::types::color::Color;
use crate::types::search_limits::{SearchLimits, TimeLimit};
use crate::types::uci_move::UCIMove;
use crate::{Printer, ThreadSpawner};
use std::iter::Peekable;
use std::marker::PhantomData;
use std::str::{FromStr, SplitAsciiWhitespace};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

/// Default transposition table size in MB
const DEFAULT_HASH_SIZE: usize = 1;

/// Default number of threads
const DEFAULT_THREADS: u8 = 1;

pub enum EngineMessage {
    Command(String),
    Response(String),
    Terminate,
}

pub struct EngineUCI<S: ThreadSpawner, P: Printer> {
    board: Board,
    engine_tx: Sender<EngineMessage>,
    threadpool: ThreadPool<S>,
    transposition_table: Arc<TranspositionTable>,
    ignore_commands: bool,
    _marker: PhantomData<P>,
}

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    IsReady,
    NewGame,
    SetOption { name: String, value: Option<String> },
    Position(StartingPosition, Vec<UCIMove>),
    Go(SearchLimits),
    Perft { depth: u8 },
    Debug,
    Stop,
    Quit,
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

impl<S: ThreadSpawner, P: Printer> EngineUCI<S, P> {
    pub fn new(engine_tx: Sender<EngineMessage>) -> Self {
        Self {
            board: Default::default(),
            engine_tx,
            threadpool: ThreadPool::<S>::new(DEFAULT_THREADS),
            transposition_table: Arc::new(TranspositionTable::new(DEFAULT_HASH_SIZE)),
            ignore_commands: false,
            _marker: Default::default(),
        }
    }

    pub fn run(mut self, engine_rx: Receiver<EngineMessage>) {
        loop {
            let Ok(input) = engine_rx.recv() else {
                break;
            };

            match input {
                EngineMessage::Command(message) => {
                    if !self.ignore_commands {
                        self.receive_command(&message)
                    }
                }
                EngineMessage::Response(message) => P::println(&message),
                EngineMessage::Terminate => break,
            }
        }
    }

    fn receive_command(&mut self, message: &str) {
        let command = self.parse_command(message);
        match command {
            Ok(command) => self.process_command(command),
            Err(err) => eprintln!("Parsing error: {:?}", err),
        }
    }

    fn parse_command(&self, message: &str) -> Result<Command, ParseCommandError> {
        let mut parts = message.split_ascii_whitespace().peekable();
        let cmd = parts.next().ok_or(ParseCommandError::MissingParts)?;

        let command = match cmd {
            "uci" => Command::Uci,
            "setoption" => parse_setoption(parts)?,
            "isready" => Command::IsReady,
            "ucinewgame" => Command::NewGame,
            "position" => parse_position(parts)?,
            "go" => parse_go(parts)?,
            "perft" => parse_perft(parts)?,
            "debug" => Command::Debug,
            "quit" => Command::Quit,
            "stop" => Command::Stop,
            _ => return Err(ParseCommandError::UnknownCommand),
        };

        Ok(command)
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::Uci => {
                P::println("id name Saiph");
                P::println("id author Yousif");

                P::println(&format!(
                    "option name Hash type spin default {DEFAULT_HASH_SIZE} min 1 max 33554432"
                ));

                P::println(&format!(
                    "option name Threads type spin default {DEFAULT_THREADS} min 1 max 255"
                ));
                P::println("uciok");
            }
            Command::IsReady => {
                P::println("readyok");
            }
            Command::SetOption { name, value } => match name.as_str() {
                "Threads" => {
                    if let Some(num_threads) = value.and_then(|v| v.parse::<u8>().ok()) {
                        self.threadpool.resize(num_threads)
                    } else {
                        eprintln!("invalid value");
                    }
                }
                "Hash" => {
                    if let Some(size_mb) = value.and_then(|v| v.parse::<usize>().ok()) {
                        self.transposition_table = Arc::new(TranspositionTable::new(size_mb));
                    } else {
                        eprintln!("invalid value");
                    }
                }
                _ => eprintln!("invalid option"),
            },
            Command::NewGame => {
                self.threadpool.clear(self.transposition_table.clone());
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
                // The clock should be started as soon as possible even if the search has to wait in queue
                let clock = Clock::new(&limits, self.board.game_ply(), self.board.side_to_move());

                self.threadpool.search(
                    self.board.clone(),
                    limits,
                    clock,
                    self.engine_tx.clone(),
                    self.transposition_table.clone(),
                );
            }
            Command::Debug => {
                P::println(self.board.to_string().as_str());
            }
            Command::Stop => {
                self.threadpool.stop_search();
            }
            Command::Quit => {
                self.ignore_commands = true;
                self.threadpool.quit(self.engine_tx.clone());
            }
            Command::Perft { depth } => {
                perf_test::<P>(&mut self.board, depth);
            }
        }
    }
}

fn parse_perft(
    mut parts: Peekable<SplitAsciiWhitespace<'_>>,
) -> Result<Command, ParseCommandError> {
    Ok(Command::Perft {
        depth: parts
            .next()
            .ok_or(ParseCommandError::MissingParts)?
            .parse()
            .map_err(|_| ParseCommandError::InvalidNumber)?,
    })
}

fn parse_setoption(
    mut parts: Peekable<SplitAsciiWhitespace<'_>>,
) -> Result<Command, ParseCommandError> {
    let name = match [parts.next(), parts.next()] {
        [Some("name"), Some(name)] => name,
        _ => return Err(ParseCommandError::MissingParts),
    };

    let value = match [parts.next(), parts.next()] {
        [Some("value"), Some(name)] => Some(name),
        // FIXME: use correct error
        [Some(_), Some(_)] => return Err(ParseCommandError::MissingParts),
        _ => None,
    };

    Ok(Command::SetOption {
        name: name.to_owned(),
        value: value.map(|s| s.to_owned()),
    })
}

fn parse_go(mut parts: Peekable<SplitAsciiWhitespace<'_>>) -> Result<Command, ParseCommandError> {
    let mut depth: Option<u8> = None;
    let mut mate: Option<u8> = None;
    let mut time_left: [Duration; 2] = Default::default();
    let mut move_time: Option<Duration> = None;
    let mut increment: [Duration; 2] = Default::default();
    let mut moves_to_go: Option<u8> = None;
    let mut nodes: Option<u64> = None;
    let mut infinite = false;
    let mut search_moves = vec![];
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
            "ponder" => {
                todo!()
            }
            "searchmoves" => {
                while let Some(mov) = parts.peek().and_then(|m| UCIMove::from_str(m).ok()) {
                    search_moves.push(mov);
                    parts.next();
                }
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
        search_moves,
    };
    Ok(Command::Go(limits))
}

fn parse_position(
    mut parts: Peekable<SplitAsciiWhitespace<'_>>,
) -> Result<Command, ParseCommandError> {
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
