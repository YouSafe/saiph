use crate::fen_parser::Fen;
use crate::{Move, Position, Promotion};

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Vec<Move>),
    Go,
    Stop,
    Quit,
}

#[derive(Debug)]
enum Response {
    Uciok,
    ReadyOk,
    Go(Move),
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
    Custom(Fen),
}

pub struct EngineUCI {}

impl EngineUCI {
    pub fn new() -> Self {
        EngineUCI {}
    }

    pub fn receive_command(&self, message: &str) {
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
                    Some("fen") => StartingPosition::Custom(
                        starting_pos_split
                            .collect::<Vec<_>>()
                            .join(" ")
                            .parse()
                            .map_err(|_| ParseCommandError::InvalidStartingPos)?,
                    ),
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
                    .map(|move_str| self.parse_move(move_str))
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

    fn process_command(&self, command: Command) {
        match command {
            Command::Uci => {
                println!("uciok");
            }
            Command::IsReady => {
                println!("readyok");
            }
            Command::NewGame => {}
            Command::Position(start_pos, last_move) => {
                eprintln!("DEBUG: last_move: {:?}", last_move);
            }
            Command::Go => {
                println!("bestmove d2d4");
            }
            Command::Stop => {}
            Command::Quit => {
                // do nothing
            }
        }
    }

    fn parse_move(&self, move_str: &str) -> Result<Move, &'static str> {
        if move_str.len() < 4 || move_str.len() > 5 {
            return Err("Invalid move format");
        }

        let chars: Vec<char> = move_str.chars().collect();

        let from = move_str[0..2]
            .parse::<Position>()
            .map_err(|_| "Invalid 'from' position")?;

        let to = move_str[2..4]
            .parse::<Position>()
            .map_err(|_| "Invalid 'to' position")?;

        let promotion = if move_str.len() == 5 {
            match chars[4] {
                'q' => Some(Promotion::Queen),
                'b' => Some(Promotion::Bishop),
                'r' => Some(Promotion::Rook),
                'n' => Some(Promotion::Knight),
                _ => return Err("Invalid promotion"),
            }
        } else {
            None
        };

        Ok(Move {
            from,
            to,
            promotion,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::engine_uci::{Command, EngineUCI, ParseCommandError, StartingPosition};
    use crate::fen_parser::Fen;
    use crate::Move;

    #[test]
    fn test_position_command_parsing() -> Result<(), ParseCommandError> {
        let uci = EngineUCI::new();
        let command = uci.parse_command(
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves d2d4 e1e2",
        )?;
        assert_eq!(
            command,
            Command::Position(
                StartingPosition::Custom(Fen::starting_pos()),
                vec![
                    Move {
                        from: "d2".parse().unwrap(),
                        to: "d4".parse().unwrap(),
                        promotion: None,
                    },
                    Move {
                        from: "e1".parse().unwrap(),
                        to: "e2".parse().unwrap(),
                        promotion: None,
                    }
                ]
            )
        );
        Ok(())
    }
}
