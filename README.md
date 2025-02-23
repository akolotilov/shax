# shax

**shax** is a work-in-progress chess engine ([Docs](https://docs.rs/shax/latest/shax/)).

What works now:

- Move generation and validation
- Full chess rules support (castling, en passant, promotion, repetition draws, 75-move rule)

Maybe someday:

- Search algorithm
- Position evaluation
- UCI implementation

## Example

```rust
use shax::board::Board;
use shax::notation::{Color, Move};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut board = Board::default();

    for mov in board.color_moves(Color::White) {
        // Every legal move for white
        println!("{mov:?}")
    }

    // Make a move using Long Algrebraic Notation (LAN)
    board.make_move(Move::from_lan("e2e4")?)?;

    // Display current position
    println!("{board:#?}");

    Ok(())
}
```

## CLI

Since there’s no UCI yet, a basic CLI lets you play manually. Enter moves in Long Algebraic Notation:

- `e2e4` (pawn move)
- `e1g1` (castling)
- `a7a8q` (promotion to queen)

```
$ cargo run
White to move
r n b q k b n r
p p p p p p p p
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
P P P P P P P P
R N B Q K B N R
> e2e4
```
