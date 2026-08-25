#[path = "src/attacks/pattern.rs"]
mod pattern;

use chess_base::{bitboard, for_each_square};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn const_pdep(mut val: u64, mut mask: u64) -> u64 {
    let mut res = 0u64;
    let mut bb = 1u64;
    while mask != 0 {
        if (mask & 1) != 0 {
            if (val & 1) != 0 {
                res |= bb;
            }
            val >>= 1;
        }
        mask >>= 1;
        bb <<= 1;
    }
    res
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=chess_base/src/bitboard.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("pext_data.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut table = Vec::with_capacity(107648);
    let mut bishop_offsets = vec![0u32; 64];
    let mut rook_offsets = vec![0u32; 64];
    let mut bishop_masks = vec![0u64; 64];
    let mut rook_masks = vec![0u64; 64];

    let mut current_index = 0;

    for_each_square!(sq => {
        rook_offsets[sq as usize] = current_index as u32;
        let mask = pattern::ROOK_XRAY_ATTACKS[sq as usize] & !bitboard::bb_get_edge_filter(sq);
        rook_masks[sq as usize] = mask;
        let combinations = 1usize << mask.count_ones();
        for j in 0..combinations {
            let occupied = const_pdep(j as u64, mask);
            table.push(bitboard::generate_rook_attacks(sq, occupied));
            current_index += 1;
        }
    });

    for_each_square!(sq => {
        bishop_offsets[sq as usize] = current_index as u32;
        let mask = pattern::BISHOP_XRAY_ATTACKS[sq as usize] & !bitboard::bb_get_edge_filter(sq);
        bishop_masks[sq as usize] = mask;
        let combinations = 1usize << mask.count_ones();
        for j in 0..combinations {
            let occupied = const_pdep(j as u64, mask);
            table.push(bitboard::generate_bishop_attacks(sq, occupied));
            current_index += 1;
        }
    });

    writeln!(
        f,
        "pub const PEXT_TABLE: [u64; {}] = {:?};",
        table.len(),
        table
    )
    .unwrap();
    writeln!(
        f,
        "pub const PEXT_BISHOP_OFFSETS: [u32; 64] = {:?};",
        bishop_offsets
    )
    .unwrap();
    writeln!(
        f,
        "pub const PEXT_ROOK_OFFSETS: [u32; 64] = {:?};",
        rook_offsets
    )
    .unwrap();
    writeln!(
        f,
        "pub const PEXT_BISHOP_MASKS: [u64; 64] = {:?};",
        bishop_masks
    )
    .unwrap();
    writeln!(
        f,
        "pub const PEXT_ROOK_MASKS: [u64; 64] = {:?};",
        rook_masks
    )
    .unwrap();
}
