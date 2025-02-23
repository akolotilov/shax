use crate::notation::{Color, Square};
use crate::rays::get_rays_cache;
use crate::{bitscan_forward, bitscan_reverse};
use crate::{FILE_A, FILE_B, FILE_G, FILE_H, RANK_2, RANK_7};

pub fn queen_attacks(square: Square, blockers: u64) -> u64 {
    rook_attacks(square, blockers) | bishop_attacks(square, blockers)
}

pub fn rook_attacks(square: Square, blockers: u64) -> u64 {
    let rays = get_rays_cache();
    let mut bb = 0;
    let square = square as usize;
    bb |= match rays.get(bitscan_forward(rays[square].north & blockers)) {
        Some(blocker) => !blocker.north & rays[square].north,
        None => rays[square].north,
    };
    bb |= match rays.get(bitscan_reverse(rays[square].south & blockers)) {
        Some(blocker) => !blocker.south & rays[square].south,
        None => rays[square].south,
    };
    bb |= match rays.get(bitscan_forward(rays[square].east & blockers)) {
        Some(blocker) => !blocker.east & rays[square].east,
        None => rays[square].east,
    };
    bb |= match rays.get(bitscan_reverse(rays[square].west & blockers)) {
        Some(blocker) => !blocker.west & rays[square].west,
        None => rays[square].west,
    };
    bb
}

pub fn bishop_attacks(square: Square, blockers: u64) -> u64 {
    let rays = get_rays_cache();
    let mut bb = 0;
    let square = square as usize;
    bb |= match rays.get(bitscan_forward(rays[square].north_east & blockers)) {
        Some(blocker) => !blocker.north_east & rays[square].north_east,
        None => rays[square].north_east,
    };
    bb |= match rays.get(bitscan_forward(rays[square].north_west & blockers)) {
        Some(blocker) => !blocker.north_west & rays[square].north_west,
        None => rays[square].north_west,
    };
    bb |= match rays.get(bitscan_reverse(rays[square].south_east & blockers)) {
        Some(blocker) => !blocker.south_east & rays[square].south_east,
        None => rays[square].south_east,
    };
    bb |= match rays.get(bitscan_reverse(rays[square].south_west & blockers)) {
        Some(blocker) => !blocker.south_west & rays[square].south_west,
        None => rays[square].south_west,
    };
    bb
}

#[inline(always)]
pub fn king_attacks(bb: u64) -> u64 {
    (bb << 8)
        | (bb >> 8)
        | ((bb << 1) & !FILE_A)
        | ((bb >> 1) & !FILE_H)
        | ((bb >> 7) & !FILE_A)
        | ((bb << 7) & !FILE_H)
        | ((bb << 9) & !FILE_A)
        | ((bb >> 9) & !FILE_H)
}

#[inline(always)]
pub fn pawn_attacks(bb: u64, color: Color) -> u64 {
    match color {
        Color::White => ((bb << 9) & !FILE_A) | ((bb << 7) & !FILE_H),
        Color::Black => ((bb >> 9) & !FILE_H) | ((bb >> 7) & !FILE_A),
    }
}

#[inline(always)]
pub fn pseudo_pawn_advances(bb: u64, color: Color) -> u64 {
    match color {
        Color::White => (bb << 8) | ((bb & RANK_2) << 16),
        Color::Black => (bb >> 8) | ((bb & RANK_7) >> 16),
    }
}

pub fn pawn_advances(square: Square, color: Color, blockers: u64) -> u64 {
    let rays = get_rays_cache();

    let bb = 1 << square as usize;
    let advances = pseudo_pawn_advances(bb, color);
    match color {
        Color::White => match rays.get(bitscan_forward(advances & blockers)) {
            Some(ray) => advances & !blockers & !ray.north,
            None => advances & !blockers,
        },
        Color::Black => match rays.get(bitscan_reverse(advances & blockers)) {
            Some(ray) => advances & !blockers & !ray.south,
            None => advances & !blockers,
        },
    }
}

#[inline(always)]
pub fn knight_attacks(bb: u64) -> u64 {
    ((bb << 6) & !(FILE_G | FILE_H))
        | ((bb << 15) & !FILE_H)
        | ((bb >> 6) & !(FILE_A | FILE_B))
        | ((bb >> 15) & !FILE_A)
        | ((bb >> 10) & !(FILE_G | FILE_H))
        | ((bb >> 17) & !FILE_H)
        | ((bb << 10) & !(FILE_A | FILE_B))
        | ((bb << 17) & !FILE_A)
}

#[cfg(test)]
mod tests {
    use super::{Color::*, Square::*, *};

    #[test]
    fn test_rook_attacks_corners() {
        assert_eq!(rook_attacks(A1, 0x01648c2412801480), 0x01010101010101fe);
        assert_eq!(rook_attacks(H1, 0x8005640832062001), 0x808080808080807f);
        assert_eq!(rook_attacks(A8, 0x8024085272045481), 0xfe01010101010101);
        assert_eq!(rook_attacks(H8, 0x0108220826401aa1), 0x7f80808080808080);
    }

    #[test]
    fn test_rook_attacks_empty() {
        assert_eq!(rook_attacks(A1, 0x0), 0x01010101010101fe);
        assert_eq!(rook_attacks(H1, 0x0), 0x808080808080807f);
        assert_eq!(rook_attacks(A8, 0x0), 0xfe01010101010101);
        assert_eq!(rook_attacks(H8, 0x0), 0x7f80808080808080);
    }

    #[test]
    fn test_rook_attacks_random() {
        assert_eq!(rook_attacks(D5, 0x8148004a008aa02b), 0x8087608080000);
        assert_eq!(rook_attacks(D5, 0x894800cb008aa02b), 0x8087608080000);
    }

    #[test]
    fn test_bishop_attacks_corners() {
        assert_eq!(bishop_attacks(A1, 0x81141244012100d0), 0x8040201008040200);
        assert_eq!(bishop_attacks(H1, 0xc19840d208020443), 0x0102040810204000);
        assert_eq!(bishop_attacks(A8, 0x7009e01561060aa9), 0x0002040810204080);
        assert_eq!(bishop_attacks(H8, 0x012c020980209051), 0x0040201008040201);
    }

    #[test]
    fn test_bishop_attacks_empty() {
        assert_eq!(bishop_attacks(A1, 0x0), 0x8040201008040200);
        assert_eq!(bishop_attacks(H1, 0x0), 0x0102040810204000);
        assert_eq!(bishop_attacks(A8, 0x0), 0x0002040810204080);
        assert_eq!(bishop_attacks(H8, 0x0), 0x0040201008040201);
    }

    #[test]
    fn test_bishop_attacks_random() {
        assert_eq!(bishop_attacks(D5, 0x00a20180002a0094), 0x22140014220000);
        assert_eq!(bishop_attacks(D5, 0x41a20180002a4194), 0x22140014220000);
    }

    #[test]
    fn test_pawn_attacks() {
        assert_eq!(pawn_attacks(0x0000018008002200, White), 0x2401400550000);
        assert_eq!(pawn_attacks(0x0010008400110000, Black), 0x028004a002a00);
        assert_eq!(pawn_attacks(0x9900000000000000, White), 0x0000000000000);
        assert_eq!(pawn_attacks(0x0000000000000099, Black), 0x0000000000000);
    }

    #[rustfmt::skip]
    #[test]
    fn test_pseudo_pawn_advances() {
        assert_eq!(pseudo_pawn_advances(0x000000100820c300, White), 0x1008e3c30000);
        assert_eq!(pseudo_pawn_advances(0x00c3200810000000, Black), 0xc3e308100000);
        assert_eq!(pseudo_pawn_advances(0xff00000000000000, White), 0x000000000000);
        assert_eq!(pseudo_pawn_advances(0x00000000000000ff, Black), 0x000000000000);
    }

    #[test]
    fn test_knight_attacks() {
        assert_eq!(knight_attacks(0x000000100800000), 0x02044024022040);
        assert_eq!(knight_attacks(0x800000000000020), 0x22140000508800);
    }

    #[test]
    fn test_king_attacks() {
        assert_eq!(king_attacks(0x0800008100000008), 0x141cc342c3001c14);
        assert_eq!(king_attacks(0x8100000000000081), 0x42c300000000c342);
    }
}
