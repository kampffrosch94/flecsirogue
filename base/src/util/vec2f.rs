use derive_more::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};
use tween::TweenValue;

// needed because orphan rules are annoying
#[derive(Clone, Copy, Debug, Add, Sub, Mul, Div, AddAssign, SubAssign, DivAssign)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl TweenValue for Vec2f {
    fn scale(self, scale: f32) -> Self {
        self * scale
    }
}


impl From<(f32, f32)> for Vec2f {
    fn from(t: (f32, f32)) -> Self {
        Self{x: t.0, y: t.1}
    }
}

impl From<Vec2f> for (f32, f32) {
    fn from(v: Vec2f) -> Self {
        (v.x, v.y)
    }
}
