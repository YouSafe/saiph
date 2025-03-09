use std::{env, fs::File, io::Write, mem, path::Path};

use tablegen::{
    king_move::generate_king_attacks,
    knight_move::generate_knight_attacks,
    magics::{BISHOP_MAGICS, ROOK_MAGICS, SLIDER_ATTACK_TABLE_SIZE},
    pawn_move::generate_pawn_attacks,
    rays_between::generate_squares_between,
    slider_move::generate_slider_attacks,
    xray_line::generate_squares_line,
    zobrist::{generate_keys, GeneratedKeys},
    BitBoard,
};

fn write_slice_to_file<T>(file: impl AsRef<Path>, table: T) {
    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join(file.as_ref());
    let mut file = File::create(&path).unwrap();

    unsafe {
        let size = mem::size_of::<T>();
        let ptr = &table as *const T as *const u8;
        let bytes = std::slice::from_raw_parts(ptr, size);
        file.write_all(bytes).unwrap();
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../tablegen/");

    let pawn_attacks: [[BitBoard; 64]; 2] = generate_pawn_attacks();
    let king_attacks: [BitBoard; 64] = generate_king_attacks();
    let squares_between: [[BitBoard; 64]; 64] = generate_squares_between();
    let squares_line: [[BitBoard; 64]; 64] = generate_squares_line();
    let knight_attacks: [BitBoard; 64] = generate_knight_attacks();
    let slider_attacks: [BitBoard; SLIDER_ATTACK_TABLE_SIZE] = generate_slider_attacks();
    let zobrist: GeneratedKeys = generate_keys();

    write_slice_to_file("pawn_attacks", pawn_attacks);
    write_slice_to_file("king_attacks", king_attacks);
    write_slice_to_file("knight_attacks", knight_attacks);
    write_slice_to_file("slider_attacks", slider_attacks);
    write_slice_to_file("rook_magics", ROOK_MAGICS);
    write_slice_to_file("bishop_magics", BISHOP_MAGICS);
    write_slice_to_file("squares_between", squares_between);
    write_slice_to_file("squares_line", squares_line);
    write_slice_to_file("zobrist", zobrist);
}
