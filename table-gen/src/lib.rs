mod king_move;
mod knight_move;
mod pawn_move;

use crate::king_move::generate_king_attacks;
use crate::knight_move::generate_knight_attacks;
use crate::pawn_move::generate_pawn_attacks;
use chess_core::bitboard::BitBoard;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_tables() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tables.rs");
    let mut tables = File::create(dest_path).unwrap();

    let pawn_attacks = generate_pawn_attacks();
    let king_attacks = generate_king_attacks();
    let knight_attacks = generate_knight_attacks();
    write_bitboards_variable_2d(&mut tables, "PAWN_ATTACKS", &pawn_attacks).unwrap();
    write_bitboards_variable_1d(&mut tables, "KNIGHT_ATTACKS", &knight_attacks).unwrap();
    write_bitboards_variable_1d(&mut tables, "KING_ATTACKS", &king_attacks).unwrap();
}

pub fn write_bitboards_variable_2d(
    file: &mut File,
    variable_name: &str,
    pawn_attacks: &[[BitBoard; 64]; 2],
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [[BitBoard; 64]; 2] = [")?;
    for attacks_for_color in pawn_attacks {
        writeln!(file, "\t[")?;
        for board in attacks_for_color {
            writeln!(file, "\t\tBitBoard({}), ", board.0)?;
        }
        writeln!(file, "\t],")?
    }
    writeln!(file, "];\n")
}

pub fn write_bitboards_variable_1d(
    file: &mut File,
    variable_name: &str,
    attacks: &[BitBoard; 64],
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [BitBoard; 64] = [")?;
    for board in attacks {
        writeln!(file, "\tBitBoard({}), ", board.0)?;
    }
    writeln!(file, "];\n")
}
