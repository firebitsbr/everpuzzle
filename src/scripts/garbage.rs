use crate::engine::App;
use crate::helpers::*;
use crate::scripts::{Block, BlockState, Component, Grid};
use GarbageState::*;

pub enum GarbageState {
    Idle,

    Fall,

    Hang {
        counter: u32,
    },

    Clear {
        counter: u32,
        end_time: u32,
        finished: bool,
    },
}

/// garbage child data used mainly for unqiue animation
pub struct Child {
    /// y pixel offset of child
    pub y_offset: f32,

    /// hframe horizontal position in the texture atlas
    pub hframe: u32,

    /// vframe vertical position in the texture atlas
    pub vframe: u32,

    /// clear animation counter
    pub counter: u32,

    /// clear start animation frame time
    pub start_time: u32,

    /// wether the child will be randomized into a block after clear animation
    pub randomize_at_end: bool,

    /// clear animation scale
    pub scale: V2,

    /// wether the clearing animation finished on this child
    pub finished: bool,
}

impl Default for Child {
    fn default() -> Self {
        Self {
            hframe: 0,
            vframe: 0,
            counter: 0,
            start_time: 0,
            randomize_at_end: false,
            scale: V2::one(),
            y_offset: 0.,
            finished: false,
        }
    }
}

impl Child {
    /// temporary way to generate the hframes / vframes dependand on height
    pub fn gen_2d_frames(x: usize, y: usize, mut height: usize) -> (u32, u32) {
        height = height.max(1);
		//debug_assert!(height >= 1);

        if height != 1 {
            let hframe = {
                if x == 0 {
                    if y == 0 {
                        1
                    } else if y == height - 1 {
                        2
                    } else {
                        8
                    }
                } else {
                    if y == 0 {
                        if x == GRID_WIDTH - 1 {
                            3
                        } else {
                            5
                        }
                    } else {
                        if x == GRID_WIDTH - 1 {
                            if y == height - 1 {
                                4
                            } else {
                                6
                            }
                        } else {
                            if y == height - 1 {
                                7
                            } else {
                                // midle pieces
                                0
                            }
                        }
                    }
                }
            };

            (hframe, ATLAS_GARBAGE_2D)
        } else {
            Child::gen_1d_frames(x, GRID_WIDTH)
        }
    }

    /// temporary way to generate the hframes / vframes by default for 1D garbages
    pub fn gen_1d_frames(i: usize, end: usize) -> (u32, u32) {
        let hframe = {
            if i == 0 {
                0
            } else if i == end - 1 {
                2
            } else {
                1
            }
        };

        (hframe, ATLAS_GARBAGE_1D)
    }
}

/// system that holds N garbages per 1 grid
pub struct GarbageSystem {
    pub list: Vec<Garbage>,
}

impl Default for GarbageSystem {
    fn default() -> Self {
        Self { list: Vec::new() }
    }
}

impl GarbageSystem {
    /// calls the update event on each garbage
    pub fn update(&mut self, grid: &mut Grid) {
		for garbage in self.list.iter_mut() {
			if grid.id == garbage.parent_id {
				garbage.update(grid);
			}
        }
    }
	
	pub fn lowest_idle(&mut self, grid: &Grid) -> Option<usize> {
		let mut min_y = 100_000;
		
		// TODO(Skytrias): loops through all garbage
		for garbage in self.list.iter() {
				// leave early if all children are gone
			if garbage.children.is_empty() {
				return None;
			}
			
			if garbage.parent_id == grid.id {
				if let GarbageState::Idle = garbage.state {
					min_y = min_y.min(garbage.lowest_y());
				}
		}
		}
		
		if min_y != 100_000 {
			Some(min_y)
		} else {
			None
		}
	}
	
	pub fn lowest_clear(&mut self, grid: &Grid) -> Option<usize> {
		let mut min_y = 100_000;
		
		for garbage in self.list.iter() {
				// leave early if all children are gone
			if garbage.children.is_empty() {
				return None;
			}
			
			if garbage.parent_id == grid.id {
				if let GarbageState::Clear { .. } = garbage.state {
					min_y = min_y.min(garbage.lowest_y());
				}
			}
		}
		
		if min_y != 100_000 {
			Some(min_y)
		} else {
			None
		}
	}
}

/// garbage that holds N indexes to garbage children in the list
pub struct Garbage {
    /// will only update, when parent grid is calling update
	pub parent_id: usize,
	
	/// list of children indexes that exist in the grid
    pub children: Vec<usize>,

    /// list of children removed in clear, that have to be idled
    removed_children: Vec<usize>,

    /// len of children, should stay the same
    count: usize,

    /// wether the garbage has more than 6 children
    pub is_2d: bool,

    /// super state of the garbage and its children
    pub state: GarbageState,
}

impl Garbage {
    /// creates a garbage with an array of indexes that match the grid garbage children that were spawned
    pub fn new(parent_id: usize, children: Vec<usize>) -> Self {
        let count = children.len();

        Self {
			parent_id,
			children,
            count,
            state: Fall,
            is_2d: count > GRID_WIDTH,
            removed_children: Vec::new(),
        }
    }

    // NOTE(Skytrias): shouldnt be called when its 1d
    /// removes the lowest children and returns them if the garbage is still 2d
    pub fn drain_lowest(&mut self) -> Vec<usize> {
        let skip = (self.count / GRID_WIDTH - 1) * GRID_WIDTH;

        self.count = self.children.len() - GRID_WIDTH;
        self.is_2d = self.count > GRID_WIDTH;

        self.children.drain(skip..).collect()
    }

    /// returns all the lowest children indexes, if 2d skip to the bottom 6 children
    pub fn lowest(&self) -> Vec<usize> {
        if self.is_2d {
            let skip = (self.count / GRID_WIDTH - 1) * GRID_WIDTH;

            let result = self
                .children
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
	
    /// checks wether the lowest blocks below are all empty
    pub fn lowest_empty(&self, grid: &Grid) -> bool {
        let mut can_hang = true;

        for child_index in self.lowest().iter() {
            match grid[child_index + GRID_WIDTH] {
                Component::Block { .. } => can_hang = false,
                Component::Child(_) => can_hang = false,
                _ => {}
            }
        }

        can_hang
    }
	
    /// returns the highest existing children, will always work
    pub fn highest(&self) -> Vec<usize> {
        self.children
            .iter()
            .enumerate()
            .take_while(|(i, _)| *i < GRID_WIDTH)
            .map(|(_, num)| *num)
            .collect()
    }
	
	fn lowest_y(&self) -> usize {
		(self.lowest()[0] as f32 / GRID_WIDTH as f32).floor() as usize
	}
	
    /// updates the garbage variables based on each state, mostly animation based
    pub fn update(&mut self, grid: &mut Grid) {
        match &mut self.state {
            Hang { counter } => *counter += 1,
			
            Clear {
                counter,
                end_time,
                finished,
            } => {
                if *counter < *end_time {
                    let mut remove = None;

                    // TODO(Skytrias): create simple gen_2d_frames based on xycount or icount
                    let min_pos = self.children.iter().min().unwrap_or(&0);
                    let min_y = (*min_pos as f32 / GRID_WIDTH as f32).floor() as usize;

                    for (i, child_index) in self.children.iter_mut().enumerate() {
                        let mut reset = false;

                        if let Component::Child(c) = &mut grid[*child_index] {
                            if c.finished {
                                continue;
                            }

                            if c.counter > c.start_time {
                                if (c.counter - c.start_time) < CLEAR_TIME {
                                    let amt = 1.
                                        - ((c.counter - c.start_time) as f32) / (CLEAR_TIME as f32);
                                    c.scale = V2::broadcast(amt);
                                } else {
                                    if c.randomize_at_end {
                                        reset = true;
                                    } else {
                                        let (hframe, vframe) = {
                                            let pos = child_index.to_v2();

                                            // TODO(Skytrias): too complex
                                            if self.is_2d {
                                                Child::gen_2d_frames(
                                                    pos.x as usize,
                                                    pos.y as usize - min_y,
                                                    (self.count - GRID_WIDTH) / GRID_WIDTH,
                                                )
                                            } else {
                                                println!("child");
                                                Child::gen_1d_frames(pos.x as usize, self.count)
                                            }
                                        };

                                        c.vframe = vframe;
                                        c.hframe = hframe;
                                        c.scale = V2::one();
                                    }

                                    c.finished = true;
                                }
                            }

                            c.counter += 1;
                        } else {
                            panic!("GARBAGE: wasnt garbage even though it should have been");
                        }

                        // reset the block and save the i in which that block lived
                        if reset {
                            grid[*child_index] = Component::Block {
                                state: BlockState::Spawned,
                                block: Block {
                                    offset: V2::new(0., -grid.push_amount),
                                    vframe: Block::random_vframe(&mut grid.rng),

                                    // allow chains from garbage
                                    saved_chain: Some(1),
                                    ..Default::default()
                                },
                            };
							println!("{} hi", *child_index);
							remove = Some(i);
                            break;
                        }
                    }

                    // remove the saved children in i in children, save in removed_children
                    if let Some(index) = remove {
                        self.count -= 1;
                        self.removed_children.push(self.children.remove(index));
                    }

                    *counter += 1;
                } else {
                    for child_index in self.removed_children.iter() {
                        if let Component::Block { state, .. } = &mut grid[*child_index] {
                            *state = BlockState::Idle;
                        }
                    }

                    self.is_2d = self.count > GRID_WIDTH;
                    self.removed_children.clear();

                    *finished = true;
                }
            }

            _ => {}
        }
    }
}
