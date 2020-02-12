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
    pub fn gen_1d_garbage(&mut self, width: usize, offset: usize) {
        assert!(width >= 3);
        assert!(offset < GRID_WIDTH);
		
        let children: Vec<usize> = (offset..offset + width).collect();
		
        for x in offset..offset + width {
            if let Some(index) = (x, 0).to_index() {
                self.components[index] = {
                    if index == offset {
                        Components::GarbageParent(Garbage::new(children.clone()))
                    } else {
                        Components::GarbageChild(offset)
                    }
                };
            }
        }
    }
	
    // generates a line of garbage at the top of the grid
    pub fn gen_2d_garbage(&mut self, height: usize) {
        assert!(height >= 1);
		
        let children: Vec<usize> = (0..height * GRID_WIDTH).collect();
		
        for x in 0..6 {
            for y in 0..height {
                if let Some(index) = (x, y).to_index() {
                    self.components[index] = {
                        if index == 0 {
                            Components::GarbageParent(Garbage::new(children.clone()))
                        } else {
                            Components::GarbageChild(0)
                        }
                    };
                }
            }
        }
    }
	
    // check if the garbage head can currently hang
    pub fn garbage_can_hang<I: Copy + BoundIndex>(&self, i: I) -> Option<u32> {
        // get copy of indexes
        let child_indexes: Option<Vec<usize>> = self.garbage(i).map(|g| g.lowest());
		
        let mut can_hang = true;
        let mut hang_counter = 0;
		
        // loop through children, look below each and check
        if let Some(indexes) = child_indexes {
            // look for the garbage parent below only once
            {
				/*
				/let parent_index = indexes[0];
				
				// TODO(Skytrias): look for garbage child and its parent_index
				if let Some(state) = self.garbage_state(parent_index) {
				   if let GarbageStates::Hang { counter, .. } = state {
				   return Some(*counter);
				   }
				}
				*/
				
				
			}
			
				// look for normal blocks below
				for child_index in indexes.iter() {
				if let Some(ib) = (child_index + GRID_WIDTH).to_index() {
					let mut one_found = false;
					
					if !self[ib].is_empty() {
						if let Some(state) = self.block_state(ib) {
							if let BlockStates::Hang { counter, .. } = state {
								hang_counter = max(hang_counter, *counter);
								one_found = true;
							}
						}
						
						if let Components::GarbageChild(parent_index) = &self.components[ib] {
							if let Some(parent_state) = self.garbage_state(*parent_index) {
								if let GarbageStates::Hang { counter, .. } = parent_state {
									hang_counter = max(hang_counter, *counter);
									one_found = true;
								}
							}
						}
						
						if !one_found {
							can_hang = false;
						}
					}
				}
				}
			
				// set to hang if not already
				if can_hang {
				return Some(hang_counter);
				}
		  }
		
		  None
	}
	
	// detects any nearby clears happening next to the garbage head or its children
	pub fn garbage_detect_clears(&mut self) {
		  for (_, _, i) in iter_yx_rev() {
				// skip none garbage and also only allow idle to be detected
				if self.garbage(i).is_none() {
				continue;
				}
			
				let g = self.garbage(i).unwrap();
			
				if !g.state.is_idle() {
				continue;
				}
			
				let clear_found = g.children.iter().any(|&i| {
														// TODO(Skytrias): better way to avoid 0 - 1 on usize
														let neighbors = {
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
				if let Some(g) = self.garbage_mut(i) {
					g.state.to_clear();
				}
				}
		  }
	}
	
	// turns part of the garbage into random blocks, effect differs wether its 2d or not
	pub fn garbage_resolve_clears(&mut self, app: &mut App) {
		  for (_, _, i) in iter_xy() {
				if let Some(g) = self.garbage_mut(i) {
				if g.state.clear_finished() {
					if g.is_2d {
						// delete the lowest blocks and loop through those instead of all
						let lowest = g.drain_lowest();
						g.state = GarbageStates::Idle;
						
						// convert all into random in 1d
						for index in lowest.iter() {
							self.components[*index] = Components::Normal(Block::random(app));
						}
					} else {
						// convert all into random in 1d
						for index in g.children.clone().iter() {
							self.components[*index] = Components::Normal(Block::random(app));
						}
					}
				}
				}
		  }
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
				if let Some(b) = self.block_mut((x, GRID_HEIGHT - 1)) {
				if b.state.is_swap() || b.state.is_clear() {
					continue;
				}
				
				b.state = BlockStates::Bottom;
				}
		  }
	}
	
	/*
	// detects wether the block can currently hang, switches the state to hang with below counter
	pub fn block_garbage_detect_hang(&mut self) {
	   for (x, y, i) in iter_yx_rev() {
	   // garbage hang if in idle and every child can hang
	   if self.garbage(i).is_some() {
	   if self.garbage_state(i).filter(|s| s.is_idle()).is_none() {
	   continue;
	   }
	   
	   if let Some(counter) = self.garbage_can_hang(i) {
	   if let Some(state) = self.garbage_state_mut(i) {
	   state.to_hang(counter);
	   }
	   }
	   }
	   }
	   
	   for (x, y, i) in iter_yx_rev() {
	   // block hang startup, only allow idle
	   if self.block_state(i).filter(|s| s.is_idle()).is_some() {
	   if let Some(ib) = (x, y + 1).to_index() {
	   let below_empty = self[ib].is_empty();
	   let below_block_state = *self.block_state(ib).unwrap_or(&BlockStates::Idle);
	   
	   // look for garbage child parent or the parent itself
	   let below_garbage_state: GarbageStates = {
	   let mut parent_index = None;
	   if let Components::GarbageChild(index) = self.components[ib] {
	   parent_index = Some(index);
	   }
	   
	   if let Some(index) = parent_index {
	   *self.garbage_state(index).unwrap_or(&GarbageStates::Idle)
	   } else {
	   *self.garbage_state(ib).unwrap_or(&GarbageStates::Idle)
	   }
	   };
	   
	   if let Some(state) = self.block_state_mut(i) {
	   if below_empty {
	   if state.is_idle() {
	   state.to_hang(0);
	   }
	   } else {
	   if below_block_state.is_hang() {
	   *state = below_block_state;
	   }
	   
	   if below_garbage_state.is_hang() {
	   if let GarbageStates::Hang { counter, .. } = below_garbage_state {
	   state.to_hang(counter);
	   }
	   }
	   }
	   }
	   }
	   }
	   }
	}
	
	// once block / garbage hang is done it swaps index components out
	// 2 types of hang have to happen in the same yx_rev loop,
	pub fn block_garbage_resolve_hang(&mut self) {
	   // get all parent related info for hang before hand, make it easy to access via hashmap
	   let mut parent_map = HashMap::new();
	   for i in 0..GRID_TOTAL {
	   if let Some(g) = self.garbage(i) {
	   parent_map.insert(i, (self.garbage_can_hang(i).is_some(), g.state.hang_finished()));
	   }
	   }
	   
	   for (x, y, i) in iter_yx_rev() {
	   // block hang finish
	   if self.block_state(i).filter(|s| s.hang_finished()).is_some() {
	   // below in range && below still empty
	   if let Some(ib) = (x, y + 1).to_index() {
	   if self[ib].is_empty() {
	   self.components.swap(i, ib);
	   
	   let index_below_below = (x, y + 2).to_index();
	   
	   // stop hanging if below below is not empty
	   if let Some(ibb) = index_below_below {
	   if !self[ibb].is_empty() {
	   self[i].reset();
	   self[ib].reset();
	   }
	   }
	   }
	   }
	   }
	   
	   // garbage hang
	   {
	   let mut fell = false;
	   match &mut self.components[i.raw()] {
	   // swap child itself
	   Components::GarbageChild(parent_index) => {
	   if let Some((can_hang, hang_finished)) = parent_map.get(parent_index) {
	   if *hang_finished && *can_hang {
	   if let Some(ib) = (i + GRID_WIDTH).to_index() {
	   if self[ib].is_empty() {
	   *parent_index += GRID_WIDTH;
	   fell = true;
	   }
	   }
	   }
	   }
	   }
	   
	   // swap parent and its child indexes
	   Components::GarbageParent(g) => {
	   if let Some((can_hang, hang_finished)) = parent_map.get(&i) {
	   if *hang_finished {
	   if *can_hang {
	   if let Some(ib) = (i + GRID_WIDTH).to_index() {
	   if self[ib].is_empty() {
	   for index in g.children.iter_mut() {
	   *index += GRID_WIDTH;
	   }
	   
	   fell = true;
	   } else {
	   g.state = GarbageStates::Idle;
	   }
	   } else {
	   // NOTE(Skytrias): bottom?
	   g.state = GarbageStates::Idle;
	   }
	   } else {
	   g.state = GarbageStates::Idle;
	   }
	   }
	   }
	   }
	   
	   _ => {}
	   }
	   
	   // delayed swap cuz borrow
	   if fell {
	   if let Some(ib) = (i + GRID_WIDTH).to_index() {
	   self.components.swap(i, ib);
	   }
	   }
	   }
	   }
	}
	*/
	
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
	
	pub fn update(&mut self, _app: &mut App) {
		assert!(!self.components.is_empty());
		
		// NOTE(Skytrias): resolves might need to happen before detects, so there is 1 frame delay?
		//self.garbage_detect_clears();
		//self.garbage_resolve_clears(app);
		self.block_resolve_swap();
		self.block_detect_bottom();
		
		// hang detection 
		{
			for (_, _, i) in iter_xy() {
				// block hang detection
				if self.block_state(i).filter(|s| s.is_idle()).is_some() {
					if let Some(ib) = (i + GRID_WIDTH).to_index() {
						if self[ib].is_empty() {
							if let Some(state) = self.block_state_mut(i) {
								state.to_hang(0);
							}
						}
					}
				}
				
				// garbage hang detection
				if self.garbage_state(i).filter(|s| s.is_idle()).is_some() {
					let can_hang = {
						if let Some(g) = self.garbage(i) {
							g.lowest_empty(self)
						} else {
							// NOTE(Skytrias): might have to be true
							false
						}
					};
					
					if can_hang {
						if let Some(g) = self.garbage_mut(i) {
							g.state.to_hang(0);
						}
					}
				}
			}
		}
		
		// block hang finish, set all above finished block to fall state 
		{
			let mut above_fall = false;
			// look for block and empty below
			for (_, _, i) in iter_yx_rev() {
				// TODO(Skytrias): check for if below empty again? since a few frames passed
				if let Some(state) = self.block_state_mut(i) {
					match state {
						// any hang finished, set to fall and let other normal blocks above it fall too
						BlockStates::Hang { finished, .. } => {
							if *finished {
								state.to_fall();
								above_fall = true;
							}
						}
						
						// fall too if below was hang finished
						BlockStates::Idle => {
							if above_fall {
								state.to_fall();
							}
						}
						
						// NOTE(Skytrias): INCLUDES GARBAGE
						// short circuit the fall loop
						_ => {
							above_fall = false;
						}
					}
				}
				}
		}
		
		// garbage hang finish, set all above finished block to fall state 
		{
			let mut above_fall = false;
			// look for block and empty below
			for (_, _, i) in iter_yx_rev() {
				let opt_parent_index = {
					match &self[i] {
						Components::GarbageParent(_) => Some(i),
						Components::GarbageChild(new_i) => Some(*new_i),
						_ => None
					}
				};
				
				if let Some(parent_index) = opt_parent_index {
					if let Some(mut state) = self.garbage_state_mut(parent_index) {
						match &mut state {
							GarbageStates::Hang { finished, .. } => {
								if *finished {
									state.to_fall();
									println!("{}, set set parent to fall", parent_index);
									above_fall = true;
								}
							} 
							
							GarbageStates::Idle => {
								if above_fall {
									println!("{}, set above garbage to fall", parent_index);
									state.to_fall();
								}
							} 
							
							_ => {}
						}
					}
				}
				
				match &mut self[i] {
					Components::Normal(b) => {
						if b.state.is_idle() && above_fall {
							b.state.to_fall();
						}
					}
					
					Components::Empty => {
						above_fall = false;
					}
					
					_ => {}
				}
				}
		}
		
		// fall execution
		{
			for (_, _, i) in iter_yx_rev() {
				// block fall
				if self.block_state(i).filter(|s| s.is_fall()).is_some() {
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
			
				// garbage fall
				for (_, _, i) in iter_yx_rev() {
				if self.garbage_state(i).filter(|s| s.is_fall()).is_some() {
					continue;
				}
				
				let opt_lowest_empty = {
					if let Some(g) = self.garbage(i) {
						Some(g.lowest_empty(self))
					} else {
						None
					}
				};
				
				if let Some(lowest_empty) = opt_lowest_empty {
					if !lowest_empty {
						if let Some(state) = self.garbage_state_mut(i) {
							state.to_idle();
						}
					}
					
					let mut opt_children = self.garbage(i).map(|g| g.children.clone());
					
					if let Some(children) = &mut opt_children {
						println!("1: {:?}", children);
						for child_index in children.iter() {
							if let Some(parent_index) = self.garbage_child_mut(*child_index) {
								*parent_index += GRID_WIDTH;
							}
						}
						
						println!("2: {:?}", children);
						if let Some(g) = self.garbage_mut(i) {
							children.sort_by(|a, b| b.cmp(a));
							
							for child_index in children.iter_mut() {
								*child_index += GRID_WIDTH;
							}
							
							g.children = children.clone();
						}
						
						println!("3: {:?}", children);
						for child_index in children.iter() {
							self.components.swap(*child_index - GRID_WIDTH, *child_index);
						}
					}
				}
				}
		}
		
		//self.block_detect_clear();
		//self.block_resolve_clear();
		
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
				Components::GarbageParent(_) => return,
				Components::GarbageChild(_) => return,
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
	
	// garbage & and &mut accesors
	
	pub fn garbage<I: BoundIndex>(&self, index: I) -> Option<&Garbage> {
		match &self[index] {
			Components::GarbageParent(g) => Some(&g),
			_ => None,
		}
	}
	
	pub fn garbage_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut Garbage> {
		match &mut self[index] {
			Components::GarbageParent(g) => Some(g),
			_ => None,
		}
	}
	
	pub fn garbage_state<I: BoundIndex>(&self, index: I) -> Option<&GarbageStates> {
		match &self[index] {
			Components::GarbageParent(g) => Some(&g.state),
			_ => None,
		}
	}
	
	pub fn garbage_state_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut GarbageStates> {
		match &mut self[index] {
			Components::GarbageParent(g) => Some(&mut g.state),
			_ => None,
		}
	}
	
	pub fn garbage_child_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut usize> {
		match &mut self[index] {
			Components::GarbageChild(num) => Some(num),
			_ => None,
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
