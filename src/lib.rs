//! # shax
//!
//! **shax** is a work-in-progress chess engine ([GitHub](https://github.com/akolotilov/shax)).
//!
//! What works now:
//!
//! - Move generation and validation
//! - Full chess rules support (castling, en passant, promotion, repetition draws, 75-move rule)
//!
//! Maybe someday:
//!
//! - Search algorithm
//! - Position evaluation
//! - UCI implementation
//!
//! ## Example
//!
//! ```
//! use shax::board::Board;
//! use shax::notation::{Color, Move};
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut board = Board::default();
//!
//!     for mov in board.color_moves(Color::White) {
//!         // Every legal move for white
//!         println!("{mov:?}")
//!     }
//!
//!     // Make a move using long algrebraic notation (LAN)
//!     board.make_move(Move::from_lan("e2e4")?)?;
//!
//!     // Display current position
//!     println!("{board:#?}");
//!
//!     Ok(())
//! }
//! ```
//!

mod attacks;
pub mod board;
pub mod notation;
mod rays;

#[inline(always)]
fn bitscan_forward(bb: u64) -> usize {
    bb.trailing_zeros() as usize
}

#[inline(always)]
fn bitscan_reverse(bb: u64) -> usize {
    bb.leading_zeros() as usize ^ 63
}

#[allow(unused)]
const FILE_A: u64 = 0x0101010101010101;
#[allow(unused)]
const FILE_B: u64 = 0x0202020202020202;
#[allow(unused)]
const FILE_C: u64 = 0x0404040404040404;
#[allow(unused)]
const FILE_D: u64 = 0x0808080808080808;
#[allow(unused)]
const FILE_E: u64 = 0x1010101010101010;
#[allow(unused)]
const FILE_F: u64 = 0x2020202020202020;
#[allow(unused)]
const FILE_G: u64 = 0x4040404040404040;
#[allow(unused)]
const FILE_H: u64 = 0x8080808080808080;
#[allow(unused)]
const RANK_1: u64 = 0x00000000000000ff;
#[allow(unused)]
const RANK_2: u64 = 0x000000000000ff00;
#[allow(unused)]
const RANK_3: u64 = 0x0000000000ff0000;
#[allow(unused)]
const RANK_4: u64 = 0x00000000ff000000;
#[allow(unused)]
const RANK_5: u64 = 0x000000ff00000000;
#[allow(unused)]
const RANK_6: u64 = 0x0000ff0000000000;
#[allow(unused)]
const RANK_7: u64 = 0x00ff000000000000;
#[allow(unused)]
const RANK_8: u64 = 0xff00000000000000;
