use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::ops::{Index, IndexMut};

/// the grid holds all components and updates all the script logic of each component  
pub struct Grid {
    /// all components that the player can interact with
    pub components: Vec<Component>,

    /// rendering highlight for combo / chain appearing
    combo_highlight: ComboHighlight,

    /// counter till the push_amount is increased
    pub push_counter: u32,
	
    /// pixel amount of y offset of all pushable structs
    pub push_amount: f32,
	
	/// manual input sent, till a push_upwards has been called
	pub push_raise: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            components: Vec::with_capacity(GRID_TOTAL),
            combo_highlight: Default::default(),

            push_counter: 0,
            push_amount: 0.,
			push_raise: false,
		}
    }
}

impl Grid {
    /// creates empty grid for testing
    pub fn empty() -> Self {
        let components: Vec<Component> = (0..GRID_TOTAL)
            .map(|_| Component::Empty {
                size: 0,
                alive: false,
            })
            .collect();

        Self {
            components,
            ..Default::default()
        }
    }

    /// inits the grid with randomized blocks (seeded)
    pub fn new(app: &mut App) -> Self {
        let components: Vec<Component> = (0..GRID_TOTAL)
            .map(|i| {
					 if i >= GRID_WIDTH * 3 {
                    Component::Block {
                        block: Block::random(app),
                        state: BlockState::Idle,
						 }
					 } else {
						 Component::Empty {
							 size: 0,
							 alive: false,
						 }
					 }
            })
            .collect();

        Self {
            components,
            ..Default::default()
        }
    }
	
	/// sets all blocks and childs y_offset to 0, swaps them with below and sets bottom row to randoimized blocks
    pub fn push_upwards(
        &mut self,
        app: &mut App,
        garbage_system: &mut GarbageSystem,
        cursor: &mut Cursor,
							reset: bool,
							) {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let index = y * GRID_WIDTH + x;
				
                if y < GRID_HEIGHT - 1 {
                    match &mut self[index + GRID_WIDTH] {
                        Component::Block { block, .. } => block.offset.y = 0.,
                        Component::Child(g) => g.y_offset = 0.,
                        _ => {}
                    }

                    self.components.swap(index, index + GRID_WIDTH);
                } else {
                    self.components[index] = Component::Block {
                        block: Block {
                            vframe: Block::random_vframe(app),
                            offset: V2::new(0., -self.push_amount),
                            ..Default::default()
                        },
                        state: BlockState::Idle,
                    };
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
            let (hframe, vframe) = Child::gen_1d_frames(i, width);
            self.components[*index] = Component::Child(Child {
                hframe,
                vframe,
                ..Default::default()
            });
        }

        garbage_system.list.push(Garbage::new(children));
    }

    /// generates a line of garbage at the top of the grid
    pub fn gen_2d_garbage(&mut self, garbage_system: &mut GarbageSystem, height: usize) {
        debug_assert!(height >= 1);

        let mut children = Vec::with_capacity(height * 6);

        for y in 0..height {
            for x in 0..GRID_WIDTH {
                let i = y * GRID_WIDTH + x;

                children.push(i);
                let (hframe, vframe) = Child::gen_2d_frames(x, y, height);

                self.components[i] = Component::Child(Child {
                    hframe,
                    vframe,
                    ..Default::default()
                });
            }
        }

        garbage_system.list.push(Garbage::new(children));
    }

    /// swaps the 2 index components around if the block was in swap animation
    pub fn block_resolve_swap(&mut self) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let i = y * GRID_WIDTH + x;

                let mut any = None;

                if let Component::Block { state, block } = &mut self[i] {
                    if let BlockState::Swap { counter, direction } = state {
                        if *counter >= SWAP_TIME - 1 {
                            block.offset.x = 0.;
                            let next = (i as i32 + *direction) as usize;
                            *state = BlockState::Idle;
                            self.components.swap(i, next);
                            any = Some(next);
                        }
                    }
                }

                if let Some(index) = any {
                    if let Component::Block { state, block } = &mut self[i] {
                        *state = BlockState::Idle;
                        block.offset.x = 0.;
                    }
                }
            }
        }
    }
	
	/// detects wether any clear is happening, turns blocks to clear state if 3 or more blocks match vframes
    pub fn block_detect_clear(&mut self) {
        // NOTE(Skytrias): consider pushing to grid variables?
        let mut list = Vec::new();

        // get all vframes, otherwhise 99
        let vframes: Vec<u32> = (0..GRID_TOTAL)
            .map(|i| {
                if let Component::Block { block, state } = &self[i] {
                    match *state {
                        BlockState::Idle => return block.vframe,

                        BlockState::Land { counter } => {
								 if counter == 1 {
									 return block.vframe;
                            }
                        }

                        _ => return 99,
                    }
                }

                99
            })
            .collect();

        // loop through vframes and match horizontal or vertical matches, append them to list
        for x in 0..GRID_WIDTH {
            for y in 0..(GRID_HEIGHT - 1) {
                let i = y * GRID_WIDTH + x;
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
                if let Component::Block { block, state } = &mut self[*index] {
                    // TODO(Skytrias): check if idle?

                    *state = BlockState::Clear {
                        counter: 0,
                        start_time: i as u32 * CLEAR_TIME as u32,
                        end_time,
                    };
					
					if let Some(new_size) = block.saved_chain {
					if let Some(existing_size) = had_chainable {
							had_chainable = Some(new_size.max(existing_size));
						} else {
							had_chainable = Some(new_size);
						}
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
        for i in 0..GRID_TOTAL {
            if let Component::Block { state, block } = &self[i] {
                if let BlockState::Clear {
                    counter, end_time, ..
                } = state
                {
                    if *counter >= end_time - 1 {
                        self[i] = Component::Empty {
                            size: block.saved_chain.unwrap_or(0) + 1,
                            alive: true,
                        };
                    }
                }
            }
        }
    }

    /// block hang detection, if block state is idle and below is empty, set to hang   
    pub fn block_detect_hang(&mut self) {
        for i in 0..GRID_TOTAL - GRID_WIDTH {
            if let Component::Empty { .. } = &self[i + GRID_WIDTH] {
                if let Component::Block { state, block } = &mut self[i] {
                    if let BlockState::Idle = state {
                        *state = BlockState::Hang { counter: 0 };
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
        for x in (0..GRID_WIDTH).rev() {
            for y in (0..GRID_HEIGHT - 1).rev() {
                let i = y * GRID_WIDTH + x;

                // TODO(Skytrias): check for if below empty again? since a few frames passed
                match &mut self[i] {
                    Component::Block { state, .. } => {
                        match state {
                            // any hang finished, set to fall and let other normal blocks above it fall too
                            BlockState::Hang { counter } => {
                                if *counter >= HANG_TIME - 1 {
                                    *state = BlockState::Fall;
                                    above_fall = true;
                                }
                            }

                            // fall too if below was hang finished
                            BlockState::Idle => {
                                if above_fall {
                                    *state = BlockState::Fall;
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
                    Component::Child(_) => {
                        if above_fall {
                            for g in garbage_system.list.iter_mut() {
                                if let GarbageState::Idle = g.state {
                                    if g.children.iter().any(|index| *index == i) {
                                        g.state = GarbageState::Fall;
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
    }

    /// block fall execution, swap downwards if still empty below, set to idle otherwhise
    pub fn block_resolve_fall(&mut self) {
        for x in (0..GRID_WIDTH).rev() {
            for y in (0..GRID_HEIGHT - 1).rev() {
                let i = y * GRID_WIDTH + x;

                if let Component::Block { state, block } = &self[i] {
                    if let BlockState::Fall = state {
                        let mut saved_chain = None;
                            let was_saved = block.saved_chain.is_some();
						
						if let Component::Empty { alive, size } = &mut self[i + GRID_WIDTH] {
							if was_saved {
							*alive = false;
							}
							
							saved_chain = Some(*size);
							*size = 1;
							
							self.components.swap(i, i + GRID_WIDTH);
                        } else {
                            // reset blocks that were in fall and cant fall anymore
                            if let Component::Block { state, .. } = &mut self[i] {
                                *state = BlockState::Land { counter: 0 };
                            }
                        }
						
						if let Some(_) = saved_chain {
							if let Component::Block { block, .. } = &mut self[i + GRID_WIDTH] {
								block.saved_chain = saved_chain;
							}
						}
                    }
                }
            }
        }
    }

    /// block fall execution, swap downwards if still empty below, set to idle otherwhise
    pub fn block_resolve_land(&mut self) {
        for x in (0..GRID_WIDTH).rev() {
            for y in (0..GRID_HEIGHT - 1).rev() {
                let i = y * GRID_WIDTH + x;

                if let Component::Block { state, .. } = &mut self[i] {
                    if let BlockState::Land { counter } = state {
                        if *counter >= LAND_TIME - 1 {
                            *state = BlockState::Idle;
                        }
                    }
                }
            }
        }
    }

    /// garbage hang detection, loop through garbages, look if idle and below are all empty, hang 0
    pub fn garbage_detect_hang(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if let GarbageState::Idle = g.state {
                if g.lowest_empty(self) {
                    g.state = GarbageState::Hang { counter: 0 };
                } else {

                    // TODO(Skytrias): set to idle?
                }
            }
        }
    }

    /// garbage hang finish, loop through garbages, look if hang finished and set to fall
    pub fn garbage_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if let GarbageState::Hang { counter } = g.state {
                if counter >= HANG_TIME - 1 {
                    g.state = GarbageState::Fall;
                }
            }
        }
    }

    /// garbage fall, loop through garbages, if fall and below stil empty, swap components and increase index stored in .children
    pub fn garbage_resolve_fall(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if let GarbageState::Fall = g.state {
                if g.lowest_empty(self) {
                    for index in g.children.iter_mut().rev() {
                        self.components.swap(*index, *index + GRID_WIDTH);
                        *index += GRID_WIDTH;
                    }
                } else {
                    g.state = GarbageState::Idle;
                }
            }
        }
    }

    fn any_clears_started(&self, indexes: &[i32]) -> Vec<bool> {
        let mut results = Vec::with_capacity(indexes.len());

        for &index in indexes {
            let mut result = false;

            if index >= 0 && index < GRID_TOTAL as i32 {
                if let Component::Block { state, .. } = &self[index as usize] {
                    if let BlockState::Clear { counter, .. } = state {
                        if *counter == 0 {
                            result = true;
                        }
                    }
                }
            } else {
                result = false;
            }

            results.push(result);
        }

        results
    }

    /// garbage detect clear on multiple blocks, dependant on 2d factor
    pub fn garbage_detect_clear(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut().rev() {
            if let GarbageState::Idle = g.state {
                let clear_found = g.children.iter().any(|&i| {
                    // TODO(Skytrias): better way to avoid 0 - 1 on usize
                    let neighbors = {
                        // TODO(Skytrias): REFACTOR
                        if g.is_2d {
                            // above, below
                            self.any_clears_started(&[
                                i as i32 + GRID_WIDTH as i32,
                                i as i32 - GRID_WIDTH as i32,
                            ])
                        } else {
                            // above, below, right, left
                            self.any_clears_started(&[
                                i as i32 + GRID_WIDTH as i32,
                                i as i32 - GRID_WIDTH as i32,
                                i as i32 + 1 as i32,
                                i as i32 - 1 as i32,
                            ])
                        }
                    };

                    neighbors.iter().any(|b| *b)
                });

                if clear_found {
                    let len = g.children.len() as usize;
                    let lowest = g.lowest();

                    for j in 0..len {
                        let child_index = g.children[j];

                        if let Component::Child(child) = &mut self[child_index] {
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

                    g.state = GarbageState::Clear {
                        counter: 0,
                        end_time: (len as u32 + 1) * CLEAR_TIME,
                        finished: false,
                    };
                }
            }
        }
    }

    /// garbage clear resolve, checks for finished - sets state to idle - removes garbage from list if empty
    pub fn garbage_resolve_clear(&mut self, garbage_system: &mut GarbageSystem) {
        for (i, garbage) in garbage_system.list.iter_mut().enumerate() {
            if let GarbageState::Clear { finished, .. } = garbage.state {
                if finished {
                    garbage.state = GarbageState::Idle;

                    if garbage.children.is_empty() {
                        garbage_system.list.remove(i);
                        break;
                    }
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
	
	/// updates the push / raise data which offsets the grid components
    pub fn push_update(
        &mut self,
        app: &mut App,
        garbage_system: &mut GarbageSystem,
        cursor: &mut Cursor,
    ) {
        
		if self.push_counter < PUSH_TIME && !self.push_raise {
            self.push_counter += 1;
        } else {
            self.push_amount += 1.;
            let amt = self.push_amount;
            self.push_counter = 0;

            if amt < ATLAS_TILE {
                for i in 0..GRID_TOTAL {
                    match &mut self[i] {
                        Component::Block { block, .. } => block.offset.y = -amt,
                        Component::Child(g) => g.y_offset = -amt,
                        _ => {}
                    }
                }

                cursor.y_offset = -amt;
            } else {
				self.push_upwards(app, garbage_system, cursor, false);
                self.push_raise = false;
                self.push_amount = 0.;
            }
        }
    }

    /// updates all non empty components in the grid
    pub fn update_components(&mut self) {
        for component in self.components.iter_mut() {
            match component {
                Component::Block { block, state } => block.update(state),
                _ => {}
            }
        }
    }

    /// draws all the grid components as sprite / quads
    pub fn draw(&mut self, app: &mut App) {
        self.combo_highlight.draw(app);

        // draw all grid components
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let i = y * GRID_WIDTH + x;

                // set bottom row to darkened
                if y == GRID_HEIGHT - 1 {
                    if let Component::Block { block, .. } = &mut self[i] {
                        block.hframe = 2;
                    }
                }

                let position = V2::new(x as f32, y as f32) * ATLAS_SPACING;
                if let Some(sprite) = self[i].to_sprite(position) {
                    app.push_sprite(sprite.into());
                }
            }
        }
    }
}

impl Index<usize> for Grid {
    type Output = Component;

    fn index(&self, index: usize) -> &Self::Output {
        &self.components[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
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
