use crate::helpers::math::*;

/// block and garbage hang time
pub const HANG_TIME: u32 = 40;
/// block and garbage clear time
pub const CLEAR_TIME: u32 = 40;
/// amount of frames it takes to swap a block
pub const SWAP_TIME: u32 = 5;
/// amount of frames it takes to land a block, multples of 3!
pub const LAND_TIME: u32 = 24;

/// frame time until the push_counter gets reset
pub const PUSH_TIME: u32 = 100;

/// pixel size of each sprite in the texture atlas
pub const ATLAS_TILE: f32 = 32.;
/// pixel dimensions of each sprite in the texture atlas
pub const ATLAS_SPACING: V2 = V2 {
    x: ATLAS_TILE,
    y: ATLAS_TILE,
};

/// vframe position of a white texture, can be used as a rectangle replacement
pub const ATLAS_FILL: u32 = 0;
/// vframe position of the cursor texture
pub const ATLAS_CURSOR: u32 = 1;
/// vframe position of the garbage texture
pub const ATLAS_GARBAGE_1D: u32 = 9;
/// vframe position of the garbage texture
pub const ATLAS_GARBAGE_2D: u32 = 10;
pub const ATLAS_NUMBERS: u32 = 11;
pub const ATLAS_ALPHABET: u32 = 12;

/// block width size of the grid
pub const GRID_WIDTH: usize = 6;
/// block height size of the grid
pub const GRID_HEIGHT: usize = 12;
/// block width * height size of the grid
pub const GRID_TOTAL: usize = GRID_WIDTH * GRID_HEIGHT;

/// Sprite data used to render quads
#[derive(Copy, Clone)]
pub struct Sprite {
    pub position: V2,
    pub offset: V2,
    pub scale: V2,
    pub rotation: f32,
    pub hframe: u32,
    pub vframe: u32,
    pub centered: bool,
    pub tiles: V2,
    pub depth: f32,
}

/// default parameters that a sprite requires!
/// careful about scale, depth, etc
impl Default for Sprite {
    fn default() -> Self {
        Self {
            position: V2::zero(),
            offset: V2::zero(),
            scale: V2::one(),
            rotation: 0.,
            hframe: 0,
            vframe: ATLAS_FILL,
            centered: false,
            tiles: V2::one(),
            depth: 0.9,
        }
    }
}

/// data that will be sent to the gpu
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: V2,
    pub end: V2,
    pub thickness: f32,
    pub hframe: u32,
    pub vframe: u32,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            start: V2::zero(),
            end: V2::zero(),
            thickness: 10.,
            hframe: 0,
            vframe: ATLAS_FILL,
        }
    }
}

/// data that will be sent to the gpu
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Quad {
    /// model matrix that stores position, offset, scale, dimensions, etc
    pub model: M4,

    /// how many tiles the quad texture should use
    pub tiles: V2,

    /// hframe of the tile in the texture atlas
    pub hframe: f32,

    /// vframe of the tile in the texture atlas
    pub vframe: f32,

    /// vframe of the tile in the texture atlas
    pub depth: f32,
}

impl Quad {
    /// max number of quads that can be rendered
    pub const MAX: usize = 1000;

    /// byte size of the quad struct
    pub const SIZE: usize = std::mem::size_of::<Quad>();
}

/// converts a sprite into a valid quad
impl From<Sprite> for Quad {
    fn from(sprite: Sprite) -> Self {
        let dimensions = sprite.tiles * ATLAS_SPACING;

        let mut model = M4::from_translation(v3(
            sprite.position.x + sprite.offset.x,
            sprite.position.y + sprite.offset.y,
            0.,
        ));

        model = model * M4::from_nonuniform_scale(v4(sprite.scale.x, sprite.scale.y, 1., 1.));
        model = model * M4::from_nonuniform_scale(v4(dimensions.x, dimensions.y, 1., 1.));

        if sprite.centered {
            model = model * M4::from_translation(v3(-0.5, -0.5, 0.));
        }

        Quad {
            model,
            tiles: sprite.tiles,
            hframe: sprite.hframe as f32,
            vframe: sprite.vframe as f32,
            depth: sprite.depth,
        }
    }
}

pub struct Text<'a> {
    pub content: &'a str,
    pub position: V2,
    pub scale: V2,
    pub step: f32,
}

impl<'a> Default for Text<'a> {
    fn default() -> Self {
        Self {
            position: V2::zero(),
            scale: V2::one(),
            content: "",
            step: 32.,
        }
    }
}
