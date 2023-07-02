use crate::{Move, Position, Promotion};
use crate::fen_parser::Fen;

enum StartingPosition {
    Standard,
    Custom(Fen)
}

enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Option<Move>),
    Go,
    Stop,
    Quit
}

pub struct EngineUCI {

}

impl EngineUCI {
    pub fn new() -> Self {
        EngineUCI {}
    }

    pub fn receive_command(&self, message: &str) {
        let mut parts = message.split(' ');
        let cmd = parts.next().expect("command can not be empty");
        let command = match cmd {
            "uci" => Some(Command::Uci),
            "isready" => Some(Command::IsReady),
            "ucinewgame" => Some(Command::NewGame),
            "position" => {
                let args_str = parts
                    .collect::<Vec<_>>()
                    .join(" ");

                let args: Vec<_> = args_str
                    .split("moves")
                    .map(|s| s.trim())
                    .collect();

                if args.len() != 2 {
                    todo!("Error handling")
                }

                let starting_pos = match args[0] {
                    "startpos" => StartingPosition::Standard,
                    fen => StartingPosition::Custom(fen.parse().expect("provided fen is not valid"))
                };

                // Note: the filter allows having two spaces between the moves, which is not intend
                let moves = args[1]
                    .split(' ')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>();

                let last_move = self.parse_last_move(moves);

                Some(Command::Position(starting_pos, last_move))
            },
            "go" => Some(Command::Go),
            "stop" => Some(Command::Stop),
            "quit" => Some(Command::Quit),
            _ => None
        };

        if let Some(command) = command {
            self.process_command(command);
        } else {
            eprintln!("Unknown command: {cmd}");
        }
    }

    fn process_command(&self, command: Command) {
        match command {
            Command::Uci => {
                println!("uciok");
            }
            Command::IsReady => {
                println!("readyok");
            }
            Command::NewGame => {

            }
            Command::Position(start_pos, last_move) => {
                eprintln!("DEBUG: last_move: {:?}", last_move);
            }
            Command::Go => {
                println!("bestmove d2d4");
            }
            Command::Stop => {

            }
            Command::Quit => {
                // do nothing
            }
        }
    }

    fn parse_last_move(&self, move_strings: Vec<&str>) -> Option<Move> {
        let last_move_string = move_strings.last();

        // TODO: program should not panic if an invalid move is parsed
        last_move_string.map(
            |move_str | self.parse_move(move_str).expect("invalid move")
        )

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