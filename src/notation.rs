use bitflags::bitflags;
use std::error;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CastlingRights : u8 {
        const WHITE_KINGSIDE = 0b1000;
        const WHITE_QUEENSIDE = 0b0100;
        const BLACK_KINGSIDE = 0b0010;
        const BLACK_QUEENSIDE = 0b0001;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMoveError {
    NotEnoughCharacters,
    BadPromotionPiece(char),
    BadSrcFile(char),
    BadSrcRank(char),
    BadDstFile(char),
    BadDstRank(char),
}

impl fmt::Display for ParseMoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotEnoughCharacters => write!(f, "not enough characters"),
            Self::BadPromotionPiece(c) => {
                write!(f, "expected promotion piece to be one of 'rnbq', got {c:?}")
            }
            Self::BadSrcFile(c) => {
                write!(f, "expected src file to be one of 'abcdefgh', got {c:?}")
            }
            Self::BadSrcRank(c) => {
                write!(f, "expected src rank to be one of '12345678', got {c:?}")
            }
            Self::BadDstFile(c) => {
                write!(f, "expected dst file to be one of 'abcdefgh', got {c:?}")
            }
            Self::BadDstRank(c) => {
                write!(f, "expected dst rank to be one of '12345678', got {c:?}")
            }
        }
    }
}

impl error::Error for ParseMoveError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// Regular move, including captures, that is not a promotion or castling move.
    Regular(RegularMove),

    /// Promotion move, where the pawn is promoted to the specified [`PromotionPiece`].
    Promotion(PromotionMove),

    /// Castling move, where the king is moved from its starting [`Square`] to the file G
    /// (kingside) or C (queenside).
    Castling(CastlingMove),
}

impl Move {
    /// Parses a move in Long Algebraic Notation (LAN). Examples:
    ///
    /// ```
    /// use shax::notation::{
    ///     CastlingMove, Move, ParseMoveError, PromotionMove, PromotionPiece, RegularMove, Square,
    /// };
    ///
    /// fn main() -> Result<(), ParseMoveError> {
    ///     assert_eq!(
    ///         Move::from_lan("e2e4")?,
    ///         Move::Regular(RegularMove {
    ///             src: Square::E2,
    ///             dst: Square::E4
    ///         })
    ///     );
    ///     assert_eq!(
    ///         Move::from_lan("e7e5")?,
    ///         Move::Regular(RegularMove {
    ///             src: Square::E7,
    ///             dst: Square::E5
    ///         })
    ///     );
    ///     assert_eq!(
    ///         Move::from_lan("e1g1")?,
    ///         Move::Castling(CastlingMove {
    ///             src: Square::E1,
    ///             dst: Square::G1
    ///         })
    ///     );
    ///     assert_eq!(
    ///         Move::from_lan("e7e8q")?,
    ///         Move::Promotion(PromotionMove {
    ///             src: Square::E7,
    ///             dst: Square::E8,
    ///             piece: PromotionPiece::Queen
    ///         })
    ///     );
    ///     Ok(())
    /// }
    /// ```
    pub fn from_lan(mov: &str) -> Result<Self, ParseMoveError> {
        let mut mov = mov.chars();

        let src_file_char = mov.next().ok_or(ParseMoveError::NotEnoughCharacters)?;
        let src_rank_char = mov.next().ok_or(ParseMoveError::NotEnoughCharacters)?;

        let (src_file, src_rank) = (
            src_file_char as isize - 'a' as isize,
            src_rank_char as isize - '1' as isize,
        );

        if !(0..8).contains(&src_file) {
            return Err(ParseMoveError::BadSrcFile(src_file_char));
        }
        if !(0..8).contains(&src_rank) {
            return Err(ParseMoveError::BadSrcRank(src_rank_char));
        }

        let src = Square::from_repr((src_rank * 8 + src_file) as usize).unwrap();

        let dst_file_char = mov.next().ok_or(ParseMoveError::NotEnoughCharacters)?;
        let dst_rank_char = mov.next().ok_or(ParseMoveError::NotEnoughCharacters)?;

        let (dst_file, dst_rank) = (
            dst_file_char as isize - 'a' as isize,
            dst_rank_char as isize - '1' as isize,
        );

        if !(0..8).contains(&dst_file) {
            return Err(ParseMoveError::BadDstFile(dst_file_char));
        }
        if !(0..8).contains(&dst_rank) {
            return Err(ParseMoveError::BadDstRank(dst_rank_char));
        }

        let dst = Square::from_repr((dst_rank * 8 + dst_file) as usize).unwrap();

        let promote_to = match mov.next() {
            Some('r') => Some(PromotionPiece::Rook),
            Some('n') => Some(PromotionPiece::Knight),
            Some('b') => Some(PromotionPiece::Bishop),
            Some('q') => Some(PromotionPiece::Queen),
            Some(' ') => None,
            Some('\n') => None,
            Some('\r') => None,
            None => None,
            Some(other) => return Err(ParseMoveError::BadPromotionPiece(other)),
        };

        match promote_to {
            Some(piece) => Ok(Move::Promotion(PromotionMove { src, dst, piece })),
            None => {
                if matches!(
                    (src, dst),
                    (Square::E1, Square::G1)
                        | (Square::E8, Square::G8)
                        | (Square::E1, Square::C1)
                        | (Square::E8, Square::C8)
                ) {
                    Ok(Move::Castling(CastlingMove { src, dst }))
                } else {
                    Ok(Move::Regular(RegularMove { src, dst }))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingMove {
    pub src: Square,
    pub dst: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegularMove {
    pub src: Square,
    pub dst: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotionMove {
    pub src: Square,
    pub dst: Square,
    pub piece: PromotionPiece,
}

impl PromotionMove {
    pub fn all(src: Square, dst: Square) -> Vec<Move> {
        PromotionPiece::iter()
            .map(|piece| Move::Promotion(PromotionMove { src, dst, piece }))
            .collect()
    }
}

#[derive(FromRepr, EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Winner {
    White,
    Black,
    Draw,
}

#[derive(FromRepr, EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Color {
    White,
    Black,
}

impl From<Color> for Winner {
    fn from(color: Color) -> Self {
        Winner::from_repr(color as usize).unwrap()
    }
}

impl Color {
    /// ```
    /// use shax::notation::{Color, CastlingRights};
    ///
    /// assert_eq!(Color::White.opposite(), Color::Black);
    /// assert_eq!(Color::Black.opposite(), Color::White);
    /// ```
    pub const fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// ```
    /// use shax::notation::{Color, CastlingRights};
    ///
    /// assert_eq!(
    ///     Color::White.kingside_castling_flag(),
    ///     CastlingRights::WHITE_KINGSIDE
    /// );
    /// assert_eq!(
    ///     Color::Black.kingside_castling_flag(),
    ///     CastlingRights::BLACK_KINGSIDE
    /// );
    /// ```
    pub const fn kingside_castling_flag(&self) -> CastlingRights {
        match self {
            Color::White => CastlingRights::WHITE_KINGSIDE,
            Color::Black => CastlingRights::BLACK_KINGSIDE,
        }
    }

    /// ```
    /// use shax::notation::{Color, CastlingRights};
    ///
    /// assert_eq!(
    ///     Color::White.queenside_castling_flag(),
    ///     CastlingRights::WHITE_QUEENSIDE
    /// );
    /// assert_eq!(
    ///     Color::Black.queenside_castling_flag(),
    ///     CastlingRights::BLACK_QUEENSIDE
    /// );
    /// ```
    pub const fn queenside_castling_flag(&self) -> CastlingRights {
        match self {
            Color::White => CastlingRights::WHITE_QUEENSIDE,
            Color::Black => CastlingRights::BLACK_QUEENSIDE,
        }
    }

    /// ```
    /// use shax::notation::Color;
    ///
    /// assert_eq!(Color::White.promotion_rank(), 7);
    /// assert_eq!(Color::Black.promotion_rank(), 0);
    /// ```
    pub const fn promotion_rank(&self) -> usize {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

#[derive(FromRepr, EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(FromRepr, EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum PromotionPiece {
    Rook = 1,
    Knight = 2,
    Bishop = 3,
    Queen = 4,
}

impl From<PromotionPiece> for Piece {
    fn from(piece: PromotionPiece) -> Self {
        Piece::from_repr(piece as usize).unwrap()
    }
}

#[rustfmt::skip]
#[derive(FromRepr, EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    pub fn rank(&self) -> usize {
        *self as usize & 56
    }

    pub fn file(&self) -> usize {
        *self as usize % 8
    }
}
