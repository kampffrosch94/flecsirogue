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
