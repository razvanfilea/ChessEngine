use chess_engine::perft::{
    perft_kiwipete, perft_pos3, perft_pos4, perft_pos4_mirrored, perft_pos5, perft_pos6,
    perft_start,
};

#[test]
fn test_perft_start_position() {
    let depth = if cfg!(miri) { 2 } else { 5 };
    perft_start(depth);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_kiwipete() {
    perft_kiwipete(4);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_3() {
    perft_pos3(5);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_4() {
    perft_pos4(4);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_4_mirrored() {
    perft_pos4_mirrored(4);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_5() {
    perft_pos5(4);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_6() {
    perft_pos6(4);
}
