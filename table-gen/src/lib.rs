mod bishop_move;
mod king_move;
mod knight_move;
mod magic_number;
mod pawn_move;
mod rook_move;

use crate::bishop_move::{
    generate_bishop_attacks, generate_bishop_relevant_occupancy, BishopAttacks,
};
use crate::king_move::generate_king_attacks;
use crate::knight_move::generate_knight_attacks;
use crate::magic_number::Magic;
use crate::pawn_move::generate_pawn_attacks;
use crate::rook_move::{generate_rook_attacks, generate_rook_relevant_occupancy, RookAttacks};
use chess_core::bitboard::BitBoard;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

pub fn generate_tables() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tables.rs");
    let mut tables = BufWriter::new(File::create(dest_path).unwrap());

    {
        let pawn_attacks = generate_pawn_attacks();
        let king_attacks = generate_king_attacks();
        let knight_attacks = generate_knight_attacks();

        write_bitboards_variable_2d(&mut tables, "PAWN_ATTACKS", &pawn_attacks).unwrap();
        write_bitboards_variable_1d(&mut tables, "KNIGHT_ATTACKS", &knight_attacks).unwrap();
        write_bitboards_variable_1d(&mut tables, "KING_ATTACKS", &king_attacks).unwrap();
    }

    {
        let BishopAttacks {
            attack_table,
            magic_number_table,
        } = generate_bishop_attacks();

        let relevant_occupancies = generate_bishop_relevant_occupancy();

        write_bitboards_variable_1d(
            &mut tables,
            "BISHOP_RELEVANT_OCCUPANCIES",
            &relevant_occupancies,
        )
        .unwrap();
        write_bitboards_variable_2d_bishop(&mut tables, "BISHOP_ATTACKS", &attack_table).unwrap();
        write_magic_number_table(&mut tables, "BISHOP_MAGIC_NUMBERS", &magic_number_table).unwrap();
    }

    {
        let RookAttacks {
            magic_number_table,
            attack_table,
        } = generate_rook_attacks();

        let relevant_occupancies = generate_rook_relevant_occupancy();

        write_bitboards_variable_1d(
            &mut tables,
            "ROOK_RELEVANT_OCCUPANCIES",
            &relevant_occupancies,
        )
        .unwrap();

        write_bitboards_variable_2d_rook(&mut tables, "ROOK_ATTACKS", &attack_table).unwrap();
        write_magic_number_table(&mut tables, "ROOK_MAGIC_NUMBERS", &magic_number_table).unwrap();
    }
}

// TODO: refactor remove duplicated code

fn write_magic_number_table(
    file: &mut BufWriter<File>,
    variable_name: &str,
    magic_number_table: &[Magic; 64],
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [Magic; 64] = [")?;
    for magic in magic_number_table {
        writeln!(
            file,
            "\tMagic {{ magic_number: {:#018x}, shift: {}  }}, ",
            magic.magic_number, magic.shift
        )?;
    }
    writeln!(file, "];\n")
}

pub fn write_bitboards_variable_2d(
    file: &mut BufWriter<File>,
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

pub fn write_bitboards_variable_2d_bishop(
    file: &mut BufWriter<File>,
    variable_name: &str,
    attacks: &Vec<[BitBoard; 64]>,
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [[BitBoard; 64]; 512] = [")?;
    for attacks_for_color in attacks {
        writeln!(file, "\t[")?;
        for board in attacks_for_color {
            writeln!(file, "\t\tBitBoard({}), ", board.0)?;
        }
        writeln!(file, "\t],")?
    }
    writeln!(file, "];\n")
}

pub fn write_bitboards_variable_2d_rook(
    file: &mut BufWriter<File>,
    variable_name: &str,
    attacks: &Vec<[BitBoard; 64]>,
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [[BitBoard; 64]; 4096] = [")?;
    for attacks_for_color in attacks {
        writeln!(file, "\t[")?;
        for board in attacks_for_color {
            writeln!(file, "\t\tBitBoard({}), ", board.0)?;
        }
        writeln!(file, "\t],")?
    }
    writeln!(file, "];\n")
}

pub fn write_bitboards_variable_1d(
    file: &mut BufWriter<File>,
    variable_name: &str,
    attacks: &[BitBoard; 64],
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [BitBoard; 64] = [")?;
    for board in attacks {
        writeln!(file, "\tBitBoard({}), ", board.0)?;
    }
    writeln!(file, "];\n")
}

pub fn write_u8_array_variable_1d(
    file: &mut File,
    variable_name: &str,
    array: &[u8; 64],
) -> std::io::Result<()> {
    writeln!(file, "const {variable_name}: [u8; 64] = [")?;
    for row in 0..8 {
        write!(file, "\t")?;
        for col in 0..8 {
            let index = row * 8 + col;
            write!(file, "{:2}, ", array[index])?;
        }
        writeln!(file)?;
    }
    writeln!(file, "];\n")
}
