use vek::geom::FrustumPlanes;
use vek::vec::repr_c::{Vec2, Vec3};
use vek::mat::repr_c::{Mat4};

/// matrix4 f32 type
pub type M4 = Mat4<f32>;
/// vector2 f32 type
pub type V2 = Vec2<f32>;
/// vector3 f32 type
pub type V3 = Vec3<f32>;

/// wrapper for calling into vek orthographic_lh_zo more easily
#[inline]
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> M4 {
	M4::orthographic_lh_zo(FrustumPlanes {
										   left, right, bottom, top, near, far
									   })
}

/*
#[inline]
pub fn lerp(n1: f32, n2: f32, amount: f32) -> f32 {
	(1. - amount) * n1 + amount * n2
}*/