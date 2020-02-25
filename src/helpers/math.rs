use crate::helpers::{GRID_WIDTH, GRID_HEIGHT};
use vek::geom::repr_c::Rect;
use vek::geom::FrustumPlanes;
use vek::mat::repr_c::Mat4;
use vek::vec::repr_c::{Vec2, Vec3};

/// matrix4 f32 type
pub type M4 = Mat4<f32>;
/// vector2 f32 type
pub type V2 = Vec2<f32>;
/// vector2 f32 type
pub type I2 = Vec2<i32>;
/// vector3 f32 type
pub type V3 = Vec3<f32>;
/// vector3 f32 type
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

pub trait ToIndex {
	fn to_index(self) -> Option<usize>;
}

impl ToIndex for I2 {
	fn to_index(self) -> Option<usize> {
		if self.x >= 0 && self.x < GRID_WIDTH as i32 && self.y >= 0 && self.y < GRID_HEIGHT as i32 {
			Some(self.y as usize * GRID_WIDTH + self.x as usize)
		} else {
			None
		}
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
