use chess_base::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GenType {
    Captures,
    Quiets,
    NonEvasions,
    Evasions,
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
    const CAPTURES: bool = matches!(
        Self::TYPE,
        GenType::Captures | GenType::NonEvasions | GenType::Evasions
    );
    const QUIETS: bool = matches!(
        Self::TYPE,
        GenType::Quiets | GenType::NonEvasions | GenType::Evasions
    );
    const EVASIONS: bool = matches!(Self::TYPE, GenType::Evasions);
}
pub struct Captures;
impl MoveGenType for Captures {
    const TYPE: GenType = GenType::Captures;
}

pub struct Quiets;
impl MoveGenType for Quiets {
    const TYPE: GenType = GenType::Quiets;
}

pub struct NonEvasions;
impl MoveGenType for NonEvasions {
    const TYPE: GenType = GenType::NonEvasions;
}

pub struct Evasions;
impl MoveGenType for Evasions {
    const TYPE: GenType = GenType::Evasions;
}
