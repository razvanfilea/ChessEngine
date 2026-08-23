use crate::board::Board;
use chess_base::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GenType {
    All,
    Captures,
    Quiets,
}

pub trait Player {
    const COLOR: Color;
}
pub struct White;
impl Player for White {
    const COLOR: Color = Color::White;
}
pub struct Black;
impl Player for Black {
    const COLOR: Color = Color::Black;
}

pub trait MoveGenType {
    const TYPE: GenType;
}
pub struct AllMoves;
impl MoveGenType for AllMoves {
    const TYPE: GenType = GenType::All;
}
pub struct GenCaptures;
impl MoveGenType for GenCaptures {
    const TYPE: GenType = GenType::Captures;
}
pub struct GenQuiets;
impl MoveGenType for GenQuiets {
    const TYPE: GenType = GenType::Quiets;
}

pub fn generate_moves<Us: Player, Type: MoveGenType>(board: &Board) {
    let us = Us::COLOR;
    let them = !us;
}
