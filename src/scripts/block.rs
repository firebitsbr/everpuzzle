use crate::engine::App;
use crate::helpers::*;
use crate::scripts::Gridable;
use BlockStates::*;

const SWAP_TIME: u32 = 5;
const CLEAR_TIME: u32 = 50;

#[derive(Copy, Clone, Debug)]
pub enum SwapDirection {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum BlockStates {
    Idle,
    Hang {
        counter: u32,
        finished: bool,
    },
    Swap {
        counter: u32,
        direction: SwapDirection,
        finished: bool,
    },
    Bottom,
    Clear {
        counter: u32,
        finished: bool,
    },
}

pub struct Block {
    pub hframe: u32,
    pub vframe: u32,
    pub state: BlockStates,
    pub offset: V2,
    pub scale: f32,
}

impl Gridable for BlockStates {
    // returns true if the block is idle
    fn is_idle(self) -> bool {
        match self {
            Idle => true,
            _ => false,
        }
    }

    // returns true if the block is hang
    fn is_hang(self) -> bool {
        match self {
            Hang { .. } => true,
            _ => false,
        }
    }

    // returns true if the block hang state has finished counting up
    fn hang_started(self) -> bool {
        match self {
            Hang { counter, .. } => counter == 1,
            _ => false,
        }
    }

    // returns true if the block hang state has finished counting up
    fn hang_finished(self) -> bool {
        match self {
            Hang { finished, .. } => finished,
            _ => false,
        }
    }

    // returns true if the block is clear
    fn is_clear(self) -> bool {
        match self {
            Clear { .. } => true,
            _ => false,
        }
    }

    // returns true if the block swap state has finished counting up
    fn clear_finished(self) -> bool {
        match self {
            Clear { finished, .. } => finished,
            _ => false,
        }
    }

    // returns true if the block swap state has finished counting up
    fn clear_started(self) -> bool {
        match self {
            Clear { counter, .. } => counter == 1,
            _ => false,
        }
    }

    // helpers for state data

    fn to_hang(&mut self, counter: u32) {
        *self = Hang {
            counter,
            finished: false,
        };
    }

    fn to_clear(&mut self) {
        *self = Clear {
            counter: 0,
            finished: false,
        };
    }
}

// TODO(Skytrias): make inline?
impl BlockStates {
    // returns true if the block is swap
    pub fn is_swap(self) -> bool {
        match self {
            Swap { .. } => true,
            _ => false,
        }
    }

    // returns true if the block swap state has finished counting up
    pub fn swap_finished(self) -> bool {
        match self {
            Swap { finished, .. } => finished,
            _ => false,
        }
    }

    // returns true if the block is real meaning its idle or at the bottom
    pub fn is_real(self) -> bool {
        self.is_idle() || self.is_bottom()
    }

    // returns true if the block is at the bottom of the grid
    pub fn is_bottom(self) -> bool {
        match self {
            Bottom => true,
            _ => false,
        }
    }

    pub fn to_swap(&mut self, direction: SwapDirection) {
        *self = Swap {
            counter: 0,
            direction,
            finished: false,
        };
    }

    pub fn as_gridable(&self) -> Box<&dyn Gridable> {
        Box::new(self)
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            hframe: 0,
            vframe: 2,
            state: Idle,
            offset: V2::zero(),
            scale: 1.,
        }
    }
}

impl Block {
    pub fn random(app: &mut App) -> Self {
        Self {
            vframe: (app.rand_int(5) + 2) as u32,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.state = Idle;
        self.offset.x = 0.;
    }

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

            Clear { counter, finished } => {
                if *counter < CLEAR_TIME {
                    self.scale = 1. - (*counter as f32) / (CLEAR_TIME as f32);
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
