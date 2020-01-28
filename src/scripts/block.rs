use crate::helpers::*;
use BlockStates::*;

const HANG_TIME: u32 = 20;
const SWAP_TIME: u32 = 5;
const CLEAR_TIME: u32 = 100;

pub struct Block {
    pub hframe: f32,
    pub vframe: f32,
    pub state: BlockStates,
    pub offset: V2,
    pub scale: f32,
}

#[derive(Copy, Clone)]
pub enum BlockStates {
    Idle,
    Hang(HangState),
    Swap(SwapState),
    Bottom,
    Clear(ClearState),
}

#[derive(Copy, Clone)]
pub struct SwapState {
	pub counter: u32,
	pub direction: SwapDirection,
	pub finished: bool,
}

impl SwapState {
	pub fn new(direction: SwapDirection) -> Self {
		Self {
			counter: 0,
			direction,
			finished: false,
		}
	}
}

// TODO(Skytrias): hang and clear as the same, use a default one?

#[derive(Copy, Clone)]
pub struct HangState {
	pub counter: u32,
	pub finished: bool,
}

impl Default for HangState {
	fn default() -> Self {
		Self {
			counter: 0,
			finished: false,
		}
	}
}

#[derive(Copy, Clone)]
pub struct ClearState {
	pub counter: u32,
	pub finished: bool,
}

impl Default for ClearState {
	fn default() -> Self {
		Self {
			counter: 0,
			finished: false,
		}
	}
}

#[derive(Copy, Clone)]
pub enum SwapDirection {
    Left,
    Right,
}

// TODO(Skytrias): make inline?
impl BlockStates {
    // returns true if the block is idle
    pub fn is_idle(&self) -> bool {
        match self {
            Idle => true,
            _ => false,
        }
    }
	
    // returns true if the block is hang
    pub fn is_hang(&self) -> bool {
        match self {
            Hang(..) => true,
            _ => false,
        }
    }
	
    // returns true if the block hang state has finished counting up
    pub fn hang_finished(&self) -> bool {
        match self {
            Hang(state) => state.finished,
            _ => false,
        }
    }
	
    // returns true if the block is swap
    pub fn is_swap(&self) -> bool {
        match self {
            Swap(..) => true,
            _ => false,
        }
    }
	
    // returns true if the block is real meaning its idle or at the bottom
    pub fn is_real(&self) -> bool {
        self.is_idle() || self.is_bottom()
    }
	
    // returns true if the block swap state has finished counting up
    pub fn swap_finished(&self) -> bool {
        match self {
            Swap(state) => state.finished,
            _ => false,
        }
    }
	
    // returns true if the block is at the bottom of the grid
    pub fn is_bottom(&self) -> bool {
        match self {
            Bottom => true,
            _ => false,
        }
    }
	
    // returns true if the block is clear
    pub fn is_clear(&self) -> bool {
        match self {
            Clear(..) => true,
            _ => false,
        }
    }
	
    // returns true if the block swap state has finished counting up
    pub fn clear_finished(&self) -> bool {
        match self {
            Clear(state) => state.finished,
            _ => false,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            hframe: 0.,
            vframe: 2.,
            state: Idle,
            offset: V2::zero(),
            scale: 1.,
        }
    }
}

impl Block {
    pub fn new(vframe: f32) -> Self {
        Self {
            vframe,
            ..Default::default()
        }
    }
	
    pub fn reset(&mut self) {
        self.state = Idle;
        self.offset.x = 0.;
    }
	
    pub fn update(&mut self) {
        match &mut self.state {
            Hang(state) => {
                if state.counter < HANG_TIME {
                    state.counter += 1;
                } else {
                    state.finished = true;
                }
            }
			
            Swap(state) => {
                if state.counter < SWAP_TIME {
                    self.offset.x = match state.direction {
                        SwapDirection::Left => -(state.counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE,
                        SwapDirection::Right => (state.counter as f32) / (SWAP_TIME as f32) * ATLAS_TILE,
                    };
					
                    state.counter += 1;
                } else {
                    state.finished = true;
                }
            }
			
            Clear(state) => {
                if state.counter < CLEAR_TIME {
                    self.scale = (state.counter as f32) / (CLEAR_TIME as f32);
					
                    state.counter += 1;
                } else {
                    state.finished = true;
                }
            }
			
            _ => {}
        }
    }
}
