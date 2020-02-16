use crate::helpers::math::*;

/// block and garbage hang time
pub const HANG_TIME: u32 = 20;
/// block and garbage clear time
pub const CLEAR_TIME: u32 = 20;

/// pixel size of each sprite in the texture atlas
pub const ATLAS_TILE: f32 = 32.;
/// pixel dimensions of each sprite in the texture atlas
pub const ATLAS_SPACING: V2 = V2::new(
									  ATLAS_TILE,
									  ATLAS_TILE,
									  );

/// vframe position of a white texture, can be used as a rectangle replacement
pub const ATLAS_FILL: u32 = 0;
/// vframe position of the cursor texture
pub const ATLAS_CURSOR: u32 = 1;
/// vframe position of the garbage texture
pub const ATLAS_GARBAGE: u32 = 11;

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
    pub dimensions: V2,
    pub offset: V2,
    pub scale: V2,
    pub rotation: f32,
    pub hframe: u32,
    pub vframe: u32,
    pub centered: bool,
	pub tiles: V2,
}

/// default parameters that a sprite requires! 
/// careful about scale, depth, etc
impl Default for Sprite {
    fn default() -> Self {
        Self {
            position: V2::zero(),
            dimensions: V2::broadcast(ATLAS_TILE),
            offset: V2::zero(),
            scale: V2::one(),
            rotation: 0.,
            hframe: 0,
            vframe: ATLAS_FILL,
            centered: false,
			tiles: V2::one(),
        }
    }
}