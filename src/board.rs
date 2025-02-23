use crate::attacks;
use crate::bitscan_forward;
use crate::notation::{
    CastlingMove, CastlingRights, Color, Move, Piece, PromotionMove, RegularMove, Square, Winner,
};
use std::error;
use std::fmt;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    NothingToMove,
    IllegalMove,
    PinnedMove,
    GameEnded,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NothingToMove => write!(f, "nothing to move"),
            Self::IllegalMove => write!(f, "illegal move"),
            Self::PinnedMove => write!(f, "pinned move"),
            Self::GameEnded => write!(f, "game ended"),
        }
    }
}

impl error::Error for MoveError {}

#[derive(Clone)]
pub struct Board {
    bitboards: [[u64; 6]; 2],
    en_passant: Option<u64>,
    active: Color,
    winner: Option<Winner>,
    castling: CastlingRights,

    /// History for detecting fivefold repetition (FIDE 9.6.1).
    ///
    /// Per [FIDE rules](https://handbook.fide.com/chapter/E012023), the game is drawn
    /// if the same position occurs five times. The history is cleared after pawn moves
    /// and captures because these moves make it impossible to return to previous positions.
    history: Vec<[[u64; 6]; 2]>,

    /// Counter for the 75-move rule (FIDE 9.6.2).
    ///
    /// The game is drawn when this reaches 150 (75 moves by each player). According to
    /// [FIDE rules](https://handbook.fide.com/chapter/E012023), checkmate takes precedence
    /// if achieved on the move that would otherwise trigger this draw condition.
    halfclock: usize,
}

impl fmt::Debug for Board {
    /// Example of starting position:
    ///
    /// ```
    /// use shax::board::Board;
    ///
    /// let board = Board::default();
    /// assert_eq!(
    ///     format!("{board:#?}").split('\n').collect::<Vec<&str>>(),
    ///     [
    ///         "r n b q k b n r ",
    ///         "p p p p p p p p ",
    ///         ". . . . . . . . ",
    ///         ". . . . . . . . ",
    ///         ". . . . . . . . ",
    ///         ". . . . . . . . ",
    ///         "P P P P P P P P ",
    ///         "R N B Q K B N R ",
    ///     ]
    /// );
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let chars = ['P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k'];
            let mut ascii = String::new();
            for rank in (0..8).rev() {
                for file in 0..8 {
                    let square = Square::from_repr(rank * 8 + file).unwrap();
                    let char = match self.on_square(square) {
                        Some((color, piece)) => chars[piece as usize + (color as usize * 6)],
                        None => '.',
                    };
                    ascii.push(char);
                    ascii.push(' ');
                }
                if rank != 0 {
                    ascii.push('\n');
                }
            }
            f.write_str(&ascii)?;
        } else {
            f.debug_struct("Board")
                .field("active", &self.active())
                .field("winner", &self.winner())
                .field("en_passant", &self.en_passant())
                .field("castling", &self.castling())
                .field("halfclock", &self.halfclock)
                .finish_non_exhaustive()?;
        }
        Ok(())
    }
}

const DEFAULT: [[u64; 6]; 2] = [
    [
        // White
        0x000000000000ff00, // Pawn
        0x0000000000000081, // Rook
        0x0000000000000042, // Knight
        0x0000000000000024, // Bishop
        0x0000000000000008, // Queen
        0x0000000000000010, // King
    ],
    [
        // Black
        0x00ff000000000000, // Pawn
        0x8100000000000000, // Rook
        0x4200000000000000, // Knight
        0x2400000000000000, // Bishop
        0x0800000000000000, // Queen
        0x1000000000000000, // King
    ],
];

impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: DEFAULT,
            en_passant: None,
            castling: CastlingRights::all(),
            active: Color::White,
            winner: None,
            history: Vec::with_capacity(150),
            halfclock: 0,
        }
    }
}

impl Board {
    /// Returns the en passant target square, if an en passant capture is possible this turn.
    pub fn en_passant(&self) -> Option<Square> {
        match self.en_passant {
            Some(bb) => Square::from_repr(bb.trailing_zeros() as usize),
            None => None,
        }
    }

    /// Returns the color of the currently active player (whose turn it is).
    pub fn active(&self) -> Color {
        self.active
    }

    /// Returns the game winner if the game has concluded, or [`None`] if still in progress.
    pub fn winner(&self) -> Option<Winner> {
        self.winner
    }

    /// Returns current castling rights for both players.
    /// These flags track whether kings/rooks have moved, not directly indicating legal moves.
    /// Use [`Board::castling_moves`] to calculate actual castling possibilities.
    pub fn castling(&self) -> CastlingRights {
        self.castling
    }

    fn bitboard(&self, color: Color, piece: Piece) -> u64 {
        self.bitboards[color as usize][piece as usize]
    }

    fn bitboard_mut(&mut self, color: Color, piece: Piece) -> &mut u64 {
        &mut self.bitboards[color as usize][piece as usize]
    }

    fn occupied_mask(&self) -> u64 {
        self.bitboards.iter().flatten().fold(0, |acc, bb| acc | bb)
    }

    fn occupied_by_color_mask(&self, color: Color) -> u64 {
        self.bitboards[color as usize]
            .iter()
            .fold(0, |acc, bb| acc | bb)
    }

    /// Returns an iterator over all squares occupied by the specified color and piece type.
    pub fn squares(&self, color: Color, piece: Piece) -> impl Iterator<Item = Square> {
        let bitboard = self.bitboard(color, piece);
        (0..64).filter_map(move |dst| {
            if (bitboard & (1 << dst)) != 0 {
                Square::from_repr(dst)
            } else {
                None
            }
        })
    }

    /// Returns the (color, piece) pair occupying the given square, if any.
    pub fn on_square(&self, square: Square) -> Option<(Color, Piece)> {
        self.flat_enumerate()
            .filter_map(|(color, piece, bb)| {
                if *bb & (1 << square as usize) != 0 {
                    Some((color, piece))
                } else {
                    None
                }
            })
            .next()
    }

    /// Attempts to execute a move on the board.
    pub fn make_move(&mut self, mov: Move) -> Result<(), MoveError> {
        if self.winner.is_some() {
            return Err(MoveError::GameEnded);
        }

        match mov {
            Move::Regular(regular) => {
                let (_, piece) = self
                    .on_square(regular.src)
                    .ok_or(MoveError::NothingToMove)?;

                self.validate_move(self.active, piece, regular.src, regular.dst)?;
                self.execute_regular_move(self.active, piece, regular);
            }
            Move::Promotion(promotion) => {
                self.validate_move(self.active, Piece::Pawn, promotion.src, promotion.dst)?;
                self.execute_promotion_move(self.active, promotion);
            }
            Move::Castling(castling) => {
                if self
                    .castling_moves(self.active, castling.src)
                    .any(|mov| mov == Move::Castling(CastlingMove { ..castling }))
                {
                    self.execute_castling_move(self.active, castling);
                } else {
                    return Err(MoveError::IllegalMove);
                }
            }
        }

        self.update_game_state(self.active);

        Ok(())
    }

    fn validate_move(
        &self,
        color: Color,
        piece: Piece,
        src: Square,
        dst: Square,
    ) -> Result<(), MoveError> {
        if !self.is_pseudo_legal_move(color, piece, src, dst) {
            return Err(MoveError::IllegalMove);
        }

        if self.is_move_pinned(color, piece, src, dst) {
            return Err(MoveError::PinnedMove);
        }

        Ok(())
    }

    fn execute_regular_move(&mut self, color: Color, piece: Piece, mov: RegularMove) {
        if let Some((blocker_color, blocker_piece)) = self.on_square(mov.dst) {
            self.reset_draw_conditions();
            self.remove_piece(blocker_color, blocker_piece, mov.dst);
        } else if Piece::Pawn == piece {
            self.reset_draw_conditions();
        } else {
            self.update_draw_conditions();
        }

        self.force_move(color, piece, mov.src, mov.dst);
        self.update_castling(color, piece, mov.src);
        self.update_en_passant(color, piece, mov.src, mov.dst);
    }

    fn execute_promotion_move(&mut self, color: Color, mov: PromotionMove) {
        self.reset_draw_conditions();

        if let Some((blocker_color, blocker_piece)) = self.on_square(mov.dst) {
            self.remove_piece(blocker_color, blocker_piece, mov.dst);
        }

        self.remove_piece(color, Piece::Pawn, mov.src);
        self.add_piece(color, mov.piece.into(), mov.dst);
    }

    fn execute_castling_move(&mut self, color: Color, mov: CastlingMove) {
        self.update_draw_conditions();
        match (mov.src, mov.dst) {
            (Square::E1, Square::G1) => {
                self.force_move(color, Piece::Rook, Square::H1, Square::F1);
                self.force_move(color, Piece::King, Square::E1, Square::G1);
            }
            (Square::E8, Square::G8) => {
                self.force_move(color, Piece::Rook, Square::H8, Square::F8);
                self.force_move(color, Piece::King, Square::E8, Square::G8);
            }
            (Square::E1, Square::C1) => {
                self.force_move(color, Piece::Rook, Square::A1, Square::D1);
                self.force_move(color, Piece::King, Square::E1, Square::C1);
            }
            (Square::E8, Square::C8) => {
                self.force_move(color, Piece::Rook, Square::A8, Square::D8);
                self.force_move(color, Piece::King, Square::E8, Square::C8);
            }
            _ => unreachable!(),
        }
    }

    fn update_en_passant(&mut self, color: Color, piece: Piece, src: Square, dst: Square) {
        let (src, dst) = (src as usize, dst as usize);
        self.en_passant = if piece == Piece::Pawn && src.abs_diff(dst) == 16 {
            Some(match color {
                Color::White => (1 << src) << 8,
                Color::Black => (1 << src) >> 8,
            })
        } else {
            None
        }
    }

    fn update_game_state(&mut self, moved_color: Color) {
        let opposite = moved_color.opposite();

        if !self.color_has_moves(opposite) {
            if self.is_king_attacked(opposite) {
                self.winner = Some(moved_color.into())
            } else {
                self.winner = Some(Winner::Draw)
            }
        } else if self.is_fivefold_repetition() || self.halfclock >= 150 {
            self.winner = Some(Winner::Draw)
        } else {
            self.winner = None
        }

        self.active = opposite;
    }

    fn update_castling(&mut self, color: Color, piece: Piece, src: Square) {
        match piece {
            Piece::King => self.handle_king_move(color),
            Piece::Rook => self.handle_rook_move(color, src),
            _ => (),
        }
    }

    fn handle_king_move(&mut self, color: Color) {
        self.castling
            .remove(color.kingside_castling_flag() | color.queenside_castling_flag());
    }

    fn handle_rook_move(&mut self, color: Color, src: Square) {
        let (kingside, queenside) = match color {
            Color::White => (Square::H1, Square::A1),
            Color::Black => (Square::H8, Square::A8),
        };

        if src == kingside {
            self.castling.remove(color.kingside_castling_flag())
        } else if src == queenside {
            self.castling.remove(color.queenside_castling_flag())
        }
    }

    fn reset_draw_conditions(&mut self) {
        self.history.clear();
        self.halfclock = 0;
    }

    fn update_draw_conditions(&mut self) {
        self.history.push(self.bitboards);
        self.halfclock += 1;
    }

    /// Returns all legal moves for pieces of the specified type and color.
    pub fn moves(&self, color: Color, piece: Piece) -> impl Iterator<Item = Move> + '_ {
        self.squares(color, piece)
            .flat_map(move |src| self.square_moves(color, piece, src))
    }

    /// Returns all legal moves available for the specified color.
    pub fn color_moves(&self, color: Color) -> impl Iterator<Item = Move> + '_ {
        Piece::iter().flat_map(move |piece| self.moves(color, piece))
    }

    /// Returns legal moves for a specific piece (color and type) originating from a given square.
    /// Useful for generating moves when a player selects a particular piece on the board.
    pub fn square_moves(
        &self,
        color: Color,
        piece: Piece,
        src: Square,
    ) -> impl Iterator<Item = Move> + '_ {
        let mask = self.pseudo_moves_mask(color, piece, src).unwrap_or(0);
        (0..64)
            .filter_map(move |dst| {
                if (mask & (1 << dst)) != 0 {
                    Square::from_repr(dst)
                } else {
                    None
                }
            })
            .filter(move |&dst| !self.is_move_pinned(color, piece, src, dst))
            .flat_map(move |dst| self.generate_moves(color, piece, src, dst))
    }

    /// Returns legal castling moves for the specified color's king.
    pub fn castling_moves(&self, color: Color, src: Square) -> impl Iterator<Item = Move> {
        let mut moves = Vec::new();
        let empty_mask = !self.occupied_mask();

        if self.castling.contains(color.kingside_castling_flag()) {
            let path_mask = match color {
                Color::White => 0x60, // f1 and g1
                Color::Black => 0x6000000000000000,
            };
            let mut path = match color {
                Color::White => [Square::E1, Square::F1, Square::G1].iter(),
                Color::Black => [Square::E1, Square::F8, Square::G8].iter(),
            };
            if empty_mask & path_mask == path_mask
                && path.all(|&sq| !self.is_square_attacked(color, sq))
            {
                moves.push(Move::Castling(CastlingMove {
                    src,
                    dst: Square::from_repr(src as usize + 2).unwrap(),
                }))
            }
        }

        if self.castling.contains(color.queenside_castling_flag()) {
            let path_mask = match color {
                Color::White => 0x0E, // b1, c1, d1
                Color::Black => 0x0E00000000000000,
            };
            let mut path = match color {
                Color::White => [Square::E1, Square::D1, Square::C1, Square::B1].iter(),
                Color::Black => [Square::E8, Square::D8, Square::C8, Square::B8].iter(),
            };
            if empty_mask & path_mask == path_mask
                && path.all(|&sq| !self.is_square_attacked(color, sq))
            {
                moves.push(Move::Castling(CastlingMove {
                    src,
                    dst: Square::from_repr(src as usize - 2).unwrap(),
                }))
            }
        }

        moves.into_iter()
    }

    /// Returns [`true`] if the specified color has any legal moves available.
    /// Useful for detecting checkmate (no moves + in check) or stalemate (no moves + not in check).
    pub fn color_has_moves(&self, color: Color) -> bool {
        self.color_moves(color).next().is_some()
    }

    fn pseudo_moves_mask(&self, color: Color, piece: Piece, src: Square) -> Option<u64> {
        let friendly = self.occupied_by_color_mask(color);
        let enemy = self.occupied_by_color_mask(color.opposite());
        let occupied = friendly | enemy;
        let bb = 1 << src as usize;

        let moves = match piece {
            Piece::Pawn => {
                (attacks::pawn_attacks(bb, color) & (enemy | self.en_passant.unwrap_or(0)))
                    | attacks::pawn_advances(src, color, occupied)
            }
            Piece::Rook => attacks::rook_attacks(src, occupied),
            Piece::Knight => attacks::knight_attacks(bb),
            Piece::Bishop => attacks::bishop_attacks(src, occupied),
            Piece::Queen => attacks::queen_attacks(src, occupied),
            Piece::King => attacks::king_attacks(bb),
        } & !friendly;

        (moves != 0).then_some(moves)
    }

    fn generate_moves(
        &self,
        color: Color,
        piece: Piece,
        src: Square,
        dst: Square,
    ) -> impl Iterator<Item = Move> {
        let mut moves = Vec::new();
        if Piece::Pawn == piece && dst.rank() == color.promotion_rank() {
            moves.extend(PromotionMove::all(src, dst))
        } else if Piece::King == piece {
            let mut castling_moves = self.castling_moves(color, src).peekable();
            if castling_moves.peek().is_some() {
                moves.extend(castling_moves)
            } else {
                moves.push(Move::Regular(RegularMove { src, dst }))
            }
        } else {
            moves.push(Move::Regular(RegularMove { src, dst }))
        }
        moves.into_iter()
    }

    fn is_fivefold_repetition(&self) -> bool {
        self.history
            .iter()
            .filter(|&bitboards| *bitboards == self.bitboards)
            .count()
            >= 5
    }

    fn is_square_attacked(&self, color: Color, square: Square) -> bool {
        let opponent = color.opposite();
        let occupied = self.occupied_mask();

        let pawns = self.bitboard(opponent, Piece::Pawn);
        if attacks::pawn_attacks(1 << square as usize, color) & pawns != 0 {
            return true;
        }

        let knights = self.bitboard(opponent, Piece::Knight);
        if attacks::knight_attacks(1 << square as usize) & knights != 0 {
            return true;
        }

        let kings = self.bitboard(opponent, Piece::King);
        if attacks::king_attacks(1 << square as usize) & kings != 0 {
            return true;
        }

        let bishops = self.bitboard(opponent, Piece::Bishop);
        let rooks = self.bitboard(opponent, Piece::Rook);
        let queens = self.bitboard(opponent, Piece::Queen);

        (attacks::bishop_attacks(square, occupied) & (bishops | queens) != 0)
            || (attacks::rook_attacks(square, occupied) & (rooks | queens) != 0)
    }

    fn is_king_attacked(&self, color: Color) -> bool {
        let king = self.bitboard(color, Piece::King);
        match Square::from_repr(bitscan_forward(king)) {
            Some(sq) => self.is_square_attacked(color, sq),
            None => false,
        }
    }

    fn is_pseudo_legal_move(&self, color: Color, piece: Piece, src: Square, dst: Square) -> bool {
        let moves_mask = self.pseudo_moves_mask(color, piece, src);
        moves_mask.is_some_and(|mask| mask & (1 << dst as usize) != 0)
    }

    fn is_move_pinned(&self, color: Color, piece: Piece, src: Square, dst: Square) -> bool {
        let mut board = self.clone();
        board.force_move(color, piece, src, dst);
        if let Some(blocker) = self.on_square(dst) {
            let (color, piece) = blocker;
            board.remove_piece(color, piece, dst);
        }
        board.is_king_attacked(color)
    }

    fn flat_enumerate(&self) -> impl Iterator<Item = (Color, Piece, &u64)> {
        self.bitboards
            .iter()
            .enumerate()
            .flat_map(|(color, pieces)| {
                pieces.iter().enumerate().map(move |(piece, bb)| {
                    (
                        Color::from_repr(color).unwrap(),
                        Piece::from_repr(piece).unwrap(),
                        bb,
                    )
                })
            })
    }

    fn force_move(&mut self, color: Color, piece: Piece, src: Square, dst: Square) {
        self.remove_piece(color, piece, src);
        self.add_piece(color, piece, dst);
    }

    fn remove_piece(&mut self, color: Color, piece: Piece, square: Square) {
        *self.bitboard_mut(color, piece) &= !(1 << square as usize)
    }

    fn add_piece(&mut self, color: Color, piece: Piece, square: Square) {
        *self.bitboard_mut(color, piece) |= 1 << square as usize
    }
}
