use crate::helpers::math::*;

// block and garbage hang time
pub const HANG_TIME: u32 = 20;

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
pub const ATLAS_NUMBERS: f32 = 9.;
pub const ATLAS_ALPHABET: f32 = 10.;
pub const ATLAS_CURSOR: f32 = 1.;
pub const ATLAS_GARBAGE: f32 = 11.;

// grid specific
pub const GRID_WIDTH: usize = 6;
pub const GRID_HEIGHT: usize = 12;
pub const GRID_TOTAL: usize = GRID_WIDTH * GRID_HEIGHT;

pub const SPRITE_LEN: usize = 50;
pub const TEXT_LEN: usize = 50;

// TODO(Skytrias): consider removing visible, since we just dont push it to push_sprite
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

impl Sprite {
    pub fn empty() -> Self {
        Self {
            visible: 0.,
            ..Default::default()
        }
    }
}

// differentiate between a value or a string being sent to push_text
pub enum TextVariant {
    Value(f32),
    Characters(&'static str),
}

impl From<f32> for TextVariant {
    fn from(value: f32) -> Self {
        TextVariant::Value(value)
    }
}

impl From<u32> for TextVariant {
    fn from(value: u32) -> Self {
        TextVariant::Value(value as f32)
    }
}

impl From<&'static str> for TextVariant {
    fn from(chars: &'static str) -> Self {
        TextVariant::Characters(chars)
    }
}

// data sent via push_text, optional values so you dont have to fill all
pub struct Text {
    pub variant: TextVariant,
    pub position: V2,
    pub dimensions: V2,
    pub centered: bool,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            variant: TextVariant::Value(0.),
            position: V2::zero(),
            dimensions: V2::both(20.),
            centered: false,
        }
    }
}
