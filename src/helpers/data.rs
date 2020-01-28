use crate::helpers::math::*;

// colors
pub const WHITE: V4 = V4 {
    x: 1.,
    y: 1.,
    z: 1.,
    w: 1.,
};
pub const BLACK: V4 = V4 {
    x: 0.,
    y: 0.,
    z: 0.,
    w: 1.,
};
pub const RED: V4 = V4 {
    x: 1.,
    y: 0.,
    z: 0.,
    w: 1.,
};
pub const GREEN: V4 = V4 {
    x: 0.,
    y: 1.,
    z: 0.,
    w: 1.,
};
pub const BLUE: V4 = V4 {
    x: 0.,
    y: 0.,
    z: 1.,
    w: 1.,
};

// change the color alpha part only
pub fn color_alpha(color: V4, alpha: f32) -> V4 {
    v4(color.x, color.y, color.z, color.w * alpha)
}

// pixel size of each sprite in the atlas
pub const ATLAS_TILE: f32 = 32.;
pub const ATLAS_SPACING: V2 = V2 {
    x: ATLAS_TILE,
    y: ATLAS_TILE,
};
pub const ATLAS_FILL: f32 = 0.;
pub const ATLAS_NUMBERS: f32 = 8.;
pub const ATLAS_ALPHABET: f32 = 9.;
pub const ATLAS_CURSOR: f32 = 1.;

// grid specific
pub const GRID_WIDTH: usize = 6;
pub const GRID_HEIGHT: usize = 12;
pub const GRID_TOTAL: usize = GRID_WIDTH * GRID_HEIGHT;

// how many sprites can be drawn each frame
pub const SPRITE_AMOUNT: usize = 100;
pub const TEXT_AMOUNT: usize = 20;

// generic sprite
#[derive(Copy, Clone)]
pub struct Sprite {
    pub color: V4,
    pub position: V2,
    pub dimensions: V2,
    pub offset: V2,
    pub scale: V2,
    pub rotation: f32,
    pub hframe: f32,
    pub vframe: f32,
    pub visible: f32,
    pub depth: f32,
    pub centered: f32,
    pub temp1: f32,
    pub temp2: f32,
}

// default parameters that a sprite requires! careful about scale, depth etc
impl Default for Sprite {
    fn default() -> Self {
        Self {
            color: WHITE,
            position: V2::zero(),
            dimensions: v2(ATLAS_TILE, ATLAS_TILE),
            offset: V2::zero(),
            scale: V2::one(),
            rotation: 0.,
            hframe: 0.,
            vframe: ATLAS_FILL,
            visible: 1.,
            depth: 0.9,
            centered: 0.,
            temp1: 0.,
            temp2: 0.,
        }
    }
}
