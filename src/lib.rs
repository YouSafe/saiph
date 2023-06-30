enum StartingPosition {
    Standard,
    Custom(/* Todo: fen*/)
}

struct Position {
    rank: u8,
    file: u8
}

struct Move {
    from: Position,
    to: Position,
}

enum Command {
    Uci,
    IsReady,
    NewGame,
    Position(StartingPosition, Vec<Move>),
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
            "position" => Some(Command::Position(StartingPosition::Standard, vec![])),
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
            Command::Position(start_pos, moves) => {

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
}