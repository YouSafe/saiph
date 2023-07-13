mod king_move;
mod pawn_move;

use crate::king_move::{generate_king_attacks, write_king_attacks};
use crate::pawn_move::{generate_pawn_attacks, write_pawn_attacks};
use std::env;
use std::fs::File;
use std::path::Path;

pub fn generate_tables() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tables.rs");
    let mut tables = File::create(dest_path).unwrap();

    let pawn_attacks = generate_pawn_attacks();
    let king_attacks = generate_king_attacks();
    write_pawn_attacks(&mut tables, &pawn_attacks).unwrap();
    write_king_attacks(&mut tables, &king_attacks).unwrap();
}
