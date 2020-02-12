use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use GarbageStates::*;

const CLEAR_TIME: u32 = 100;

// TODO(Skytrias): check for bottom?

#[derive(Copy, Clone, Debug)]
pub enum GarbageStates {
    Idle,
    Hang { counter: u32, finished: bool },
    Fall,
    Clear { counter: u32, finished: bool },
}

impl GarbageStates {
    // returns true if the garbage is idle
    pub fn is_idle(self) -> bool {
        match self {
            Idle => true,
            _ => false,
        }
    }
	
    // returns true if the garbage is hang
    pub fn is_hang(self) -> bool {
        match self {
            Hang { .. } => true,
            _ => false,
        }
    }
	
    // returns true if the garbage hang state has finished counting up
    pub fn hang_started(self) -> bool {
        match self {
            Hang { counter, .. } => counter == 1,
            _ => false,
        }
    }
	
    // returns true if the garbage hang state has finished counting up
    pub fn hang_finished(self) -> bool {
        match self {
            Hang { finished, .. } => finished,
            _ => false,
        }
    }
	
    // returns true if the garbage is hang
    pub fn is_fall(self) -> bool {
        match self {
			Fall => true,
            _ => false,
        }
    }
	
    // returns true if the garbage is clear
    pub fn is_clear(self) -> bool {
        match self {
            Clear { .. } => true,
            _ => false,
        }
    }
	
    // returns true if the garbage clear state has finished counting up
    pub fn clear_started(self) -> bool {
        match self {
            Clear { counter, .. } => counter == 1,
            _ => false,
        }
    }
	
    // returns true if the garbage clear state has finished counting up
    pub fn clear_finished(self) -> bool {
        match self {
            Clear { finished, .. } => finished,
            _ => false,
        }
    }
	
    pub fn to_idle(&mut self) {
        *self = Idle;
    }
	
    pub fn to_hang(&mut self, counter: u32) {
        *self = Hang {
            counter,
            finished: false,
        };
    }
	
    pub fn to_clear(&mut self) {
        *self = Clear {
            counter: 0,
            finished: false,
        };
    }
	
    pub fn to_fall(&mut self) {
        *self = Fall;
    }
}

pub struct Garbage {
    pub children: Vec<usize>,
    pub count: usize, // len of children, should stay the same
    pub is_2d: bool,  // wether the garbage has more than 6 children
	
    pub hframe: u32,
    pub vframe: u32,
    pub state: GarbageStates,
    pub offset: V2,
}

impl Default for Garbage {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            count: 0,
            hframe: 0,
            vframe: ATLAS_GARBAGE as u32,
            state: Idle,
            offset: V2::zero(),
            is_2d: false,
        }
    }
}

impl Garbage {
    pub fn new(children: Vec<usize>) -> Self {
        let count = children.len();
		
        Self {
            children,
            count,
            is_2d: count > GRID_WIDTH,
            ..Default::default()
        }
    }
	
    // NOTE(Skytrias): shouldnt be called when its 1d
    // removes the lowest children and returns them if the garbage is still 2d
    pub fn drain_lowest(&mut self) -> Vec<usize> {
        let skip = (self.count / GRID_WIDTH - 1) * GRID_WIDTH;
		
        self.count = self.children.len() - GRID_WIDTH;
        self.is_2d = self.count > GRID_WIDTH;
		
        self.children.drain(skip..).collect()
    }
	
    // depends on dimensions, if 2d skip to the bottom of the children
    pub fn lowest(&self) -> Vec<usize> {
        if self.is_2d {
            let skip = (self.count / GRID_WIDTH - 1) * GRID_WIDTH;
			
            // TODO(Skytrias): double check
            self.children
                .iter()
                .skip(skip)
                .enumerate()
                .take_while(|(i, _)| *i < GRID_WIDTH)
                .map(|(_, num)| *num)
                .collect()
        } else {
            self.highest()
        }
    }
	
	// checks wether the lowest blocks below are all empty
	pub fn lowest_empty(&self, grid: &Grid) -> bool {
		let mut can_hang = true;
		
		for child_index in self.lowest().iter() {
				if !grid[child_index + GRID_WIDTH].is_empty() {
				can_hang = false;
				}
		}
		
		can_hang
	}
	
    // gets the highest children, will always work
    pub fn highest(&self) -> Vec<usize> {
        self.children
            .iter()
            .enumerate()
            .take_while(|(i, _)| *i < GRID_WIDTH)
            .map(|(_, num)| *num)
            .collect()
    }
	
    pub fn reset(&mut self) {
        self.state = Idle;
    }
	
    pub fn update(&mut self) {
        match &mut self.state {
            Idle => {
                self.hframe = 1;
            }
			
            Hang { counter, finished } => {
                if *counter < HANG_TIME {
                    *counter += 1;
					self.hframe = 2;
				} else {
					self.hframe = 1;
					*finished = true;
                }
            }
			
            Clear { counter, finished } => {
                if *counter < CLEAR_TIME {
                    self.hframe = 1;
                    *counter += 1;
                } else {
                    *finished = true;
                }
            }
			
			Fall =>
			{
				self.hframe = 3;
				}
		}
    }
}
