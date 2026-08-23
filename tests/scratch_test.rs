use chess_base::bitboard;

#[test]
fn test_scratch() {
    assert_eq!(bitboard::bb_scan_reverse(2), 1);
}
