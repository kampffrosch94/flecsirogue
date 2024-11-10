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

impl<T> From<T> for Vec2f
where
    T: Into<(f32, f32)>,
{
    fn from(t: T) -> Self {
        let t: (f32, f32) = t.into();
        Self { x: t.0, y: t.1 }
    }
}

impl Vec2f {
    pub fn to_tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }
}
