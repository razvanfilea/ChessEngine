use std::mem::MaybeUninit;

use chess_base::prelude::*;

pub struct MoveList {
    moves: [MaybeUninit<Move>; 256],
    size: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            size: 0,
        }
    }
}

impl MoveList {
    pub const fn as_ptr(&mut self) -> MoveListPtr {
        MoveListPtr(self.current_ptr())
    }

    pub const fn update_size(&mut self, new_position: MoveListPtr) {
        let size = unsafe { new_position.0.offset_from(self.current_ptr()) };
        self.size = size as usize;
    }

    pub const fn as_slice(&self) -> &[Move] {
        unsafe { core::slice::from_raw_parts(self.moves.as_ptr() as *const Move, self.size) }
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    const fn current_ptr(&mut self) -> *mut Move {
        (unsafe { self.moves.as_mut_ptr().add(self.size) }) as *mut Move
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MoveListPtr(pub *mut Move);

impl MoveListPtr {
    #[inline(always)]
    pub const fn push(&mut self, from: Sq, to: Sq, flags: MoveFlags) {
        unsafe {
            self.0.write(Move::new(from, to, flags));
            self.0 = self.0.add(1);
        }
    }

    #[inline(always)]
    pub const fn push_promotions(&mut self, from: Sq, to: Sq, is_capture: bool) {
        let moves = if is_capture {
            [
                Move::new(from, to, MoveFlags::PromoCaptureQueen),
                Move::new(from, to, MoveFlags::PromoCaptureRook),
                Move::new(from, to, MoveFlags::PromoCaptureBishop),
                Move::new(from, to, MoveFlags::PromoCaptureKnight),
            ]
        } else {
            [
                Move::new(from, to, MoveFlags::PromoQueen),
                Move::new(from, to, MoveFlags::PromoRook),
                Move::new(from, to, MoveFlags::PromoBishop),
                Move::new(from, to, MoveFlags::PromoKnight),
            ]
        };

        unsafe {
            let ptr = self.0 as *mut [Move; 4];
            ptr.write(moves);
            self.0 = self.0.add(4);
        }
    }
}
