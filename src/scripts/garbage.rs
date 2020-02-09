use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use GarbageStates::*;

const CLEAR_TIME: u32 = 100;

// TODO(Skytrias): check for bottom?

// set of functions both garbage and the block share
pub trait Gridable {
	fn is_idle(&self) -> bool;
	fn is_hang(&self) -> bool;
	fn is_clear(&self) -> bool;
	
	fn clear_started(&self) -> bool;
	fn hang_started(&self) -> bool;
	
	fn hang_finished(&self) -> bool;
	fn clear_finished(&self) -> bool;
	
    fn to_hang(&mut self, counter: u32);
    fn to_clear(&mut self);
}

#[derive(Copy, Clone, Debug)]
pub enum GarbageStates {
    Idle,
    Hang { counter: u32, finished: bool },
    Clear { counter: u32, finished: bool },
}

impl Gridable for GarbageStates {
	// returns true if the garbage is idle
    fn is_idle(&self) -> bool {
        match self {
            Idle => true,
            _ => false,
        }
    }
	
    // returns true if the garbage is hang
    fn is_hang(&self) -> bool {
        match self {
            Hang { .. } => true,
            _ => false,
        }
    }
	
    // returns true if the garbage hang state has finished counting up
    fn hang_started(&self) -> bool {
        match self {
            Hang { counter, .. } => *counter == 1,
            _ => false,
        }
    }
	
    // returns true if the garbage hang state has finished counting up
    fn hang_finished(&self) -> bool {
        match self {
            Hang { finished, .. } => *finished,
            _ => false,
        }
    }
	
    // returns true if the garbage is clear
    fn is_clear(&self) -> bool {
        match self {
            Clear { .. } => true,
            _ => false,
        }
    }
	
    // returns true if the garbage clear state has finished counting up
    fn clear_started(&self) -> bool {
        match self {
			Clear { counter, .. } => *counter == 1,
            _ => false,
        }
    }
	
    // returns true if the garbage clear state has finished counting up
    fn clear_finished(&self) -> bool {
        match self {
            Clear { finished, .. } => *finished,
            _ => false,
        }
    }
	
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

impl GarbageStates {
	pub fn as_gridable(&self) -> Box<&dyn Gridable> {
		Box::new(self)
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
            Hang { counter, finished } => {
                if *counter < HANG_TIME {
                    *counter += 1;
                } else {
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
			
            _ => {}
        }
    }
}
