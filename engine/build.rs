use std::io::Write;
use std::{env, fs::File, io::BufWriter, path::Path};

use tablegen::magics::Magic;
use tablegen::BitBoard;
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

#[derive(Clone, Copy)]
pub struct State {
    indentation: usize,
}

trait FormattedWriter {
    fn typename() -> String;
    fn write(self, file: &mut impl Write, state: State) -> std::io::Result<()>;
    fn is_primitive() -> bool {
        true
    }
}

impl<const N: usize, T: FormattedWriter> FormattedWriter for [T; N] {
    fn typename() -> String {
        format!("[{}; {N}]", T::typename())
    }

    fn is_primitive() -> bool {
        false
    }

    fn write(self, file: &mut impl Write, state: State) -> std::io::Result<()> {
        write!(file, "[\n")?;

        fn indentation(file: &mut impl Write, indentation: usize) -> std::io::Result<()> {
            for _ in 0..indentation {
                write!(file, "    ")?;
            }
            Ok(())
        }

        let mut temp = Vec::new();
        let mut current_line_len = (state.indentation + 1) * 4;
        const MAX_LINE_WIDTH: usize = 80;

        indentation(file, state.indentation + 1)?;

        for (index, val) in self.into_iter().enumerate() {
            let new_state = State {
                indentation: state.indentation + 1,
            };

            if T::is_primitive() {
                temp.clear();
                val.write(&mut temp, new_state)?;
            } else {
                val.write(file, new_state)?;
            }

            let element_str = unsafe { std::str::from_utf8_unchecked(&temp) };
            let suffix = if index < N - 1 { ", " } else { "" };
            let piece_len = element_str.len() + suffix.len();

            if current_line_len + piece_len > MAX_LINE_WIDTH {
                writeln!(file)?;
                indentation(file, state.indentation + 1)?;
                current_line_len = (state.indentation + 1) * 4;
            }

            file.write_all(element_str.as_bytes())?;
            file.write_all(suffix.as_bytes())?;
            current_line_len += piece_len;
        }

        writeln!(file)?;
        indentation(file, state.indentation)?;
        write!(file, "]")
    }
}

impl FormattedWriter for u64 {
    fn typename() -> String {
        "u64".to_owned()
    }

    fn write(self, file: &mut impl Write, _state: State) -> std::io::Result<()> {
        write!(file, "{self}")
    }
}

impl FormattedWriter for Magic {
    fn typename() -> String {
        "Magic".to_owned()
    }

    fn write(self, file: &mut impl Write, _state: State) -> std::io::Result<()> {
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
    variable.write(file, State { indentation: 0 })?;
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

    write_variable(
        &mut writer,
        "PAWN_ATTACKS",
        pawn_attacks.map(|color| color.map(|BitBoard(v)| v)),
    )
    .unwrap();
    write_variable(
        &mut writer,
        "KING_ATTACKS",
        king_attacks.map(|BitBoard(v)| v),
    )
    .unwrap();
    write_variable(
        &mut writer,
        "KNIGHT_ATTACKS",
        knight_attacks.map(|BitBoard(v)| v),
    )
    .unwrap();
    write_variable(
        &mut writer,
        "SQUARES_BETWEEN",
        squares_between.map(|from| from.map(|BitBoard(v)| v)),
    )
    .unwrap();
    write_variable(
        &mut writer,
        "SQUARES_LINE",
        squares_line.map(|from| from.map(|BitBoard(v)| v)),
    )
    .unwrap();
    write_variable(
        &mut writer,
        "SLIDER_ATTACKS",
        slider_attacks.map(|BitBoard(v)| v),
    )
    .unwrap();

    let dest_path = Path::new(&out_dir).join("zobrist.rs");
    let tables = File::create(&dest_path).unwrap();
    let mut writer = BufWriter::new(tables);

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
