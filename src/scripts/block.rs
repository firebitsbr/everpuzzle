use crate::engine::App;
use crate::helpers::{V2, HANG_TIME, ATLAS_TILE, CLEAR_TIME};
use BlockStates::*;

// amount of frames it takes to swap a block
const SWAP_TIME: u32 = 5;

/// the direction a block can be swapped into
#[derive(Copy, Clone, Debug)]
pub enum SwapDirection {
    Left,
    Right,
}

/// all states a block can have
#[derive(Copy, Clone, Debug)]
pub enum BlockStates {
	/// tag that does nothing, other state detections depend on this being true
    Idle,
	
	/// hangs the block in the air until time is finished counting up
    Hang {
        counter: u32,
        finished: bool,
    },
	
	/// tag to note that the block is currently falling
    Fall,
	
	/// swap animation, with a direction of where the swap is directed to 
	Swap {
        counter: u32,
        direction: SwapDirection,
        finished: bool,
    },
    
	/// tag to note that the block is at the "bottom" of the grid
	Bottom,
	
	/// clear animation, with a specific starting and end time, since clears happen delayed
    Clear {
        counter: u32,
        start_time: u32,
        end_time: u32,
		finished: bool,
    },
	
	/// tag to halt any other state, have to manually set it to idle 
	Spawned, 
}

impl BlockStates {
    /// returns true if the block is idle
    pub fn is_idle(self) -> bool {
        match self {
            Idle => true,
            _ => false,
        }
    }
	
    /// returns true if the block is clear
    pub fn is_clear(self) -> bool {
        match self {
            Clear { .. } => true,
            _ => false,
        }
    }
	
	/// returns true if the block is swap
    pub fn is_swap(self) -> bool {
        match self {
            Swap { .. } => true,
            _ => false,
        }
    }
	
    /// returns true if the block is real meaning its idle or at the bottom
    pub fn is_real(self) -> bool {
        self.is_idle() || self.is_bottom()
    }
	
    /// returns true if the block is at the bottom of the grid
    pub fn is_bottom(self) -> bool {
        match self {
            Bottom => true,
            _ => false,
        }
    }
	
	/// returns true if the lbock is currently falling
	pub fn is_fall(self) -> bool {
		match self {
			Fall => true,
			_ => false
		}
	}
	
    /// returns true if the block swap state has finished counting up
    pub fn clear_finished(self) -> bool {
        match self {
            Clear { finished, .. } => finished,
            _ => false,
        }
    }
	
    /// returns true if the block swap state has finished counting up
    pub fn clear_started(self) -> bool {
        match self {
            Clear { counter, .. } => counter == 1,
            _ => false,
        }
    }
	
    /// change the state to hang with defaults
    pub fn to_hang(&mut self, counter: u32) {
        *self = Hang {
            counter,
            finished: false,
        };
    }
	
    /// change the state to clear with defaults
    pub fn to_clear(&mut self, start_time: u32, end_time: u32) {
        *self = Clear {
			start_time,
			end_time,
            finished: false,
            counter: 0,
		};
    }
	
	/// change the state to swap with the given direction
    pub fn to_swap(&mut self, direction: SwapDirection) {
        *self = Swap {
            counter: 0,
            direction,
            finished: false,
        };
    }
	
    /// change the state to fall
    pub fn to_fall(&mut self) {
        *self = Fall;
    }
	
	/// change the state to idle
	pub fn to_idle(&mut self) {
        *self = Idle;
    }
}

/// block data used for unique block rendering and unique state 
pub struct Block {
    pub hframe: u32,
    pub vframe: u32,
    pub offset: V2,
    pub scale: V2,
    pub state: BlockStates,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            hframe: 0,
            vframe: 2,
            state: Idle,
            offset: V2::zero(),
            scale: V2::one(),
        }
    }
}

impl Block {
    /// creates a block with a "randomized" vframe
	pub fn random(app: &mut App) -> Self {
        Self {
            vframe: (app.rand_int(5) + 3) as u32,
            ..Default::default()
        }
    }
	
	/// creates a "randomized" block and sets it to spawned, having to turn it to idle manually at some point
	pub fn random_clear(app: &mut App) -> Self {
        Self {
			state: Spawned,
            vframe: (app.rand_int(5) + 3) as u32,
            ..Default::default()
        }
    }
	
	/// sets the state to idle and offset.x back to 0
    pub fn reset(&mut self) {
        self.state = Idle;
        self.offset.x = 0.;
    }
	
	/// updates the block variables based on each state, mostly animation based
    pub fn update(&mut self) {
        match &mut self.state {
            Hang { counter, finished } => {
                if *counter < HANG_TIME {
                    *counter += 1;
                } else {
                    *finished = true;
                }
            }
			
            Swap {
                counter,
                direction,
                finished,
            } => {
                if *counter < SWAP_TIME {
                    self.offset.x = match *direction {
                        SwapDirection::Left => -(*counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE,
                        SwapDirection::Right => (*counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE,
                    };
					
                    *counter += 1;
                } else {
                    *finished = true;
                }
            }
			
            Clear { counter, finished, start_time, end_time } => {
                if *counter < *end_time {
					if *counter > *start_time {
						if (*counter - *start_time) < CLEAR_TIME {
							 let amt = 1. - ((*counter - *start_time) as f32) / (CLEAR_TIME as f32);
							self.scale = V2::broadcast(amt);
						} else {
							self.scale = V2::zero();
						}
						
					}
					
						self.hframe = 1;
                    *counter += 1;
                } else {
                    *finished = true;
                }
            }
			
            _ => {}
        }
    }
}
