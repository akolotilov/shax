use shax::board::Board;
use shax::notation::Move;
use std::io::{self, Write};

#[allow(dead_code)]
fn ascii_u64(bb: u64) -> String {
    let mut ascii = String::with_capacity(64 * 2 + 8);
    for rank in (0..8).rev() {
        for file in 0..8 {
            let sq = rank * 8 + file;
            ascii.push(if (bb >> sq) & 1 != 0 { 'x' } else { '.' });
            ascii.push(' ');
        }
        if rank != 0 {
            ascii.push('\n');
        }
    }
    ascii
}

fn main() -> io::Result<()> {
    let mut board = Board::default();
    let mut buffer = String::new();
    let mut error = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        buffer.clear();

        if board.winner().is_none() {
            println!("{:#?} to move", board.active());
        } else {
            println!("Winner: {:?}", board.winner())
        }

        println!("{board:#?}");

        if !error.is_empty() {
            println!("ERROR: {error}");
        }

        print!("> ");
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        match Move::from_lan(&buffer) {
            Ok(mov) => match board.make_move(mov) {
                Ok(()) => (),
                Err(err) => {
                    error = format!("{err:?}");
                    continue;
                }
            },
            Err(err) => {
                error = format!("{err}");
                continue;
            }
        }

        error.clear();
    }
}
