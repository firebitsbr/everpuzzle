use crate::engine::App;
use crate::helpers::*;
use BlockState::*;

/// the direction a block can be swapped into
#[derive(Copy, Clone, Debug)]
pub enum SwapDirection {
    Left,
    Right,
}

block_state!(
			 BlockState,
			 {
				 Idle, idle,
				 Fall, fall,
				 Bottom, bottom,
				 Spawned, spawned,
				 },
			 {
				 Hang, hang {
					 
				 },
				 
				 Swap, swap {
					 direction: SwapDirection,
				 },
				 
				 Clear, clear {
					 start_time: u32,
					 end_time: u32,
				 },
				 
				 Land, land {
					 
				 },
			 }
			 );

impl BlockState {
    /// returns true if the block is real meaning its idle or at the bottom
    pub fn is_real(self) -> bool {
        self.is_idle() || self.is_bottom()
    }
}

/// block data used for unique block rendering and unique state
pub struct Block {
    pub hframe: u32,
    pub vframe: u32,
    pub offset: V2,
    pub scale: V2,
    pub state: BlockState,
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
                if *counter < HANG_TIME - 1 {
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
                if *counter < SWAP_TIME - 1 {
                    self.offset.x = match *direction {
                        SwapDirection::Left => -(*counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE,
                        SwapDirection::Right => (*counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE,
                    };

                    *counter += 1;
                } else {
                    *finished = true;
                }
            }

            Clear {
                counter,
                finished,
                start_time,
                end_time,
            } => {
                if *counter < *end_time - 1 {
                    if *counter > *start_time {
                        if (*counter - *start_time) < CLEAR_TIME - 1 {
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

            Fall => {
                self.hframe = 3;
            }

            Bottom => {
                self.hframe = 2;
            }

            Land { counter, finished } => {
                if *counter < LAND_TIME - 1 {
                    self.hframe = 3 + ((*counter as f32 / LAND_TIME as f32) * 3.).floor() as u32;
                    *counter += 1;
                } else {
                    self.hframe = 0;
                    *finished = true;
                }
            }

            _ => {}
        }
    }
}
