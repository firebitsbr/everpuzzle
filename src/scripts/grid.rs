use crate::engine::App;
use crate::scripts::*;
use crate::helpers::*;
use std::ops::{Index, IndexMut};
use num_traits::cast::ToPrimitive;

pub struct Grid {
	pub components: Vec<Components>,
	placeholder: Components,
	
	flood_x_count: u32,
	flood_x_history: Vec<usize>,
	flood_y_count: u32,
	flood_y_history: Vec<usize>,
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			components: Vec::with_capacity(GRID_TOTAL),
			placeholder: Components::Placeholder,
			
			flood_x_count: 0,
			flood_x_history: Vec::with_capacity(GRID_WIDTH),
			
			flood_y_count: 0,
			flood_y_history: Vec::with_capacity(GRID_HEIGHT),
		}
	}
}

impl Grid {
	pub fn new(app: &mut App) -> Self {
		let mut components = Vec::with_capacity(GRID_TOTAL);
		
		for _x in 0..GRID_WIDTH {
			for _y in 0..GRID_HEIGHT {
				if app.rand_int(1) == 0 {
					components.push(Components::Empty);
				} else {
					components.push(Components::Normal(Block::new((app.rand_int(5) + 2) as f32)));
				}
			}
		}
		
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
		for i in 0..GRID_TOTAL - 1 {
			if let Components::Normal(b) = &mut self[i] {
				if let BlockStates::Swap(_, direction, finished) = b.state {
					if finished {
						// offset depends on the swap direction
						let offset = match direction {
							SwapDirection::Left => i - 1,
							SwapDirection::Right => i + 1,
						};
						
						self.components.swap(i, offset);
						&mut self[i].reset();
						&mut self[offset].reset();
					}
				}
			}
		}
		
		// set bottom row to bottom state
		for x in 0..GRID_WIDTH {
			if let Components::Normal(b) = self.from_xy_mut(x, GRID_HEIGHT - 1) {
				// skip swap and clear, since bottom blocks can do those
				if b.state.is_swap() || b.state.is_clear() {
					continue;
				}
				
				b.state = BlockStates::Bottom;
			}
		}
		
		// hang setting
		for x in (0..GRID_WIDTH).rev() {
			for y in (0..GRID_HEIGHT).rev() {
				if let Some(i) = xy_to_index(x, y) {
					if let Some(ib) = xy_to_index(x, y + 1) {
						let below_empty = self[ib].is_none();
						let below_state = self.block_state(ib)
							.unwrap_or(&BlockStates::Idle)
							.clone();
						
						if let Components::Normal(b) = &mut self[i] {
							if below_empty {
								if b.state.is_idle() {
									b.state = BlockStates::Hang(0, false);
								}
							} else {
								if !below_state.is_bottom() && below_state.is_hang() {
									b.state = below_state;
								}
							} 
						}
					}
				}
			}
		}
		
		// hang finished execution
		for x in (0..GRID_WIDTH).rev() {
			for y in (0..GRID_HEIGHT).rev() {
				if let Some(i) = xy_to_index(x, y) {
					let should_fall = self.block_state(i)
						.map(|s| s.is_hang() && s.hang_finished())
						.unwrap_or(false);
					
					if should_fall {
						let index_below = xy_to_index(x, y + 1);
						
						if let Some(ib) = index_below {
							self.components.swap(i, ib);
							
							let index_below_below = xy_to_index(x, y + 2);
							
							if let Some(ibb) = index_below_below {
								if !self[ibb].is_none() {
									&mut self[i].reset();
									&mut self[ib].reset();
								} 
							} 
						} 
					}
				}
			}
		}
		
		// set bottom row to bottom state
		for x in 0..GRID_WIDTH {
			for y in 0..GRID_HEIGHT {
				let index = xy_to_index(x, y).unwrap_or(0); // always real value
				
				let mut vframe = 0.;
				if let Components::Normal(b) = &self[index] {
					if b.state.is_real() {
						vframe = b.vframe;
					}
				}
				
				if vframe != 0. {
					self.flood_fill_x_count(x as f32, y as f32, vframe as f32);
					self.flood_fill_y_count(x as f32, y as f32, vframe as f32);
					
					if self.flood_x_count > 2 {
						// TODO(Skytrias): bad to clone!
						for clear_index in self.flood_x_history.clone().iter() {
							if let Components::Normal(b) = &mut self[*clear_index] {
								b.state = BlockStates::Clear(0, false);
							}
						}
					}
					
					if self.flood_y_count > 2 {
						// TODO(Skytrias): bad to clone!
						for clear_index in self.flood_y_history.clone().iter() {
							if let Components::Normal(b) = &mut self[*clear_index] {
								b.state = BlockStates::Clear(0, false);
							}
						}
					}
					
					self.flood_x_count = 0;
					self.flood_x_history.clear();
					self.flood_y_count = 0;
					self.flood_y_history.clear();
				}
			}
		}
		
		// set bottom row to bottom state
		for i in 0..GRID_TOTAL {
			let mut finished = false;
			if let Components::Normal(b) = &self[i] {
				if b.state.is_clear() && b.state.clear_finished() {
					finished = true
				}
			}
			
			if finished {
				self.components[i] = Components::Empty;
			}
		}
		
		// update state timers
		for c in self.components.iter_mut() {
			c.update();
		}
	}
	
	pub fn flood_fill_x_count(&mut self, x: f32, y: f32, vframe: f32) {
		if let Some(index) = xy_to_index(x, y) {
			// skip already checked ones
			if self.flood_x_history.contains(&index) {
				return;
			}
			
			// dont allow empty components
			if let Components::Empty = &self[index] {
				return;
			}
			
			// only allow the same vframe to be counted
			if let Components::Normal(b) =  &self[index] {
				if b.vframe != vframe || !b.state.is_real() {
					return;
				}
			}
			
			self.flood_x_history.push(index);
			self.flood_x_count += 1;
			
			// repeat recursively around the component, gaining counts
			self.flood_fill_x_count(x + 1., y, vframe);
			self.flood_fill_x_count(x - 1., y, vframe);
		}
	}
	
	pub fn flood_fill_y_count(&mut self, x: f32, y: f32, vframe: f32) {
		if let Some(index) = xy_to_index(x, y) {
			// skip already checked ones
			if self.flood_y_history.contains(&index) {
				return;
			}
			
			// dont allow empty components
			if let Components::Empty = &self[index] {
				return;
			}
			
			// only allow the same vframe to be counted
			if let Components::Normal(b) =  &self[index] {
				if b.vframe != vframe || !b.state.is_real() {
					return;
				}
			}
			
			self.flood_y_history.push(index);
			self.flood_y_count += 1;
			
			// repeat recursively around the component, gaining counts
			self.flood_fill_y_count(x, y + 1., vframe);
			self.flood_fill_y_count(x, y - 1., vframe);
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
				let index = xy_to_index(x, y).unwrap_or(0); // always real
				
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
	
	// TODO(Skytrias): might be slow!
	// NOTE(Skytrias): helpers to get a component from any numerical type of x and y
	pub fn from_xy<T: ToPrimitive>(&self, x: T, y: T) -> &Components {
		if let Some(index) = xy_to_index(x, y) {
			&self.components[index]
		} else {
			&self.placeholder
		}
	}
	
	pub fn from_xy_mut<T: ToPrimitive>(&mut self, x: T, y: T) -> &mut Components {
		if let Some(index) = xy_to_index(x, y) {
			&mut self.components[index]
		} else {
			&mut self.placeholder
		}
	}
	
	// helpers 
	pub fn block(&self, index: usize) -> Option<&Block> {
		match &self[index] {
			Components::Normal(b) => Some(&b),
			_ => None
		}
	}
	
	pub fn block_state(&self, index: usize) -> Option<&BlockStates> {
		match &self[index] {
			Components::Normal(b) => Some(&b.state),
			_ => None
		}
	}
}

impl Index<usize> for Grid {
	type Output = Components;
	
	fn index(&self, i: usize) -> &Self::Output {
		if i < GRID_TOTAL {
			&self.components[i]
		} else {
			&self.placeholder
		}
	}
}

impl IndexMut<usize> for Grid {
	fn index_mut(&mut self, i: usize) -> &mut Self::Output {
		if i < GRID_TOTAL {
			&mut self.components[i]
		} else {
			&mut self.placeholder
		}
	}
}

impl Index<V2> for Grid {
	type Output = Components;
	
	fn index(&self, v: V2) -> &Self::Output {
		if let Some(i) = v.to_index() {
			&self.components[i]
		} else {
			&self.placeholder
		}
	}
}

impl IndexMut<V2> for Grid {
	fn index_mut(&mut self, v: V2) -> &mut Self::Output {
		if let Some(i) = v.to_index() {
			&mut self.components[i]
		} else {
			// TODO(Skytrias): dont be able to mutify the placeholder
			&mut self.placeholder
		}
	}
}
