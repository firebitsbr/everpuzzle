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
pub const PUSH_TIME: u32 = 50;

/// pixel size of each sprite in the texture atlas
pub const ATLAS_TILE: f32 = 32.;
/// pixel dimensions of each sprite in the texture atlas
pub const ATLAS_SPACING: V2 = V2::new(ATLAS_TILE, ATLAS_TILE);

/// vframe position of a white texture, can be used as a rectangle replacement
pub const ATLAS_FILL: u32 = 0;
/// vframe position of the cursor texture
pub const ATLAS_CURSOR: u32 = 1;
/// vframe position of the garbage texture
pub const ATLAS_GARBAGE_1D: u32 = 9;
/// vframe position of the garbage texture
pub const ATLAS_GARBAGE_2D: u32 = 10;

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