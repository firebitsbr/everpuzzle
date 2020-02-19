use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use wgpu_glyph::Section;

/// horizontal is -x and +x, vertical is -y and +y
#[derive(Copy, Clone)]
enum FloodDirection {
    Horizontal,
    Vertical,
}

/// the grid holds all components and updates all the script logic of each component  
pub struct Grid {
    pub components: Vec<Component>,
    placeholder: Component,
combo_highlight: ComboHighlight,
	}

impl Default for Grid {
    fn default() -> Self {
        Self {
            components: Vec::with_capacity(GRID_TOTAL),
            placeholder: Component::Placeholder,
            combo_highlight: Default::default(),
        }
    }
}

#[derive(Debug)]
struct ClearData {
	vframe: u32,
	indexes: Vec<usize>,
}

impl Grid {
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
                    Component::Normal(Block::random(app))
                }
            })
            .collect();

        Self {
            components,
            ..Default::default()
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

        for index in children.iter() {
            self.components[*index].to_garbage();
        }

        garbage_system.list.push(Garbage::new(children));
    }

    /// generates a line of garbage at the top of the grid
    pub fn gen_2d_garbage(&mut self, garbage_system: &mut GarbageSystem, height: usize) {
        debug_assert!(height >= 1);

        let mut children = Vec::with_capacity(height * 6);

        for y in 0..height {
            for x in 0..6 {
                if let Some(index) = (x, y).to_index() {
                    children.push(index);
                    self.components[index].to_garbage();
                }
            }
        }

        garbage_system.list.push(Garbage::new(children));
    }

    /// swaps the 2 index components around if the block was in swap animation
    pub fn block_resolve_swap(&mut self) {
        for (_, _, i) in iter_xy() {
            if let Some(b) = self.block(i) {
                if let BlockState::Swap {
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

    /// sets the last row of blocks to bottom state
    pub fn block_detect_bottom(&mut self) {
		for (_, y, i) in iter_xy() {
		if let Some(state) = self.block_state_mut(i) {
				if y == GRID_HEIGHT - 1 {
					if state.is_swap() || state.is_clear() {
						continue;
					}
					
					state.to_bottom();
				} else {
					// NOTE(Skytrias): might not be needed anymore
					if state.is_bottom() {
						state.to_idle();
					}
				}
			}
        }
    }

	pub fn block_detect_clear(&mut self) {
		// NOTE(Skytrias): consider pushing to grid variables?
		let mut list = Vec::new();
		
		// get all vframes, otherwhise 99
		let vframes: Vec<u32> = (0..GRID_TOTAL)
			.map(|i| self.block(i).map(|b| {
												if b.state.is_idle() || b.state.land_started() {
												b.vframe
												} else {
													99
												}
											}).unwrap_or(99))
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
					block.state.to_clear((i * CLEAR_TIME as usize) as u32, end_time);
					
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
        for (_, _, i) in iter_xy() {
            let finished = self.block_state_check(i, |s| s.clear_finished());
			
            if finished {
				let size = self.block(i).unwrap().was_chainable.unwrap_or(0);
				self.components[i] = Component::Chainable(size + 1);
			}
        }
    }

    /// block hang detection, if block state is idle and below is empty, set to hang   
    pub fn block_detect_hang(&mut self) {
        for (_, _, i) in iter_xy() {
            if self.block_state_check(i, |s| s.is_idle()) {
                if let Some(ib) = (i + GRID_WIDTH).to_index() {
                    if self[ib].is_empty() {
                        if let Some(state) = self.block_state_mut(i) {
                            state.to_hang();
                        }
                    }
                }
            }
        }
    }

    /// loops upwards, checks if a block hang finished, sets all real above the block to fall, even garbage, garbage fall might fail in fall resolve
    pub fn block_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
        // block hang finish, set all above finished block to fall state
        let mut above_fall = false;
        // look for block and empty below
        for (_, _, i) in iter_yx_rev() {
            // TODO(Skytrias): check for if below empty again? since a few frames passed
            match &mut self[i] {
                Component::Normal(b) => {
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
                Component::GarbageChild { .. } => {
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

    /// block fall execution, swap downwards if still empty below, set to idle otherwhise
    pub fn block_resolve_fall(&mut self) {
        for (_, _, i) in iter_yx_rev() {
            if self.block_state_check(i, |s| s.is_fall()) {
                if let Some(ib) = (i + GRID_WIDTH).to_index() {
                    if self[ib].is_empty() {
                        let was_chainable = {
						if let Component::Chainable(size) = self[ib] {
							if let Component::Normal(b) = &mut self[i] {
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

                        self.components[child_index] = Component::GarbageChild(Child {
                            start_time: j as u32 * 20,
                            randomize_at_end: lowest.iter().any(|&index| index == child_index),
                            ..Default::default()
                        });
                    }

                    g.state.to_clear(len as u32);
                }
            }
        }
    }

    /// garbage clear resolve
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
                            self.components[*index] = Component::Normal(Block::random(app));
                        }
                    } else {
                        // convert all into random in 1d
                        for index in g.children.clone().iter() {
                            self.components[*index] = Component::Normal(Block::random(app));
                        }

                        remove_garbage = true;
                    }
                }
            }

            // garbage was empty and gets removed entirely
            if remove_garbage {
                garbage_system.list.remove(i);
                return;
            }
        }
    }

    /// updates all components in the grid and the garbage system
    pub fn update(&mut self, app: &mut App, garbage_system: &mut GarbageSystem) {
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
        self.garbage_resolve_clear(app, garbage_system);

        // resolve any clear
        self.block_detect_clear();
        self.garbage_detect_clear(garbage_system);
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
            Component::Normal(b) => Some(&b),
            _ => None,
        }
    }
	
    /// returns a block from the specified grid_index
    pub fn block_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut Block> {
        match &mut self[index] {
            Component::Normal(b) => Some(b),
            _ => None,
        }
    }
	
    /// returns any state if the component is a block
    pub fn block_state<I: BoundIndex>(&self, index: I) -> Option<&BlockState> {
        match &self[index] {
            Component::Normal(b) => Some(&b.state),
            _ => None,
        }
    }

    /// returns any state if the component is a block
    pub fn block_state_mut<I: BoundIndex>(&mut self, index: I) -> Option<&mut BlockState> {
        match &mut self[index] {
            Component::Normal(b) => Some(&mut b.state),
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
            Component::Normal(b) => predicate(&b.state),
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
