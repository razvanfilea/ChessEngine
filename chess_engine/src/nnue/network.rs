use std::hint::assert_unchecked;

use chess_core::prelude::*;

pub const INPUT_FEATURES: usize = 41024;
pub const LAYER_1_SIZE: usize = 256;
pub const LAYER_2_SIZE: usize = 32;
pub const LAYER_3_SIZE: usize = 32;

pub struct Network {
    pub feature_weights: Box<[i16; INPUT_FEATURES * LAYER_1_SIZE]>,
    pub feature_biases: Box<[i16; LAYER_1_SIZE]>,
    pub layer2_weights: Box<[i8; LAYER_1_SIZE * Color::NB * LAYER_2_SIZE]>,
    pub layer2_biases: Box<[i32; LAYER_2_SIZE]>,
    pub layer3_weights: Box<[i8; LAYER_2_SIZE * LAYER_3_SIZE]>,
    pub layer3_biases: Box<[i32; LAYER_3_SIZE]>,
    pub output_weights: Box<[i8; LAYER_3_SIZE]>,
    pub output_biases: i32,
}

// Helper functions using const generics to keep the parsing logic clean
fn read_i16<const N: usize>(bytes: &[u8], offset: &mut usize) -> Box<[i16; N]> {
    let slice = &bytes[*offset..*offset + N * 2];
    *offset += N * 2;
    slice
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .try_into()
        .unwrap()
}

fn read_i32<const N: usize>(bytes: &[u8], offset: &mut usize) -> Box<[i32; N]> {
    let slice = &bytes[*offset..*offset + N * 4];
    *offset += N * 4;
    slice
        .chunks_exact(4)
        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .try_into()
        .unwrap()
}

fn read_i8<const N: usize>(bytes: &[u8], offset: &mut usize) -> Box<[i8; N]> {
    let slice = &bytes[*offset..*offset + N];
    *offset += N;
    slice
        .iter()
        .map(|&b| b as i8)
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .try_into()
        .unwrap()
}

impl Network {
    pub fn load() -> Self {
        let bytes = include_bytes!("nn-04cf2b4ed1da.nnue");

        // Skip Header: version (4) + hash (4) + desc_len (4) = 12 bytes
        let desc_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let mut offset = 12 + desc_len;

        Self {
            feature_biases: read_i16(bytes, &mut offset),
            feature_weights: read_i16(bytes, &mut offset),

            layer2_biases: read_i32(bytes, &mut offset),
            layer2_weights: read_i8(bytes, &mut offset),

            layer3_biases: read_i32(bytes, &mut offset),
            layer3_weights: read_i8(bytes, &mut offset),

            output_biases: read_i32::<1>(bytes, &mut offset)[0],
            output_weights: read_i8(bytes, &mut offset),
        }
    }

    #[inline(always)]
    pub fn get_half_kp_weight<'a>(
        &'a self,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        piece_sq: Sq,
    ) -> &'a [i16; LAYER_1_SIZE] {
        let typ = piece.piece();
        debug_assert!(typ != Piece::King);
        unsafe {
            assert_unchecked(typ != Piece::King);
        }
        let piece_index = typ as usize + if king_color == piece.color() { 0 } else { 5 };

        let (oriented_ksq, oriented_psq) = if king_color == Color::White {
            (king_sq, piece_sq)
        } else {
            (king_sq.rotate_180(), piece_sq.rotate_180()) // Rotate 180 degrees for Black
        };

        let index = (oriented_ksq as usize * 641) + (piece_index * 64) + oriented_psq as usize + 1;
        let offset = index * LAYER_1_SIZE;

        unsafe {
            self.feature_weights[offset..offset + LAYER_1_SIZE]
                .try_into()
                .unwrap_unchecked()
        }
    }
}
