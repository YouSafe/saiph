use std::io::Write;
use std::{env, fs::File, io::BufWriter, path::Path};

use tablegen::magics::Magic;
use tablegen::{
    king_move::generate_king_attacks,
    knight_move::generate_knight_attacks,
    magics::{BISHOP_MAGICS, ROOK_MAGICS, SLIDER_ATTACK_TABLE_SIZE},
    pawn_move::generate_pawn_attacks,
    rays_between::generate_squares_between,
    slider_move::generate_slider_attacks,
    xray_line::generate_squares_line,
    zobrist::{generate_keys, GeneratedKeys},
};
use types::castling_rights::PerCastlingRightsConfig;
use types::piece::PerPieceType;
use types::square::PerFile;
use types::{bitboard::BitBoard, color::PerColor, square::PerSquare};

#[derive(Clone, Copy)]
pub struct State {
    identation: usize,
}

trait FormattedWriter {
    fn typename() -> String;
    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()>;
    fn wrap() -> usize {
        1
    }
}

impl<T: FormattedWriter> FormattedWriter for PerSquare<T> {
    fn typename() -> String {
        format!("PerSquare<{}>", T::typename())
    }

    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()> {
        write!(file, "PerSquare::new(")?;
        self.into_inner().write(file, state)?;
        write!(file, ")")
    }
}

impl<T: FormattedWriter> FormattedWriter for PerPieceType<T> {
    fn typename() -> String {
        format!("PerPieceType<{}>", T::typename())
    }

    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()> {
        write!(file, "PerPieceType::new(")?;
        self.into_inner().write(file, state)?;
        write!(file, ")")
    }
}

impl<T: FormattedWriter> FormattedWriter for PerColor<T> {
    fn typename() -> String {
        format!("PerColor<{}>", T::typename())
    }

    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()> {
        write!(file, "PerColor::new(")?;
        self.into_inner().write(file, state)?;
        write!(file, ")")
    }
}

impl<T: FormattedWriter> FormattedWriter for PerFile<T> {
    fn typename() -> String {
        format!("PerFile<{}>", T::typename())
    }

    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()> {
        write!(file, "PerFile::new(")?;
        self.into_inner().write(file, state)?;
        write!(file, ")")
    }
}

impl<T: FormattedWriter> FormattedWriter for PerCastlingRightsConfig<T> {
    fn typename() -> String {
        format!("PerCastlingRightsConfig<{}>", T::typename())
    }

    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()> {
        write!(file, "PerCastlingRightsConfig::new(")?;
        self.into_inner().write(file, state)?;
        write!(file, ")")
    }
}

impl<const N: usize, T: FormattedWriter> FormattedWriter for [T; N] {
    fn typename() -> String {
        format!("[{}; {N}]", T::typename())
    }

    fn write(self, file: &mut BufWriter<File>, state: State) -> std::io::Result<()> {
        write!(file, "[")?;

        let len = self.len();

        fn identation(file: &mut BufWriter<File>, identation: usize) -> std::io::Result<()> {
            for _ in 0..identation {
                write!(file, "\t")?;
            }
            Ok(())
        }

        writeln!(file)?;

        let mut new_line = true;
        for (index, val) in self.into_iter().enumerate() {
            if new_line {
                identation(file, state.identation + 1)?;
                new_line = false;
            }

            val.write(
                file,
                State {
                    identation: state.identation + 1,
                },
            )?;
            if index < len - 1 {
                write!(file, ",")?;
            }

            if (index + 1) % T::wrap() == 0 || index == len - 1 {
                writeln!(file)?;
                new_line = true;
            }
        }
        identation(file, state.identation)?;

        write!(file, "]")
    }
}

impl FormattedWriter for BitBoard {
    fn typename() -> String {
        "BitBoard".to_owned()
    }

    fn write(self, file: &mut BufWriter<File>, _state: State) -> std::io::Result<()> {
        write!(file, "BitBoard({:#x})", self.0)
    }

    fn wrap() -> usize {
        3
    }
}

impl FormattedWriter for u64 {
    fn typename() -> String {
        "u64".to_owned()
    }

    fn write(self, file: &mut BufWriter<File>, _state: State) -> std::io::Result<()> {
        write!(file, "{self}")
    }

    fn wrap() -> usize {
        4
    }
}

impl FormattedWriter for Magic {
    fn typename() -> String {
        "Magic".to_owned()
    }

    fn write(self, file: &mut BufWriter<File>, _state: State) -> std::io::Result<()> {
        write!(
            file,
            "Magic {{ magic: {}, offset: {}, mask: {} }}",
            self.magic, self.offset, self.mask
        )
    }
}

fn write_variable<T: FormattedWriter>(
    file: &mut BufWriter<File>,
    name: &str,
    variable: T,
) -> std::io::Result<()> {
    write!(file, "pub static {name}: {} = ", T::typename())?;
    variable.write(file, State { identation: 0 })?;
    writeln!(file, ";")
}

fn main() {
    println!("cargo:rerun-if-changed=../tablegen/");

    let pawn_attacks = generate_pawn_attacks();
    let king_attacks = generate_king_attacks();
    let squares_between = generate_squares_between();
    let squares_line = generate_squares_line();
    let knight_attacks = generate_knight_attacks();
    let slider_attacks: [BitBoard; SLIDER_ATTACK_TABLE_SIZE] = generate_slider_attacks();

    let zobrist: GeneratedKeys = generate_keys();

    let out_dir = env::var("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("tables.rs");
    let tables = File::create(&dest_path).unwrap();
    let mut writer = BufWriter::new(tables);

    write_variable(&mut writer, "PAWN_ATTACKS", pawn_attacks).unwrap();
    write_variable(&mut writer, "KING_ATTACKS", king_attacks).unwrap();
    write_variable(&mut writer, "KNIGHT_ATTACKS", knight_attacks).unwrap();
    write_variable(&mut writer, "SQUARES_BETWEEN", squares_between).unwrap();
    write_variable(&mut writer, "SQUARES_LINE", squares_line).unwrap();
    write_variable(&mut writer, "SLIDER_ATTACKS", slider_attacks).unwrap();

    let dest_path = Path::new(&out_dir).join("zobrist.rs");
    let tables = File::create(&dest_path).unwrap();
    let mut writer = BufWriter::new(tables);
    writeln!(&mut writer, "use types::piece::PerPieceType;").unwrap();
    writeln!(&mut writer, "use types::square::PerSquare;").unwrap();
    writeln!(&mut writer, "use types::square::PerFile;").unwrap();
    writeln!(&mut writer, "use types::color::PerColor;").unwrap();
    writeln!(
        &mut writer,
        "use types::castling_rights::PerCastlingRightsConfig;"
    )
    .unwrap();

    write_variable(&mut writer, "PIECE_KEYS", zobrist.piece_keys).unwrap();
    write_variable(&mut writer, "EN_PASSANT_KEYS", zobrist.en_passant_keys).unwrap();
    write_variable(&mut writer, "CASTLE_KEYS", zobrist.castle_keys).unwrap();
    write_variable(&mut writer, "SIDE_KEY", zobrist.side_key).unwrap();

    let dest_path = Path::new(&out_dir).join("magics.rs");
    let tables = File::create(&dest_path).unwrap();
    let mut writer = BufWriter::new(tables);
    write_variable(&mut writer, "ROOK_MAGICS", ROOK_MAGICS).unwrap();
    write_variable(&mut writer, "BISHOP_MAGICS", BISHOP_MAGICS).unwrap();
}
