use crate::{FILE_A, FILE_H};
use std::sync::OnceLock;

static RAYS_CACHE: OnceLock<[Ray; 64]> = OnceLock::new();

pub fn get_rays_cache() -> &'static [Ray; 64] {
    RAYS_CACHE.get_or_init(get_rays)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ray {
    pub north: u64,
    pub south: u64,
    pub east: u64,
    pub west: u64,
    pub north_east: u64,
    pub north_west: u64,
    pub south_east: u64,
    pub south_west: u64,
}

fn get_rays() -> [Ray; 64] {
    let mut rays = [Ray::default(); 64];

    for (square, ray) in rays.iter_mut().enumerate() {
        ray.north = north_ray(square);
        ray.south = south_ray(square);
        ray.east = east_ray(square);
        ray.west = west_ray(square);
    }

    let mut north_east_slider: u64 = 0x8040201008040200;
    let mut south_east_slider: u64 = 0x0002040810204080;
    for file in 0..8 {
        let mut north_slider = north_east_slider;
        let mut south_slider = south_east_slider;
        for rank8 in (0..64).step_by(8) {
            rays[rank8 + file].north_east = north_slider;
            north_slider = north_one(north_slider);
        }
        for rank8 in (0..64).step_by(8).rev() {
            rays[rank8 + file].south_east = south_slider;
            south_slider = south_one(south_slider);
        }
        north_east_slider = east_one(north_east_slider);
        south_east_slider = east_one(south_east_slider);
    }

    let mut north_west_slider: u64 = 0x0102040810204000;
    let mut south_west_slider: u64 = 0x0040201008040201;
    for file in (0..8).rev() {
        let mut north_slider = north_west_slider;
        let mut south_slider = south_west_slider;
        for rank8 in (0..64).step_by(8) {
            rays[rank8 + file].north_west = north_slider;
            north_slider = north_one(north_slider);
        }
        for rank8 in (0..64).step_by(8).rev() {
            rays[rank8 + file].south_west = south_slider;
            south_slider = south_one(south_slider);
        }
        north_west_slider = west_one(north_west_slider);
        south_west_slider = west_one(south_west_slider);
    }

    rays
}

#[inline(always)]
fn north_ray(square: usize) -> u64 {
    (FILE_A << 8) << square
}

#[inline(always)]
fn south_ray(square: usize) -> u64 {
    (FILE_H >> 8) >> (square ^ 63)
}

#[inline(always)]
fn east_ray(square: usize) -> u64 {
    ((1 << (square | 7)) - (1 << square)) << 1
}

#[inline(always)]
fn west_ray(square: usize) -> u64 {
    (1 << square) - (1 << (square & 56))
}

#[inline(always)]
fn north_one(bb: u64) -> u64 {
    bb << 8
}

#[inline(always)]
fn south_one(bb: u64) -> u64 {
    bb >> 8
}

#[inline(always)]
fn east_one(bb: u64) -> u64 {
    (bb << 1) & !FILE_A
}

#[inline(always)]
fn west_one(bb: u64) -> u64 {
    (bb >> 1) & !FILE_H
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rays_north() {
        let rays = get_rays_cache();
        let (c3, f3) = (18, 21);
        assert_eq!(rays[c3].north, 0x0404040404000000);
        assert_eq!(rays[f3].north, 0x2020202020000000);
    }

    #[test]
    fn test_rays_south() {
        let rays = get_rays_cache();
        let (c6, f6) = (42, 45);
        assert_eq!(rays[c6].south, 0x0404040404);
        assert_eq!(rays[f6].south, 0x2020202020);
    }

    #[test]
    fn test_rays_west() {
        let rays = get_rays_cache();
        let (f3, f6) = (21, 45);
        assert_eq!(rays[f3].west, 0x0000001f0000);
        assert_eq!(rays[f6].west, 0x1f0000000000);
    }

    #[test]
    fn test_rays_east() {
        let rays = get_rays_cache();
        let (c3, c6) = (18, 42);
        assert_eq!(rays[c3].east, 0x000000f80000);
        assert_eq!(rays[c6].east, 0xf80000000000);
    }

    #[test]
    fn test_rays_north_east() {
        let rays = get_rays_cache();
        let (c3, f4, c5) = (18, 29, 34);
        assert_eq!(rays[c3].north_east, 0x8040201008000000);
        assert_eq!(rays[f4].north_east, 0x0000804000000000);
        assert_eq!(rays[c5].north_east, 0x2010080000000000);
    }

    #[test]
    fn test_rays_north_west() {
        let rays = get_rays_cache();
        let (f3, f4, c5) = (21, 29, 34);
        assert_eq!(rays[f3].north_west, 0x102040810000000);
        assert_eq!(rays[f4].north_west, 0x204081000000000);
        assert_eq!(rays[c5].north_west, 0x001020000000000);
    }

    #[test]
    fn test_rays_south_east() {
        let rays = get_rays_cache();
        let (c4, f5, c6) = (26, 37, 42);
        assert_eq!(rays[c4].south_east, 0x000081020);
        assert_eq!(rays[f5].south_east, 0x040800000);
        assert_eq!(rays[c6].south_east, 0x810204080);
    }

    #[test]
    fn test_rays_south_west() {
        let rays = get_rays_cache();
        let (f4, c5, f6) = (29, 34, 45);
        assert_eq!(rays[f4].south_west, 0x0000100804);
        assert_eq!(rays[c5].south_west, 0x0002010000);
        assert_eq!(rays[f6].south_west, 0x1008040201);
    }
}
