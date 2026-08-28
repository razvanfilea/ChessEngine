use bitflags::bitflags;

use crate::Sq;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
    pub struct CastlingRights: u8 {
        const WHITE_00 = 1 << 0;
        const WHITE_000 = 1 << 1;
        const BLACK_00 = 1 << 2;
        const BLACK_000 = 1 << 3;

        const WHITE_ANY = Self::WHITE_00.bits() | Self::WHITE_000.bits();
        const BLACK_ANY = Self::BLACK_00.bits() | Self::BLACK_000.bits();
        const ALL = Self::WHITE_ANY.bits() | Self::BLACK_ANY.bits();
    }
}

static CASTLING_RIGHTS_MASKS: [CastlingRights; Sq::NB] = {
    let mut masks = [CastlingRights::ALL; Sq::NB];
    masks[Sq::A1 as usize] = CastlingRights::from_bits_truncate(
        CastlingRights::ALL.bits() & !CastlingRights::WHITE_000.bits(),
    );
    masks[Sq::E1 as usize] = CastlingRights::from_bits_truncate(
        CastlingRights::ALL.bits() & !CastlingRights::WHITE_ANY.bits(),
    );
    masks[Sq::H1 as usize] = CastlingRights::from_bits_truncate(
        CastlingRights::ALL.bits() & !CastlingRights::WHITE_00.bits(),
    );
    masks[Sq::A8 as usize] = CastlingRights::from_bits_truncate(
        CastlingRights::ALL.bits() & !CastlingRights::BLACK_000.bits(),
    );
    masks[Sq::E8 as usize] = CastlingRights::from_bits_truncate(
        CastlingRights::ALL.bits() & !CastlingRights::BLACK_ANY.bits(),
    );
    masks[Sq::H8 as usize] = CastlingRights::from_bits_truncate(
        CastlingRights::ALL.bits() & !CastlingRights::BLACK_00.bits(),
    );
    masks
};

impl CastlingRights {
    /// Returns the mask to AND with the current rights when a move travels
    /// `from` -> `to`, clearing any rights invalidated by that move.
    #[inline(always)]
    pub const fn mask_for_move(from: Sq, to: Sq) -> CastlingRights {
        CastlingRights::from_bits_truncate(
            CASTLING_RIGHTS_MASKS[from as usize].bits() & CASTLING_RIGHTS_MASKS[to as usize].bits(),
        )
    }
}
