use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::helpers::GRID_WIDTH;

// empty types for size comparison
pub type Matrix = [f32; 16];

pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Matrix {
    let mut m = [0.; 16];
	
    m[0] = 2. / (right - left);
    m[5] = 2. / (top - bottom);
    m[10] = 2. / (far - near);
    m[12] = -(right + left) / (right - left);
    m[13] = -(top + bottom) / (top - bottom);
    m[14] = -(far + near) / (far - near);
    m[15] = 1.;
	
    m
}

// own math vectors since its easy and i dont focus on optimization yet
// only includes fns i actually use in the game

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct V4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
pub fn v4(x: f32, y: f32, z: f32, w: f32) -> V4 {
    V4 { x, y, z, w }
}

impl V4 {
    #[inline]
		pub fn zero() -> V4 {
        v4(0., 0., 0., 0.)
    }
	
    #[inline]
		pub fn one() -> V4 {
        v4(1., 1., 1., 1.)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub fn v2(x: f32, y: f32) -> V2 {
    V2 { x, y }
}

impl V2 {
    #[inline]
		pub fn zero() -> V2 {
        v2(0., 0.)
    }
	
    #[inline]
		pub fn one() -> V2 {
        v2(1., 1.)
    }
	
    #[inline]
		pub fn both(xy: f32) -> V2 {
        v2(xy, xy)
    }
}

impl From<usize> for V2 {
	fn from(num: usize) -> Self {
		v2(
		   (num % GRID_WIDTH) as f32,
		   (num / GRID_WIDTH) as f32,
		   )
	}
}

impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for V2 {
    type Output = V2;
	
    #[inline]
		fn add(self, other: Self::Output) -> Self::Output {
        v2(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for V2 {
    #[inline]
		fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl Sub for V2 {
    type Output = V2;
	
    #[inline]
		fn sub(self, other: Self::Output) -> Self::Output {
        v2(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for V2 {
    #[inline]
		fn sub_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x - other.x,
            y: self.y - other.y,
        };
    }
}

impl Div<V2> for V2 {
    type Output = V2;
	
    #[inline]
		fn div(self, other: Self::Output) -> Self::Output {
        v2(self.x / other.x, self.y / other.y)
    }
}

impl Div<f32> for V2 {
    type Output = V2;
	
    #[inline]
		fn div(self, other: f32) -> Self::Output {
        v2(self.x / other, self.y / other)
    }
}

impl Div<V2> for f32 {
    type Output = V2;
	
    #[inline]
		fn div(self, other: V2) -> V2 {
        v2(self / other.x, self / other.y)
    }
}

impl DivAssign for V2 {
    #[inline]
		fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl Mul<V2> for V2 {
    type Output = V2;
	
    #[inline]
		fn mul(self, other: Self::Output) -> Self::Output {
        v2(self.x * other.x, self.y * other.y)
    }
}

impl Mul<f32> for V2 {
    type Output = V2;
	
    #[inline]
		fn mul(self, other: f32) -> Self::Output {
        v2(self.x * other, self.y * other)
    }
}

impl Mul<V2> for f32 {
    type Output = V2;
	
    #[inline]
		fn mul(self, other: V2) -> V2 {
        v2(self * other.x, self * other.y)
    }
}

impl MulAssign for V2 {
    #[inline]
		fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Neg for V2 {
    type Output = V2;
	
    #[inline]
		fn neg(self) -> Self::Output {
        v2(-self.x, -self.y)
    }
}
