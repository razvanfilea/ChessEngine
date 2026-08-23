use chess_base::prelude::*;

#[test]
fn test_color_defaults() {
    assert_eq!(Color::default(), Color::Black);
}

#[test]
fn test_color_as_bool() {
    assert_eq!(Color::Black.as_bool(), false);
    assert_eq!(Color::White.as_bool(), true);
}

#[test]
fn test_color_not() {
    assert_eq!(!Color::Black, Color::White);
    assert_eq!(!Color::White, Color::Black);
}

#[test]
fn test_color_from_bool() {
    assert_eq!(Color::from(false), Color::Black);
    assert_eq!(Color::from(true), Color::White);
}
