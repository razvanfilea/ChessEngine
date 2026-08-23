use std::hint::unreachable_unchecked;

use crate::{Pieces, Sq};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move(u16);

impl Move {
    pub const fn new(from: Sq, to: Sq, flags: MoveFlags) -> Self {
        Self(from.as_u8() as u16 | ((to.as_u8() as u16) << 6) | ((flags as u16) << 12))
    }

    #[inline(always)]
    pub const fn new_quiet(from: Sq, to: Sq) -> Self {
        Self::new(from, to, MoveFlags::Quiet)
    }

    #[inline(always)]
    pub const fn from(self) -> Sq {
        unsafe { Sq::from_raw_unchecked((self.0 & 63) as u8) }
    }

    #[inline(always)]
    pub const fn to(self) -> Sq {
        unsafe { Sq::from_raw_unchecked(((self.0 >> 6) & 63) as u8) }
    }

    #[inline(always)]
    pub const fn flags(self) -> MoveFlags {
        unsafe { std::mem::transmute(self.flags_bits()) }
    }

    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self.flags_bits() & 0b0100) != 0
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.flags_bits() & 0b1000) != 0
    }

    #[inline(always)]
    pub const fn promotion_piece(self) -> Option<Pieces> {
        if self.is_promotion() {
            match self.flags_bits() & 0b0011 {
                0 => Some(Pieces::Knight),
                1 => Some(Pieces::Bischop),
                2 => Some(Pieces::Rook),
                3 => Some(Pieces::Queen),
                _ => unsafe {
                    debug_assert!(false, "unreachable promotion bits");
                    unreachable_unchecked()
                },
            }
        } else {
            None
        }
    }

    const fn flags_bits(self) -> u8 {
        (self.0 >> 12) as u8
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MoveFlags {
    #[default]
    Quiet = 0b0000,
    DoublePawn = 0b0001,
    CastleKing = 0b0010,
    CastleQueen = 0b0011,
    Capture = 0b0100,
    EnPassant = 0b0101,
    PromoKnight = 0b1000,
    PromoBishop = 0b1001,
    PromoRook = 0b1010,
    PromoQueen = 0b1011,
    PromoCaptureKnight = 0b1100,
    PromoCaptureBishop = 0b1101,
    PromoCaptureRook = 0b1110,
    PromoCaptureQueen = 0b1111,
}

impl MoveFlags {}
