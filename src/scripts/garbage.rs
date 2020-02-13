use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use GarbageStates::*;

const CLEAR_TIME: u32 = 20;

pub struct Child {
	pub counter: u32,
	pub start_time: u32,
	pub randomize_at_end: bool,
	pub scale: f32, 
}

impl Default for Child {
	fn default() -> Self {
		Self {
			counter: 0,
			start_time: 0,
			randomize_at_end: false,
			scale: 1.,
		}
	}
}

pub struct GarbageSystem {
	pub list: Vec<Garbage>,
}

impl Default for GarbageSystem {
	fn default() -> Self {
		Self {
			list: Vec::new(),
		}
	}
}

impl GarbageSystem {
	pub fn update(&mut self, app: &mut App, grid: &mut Grid) {
		for garbage in self.list.iter_mut() {
			garbage.update(app, grid);
		}
	}
}

// TODO(Skytrias): check for bottom?

#[derive(Copy, Clone, Debug)]
pub enum GarbageStates {
    Idle,
    Hang { counter: u32, finished: bool },
    Fall,
    Clear { counter: u32, end_time: u32, finished: bool },
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
	
    pub fn to_clear(&mut self, children_count: u32) {
        *self = Clear {
            counter: 0,
            finished: false,
			end_time: children_count * CLEAR_TIME,
		};
    }
	
    pub fn to_fall(&mut self) {
        *self = Fall;
    }
}

pub struct Garbage {
    pub children: Vec<usize>,
    count: usize, // len of children, should stay the same
    pub is_2d: bool,  // wether the garbage has more than 6 children
	pub state: GarbageStates,
	removed_children: Vec<usize>, // list of children removed in clear, have to be idled
}

impl Default for Garbage {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            count: 0,
            state: Idle,
            is_2d: false,
			removed_children: Vec::new(),
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
			
            let result = self.children
                .iter()
                .skip(skip)
                .enumerate()
                .take_while(|(i, _)| *i < GRID_WIDTH)
                .map(|(_, num)| *num)
                .collect();
			
			result
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
	
    pub fn update(&mut self, app: &mut App, grid: &mut Grid) {
        match &mut self.state {
            Hang { counter, finished } => {
                if *counter < HANG_TIME {
                    *counter += 1;
				} else {
					*finished = true;
                }
            }
			
            Clear { counter, end_time, finished } => {
                if *counter < *end_time {
                    let mut remove = None;
					
					for (i, child_index) in self.children.iter_mut().enumerate() {
						let mut reset = false;
						
						if let Components::GarbageChild(c) = &mut grid[*child_index] {
							if c.counter > c.start_time {
								if (c.counter - c.start_time) < CLEAR_TIME {
									c.scale = 1. - ((c.counter - c.start_time) as f32) / (CLEAR_TIME as f32);
								} else {
									if c.randomize_at_end {
										reset = true;
									} else {
										c.scale = 1.;
									}
								}
							}
							
							c.counter += 1;
						} else {
							panic!("GARBAGE: wasnt garbage even though it should have been");
						}
						
						if reset {
							grid[*child_index] = Components::Normal(Block::random_clear(app));
							remove = Some(i);
							break;
						} 
					}
					
					if let Some(index) = remove {
						self.removed_children.push(self.children.remove(index));
					}
						
					*counter += 1;
                } else {
					for child_index in self.removed_children.iter() {
						if let Some(state) = grid.block_state_mut(*child_index) {
							state.to_idle();
						}
					}
					self.removed_children.clear();
					
					*finished = true;
                }
            }
			
			_ => {}
		}
    }
}
