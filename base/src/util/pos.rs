use std::ops::Sub;
use std::ops::{Add, AddAssign};

use derive_more::*;
use flecs_ecs::prelude::Component;
use nanoserde::{DeJson, SerJson};

#[derive(Clone, Copy, Hash, PartialEq, Eq, From, Into, Debug, Component, DeJson, SerJson)]
#[meta]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    #[allow(dead_code)]
    pub fn circle_around(&self, radius: u32) -> Vec<Self> {
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
    pub fn neighbors(&self) -> Vec<Self> {
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

    pub fn distance(&self, other: Self) -> i32 {
        let dx = i32::abs(self.x - other.x);
        let dy = i32::abs(self.y - other.y);
        i32::max(dx, dy)
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

impl Into<(isize, isize)> for &Pos {
    fn into(self) -> (isize, isize) {
        (self.x as _, self.y as _)
    }
}

impl From<(isize, isize)> for Pos {
    fn from(value: (isize, isize)) -> Self {
        Self {
            x: value.0 as _,
            y: value.1 as _,
        }
    }
}

#[derive(Clone, Mul, Copy, Hash, PartialEq, Eq, From, Into, Debug, Component, DeJson, SerJson)]
#[meta]
pub struct Direction {
    pub x: i32,
    pub y: i32,
}

impl Add<Direction> for Pos {
    type Output = Pos;

    fn add(self, rhs: Direction) -> Self::Output {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Direction> for Pos {
    fn add_assign(&mut self, rhs: Direction) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Direction> for Pos {
    type Output = Pos;

    fn sub(self, rhs: Direction) -> Self::Output {
        Pos {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<Pos> for Pos {
    type Output = Direction;

    fn sub(self, rhs: Pos) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
