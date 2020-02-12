use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::cmp::max;
use std::ops::{Index, IndexMut};
use std::collections::HashMap;

// like shader
#[derive(Copy, Clone, Debug)]
pub struct GridBlock {
    pub hframe: u32,
    pub vframe: u32,
    pub visible: i32,
    pub scale: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub temp1: f32,
    pub temp2: f32,
}

impl Default for GridBlock {
    fn default() -> Self {
        Self {
            hframe: 0,
            vframe: 0,
            visible: -1,
            scale: 1.,
            x_offset: 0.,
            y_offset: 0.,
            temp1: 0.,
            temp2: 0.,
        }
    }
}

#[derive(Copy, Clone)]
enum FloodDirection {
    Horizontal, // -x and +x
    Vertical,   // -y and +y
}

pub struct Grid {
    pub components: Vec<Components>,
    placeholder: Components,
	
    flood_horizontal_count: u32,
    flood_horizontal_history: Vec<usize>,
    flood_vertical_count: u32,
    flood_vertical_history: Vec<usize>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            components: Vec::with_capacity(GRID_TOTAL),
            placeholder: Components::Placeholder,
			
            flood_horizontal_count: 0,
            flood_horizontal_history: Vec::with_capacity(GRID_WIDTH),
			
            flood_vertical_count: 0,
            flood_vertical_history: Vec::with_capacity(GRID_HEIGHT),
		}
    }
}

impl Grid {
    pub fn new(app: &mut App) -> Self {
        let components: Vec<Components> = (0..GRID_TOTAL)
            .map(|_| {
					 if app.rand_int(1) == 0 {
						 Components::Empty
					 } else {
						 Components::Normal(Block::random(app))
					 }
				 })
            .collect();
		
        Self {
            components,
            ..Default::default()
        }
    }
	
    // generates a line of garbage at the top of the grid
    pub fn gen_1d_garbage(&mut self, garbage_system: &mut GarbageSystem, width: usize, offset: usize) {
        assert!(width >= 3);
        assert!(offset < GRID_WIDTH);
		
        let children: Vec<usize> = (offset..offset + width).collect();
		
		for index in children.iter() {
			self.components[*index] = Components::GarbageChild;
		}
		
		garbage_system.list.push(Garbage::new(children));
	}
	
    // generates a line of garbage at the top of the grid
    pub fn gen_2d_garbage(&mut self, garbage_system: &mut GarbageSystem, height: usize) {
        assert!(height >= 1);
		
        let mut children = Vec::with_capacity(height * 6);
		
        for x in 0..6 {
            for y in 0..height {
				if let Some(index) = (x, y).to_index() {
                    children.push(index);
					self.components[index] = Components::GarbageChild;
				}
			}
		}
		
		garbage_system.list.push(Garbage::new(children));
	}
	
	// swaps the 2 index components around if the block was in swap animation
	pub fn block_resolve_swap(&mut self) {
		for (_, _, i) in iter_xy() {
			if let Some(b) = self.block(i) {
				if let BlockStates::Swap {
					finished,
					direction,
					..
				} = b.state
				{
					if !finished {
						continue;
					}
					
					let offset = match direction {
						SwapDirection::Left => i - 1,
						SwapDirection::Right => i + 1,
					};
					
					self.components.swap(i, offset);
					self[i].reset();
					self[offset].reset();
				}
			}
		}
	}
	
	// NOTE(Skytrias): might not be necessary anymore, depends on hang
	// sets the last row of blocks to bottom state
	pub fn block_detect_bottom(&mut self) {
		// block set bottom row to bottom state
		for x in 0..GRID_WIDTH {
			if let Some(state) = self.block_state_mut((x, GRID_HEIGHT - 1)) {
				if state.is_swap() || state.is_clear() {
					continue;
				}
				
				*state = BlockStates::Bottom;
			}
		}
	}
	
	// block flood fill check for the current color of the block for any other near colors
	pub fn block_detect_clear(&mut self) {
		for (x, y, i) in iter_xy() {
				let frame = self
				.block(i)
				.filter(|b| b.state.is_real())
				.map(|b| b.vframe);
				
				if let Some(vframe) = frame {
				self.flood_check(x, y, vframe, FloodDirection::Horizontal);
				self.flood_check(x, y, vframe, FloodDirection::Vertical);
				
				let mut histories = [
									 self.flood_horizontal_history.clone(),
									 self.flood_vertical_history.clone(),
									 ].concat();
				histories.dedup();
				
				let histories_length = histories.len();
				
				let mut offset: usize = 0;
				let end_time = (histories_length * CLEAR_TIME as usize) as u32;
				if self.flood_horizontal_count > 2 {
					for (j, clear_index) in self.flood_horizontal_history.clone().iter().enumerate() {
						if let Some(state) = self.block_state_mut(*clear_index) {
							state.to_clear((j * CLEAR_TIME  as usize) as u32, end_time);
						}
					}
					
					offset = self.flood_horizontal_count as usize;
				}
				
				if self.flood_vertical_count > 2 {
					for (j, clear_index) in self.flood_vertical_history.clone().iter().enumerate() {
						if let Some(state) = self.block_state_mut(*clear_index) {
							state.to_clear(((offset + j) * CLEAR_TIME as usize) as u32, end_time);
						}
					}
				}
				
				self.flood_horizontal_count = 0;
				self.flood_horizontal_history.clear();
				self.flood_vertical_count = 0;
				self.flood_vertical_history.clear();
				}
		}
	}
	
	// clear the component if clear state is finished
	pub fn block_resolve_clear(&mut self) {
		for (_, _, i) in iter_xy() {
				let finished = self
				.block_state(i)
				.map(|s| s.is_clear() && s.clear_finished())
				.unwrap_or(false);
				
				if finished {
				self.components[i] = Components::Empty;
				}
		}
	}
	
	// block hang detection, if block state is idle and below is empty, set to hang   
	pub fn block_detect_hang(&mut self) {
		for (_, _, i) in iter_xy() {
			if self.block_state_check(i, |s| s.is_idle()) {
			//if self.block_state(i).filter(|s| s.is_idle()).is_some() {
				if let Some(ib) = (i + GRID_WIDTH).to_index() {
					if self[ib].is_empty() {
						if let Some(state) = self.block_state_mut(i) {
							state.to_hang(0);
						}
					}
				}
				}
		}
	}
	
	// loops upwards, checks if a block hang finished, sets all real above the block to fall, even garbage, garbage fall might fail in fall resolve
	pub fn block_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
		// block hang finish, set all above finished block to fall state 
		let mut above_fall = false;
		// look for block and empty below
		for (_, _, i) in iter_yx_rev() {
				// TODO(Skytrias): check for if below empty again? since a few frames passed
				match &mut self[i] {
				Components::Normal(b) => {
					match b.state {
						// any hang finished, set to fall and let other normal blocks above it fall too
						BlockStates::Hang { finished, .. } => {
							if finished {
								b.state.to_fall();
								above_fall = true;
							}
						}
						
						// fall too if below was hang finished
						BlockStates::Idle => {
							if above_fall {
								b.state.to_fall();
							}
						}
						
						// NOTE(Skytrias): INCLUDES GARBAGE
						// short circuit the fall loop
						_ => {
							above_fall = false;
						}
					}
				}
				
				// if child, look it up in any garbage children, set to hang if idle
				Components::GarbageChild => {
					if above_fall {
						for g in garbage_system.list.iter_mut() {
							if g.state.is_idle() {
								if g.children.iter().any(|index| *index == i) {
									g.state.to_fall();
								}
							}
						}
					}
				}
				
				// on empty/anything else set to false
				_ => { 
					above_fall = false;
				}
				}
		}
	}
	
	// block fall execution, swap downwards if still empty below, set to idle otherwhise
	pub fn block_resolve_fall(&mut self) {
		for (_, _, i) in iter_yx_rev() {
				if self.block_state_check(i, |s| s.is_fall()) {
				if let Some(ib) = (i + GRID_WIDTH).to_index() {
					if self[ib].is_empty() {
						self.components.swap(i, ib);
					} else {
						// reset blocks that were in fall and cant fall anymore
						if let Some(state) = self.block_state_mut(i) {
							state.to_idle();
						}
					}
				}
				}
		}
	}
	
	// garbage hang detection, loop through garbages, look if idle and below are all empty, hang 0
	pub fn garbage_detect_hang(&mut self, garbage_system: &mut GarbageSystem) {
		for g in garbage_system.list.iter_mut() {
				if g.state.is_idle() {
				if g.lowest_empty(self) {
					g.state.to_hang(0);
				} else {
					// TODO(Skytrias): set to idle?
				}
				}
		}
	}
	
	// garbage hang finish, loop through garbages, look if hang finished and set to fall 
	pub fn garbage_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
		for g in garbage_system.list.iter_mut() {
				if g.state.hang_finished() {
				g.state.to_fall();
				}
		}
	}
	
	// garbage fall, loop through garbages, if fall and below stil empty, swap components and increase index stored in .children
	pub fn garbage_resolve_fall(&mut self, garbage_system: &mut GarbageSystem) {
		for g in garbage_system.list.iter_mut() {
				if g.state.is_fall() {
				if g.lowest_empty(self) {
					for index in g.children.iter_mut() {
						self.components.swap(*index, *index + GRID_WIDTH);
						*index += GRID_WIDTH;
					}
				} else {
					g.state.to_idle();
				}
				}
		}
	}
	
	// TODO(Skytrias): look for other garbages that are clearing too!
	// TODO(Skytrias): garbage child clear start count in as well!
	// garbage detect clear on multiple blocks, dependant on 2d factor
	pub fn garbage_detect_clear(&mut self, garbage_system: &mut GarbageSystem) {
			for g in garbage_system.list.iter_mut() {
				if g.state.is_idle() {
					let clear_found = g.children.iter().any(|&i| {
																// TODO(Skytrias): better way to avoid 0 - 1 on usize
																let neighbors = {
		// TODO(Skytrias): REFACTOR
																	if g.is_2d {
																		// above, below
																		vec![
																			 self[i + GRID_WIDTH].clear_started(),
																			 if i as i32 - GRID_WIDTH as i32 > 0 {
																				 self[i - GRID_WIDTH].clear_started()
																			 } else {
																				 false
																			 },
																			 ]
																	} else {
																		// above, below, right, left
																		vec![
																			 self[i + GRID_WIDTH].clear_started(),
																			 
																			 if i as i32 - GRID_WIDTH as i32 > 0 {
																				 self[i - GRID_WIDTH].clear_started()
																			 } else {
																				 false
																			 },
																			 
																			 self[i + 1].clear_started(),
																			 
																			 if i as i32 - 1 as i32 > 0 {
																				 self[i - 1].clear_started()
																			 } else {
																				 false
																			 }
																			 ]
																	}
																};
																
																neighbors.iter().any(|b| *b)
															});
					
					if clear_found {
						g.state.to_clear();
					}
				}
		}
	}
	
	
	// garbage clear resolve
	pub fn garbage_resolve_clear(&mut self, app: &mut App, garbage_system: &mut GarbageSystem) {
		for i in 0..garbage_system.list.len() {
				let mut remove_garbage = false;
				
				{
				let g = &mut garbage_system.list[i];
				
				if g.state.clear_finished() {
					if g.is_2d {
						// delete the lowest blocks and loop through those instead of all
						let lowest = g.drain_lowest();
						g.state.to_idle();
						
						// convert all into random in 1d
						for index in lowest.iter() {
							self.components[*index] = Components::Normal(Block::random(app));
						}
					} else {
						// convert all into random in 1d
						for index in g.children.clone().iter() {
							self.components[*index] = Components::Normal(Block::random(app));
						}
						
						remove_garbage = true;
					}
				}
				}
				
				if remove_garbage {
				garbage_system.list.remove(i);
				return;
			}
		}
	}
	
	pub fn update(&mut self, app: &mut App, garbage_system: &mut GarbageSystem) {
		assert!(!self.components.is_empty());
		
		// NOTE(Skytrias): resolves might need to happen before detects, so there is 1 frame delay?
		self.block_resolve_swap();
		self.block_detect_bottom();
		self.block_detect_hang();
		self.block_resolve_hang(garbage_system);
		self.block_resolve_fall();
		
		self.garbage_detect_hang(garbage_system);
		self.garbage_resolve_hang(garbage_system);
		self.garbage_resolve_fall(garbage_system);
		
		self.block_detect_clear();
		self.block_resolve_clear();
		
		self.garbage_detect_clear(garbage_system);
		self.garbage_resolve_clear(app, garbage_system);
		
		// update all components
		for c in self.components.iter_mut() {
				c.update();
		}
	}
	
	fn flood_check(&mut self, x: usize, y: usize, vframe: u32, direction: FloodDirection) {
		if let Some(index) = (x, y).to_index() {
				// dont allow empty components
				match self[index] {
				Components::Empty => return,
				Components::GarbageChild => return,
				_ => {}
				}
				
				// only allow the same vframe to be counted
				if let Components::Normal(b) = &self[index] {
				if b.vframe != vframe || !b.state.is_real() {
					return;
				}
				}
				
				// TODO(Skytrias): could go into standalone function
				match direction {
				FloodDirection::Horizontal => {
					// skip already checked ones
					if self.flood_horizontal_history.contains(&index) {
						return;
					}
					
					self.flood_horizontal_history.push(index);
					self.flood_horizontal_count += 1;
					
					// repeat recursively around the component, gaining counts
					self.flood_check(x + 1, y, vframe, FloodDirection::Horizontal);
					
					if x > 1 {
						self.flood_check(x - 1, y, vframe, FloodDirection::Horizontal);
					}
				}
				
				FloodDirection::Vertical => {
					// skip already checked ones
					if self.flood_vertical_history.contains(&index) {
						return;
					}
					
					self.flood_vertical_history.push(index);
					self.flood_vertical_count += 1;
					
					// repeat recursively around the component, gaining counts
					self.flood_check(x, y + 1, vframe, FloodDirection::Vertical);
					
					if y > 1 {
						self.flood_check(x, y - 1, vframe, FloodDirection::Vertical);
					}
				}
				}
		}
	}
	
	pub fn draw(&mut self, app: &mut App, frame: &wgpu::SwapChainOutput<'_>) {
		assert!(self.components.len() != 0);
		
		// gather info
		// TODO(Skytrias): convert to gridblock
		let data: Vec<GridBlock> = self.components.iter().map(|c| c.to_grid_block()).collect();
		
		app.draw_grid(&data, frame);
	}
	
	// block & and &mut accesors
	
	// returns a block from the specified grid_index
	pub fn block<I: BoundIndex>(&self, index: I) -> Option<&Block> {
		match &self[index] {
				Components::Normal(b) => Some(&b),
				_ => None,
		}
	}
	
	// returns a block from the specified grid_index
	pub fn block_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut Block> {
		match &mut self[index] {
				Components::Normal(b) => Some(b),
				_ => None,
		}
	}
	
	// returns any state if the component is a block
	pub fn block_state<I: BoundIndex>(&self, index: I) -> Option<&BlockStates> {
		match &self[index] {
				Components::Normal(b) => Some(&b.state),
				_ => None,
		}
	}
	
	// returns any state if the component is a block
	pub fn block_state_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut BlockStates> {
		match &mut self[index] {
				Components::Normal(b) => Some(&mut b.state),
				_ => None,
		}
	}
	
	// experimental helpers
	
	// closure on block if it exists
	pub fn block_state_check<I, P>(&mut self, index: I, predicate: P) -> bool
		where I: BoundIndex, P: FnOnce(&BlockStates) -> bool {
		match &self[index] {
			Components::Normal(b) => predicate(&b.state),
			_ => false
		}
	}
}

impl<I: BoundIndex> Index<I> for Grid {
	type Output = Components;
	
	fn index(&self, bound_index: I) -> &Self::Output {
		if let Some(i) = bound_index.to_index() {
				&self.components[i]
		} else {
				&self.placeholder
		}
	}
}

impl<I: BoundIndex> IndexMut<I> for Grid {
	fn index_mut(&mut self, bound_index: I) -> &mut Self::Output {
		if let Some(i) = bound_index.to_index() {
				&mut self.components[i]
		} else {
				&mut self.placeholder
		}
	}
}
