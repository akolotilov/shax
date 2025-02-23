use shax::board::Board;
use shax::notation::{Move, Winner};

fn make_move(board: &mut Board, mov: &str) {
    board
        .make_move(Move::from_lan(mov).unwrap_or_else(|err| panic!("Parsing {mov} failed: {err}")))
        .unwrap_or_else(|err| panic!("Move {mov} failed: {err:?}, {board:?}\n{board:#?}"));
}

#[test]
fn test_anderssen_vs_kieseritzky() {
    /*
     *  Adolf Anderssen vs. Lionel Adalbert Bagration Felix Kieseritzky
     *  1-0 / London / 1851.??.??
     */
    let mut board = Board::default();
    for mov in [
        "e2e4", "e7e5", "f2f4", "e5f4", "f1c4", "d8h4", "e1f1", "b7b5", "c4b5", "g8f6", "g1f3",
        "h4h6", "d2d3", "f6h5", "f3h4", "h6g5", "h4f5", "c7c6", "g2g4", "h5f6", "h1g1", "c6b5",
        "h2h4", "g5g6", "h4h5", "g6g5", "d1f3", "f6g8", "c1f4", "g5f6", "b1c3", "f8c5", "c3d5",
        "f6b2", "f4d6", "c5g1", "e4e5", "b2a1", "f1e2", "b8a6", "f5g7", "e8d8", "f3f6", "g8f6",
        "d6e7",
    ] {
        make_move(&mut board, mov)
    }
    assert_eq!(board.winner(), Some(Winner::White));
}

#[test]
fn test_kasparov_vs_topalov() {
    /*
     *  Garry Kasparov vs. Veselin Topalov
     *  1-0 / Wijk aan Zee / 1999.??.??
     */
    let mut board = Board::default();
    for mov in [
        "e2e4", "d7d6", "d2d4", "g8f6", "b1c3", "g7g6", "c1e3", "f8g7", "d1d2", "c7c6", "f2f3",
        "b7b5", "g1e2", "b8d7", "e3h6", "g7h6", "d2h6", "c8b7", "a2a3", "e7e5", "e1c1", "d8e7",
        "c1b1", "a7a6", "e2c1", "e8c8", "c1b3", "e5d4", "d1d4", "c6c5", "d4d1", "d7b6", "g2g3",
        "c8b8", "b3a5", "b7a8", "f1h3", "d6d5", "h6f4", "b8a7", "h1e1", "d5d4", "c3d5", "b6d5",
        "e4d5", "e7d6", "d1d4", "c5d4", "e1e7", "a7b6", "f4d4", "b6a5", "b2b4", "a5a4", "d4c3",
        "d6d5", "e7a7", "a8b7", "a7b7", "d5c4", "c3f6", "a4a3", "f6a6", "a3b4", "c2c3", "b4c3",
        "a6a1", "c3d2", "a1b2", "d2d1", "h3f1", "d8d2", "b7d7", "d2d7", "f1c4", "b5c4", "b2h8",
        "d7d3", "h8a8", "c4c3", "a8a4", "d1e1", "f3f4", "f7f5", "b1c1", "d3d2", "a4a7",
    ] {
        make_move(&mut board, mov)
    }
    assert_eq!(board.winner(), None); // Topalov resigned
}

#[test]
fn test_draw_repetition() {
    let mut board = Board::default();
    for mov in [
        "e2e4", "e7e5", "f1e2", "f8e7", "e2f1", "e7f8", "f1e2", "f8e7", "e2f1", "e7f8", "f1e2",
        "f8e7", "e2f1", "e7f8", "f1e2", "f8e7", "e2f1", "e7f8", "f1e2", "f8e7", "e2f1", "e7f8",
    ] {
        make_move(&mut board, mov)
    }
    assert_eq!(board.winner(), Some(Winner::Draw));
}
