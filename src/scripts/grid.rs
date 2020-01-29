use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
enum FloodDirection {
	Horizontal, // -x and +x
	Vertical, // -y and +y
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
						 Components::Normal(Block::new((app.rand_int(5) + 2) as f32))
					 }
				 })
			.collect();
		
        Self {
            components,
            ..Default::default()
        }
    }
	
    pub fn update(&mut self, _app: &mut App) {
        if self.components.len() == 0 {
            return;
        }
		
        // check for swap state
        for (_, _, i) in iter_xy() {
			let result = self.block_swap(i)
				.filter(|s| s.finished)
				.map(|s| match s.direction {
						 SwapDirection::Left => i - 1,
						 SwapDirection::Right => i + 1,
					 });
			
			if let Some(offset) = result {
				self.components.swap(i, offset);
				&mut self[i].reset();
				&mut self[offset].reset();
			}
		}
		
		// set bottom row to bottom state
		for x in 0..GRID_WIDTH {
			if let Some(b) = self.block_mut((x, GRID_HEIGHT - 1)){
				if b.state.is_swap() || b.state.is_clear() {
					continue;
				}
				
				b.state = BlockStates::Bottom;
			}
		}
		
		// hang setting
		for (x, y, i) in iter_yx_rev() {
			if let Some(ib) = (x, y + 1).to_index() {
				let below_empty = self[ib].is_none();
				let below_state =
					self.block_state(ib).unwrap_or(&BlockStates::Idle).clone();
				
				if let Some(state) = self.block_state_mut(i) {
					if below_empty {
						if state.is_idle() {
							*state = BlockStates::Hang(Default::default());
						}
					} else {
						if !below_state.is_bottom() && below_state.is_hang() {
							*state = below_state;
						}
					}
				}
			}
		}
		
		// hang finished execution
		for (x, y, i) in iter_yx_rev() {
			let should_fall = self
				.block_state(i)
				.filter(|s| s.is_hang() && s.hang_finished())
				.is_some();
			
			if should_fall {
				let index_below = (x, y + 1).to_index();
				
				if let Some(ib) = index_below {
					self.components.swap(i, ib);
					
					let index_below_below = (x, y + 2).to_index();
					
					if let Some(ibb) = index_below_below {
						if !self[ibb].is_none() {
							&mut self[i].reset();
							&mut self[ib].reset();
						}
					}
				}
			}
		}
		
		// flood fill check for the current color of the block for any other near colors
		for (x, y, i) in iter_xy() {
			let frame = self.block(i)
				.filter(|b| b.state.is_real())
				.map(|b| b.vframe);
			
			if let Some(vframe) = frame {
				self.flood_check(x, y, vframe as f32, FloodDirection::Horizontal);
				self.flood_check(x, y, vframe as f32, FloodDirection::Vertical);
				
				if self.flood_horizontal_count > 2 {
					// TODO(Skytrias): bad to clone!
					for clear_index in self.flood_horizontal_history.clone().iter() {
						if let Components::Normal(b) = &mut self[*clear_index] {
							b.state = BlockStates::Clear(Default::default());
						}
					}
				}
				
				if self.flood_vertical_count > 2 {
					// TODO(Skytrias): bad to clone!
					for clear_index in self.flood_vertical_history.clone().iter() {
						if let Components::Normal(b) = &mut self[*clear_index] {
							b.state = BlockStates::Clear(Default::default());
						}
					}
				}
				
				self.flood_horizontal_count = 0;
				self.flood_horizontal_history.clear();
				self.flood_vertical_count = 0;
				self.flood_vertical_history.clear();
			}
		}
		
		// clear the component if clear state is finished
		for (_, _, i) in iter_xy() {
			let finished = self.block_state(i)
				.map(|s| s.is_clear() && s.clear_finished())
				.unwrap_or(false);
			
			if finished {
				self.components[i] = Components::Empty;
			}
		}
		
		// update state timers
		for c in self.components.iter_mut() {
			c.update();
		}
	}
	
	fn flood_check(&mut self, x: usize, y: usize, vframe: f32, direction: FloodDirection) {
		if let Some(index) = (x, y).to_index() {
			// dont allow empty components
			if let Components::Empty = &self[index] {
				return;
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
	
	pub fn draw(&mut self, app: &mut App) {
		// draw traversal info
		if false {
			let mut i = 0;
			for x in (0..GRID_WIDTH).rev() {
				for y in (0..GRID_HEIGHT).rev() {
					app.draw_number(i as f32, v2(x as f32, y as f32) * ATLAS_SPACING);
					i += 1;
				}
			}
		}
		
		// NOTE(Skytrias): dirty check
		if self.components.len() == 0 {
			return;
		}
		
		// draw clear info
		for x in 0..GRID_WIDTH {
			for y in 0..GRID_HEIGHT {
				let index = (x, y).no_bounds();
				
				if let Components::Normal(b) = &self[index] {
					if b.state.is_clear() {
						/*
						app.push_sprite(Sprite {
											depth: 0.01,
											position: v2(x as f32, y as f32) * ATLAS_SPACING,
											color: color_alpha(RED, 0.5),
											..Default::default()
										});*/
					}
				}
			}
		}
		
		// NOTE(Skytrias): send and draw ubo data
		let mut data: Vec<V4> = Vec::new();
		for c in self.components.iter() {
			data.push(v4(0., c.vframe(), c.visible(), 1.));
			data.push(v4(c.x_offset(), c.y_offset(), 0., 0.));
		}
		app.draw_grid(&data);
	}
	
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
	
	pub fn block_swap<I: BoundIndex>(&self, index: I) -> Option<&SwapState> {
		match &self[index] {
			Components::Normal(b) => match &b.state {
				BlockStates::Swap(s) => Some(s),
				_ => None
			},
			_ => None
		}
	}
}

impl<T: BoundIndex> Index<T> for Grid {
	type Output = Components;
	
	fn index(&self, grid_index: T) -> &Self::Output {
		if let Some(i) = grid_index.to_index() {
			&self.components[i]
		} else {
			&self.placeholder
		}
	}
}

impl<T: BoundIndex> IndexMut<T> for Grid {
	fn index_mut(&mut self, grid_index: T) -> &mut Self::Output {
		if let Some(i) = grid_index.to_index() {
			&mut self.components[i]
		} else {
			&mut self.placeholder
		}
	}
}