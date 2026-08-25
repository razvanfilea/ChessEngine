use std::time::Instant;

use lucky_chess::{board::Board, perft::perft, uci::UciState};

fn main() {
    let start_pos = Board::start_pos();
    assert_eq!(perft(&start_pos, 1), 20);
    assert_eq!(perft(&start_pos, 2), 400);
    assert_eq!(perft(&start_pos, 3), 8902);
    assert_eq!(perft(&start_pos, 4), 197281);
    assert_eq!(perft(&start_pos, 5), 4865609);

    let instant = Instant::now();
    assert_eq!(perft(&start_pos, 6), 119060324);
    println!("Perft 6 end: {:?}", instant.elapsed());

    let instant = Instant::now();
    assert_eq!(perft(&start_pos, 7), 3195901860);
    println!("Perft 7 end: {:?}", instant.elapsed());

    let instant = Instant::now();
    assert_eq!(perft(&start_pos, 8), 84998978956);
    println!("Perft 8 end: {:?}", instant.elapsed());

    // let mut uci = UciState::default();
    //
    // while uci.uci_loop() {}
}
