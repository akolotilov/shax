use shax::board::Board;
use shax::notation::{Color, Move};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut board = Board::default();

    for mov in board.color_moves(Color::White) {
        // Every legal move for white
        println!("{mov:?}")
    }

    // Make a move using long algrebraic notation (LAN)
    board.make_move(Move::from_lan("e2e4")?)?;

    // Display current position
    println!("{board:#?}");

    Ok(())
}
