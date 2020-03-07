use crate::helpers::{GRID_HEIGHT, GRID_WIDTH};
use ultraviolet::{
	vec::{Vec2, Vec3, Vec4},
	int::Vec2i,
	mat::Mat4,
	projection::lh_yup::orthographic_gl,
};
	
/// easy access to pi when using math helpers
pub const PI: f32 = std::f32::consts::PI;
/// matrix4 f32 type
pub type M4 = Mat4;
/// vector2 type with 2 f32
pub type V2 = Vec2;
/// vector2 type with 2 i32
pub type I2 = Vec2i;
/// vector3 type with 3 f32
pub type V3 = Vec3;
/// vector4 type with 4 f32
pub type V4 = Vec4;

#[inline]
pub fn v2(x: f32, y: f32) -> V2 {
	V2::new(x, y)
}

#[inline]
pub fn v3(x: f32, y: f32, z: f32) -> V3 {
	V3::new(x, y, z)
}

#[inline]
pub fn v4(x: f32, y: f32, z: f32, w: f32) -> V4 {
	V4::new(x, y, z, w)
}

#[inline]
pub fn i2(x: i32, y: i32) -> I2 {
	I2::new(x, y)
}

/// wrapper for calling into vek orthographic_lh_zo more easily
#[inline]
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> M4 {
    orthographic_gl(left, right, bottom, top, near, far)
}

#[inline]
pub fn lerp(n1: f32, n2: f32, amount: f32) -> f32 {
    (1. - amount) * n1 + amount * n2
}

/// NOTE(Skytrias): no out of bounds safety garantee
pub trait ToIndex {
    fn to_index(self) -> usize;
}

impl ToIndex for I2 {
    fn to_index(self) -> usize {
        self.y as usize * GRID_WIDTH + self.x as usize
    }
}

pub trait ToV2 {
    fn to_v2(&self) -> V2;
}

impl ToV2 for usize {
    fn to_v2(&self) -> V2 {
        V2::new(
            (*self % GRID_WIDTH) as f32,
            (*self as f32 / GRID_WIDTH as f32).floor(),
        )
    }
}

impl ToV2 for i32 {
    fn to_v2(&self) -> V2 {
        V2::new(
            (*self % GRID_WIDTH as i32) as f32,
            (*self as f32 / GRID_WIDTH as f32).floor(),
        )
    }
}

pub trait ToI2 {
    fn to_i2(&self) -> I2;
}

impl ToI2 for usize {
    fn to_i2(&self) -> I2 {
        I2::new(
            (*self % GRID_WIDTH) as i32,
            (*self as f32 / GRID_WIDTH as f32).floor() as i32,
        )
    }
}

impl ToI2 for i32 {
    fn to_i2(&self) -> I2 {
        I2::new(
            (*self % GRID_WIDTH as i32) as i32,
            (*self as f32 / GRID_WIDTH as f32).floor() as i32,
        )
    }
}
