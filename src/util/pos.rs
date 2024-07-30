use std::ops::{Add, AddAssign};

use derive_more::From;
use flecs_ecs::prelude::Component;

#[derive(Clone, Copy, Hash, PartialEq, From, Debug, Component)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
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
