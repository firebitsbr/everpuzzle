use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::ops::{Index, IndexMut};

/// frame time until the push_counter gets reset
const PUSH_TIME: u32 = 100;

/// the grid holds all components and updates all the script logic of each component  
pub struct Grid {
    /// all components that the player can interact with
    pub components: Vec<Component>,

    /// placeholder component so that &component and &mut component can be shared around
    placeholder: Component,

    /// rendering highlight for combo / chain appearing
    combo_highlight: ComboHighlight,

    /// counter till the push_amount is increased
    push_counter: u32,

    /// pixel amount of y offset of all pushable structs
    pub push_amount: f32,
	
	/// the index that is iterated through
	index: Option<i32>,
	
	/// steps through the iteration
	steps: i32,
	
	i: i32,
	
	/// index converted into x positions in the grid used for limiting bounds
	pub x: i32,
	
	/// index converted into y positions in the grid used for limiting bounds
	pub y: i32,
	
	/// last index that can be returned to
	old_index: Option<usize>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            components: Vec::with_capacity(GRID_TOTAL),
            placeholder: Component::Placeholder,
            combo_highlight: Default::default(),

            push_counter: 0,
            push_amount: 0.,
			
			index: None,
			old_index: None,
			steps: 0,
			x: 0,
			y: 0,
			i: 0,
		}
    }
}

// TODO(Skytrias): remove i32 casts

fn x_in_bounds(x: i32) -> bool {
	x >= 0 && x < GRID_WIDTH as i32
}

fn y_in_bounds(y: i32) -> bool {
	 y >= 0 && y < GRID_HEIGHT as i32
}

fn index_in_bounds(index: i32) -> bool {
	index >= 0 && index < GRID_TOTAL as i32
}

impl Grid {
    pub fn iter_xy(&mut self) -> bool {
		if self.steps != 0 {
			if self.x < GRID_WIDTH as i32 - 1 {
			self.x += 1;
		} else {
			self.y += 1;
			self.x = 0;
		}
		} 
			
		self.steps += 1;
		self.i = self.steps - 1;
		self.index = Some(self.i);
		
		if self.steps < GRID_TOTAL as i32 {
			true
		} else {
			self.steps = 0;
			self.x = 0;
			self.y = 0;
			self.i = 0;
			self.index = None;
			false
		}
	}
	
	pub fn iter_yx_rev(&mut self) -> bool {
		if self.steps == 0 {
			self.x = GRID_WIDTH as i32 - 1;
			self.y = GRID_HEIGHT as i32 - 1;
		}
		
			if self.y != 0 {
				self.y -= 1;
			} else {
				// let last self.y != 0 go through
				if self.x != 0 {
					self.x -= 1;
				}
				
				self.y = GRID_HEIGHT as i32 - 1;
			}
		
		self.i = self.y * GRID_WIDTH as i32 + self.x;
		self.index = Some(self.i);
		
		self.steps += 1;
		
		if self.steps < GRID_TOTAL as i32 {
			true
		} else {
			self.steps = 0;
			self.x = 0;
			self.y = 0;
			self.i = 0;
			self.index = None;
			false
		}
	}
	
	fn reset_index(&mut self) -> Option<i32> {
		let result = self.index;
			self.index = Some(self.i);
		result
	}
	
	// TODO(Skytrias): remove boilerplate
	
	pub fn left(&mut self) -> &mut Grid {
		self.index = if x_in_bounds(self.x as i32 - 1) {
			Some(self.i - 1)
		} else {
			None
		};
		
		self
	}
	
	pub fn right(&mut self) -> &mut Grid {
		self.index = if x_in_bounds(self.x as i32 + 1) {
			 Some(self.i + 1)
		} else {
			None
		};
		
		self
	}
	
	pub fn below(&mut self) -> &mut Grid {
		self.index = if y_in_bounds(self.y as i32 + 1) {
			Some(self.i + GRID_WIDTH as i32)
		} else {
			None
		};
		
		self
	}
	
	pub fn above(&mut self) -> &mut Grid {
		self.index = if y_in_bounds(self.y as i32 - 1) {
			Some(self.i - GRID_WIDTH as i32)
		} else {
			None
		};
		
		self
	}
	
	pub fn component(&mut self) -> Option<&Component> {
		self.reset_index().map(move |i| &self[i])
	}
	
	pub fn component_mut(&mut self) -> Option<&mut Component> {
		self.reset_index().map(move |i| &mut self[i])
	}
	
	pub fn x_block(&mut self) -> Option<&Block> {
		self.component().and_then(|c| match c {
										Component::Block(b) => Some(b),
										_ => None
									})
			}
	
	pub fn x_block_mut(&mut self) -> Option<&mut Block> {
		self.component_mut().and_then(|c| match c {
											Component::Block(b) => Some(b),
										_ => None
										})
		}
	
	pub fn state(&mut self) -> Option<&BlockState> {
		self.x_block().map(|b| &b.state)
		}
	
	pub fn state_mut(&mut self) -> Option<&mut BlockState> {
		self.x_block_mut().map(|b| &mut b.state)
	}
	
	pub fn chain(&mut self) -> Option<usize> {
		self.component().and_then(|c| match c {
									  Component::Chainable(size) => Some(*size),
									  _ => None
								  })
			}
	
	pub fn x_empty(&mut self) -> bool {
			self.component().filter(|c| match c {
											Component::Empty => true,
											Component::Chainable(_) => true,
										_ => false
									  }).is_some()
		}

	/// old_index must have been set!
	pub fn swap(&mut self) {
		let index_from = self.i as usize;
		let index_to = self.index.expect("index should be valid!");
		self.components.swap(index_from as usize, index_to as usize);
	}
	
	/// old_index must have been set!
	pub fn reset(&mut self) {
		self.component_mut().map(|c| c.reset()); 
	}
	
	//////////
	
	/// creates empty grid for testing
    pub fn empty() -> Self {
        let components: Vec<Component> = (0..GRID_TOTAL).map(|_| Component::Empty).collect();

        Self {
            components,
            ..Default::default()
        }
    }
	
    /// inits the grid with randomized blocks (seeded)
    pub fn new(app: &mut App) -> Self {
        let components: Vec<Component> = (0..GRID_TOTAL)
            .map(|_| {
                if app.rand_int(1) == 0 {
                    Component::Empty
                } else {
                    Component::Block(Block::random(app))
                }
            })
            .collect();

        Self {
            components,
            ..Default::default()
        }
    }

    // TODO(Skytrias): use new bound iterator yx_non_rev
    pub fn push_upwards(
        &mut self,
        app: &mut App,
        garbage_system: &mut GarbageSystem,
        cursor: &mut Cursor,
    ) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let index = (x, y).raw();

                if y < GRID_HEIGHT - 1 {
                    match &mut self[index + GRID_WIDTH] {
                        Component::Block(b) => b.offset.y = 0.,
                        Component::Child(g) => g.y_offset = 0.,
                        _ => {}
                    }

                    self.components.swap(index, index + GRID_WIDTH);
                } else {
                    self.components[index] = Component::Block(Block {
                        state: BlockState::Bottom,
                        vframe: Block::random_vframe(app),
                        offset: V2::new(0., -self.push_amount),
                        ..Default::default()
                    });
                }
            }
        }

        // TODO(Skytrias): detection for out of bounds?
        // shift up the garbage children indexes
        for garbage in garbage_system.list.iter_mut() {
            for child_index in garbage.children.iter_mut() {
                *child_index -= GRID_WIDTH;
            }
        }

        // shift up the cursor if still in grid range
        if cursor.position.y > 0 {
            cursor.position.y -= 1;
            cursor.last_position.y -= 1;
            cursor.goal_position.y -= 1. * ATLAS_TILE;
            cursor.y_offset = 0.;
        }
    }

    /// generates a line of garbage at the top of the grid
    pub fn gen_1d_garbage(
        &mut self,
        garbage_system: &mut GarbageSystem,
        width: usize,
        offset: usize,
    ) {
        debug_assert!(width >= 3);
        debug_assert!(offset < GRID_WIDTH);

        let children: Vec<usize> = (offset..offset + width).collect();

        for (i, index) in children.iter().enumerate() {
            self.components[*index].to_garbage(Child::gen_1d_frames(i, width));
        }

        garbage_system.list.push(Garbage::new(children));
    }

    /// generates a line of garbage at the top of the grid
    pub fn gen_2d_garbage(&mut self, garbage_system: &mut GarbageSystem, height: usize) {
        debug_assert!(height >= 1);

        let mut children = Vec::with_capacity(height * 6);

        for y in 0..height {
            for x in 0..GRID_WIDTH {
                if let Some(index) = (x, y).to_index() {
                    children.push(index);
                    self.components[index].to_garbage(Child::gen_2d_frames(x, y, height));
                }
            }
        }

        garbage_system.list.push(Garbage::new(children));
    }

    /// swaps the 2 index components around if the block was in swap animation
    pub fn block_resolve_swap(&mut self) {
		while self.iter_xy() {
			if self.state().swap_finished() {
				let direction = self.state().swap_direction().unwrap();
				
				match direction {
					SwapDirection::Left => {
						self.left().swap();
						self.left().reset();
					}
					
					SwapDirection::Right => {
						self.right().swap();
						self.right().reset();
					}
				}
				
				self.reset();
			}
		}
    }

    /// sets the last row of blocks to bottom state
    pub fn block_detect_bottom(&mut self) {
        while self.iter_xy() {
			if self.y == GRID_HEIGHT as i32 - 1 {
				if !(self.state().is_swap() || self.state().is_clear()) {
					self.state_mut().to_bottom();
				}
				} else {
				if self.state().is_bottom() {
					self.state_mut().to_idle();
				}
				}
		}
		}

    pub fn block_detect_clear(&mut self) {
        // NOTE(Skytrias): consider pushing to grid variables?
        let mut list = Vec::new();

        // get all vframes, otherwhise 99
        let vframes: Vec<u32> = (0..GRID_TOTAL)
            .map(|i| {
                self.block(i)
                    .map(|b| {
                        if b.state.is_idle() || b.state.land_started() {
                            b.vframe
                        } else {
                            99
                        }
                    })
                    .unwrap_or(99)
            })
            .collect();

        // loop through vframes and match horizontal or vertical matches, append them to list
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let i = (x, y).raw();
                let hv0 = vframes[i];

                if x > 1 {
                    let h1 = vframes[i - 1];
                    let h2 = vframes[i - 2];

                    if hv0 != 99 && hv0 == h1 && hv0 == h2 {
                        let mut temp = vec![i, i - 1, i - 2];
                        list.append(&mut temp);
                    }
                }

                if y > 1 {
                    let v1 = vframes[i - GRID_WIDTH];
                    let v2 = vframes[i - GRID_WIDTH * 2];

                    if hv0 != 99 && hv0 == v1 && hv0 == v2 {
                        let mut temp = vec![i, i - GRID_WIDTH, i - GRID_WIDTH * 2];
                        list.append(&mut temp);
                    }
                }
            }
        }

        if list.len() != 0 {
            // clear duplicates and sort
            list.sort();
            list.dedup();
            let length = list.len();

            let end_time = (length * CLEAR_TIME as usize) as u32;

            let mut had_chainable = None;
            for (i, index) in list.iter().enumerate() {
                if let Some(block) = self.block_mut(*index) {
                    block
                        .state
                        .to_clear((i * CLEAR_TIME as usize) as u32, end_time);

                    if let Some(size) = block.was_chainable {
                        had_chainable = Some(size);
                    }
                }
            }

            // push chainable even if count was 3
            if let Some(size) = had_chainable {
                self.combo_highlight.push_chain(size as u32 + 1);
            }

            // always send combo info
            self.combo_highlight.push_combo(length as u32);
        }
    }

    /// clear the component if clear state is finished
    pub fn block_resolve_clear(&mut self) {
        while self.iter_xy() {
			if self.state().clear_finished() {
				let size = self.x_block().get_chain().unwrap_or(0);
				self.component_mut().to_chainable(size + 1);
				}
		}
    }

    /// block hang detection, if block state is idle and below is empty, set to hang   
    pub fn block_detect_hang(&mut self) {
		
		 for (_, _, i) in iter_xy() {
			let finished = self.block_state_check(i, |s| s.clear_finished());

            if finished {
                let size = self.block(i).unwrap().was_chainable.unwrap_or(0);
                self.components[i] = Component::Chainable(size + 1);
            }
        }
  }
  
	  /// loops upwards, checks if a block hang finished, sets all real above the block to fall, even garbage, garbage fall might fail in fall resolve
	  pub fn block_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
		  // block hang finish, set all above finished block to fall state
		  let mut above_fall = false;
		  // look for block and empty below
		  while self.iter_yx_rev() {
			  // TODO(Skytrias): check for if below empty again? since a few frames passed
			match self.component_mut().unwrap() {
				  Component::Block(b) => {
					  match b.state {
						  // any hang finished, set to fall and let other normal blocks above it fall too
						  BlockState::Hang { finished, .. } => {
							  if finished {
								  b.state.to_fall();
								  above_fall = true;
							  }
						  }
  
						  // fall too if below was hang finished
						  BlockState::Idle => {
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
				  Component::Child { .. } => {
					  if above_fall {
						  for g in garbage_system.list.iter_mut() {
							  if g.state.is_idle() {
								  if g.children.iter().any(|index| *index == self.i as usize) {
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
  
	  /// block fall execution, swap downwards if still empty below, set to idle otherwhise
	  pub fn block_resolve_fall(&mut self) {
		for (_, _, i) in iter_yx_rev() {
            if self.block_state_check(i, |s| s.is_fall()) {
                if let Some(ib) = (i + GRID_WIDTH).to_index() {
                    if self[ib].is_empty() {
                        let was_chainable = {
                            if let Component::Chainable(size) = self[ib] {
                                if let Component::Block(b) = &mut self[i] {
                                    b.was_chainable = Some(size);
                                }

                                true
                            } else {
                                false
                            }
                        };

                        if was_chainable {
                            self[ib] = Component::Empty;
                        }

                        self.components.swap(i, ib);
                    } else {
                        // reset blocks that were in fall and cant fall anymore
                        if let Some(state) = self.block_state_mut(i) {
                            state.to_land();
                        }
                    }
                }
            }
        }
    }

    /// block fall execution, swap downwards if still empty below, set to idle otherwhise
    pub fn block_resolve_land(&mut self) {
		for (_, _, i) in iter_yx_rev() {
            if self.block_state_check(i, |s| s.land_finished()) {
                if let Some(b) = self.block_mut(i) {
                    //b.can_chain = false;
                    b.state.to_idle();
                }
            }
        }
    }

    /// garbage hang detection, loop through garbages, look if idle and below are all empty, hang 0
    pub fn garbage_detect_hang(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if g.state.is_idle() {
                if g.lowest_empty(self) {
                    g.state.to_hang();
                } else {
                    // TODO(Skytrias): set to idle?
                }
            }
        }
    }

    /// garbage hang finish, loop through garbages, look if hang finished and set to fall
    pub fn garbage_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if g.state.hang_finished() {
                g.state.to_fall();
            }
        }
    }

    /// garbage fall, loop through garbages, if fall and below stil empty, swap components and increase index stored in .children
    pub fn garbage_resolve_fall(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if g.state.is_fall() {
                if g.lowest_empty(self) {
                    for index in g.children.iter_mut().rev() {
                        self.components.swap(*index, *index + GRID_WIDTH);
                        *index += GRID_WIDTH;
                    }
                } else {
                    g.state.to_idle();
                }
            }
        }
    }

    /// garbage detect clear on multiple blocks, dependant on 2d factor
    pub fn garbage_detect_clear(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut().rev() {
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
                                },
                            ]
                        }
                    };

                    neighbors.iter().any(|b| *b)
                });

                if clear_found {
                    let len = g.children.len() as usize;
                    let lowest = g.lowest();

                    for j in 0..len {
                        let child_index = g.children[j];

                        if let Some(child) = self.garbage_child_mut(child_index) {
                            // set clear hframe on each garbage child
                            if g.is_2d {
                                child.hframe = 9;
                            } else {
                                child.hframe = 3;
                            }

                            child.counter = 0;
                            child.finished = false;
                            child.start_time = j as u32 * CLEAR_TIME;
                            child.randomize_at_end =
                                lowest.iter().any(|&index| index == child_index);
                        }
                    }

                    g.state.to_clear((len as u32 + 1) * CLEAR_TIME);
                }
            }
        }
    }

    /// garbage clear resolve, checks for finished - sets state to idle - removes garbage from list if empty
    pub fn garbage_resolve_clear(&mut self, garbage_system: &mut GarbageSystem) {
        for (i, garbage) in garbage_system.list.iter_mut().enumerate() {
            if garbage.state.clear_finished() {
                garbage.state.to_idle();

                if garbage.children.is_empty() {
                    garbage_system.list.remove(i);
                    break;
                }
            }
        }
    }

    /// updates all components in the grid and the garbage system
    pub fn update(&mut self, garbage_system: &mut GarbageSystem) {
        debug_assert!(!self.components.is_empty());

        self.update_components();

        // NOTE(Skytrias): always do resolves before detects so there is 1 frame at minimum delay
        self.block_resolve_swap();
        self.block_detect_bottom();

        // resolve any lands
        self.block_resolve_land();

        // resolve any falls
        self.block_resolve_fall();
        self.garbage_resolve_fall(garbage_system);

        // resolve any hangs
        self.block_resolve_hang(garbage_system);
        self.garbage_resolve_hang(garbage_system);

        // detect any hangs
        self.block_detect_hang();
        self.garbage_detect_hang(garbage_system);

        // detect any clears
        self.block_resolve_clear();
        self.garbage_resolve_clear(garbage_system);

        // resolve any clear
        self.block_detect_clear();
        self.garbage_detect_clear(garbage_system);
    }

    pub fn push_update(
        &mut self,
        app: &mut App,
        garbage_system: &mut GarbageSystem,
        cursor: &mut Cursor,
    ) {
        if self.push_counter < PUSH_TIME {
            self.push_counter += 1;
        } else {
            self.push_amount += 1.;
            let amt = self.push_amount;
            self.push_counter = 0;

            if amt < ATLAS_TILE {
                for i in 0..GRID_TOTAL {
                    match &mut self[i] {
                        Component::Block(b) => b.offset.y = -amt,
                        Component::Child(g) => g.y_offset = -amt,
                        _ => {}
                    }
                }

                cursor.y_offset = -amt;
            } else {
                self.push_upwards(app, garbage_system, cursor);
                self.push_amount = 0.;
            }
        }
    }

    /// updates all non empty components in the grid
    pub fn update_components(&mut self) {
        for c in self.components.iter_mut() {
            c.update();
        }
    }

    /// draws all the grid components as sprite / quads
    pub fn draw(&mut self, app: &mut App) {
        self.combo_highlight.draw(app);

        // draw all grid components
        for (x, y, i) in iter_xy() {
            let position = V2::new(x as f32, y as f32) * ATLAS_SPACING;
            if let Some(sprite) = self[i].to_sprite(position) {
                app.push_sprite(sprite.into());
            }
        }
    }

    /// returns a block from the specified grid_index
    pub fn block<I: BoundIndex>(&self, index: I) -> Option<&Block> {
        match &self[index] {
            Component::Block(b) => Some(&b),
            _ => None,
        }
    }

    /// returns a block from the specified grid_index
    pub fn block_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut Block> {
        match &mut self[index] {
            Component::Block(b) => Some(b),
            _ => None,
        }
    }

    /// returns any state if the component is a block
    pub fn block_state<I: BoundIndex>(&self, index: I) -> Option<&BlockState> {
        match &self[index] {
            Component::Block(b) => Some(&b.state),
            _ => None,
        }
    }

    /// returns any state if the component is a block
    pub fn block_state_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut BlockState> {
        match &mut self[index] {
            Component::Block(b) => Some(&mut b.state),
            _ => None,
        }
    }

    /// returns any state if the component is a block
    pub fn garbage_child_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut Child> {
        match &mut self[index] {
            Component::Child(g) => Some(g),
            _ => None,
        }
    }

    /// experimental helpers

    /// closure on block if it exists
    pub fn block_state_check<I, P>(&mut self, index: I, predicate: P) -> bool
    where
        I: BoundIndex,
        P: FnOnce(&BlockState) -> bool,
    {
        match &self[index] {
            Component::Block(b) => predicate(&b.state),
            _ => false,
        }
    }

    /// helper for state asserting in tests
    pub fn assert_state<I, P>(&mut self, index: I, predicate: P)
    where
        I: BoundIndex,
        P: FnOnce(&BlockState) -> bool,
    {
        assert!(self.block_state_check(index, predicate))
    }
}

impl<I: BoundIndex> Index<I> for Grid {
    type Output = Component;

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

// TODO(Skytrias): shitty 

impl Index<i32> for Grid {
	type Output = Component;
	
	fn index(&self, index: i32) -> &Self::Output {
		&self.components[index as usize]
		}
}

impl IndexMut<i32> for Grid {
	fn index_mut(&mut self, index: i32) -> &mut Self::Output {
		&mut self.components[index as usize]
	}
}


/// state transition tests
#[cfg(test)]
mod tests {
    use super::*;

    /// showcase gen_1d working
    #[test]
    fn grid_gen_1d() {
        let mut grid = Grid::empty();
        let mut garbage_system = GarbageSystem::default();

        grid.gen_1d_garbage(&mut garbage_system, 3, 0);
        assert!(grid[0].is_garbage());
        assert!(grid[1].is_garbage());
        assert!(grid[2].is_garbage());
        assert!(grid[3].is_empty());

        grid.gen_1d_garbage(&mut garbage_system, 4, 0);
        assert!(grid[0].is_garbage());
        assert!(grid[1].is_garbage());
        assert!(grid[2].is_garbage());
        assert!(grid[3].is_garbage());
        assert!(grid[4].is_empty());

        grid = Grid::empty();
        grid.gen_1d_garbage(&mut garbage_system, 3, 1);
        assert!(grid[0].is_empty());
        assert!(grid[1].is_garbage());
        assert!(grid[2].is_garbage());
        assert!(grid[3].is_garbage());
        assert!(grid[4].is_empty());
    }

    /// check if hang to fall works in the wanted frame times
    #[test]
    fn block_hang_and_fall() {
        let mut grid = Grid::empty();
        let mut garbage_system = GarbageSystem::default();
        grid[0] = Component::Normal(Block::default());

        // hang state setting
        grid.assert_state(0, |s| s.is_idle());
        if let Some(state) = grid.block_state_mut(0) {
            state.to_hang();
        } else {
            assert!(false);
        }
        grid.assert_state(0, |s| s.is_hang());
        grid.assert_state(0, |s| s.hang_started());

        // hang state updating
        for i in 0..HANG_TIME {
            grid.update_components();

            grid.block_resolve_fall();
            grid.block_resolve_hang(&mut garbage_system);
        }

        // is in fall state now
        grid.assert_state(0, |s| s.is_fall());

        // check if fall succeeds to swap components around
        assert!(grid[0].is_block());
        assert!(grid[GRID_WIDTH].is_empty());
        grid.update_components();
        grid.block_resolve_fall();
        assert!(grid[0].is_empty());
        assert!(grid[GRID_WIDTH].is_block());
    }

    /// check if swap to idle works in the wanted frame times
    #[test]
    fn block_swap() {
        let mut grid = Grid::empty();
        let mut cursor = Cursor::default();
        cursor.position = V2::new(0., 0.);

        assert!(grid[0].is_empty());
        assert!(grid[1].is_empty());
        cursor.swap_blocks(&mut grid);
        assert!(grid[0].is_empty());
        assert!(grid[1].is_empty());

        grid[0] = Component::Normal(Block::default());

        assert!(grid[0].is_block());
        assert!(grid[1].is_empty());
        cursor.swap_blocks(&mut grid);
        assert!(grid[0].is_block());
        assert!(grid[1].is_empty());
        grid.assert_state(0, |s| s.is_swap());

        // swap state updating
        for i in 0..SWAP_TIME {
            grid.update_components();
        }

        grid.assert_state(0, |s| s.swap_finished());
        grid.update_components();

        // NOTE(Skytrias): matches the resolve / detect grid.update
        grid.block_resolve_swap();
        grid.block_detect_hang();

        assert!(grid[0].is_empty());
        assert!(grid[1].is_block());

        // block should transition to hang immediatly
        grid.assert_state(1, |s| s.is_hang());
    }
}
