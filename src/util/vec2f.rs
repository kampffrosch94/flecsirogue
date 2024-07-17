use derive_more::{Add, AddAssign, Div, DivAssign, From, Mul, Sub, SubAssign};
use macroquad::prelude::Vec2;
use tween::TweenValue;

// needed because orphan rules are annoying
#[derive(Clone, Copy, Debug, Add, Sub, Mul, Div, From, AddAssign, SubAssign, DivAssign)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl TweenValue for Vec2f {
    fn scale(self, scale: f32) -> Self {
        self * scale
    }
}

impl From<Vec2> for Vec2f {
    fn from(value: Vec2) -> Self {
        Vec2f {
            x: value.x,
            y: value.y,
        }
    }
}

impl Into<Vec2> for Vec2f {
    fn into(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
