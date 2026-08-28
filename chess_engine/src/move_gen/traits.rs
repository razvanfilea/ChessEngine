pub use chess_core::{Black, Player, White};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GenType {
    Captures,
    Quiets,
    NonEvasions,
    Evasions,
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
