use crate::engine::App;
use crate::helpers::{CLEAR_TIME, GRID_WIDTH, HANG_TIME, V2};
use crate::scripts::{Block, Component, Grid};
use GarbageState::*;

/// garbage child data used mainly for unqiue animation
pub struct Child {
    pub counter: u32,
    pub start_time: u32,
    pub randomize_at_end: bool,
    pub scale: V2,
}

impl Default for Child {
    fn default() -> Self {
        Self {
            counter: 0,
            start_time: 0,
            randomize_at_end: false,
            scale: V2::one(),
        }
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
    pub fn update(&mut self, app: &mut App, grid: &mut Grid) {
        for garbage in self.list.iter_mut() {
            garbage.update(app, grid);
        }
    }
}

block_state!(
			 GarbageState,
			 {
				 Idle, idle,
				 Fall, fall,
				 },
			 {
				 Hang, hang {
					 
				 },
				 
				 Clear, clear {
					 end_time: u32,
				 },
			 }
			 );

/// garbage that holds N indexes to garbage children in the list
pub struct Garbage {
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
    /// creates a garbage with an array of indexes that match the grid garbage children that were spawned
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
            if !grid[child_index + GRID_WIDTH].is_empty() {
                can_hang = false;
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

    /// updates the garbage variables based on each state, mostly animation based
    pub fn update(&mut self, app: &mut App, grid: &mut Grid) {
        match &mut self.state {
            Hang { counter, finished } => {
                if *counter < HANG_TIME {
                    *counter += 1;
                } else {
                    *finished = true;
                }
            }

            Clear {
                counter,
                end_time,
                finished,
            } => {
                if *counter < *end_time {
                    let mut remove = None;

                    for (i, child_index) in self.children.iter_mut().enumerate() {
                        let mut reset = false;

                        if let Component::GarbageChild(c) = &mut grid[*child_index] {
                            if c.counter > c.start_time {
                                if (c.counter - c.start_time) < CLEAR_TIME {
                                    let amt = 1.
                                        - ((c.counter - c.start_time) as f32) / (CLEAR_TIME as f32);
                                    c.scale = V2::broadcast(amt);
                                } else {
                                    if c.randomize_at_end {
                                        reset = true;
                                    } else {
                                        c.scale = V2::one();
                                    }
                                }
                            }

                            c.counter += 1;
                        } else {
                            panic!("GARBAGE: wasnt garbage even though it should have been");
                        }

                        if reset {
                            grid[*child_index] = Component::Normal(Block::random_clear(app));
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
