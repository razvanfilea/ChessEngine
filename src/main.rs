use lucky_chess::uci::UciState;

fn main() {
    // lucky_chess::perft::perft_start(8);
    // lucky_chess::perft::perft_kiwipete(5);
    // lucky_chess::perft::perft_pos3(6);
    // lucky_chess::perft::perft_pos4(5);
    // lucky_chess::perft::perft_pos4_mirrored(5);
    // lucky_chess::perft::perft_pos5(5);
    // lucky_chess::perft::perft_pos6(5);

    let mut uci = UciState::default();
    while uci.uci_loop() {}
}
