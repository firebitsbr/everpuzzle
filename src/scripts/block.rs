use crate::helpers::*;
use BlockState::*;

#[derive(Debug)]
pub enum BlockState {
    Idle,
    Hang {
        counter: u32,
    },
    Fall,
    Swap {
        counter: u32,
        direction: i32,
    },
    Land {
        counter: u32,
    },
    Clear {
        counter: u32,
        start_time: u32,
        end_time: u32,
    },
    Spawned,
}

/// block data used for unique block rendering and unique state
pub struct Block {
    /// hframe horizontal position in the texture atlas
    pub hframe: u32,

    /// vframe vertical position in the texture atlas
    pub vframe: u32,

    /// visual sprite offset
    pub offset: V2,

    /// visual sprite scale
    pub scale: V2,

    /// wether the block could result in a chain or not
    pub saved_chain: Option<usize>,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            hframe: 0,
            vframe: 2,
            offset: V2::zero(),
            scale: V2::one(),
            saved_chain: None,
        }
    }
}

impl Block {
    /// simply creates a vframe designed for the block
    pub fn random_vframe(gen: &mut oorandom::Rand32) -> u32 {
        gen.rand_range(3..8)
    }

    /// updates the block variables based on each state, mostly animation based
    pub fn update(&mut self, state: &mut BlockState) {
        match state {
            Hang { counter } => *counter += 1,

            Swap { counter, direction } => {
                self.offset.x =
                    *direction as f32 * (*counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE;
                *counter += 1;
            }

            Clear {
                counter,
                start_time,
                ..
            } => {
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
            }

            Land { counter } => {
                self.hframe = 3 + ((*counter as f32 / LAND_TIME as f32) * 3.).floor() as u32;
                *counter += 1;
            }

            Idle => {
                self.saved_chain = None;
                self.hframe = 0;
            }

            _ => {}
        }
    }
}
