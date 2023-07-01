enum StartingPosition {
    Standard,
    Custom(/* Todo: fen*/)
}

#[derive(Debug)]
struct Position {
    rank: u8,
    file: u8
}

#[derive(Debug)]
enum Promotion {
    Queen,
    Bishop,
    Rook,
    Knight,
}

// TODO: pack move data tighter
#[derive(Debug)]
struct Move {
    from: Position,
    to: Position,
    promotion: Option<Promotion>
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
                let starting_pos = match parts.next() {
                    Some("startpos") => StartingPosition::Standard,
                    None => {
                        eprintln!("position command is not valid: {message}");
                        return;
                    },
                    _ => todo!("FEN starting position not supported yet")
                };

                let last_move = match parts.next() {
                    Some("moves") => self.parse_last_move(parts.collect()),
                    Some(_) => {
                        eprintln!("position command does not include `moves` keyword");
                        None
                    },
                    _ => {
                        eprintln!("position command is not valid: {message}");
                        return;
                    },
                };

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

        let from_file = chars[0];
        let from_rank = chars[1];
        let to_file = chars[2];
        let to_rank = chars[3];

        // TODO: validate positions
        let from = Position {
            file: (from_file as u8) - ('a' as u8) + 1,
            rank: (from_rank as u8) - ('1' as u8) + 1,
        };

        let to = Position {
            file: (to_file as u8) - ('a' as u8) + 1,
            rank: (to_rank as u8) - ('1' as u8) + 1,
        };

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