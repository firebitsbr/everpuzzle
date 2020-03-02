use crate::helpers::{GRID_HEIGHT, GRID_WIDTH};
use vek::geom::repr_c::Rect;
use vek::geom::FrustumPlanes;
use vek::mat::repr_c::Mat4;
use vek::vec::repr_c::{Vec2, Vec3, Vec4};

/// easy access to pi when using math helpers
pub const PI: f32 = std::f32::consts::PI;
/// matrix4 f32 type
pub type M4 = Mat4<f32>;
/// vector2 f32 type
pub type V2 = Vec2<f32>;
/// vector2 f32 type
pub type I2 = Vec2<i32>;
/// vector3 f32 type
pub type V3 = Vec3<f32>;
/// vector4 f32 type
pub type V4 = Vec4<f32>;
/// 2 f32 vectors in rectangle
pub type R4 = Rect<f32, f32>;

/// wrapper for calling into vek orthographic_lh_zo more easily
#[inline]
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> M4 {
    M4::orthographic_lh_zo(FrustumPlanes {
        left,
        right,
        bottom,
        top,
        near,
        far,
    })
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
