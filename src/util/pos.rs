use std::ops::Sub;
use std::ops::{Add, AddAssign};

use derive_more::From;
use derive_more::Into;
use flecs_ecs::prelude::Component;

#[derive(Clone, Copy, Hash, PartialEq, From, Into, Debug, Component)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    #[allow(dead_code)]
    pub fn circle_around(self, radius: u32) -> Vec<Self> {
        let mut result = Vec::new();
        let px: i32 = self.x;
        let py: i32 = self.y;
        for r in 1..(radius + 1) {
            let r = r as i32;
            let start = if px >= r { px - r } else { 0 };
            for x in start..(px + r + 1) {
                if py >= r {
                    result.push((x, py - r).into());
                }
                result.push((x, py + r).into());
            }
            let start = if py + 1 >= r { py + 1 - r } else { 0 };
            for y in start..(py + r) {
                if px >= r {
                    result.push((px - r, y).into());
                }
                result.push((px + r, y).into());
            }
        }
        result
    }

    /// Gives the neighboring tiles to a Position
    /// Excludes borders
    /// Includes walls and empty Tiles
    pub fn neighbors(self) -> Vec<Self> {
        let x = self.x as i32;
        let y = self.y as i32;
        let poss: Vec<(i32, i32)> = vec![
            (x - 1, y),
            (x + 1, y),
            (x, y - 1),
            (x, y + 1),
            (x - 1, y - 1),
            (x - 1, y + 1),
            (x + 1, y - 1),
            (x + 1, y + 1),
        ];
        poss.into_iter()
            .filter(|&(x, y)| x >= 0 || y >= 0)
            .map(Into::into)
            .collect()
    }
}

impl Add<(i32, i32)> for Pos {
    type Output = Pos;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        Pos {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}

impl AddAssign<(i32, i32)> for Pos {
    fn add_assign(&mut self, rhs: (i32, i32)) {
        self.x += rhs.0;
        self.y += rhs.1;
    }
}

impl Sub<(i32, i32)> for Pos {
    type Output = Pos;

    fn sub(self, rhs: (i32, i32)) -> Self::Output {
        Pos {
            x: self.x - rhs.0,
            y: self.y - rhs.1,
        }
    }
}
