use chess_base::prelude::*;
use lucky_chess::move_gen::MoveList;

#[test]
fn test_move_list_push_and_slice() {
    let mut list = MoveList::default();
    assert_eq!(list.as_slice().len(), 0);

    let mut ptr = list.as_ptr();
    ptr.push(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
    ptr.push(Sq::G1, Sq::F3, MoveFlags::Quiet);
    ptr.push(Sq::F3, Sq::E5, MoveFlags::Capture);

    list.update_size(ptr);
    let slice = list.as_slice();
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0].from(), Sq::E2);
    assert_eq!(slice[0].to(), Sq::E4);
    assert_eq!(slice[0].flags(), MoveFlags::DoublePawn);

    assert_eq!(slice[1].from(), Sq::G1);
    assert_eq!(slice[1].to(), Sq::F3);
    assert_eq!(slice[1].flags(), MoveFlags::Quiet);

    assert_eq!(slice[2].from(), Sq::F3);
    assert_eq!(slice[2].to(), Sq::E5);
    assert_eq!(slice[2].flags(), MoveFlags::Capture);
}

#[test]
fn test_move_list_push_promotions() {
    let mut list = MoveList::default();
    let mut ptr = list.as_ptr();

    // Push quiet promotions (4 moves)
    ptr.push_promotions(Sq::E7, Sq::E8, false);
    // Push capture promotions (4 moves)
    ptr.push_promotions(Sq::E7, Sq::D8, true);
    list.update_size(ptr);

    let slice = list.as_slice();
    assert_eq!(slice.len(), 8);
    assert_eq!(slice[0].flags(), MoveFlags::PromoQueen);
    assert_eq!(slice[1].flags(), MoveFlags::PromoRook);
    assert_eq!(slice[2].flags(), MoveFlags::PromoBishop);
    assert_eq!(slice[3].flags(), MoveFlags::PromoKnight);

    assert_eq!(slice[4].flags(), MoveFlags::PromoCaptureQueen);
    assert_eq!(slice[5].flags(), MoveFlags::PromoCaptureRook);
    assert_eq!(slice[6].flags(), MoveFlags::PromoCaptureBishop);
    assert_eq!(slice[7].flags(), MoveFlags::PromoCaptureKnight);
}
