use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;
use std::collections::HashMap;
use std::ops::{Index, IndexMut, Range};

/// the grid holds all components and updates all the script logic of each component  
pub struct Grid {
    pub id: usize,

    /// all components that the player can interact with
    pub components: Vec<Component>,

    /// rendering highlight for combo / chain appearing
    pub combo_highlight: ComboHighlight,

    /// counter till the push_amount is increased
    pub push_counter: u32,

    /// pixel amount of y offset of all pushable structs
    pub push_amount: f32,

    /// manual input sent, till a push_upwards has been called
    pub push_raise: bool,

    /// cursor that the player controls inside the grid
    pub cursor: Cursor,

    /// random number generator, each grid will have its own generator, which will all use the same seed
    pub rng: oorandom::Rand32,
}

impl Grid {
    /// generates a randomized new line of vframes for blocks
    /// respects the current bottom row of blocks to not generate the same as the above ones
    pub fn gen_line(&mut self) -> [u32; 6] {
        let mut vframes = [0; 6];

        for (i, index) in (GRID_TOTAL - GRID_WIDTH..GRID_TOTAL).enumerate() {
            if let Component::Block { block, .. } = &self[index] {
                let mut new_num;
                let vframe = block.vframe;

                loop {
                    new_num = self.rng.rand_range(3..8);

                    // dont allow new to be the same as above
                    if new_num != vframe {
                        // if history is longer than 2, check not to create a 3x1 line
                        if i > 1 {
                            // if 2 previous were the same and it matches regenrate
                            if vframes[i - 1] == vframes[i - 2] {
                                if vframes[i - 1] != new_num {
                                    break;
                                }
                            } else {
                                break;
                            }
                        } else {
                            // simply exit
                            break;
                        }
                    }
                }

                vframes[i] = new_num;
            } else {
                // simply generate new number if block doesnt exist
                vframes[i] = self.rng.rand_range(3..8);
            }
        }

        vframes
    }

    /// non grid dependant way to generate a new field of vframes
    pub fn gen_field(rng: &mut oorandom::Rand32, skip_height: usize) -> [Option<u32>; GRID_TOTAL] {
        let mut vframes = [None; GRID_TOTAL];

        let mut last = None;
        let mut num = None;

        for i in 0..GRID_TOTAL {
            if i >= skip_height * GRID_WIDTH {
                loop {
                    num = Some(rng.rand_range(3..8));

                    // skip rand gen if last doesnt equal new
                    if num != last {
                        // only when i is below GRID_WIDTH, check for above number too
                        if i > GRID_WIDTH {
                            if let Some(above) = vframes[i - GRID_WIDTH] {
                                if above != num.unwrap() {
                                    break;
                                }
                            } else {
                                if rng.rand_range(0..5) == 1 {
                                    num = None;
                                }

                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }

                vframes[i] = num;
            } else {
                vframes[i] = None;
            }

            last = num;
        }

        vframes
    }

    /// inits the grid with randomized blocks (seeded)
    pub fn new(id: usize, seed: u64, vframes: &[Option<u32>; GRID_TOTAL]) -> Self {
        let components: Vec<Component> = (0..GRID_TOTAL)
            .map(|i| Component::spawn(vframes[i]))
            .collect();

        Self {
            id,

            components,
            combo_highlight: Default::default(),

            push_counter: 0,
            push_amount: 0.,
            push_raise: false,

            cursor: Cursor::new(if id == 1 { true } else { false }),
            rng: oorandom::Rand32::new(seed),
        }
    }

    /// resets the grid and sets it to a new randomized field
    pub fn reset(&mut self) {
        let vframes = Grid::gen_field(&mut self.rng, 5);

        for i in 0..GRID_TOTAL {
            self[i] = Component::spawn(vframes[i]);
        }

        self.combo_highlight.clear();
        self.push_raise = false;
        self.push_counter = 0;
        self.cursor.reset();
    }

    /// sets all blocks and childs y_offset to 0, swaps them with below and sets bottom row to randoimized blocks
    pub fn push_upwards(&mut self, garbage_system: &mut GarbageSystem, reset: bool) {
        let vframes = self.gen_line();

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
                            vframe: vframes[x],
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
            if garbage.parent_id == self.id {
				for child_index in garbage.children.iter_mut() {
                *child_index -= GRID_WIDTH;
					}
			}
        }

        // shift up the cursor if still in grid range
        if self.cursor.position.y > 0 {
            self.cursor.position.y -= 1;
            self.cursor.last_position.y -= 1;
            self.cursor.goal_position.y -= 1. * ATLAS_TILE;
            self.cursor.y_offset = 0.;
        }
    }

    /// generates a line of garbage at the top of the grid
    pub fn gen_1d_garbage(
        &mut self,
							  garbage_system: &mut GarbageSystem,
        width: usize,
    ) {
		let offset = self.rng.rand_range(0..(GRID_WIDTH - width + 1) as u32) as usize;
        
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

        garbage_system.list.push(Garbage::new(self.id, children));
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

        garbage_system.list.push(Garbage::new(self.id, children));
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

            // only send combo info if larger than 3
            if length > 3 {
                self.combo_highlight.push_combo(length as u32);
            }
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
                                if g.parent_id == self.id {
									if let GarbageState::Idle = g.state {
                                    if g.children.iter().any(|index| *index == i) {
                                        g.state = GarbageState::Fall;
                                    }
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
            if g.parent_id == self.id {
				if let GarbageState::Idle = g.state {
                if g.lowest_empty(self) {
                    g.state = GarbageState::Hang { counter: 0 };
                } else {

                    // TODO(Skytrias): set to idle?
                }
                }
            }
        }
    }

    /// garbage hang finish, loop through garbages, look if hang finished and set to fall
    pub fn garbage_resolve_hang(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
            if g.parent_id == self.id {
				if let GarbageState::Hang { counter } = g.state {
                if counter >= HANG_TIME - 1 {
                    g.state = GarbageState::Fall;
                }
            }
            }
        }
    }

    /// garbage fall, loop through garbages, if fall and below stil empty, swap components and increase index stored in .children
    pub fn garbage_resolve_fall(&mut self, garbage_system: &mut GarbageSystem) {
        for g in garbage_system.list.iter_mut() {
			if g.parent_id == self.id {
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
    }
	
    /// returns true if if a clear has started in the index
    fn any_clears_started(&self) -> bool {
		for i in 0..GRID_TOTAL {
                if let Component::Block { state, .. } = &self[i] {
                    if let BlockState::Clear { counter, .. } = state {
                        if *counter == 0 {
                            return true;
						}
                    }
            }
        }
		
        false
    }
	
    /// returns true if if a clear has started in the index
    fn clears_started(&self, indexes: &[i32]) -> Vec<bool> {
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
			if g.parent_id == self.id {
            if let GarbageState::Idle = g.state {
                let clear_found = g.children.iter().any(|&i| {
                    // TODO(Skytrias): better way to avoid 0 - 1 on usize
                    let neighbors = {
                        // TODO(Skytrias): REFACTOR
                        if g.is_2d {
                            // above, below
                            self.clears_started(&[
                                i as i32 + GRID_WIDTH as i32,
                                i as i32 - GRID_WIDTH as i32,
                            ])
                        } else {
                            // above, below, right, left
                            self.clears_started(&[
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
								child.start_time = (len - 1 - j) as u32 * CLEAR_TIME;
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
    }

    /// garbage clear resolve, checks for finished - sets state to idle - removes garbage from list if empty
    pub fn garbage_resolve_clear(&mut self, garbage_system: &mut GarbageSystem) {
        for (i, garbage) in garbage_system.list.iter_mut().enumerate() {
            if garbage.parent_id == self.id {
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
    }

    pub fn solve_format(&mut self, format: &Vec<Vec<u32>>) {
        if let Some(state) = self.cursor.states.get(0) {
            match state {
                CursorState::Idle => {}
                _ => return,
            }
        }

        let width = format[0].len();
        let height = format.len();

        // TODO(Skytrias): panic if no 2 or 3

        // get how many 1s or 2s there exist
        let mut goal_amount = 0;
        for x in 0..width {
            for y in 0..height {
                let num = format[y][x];
                if num == 1 || num == 2 {
                    goal_amount += 1;
                }
            }
        }

        for &vframe in &[3, 4, 5, 6, 7] {
            // move through arrays and search for pattern
            for x in 0..GRID_WIDTH - (width - 1) {
                for y in 0..GRID_HEIGHT - (height - 1) - 1 {
                    let mut goal = 0;
                    let mut goal_index = 0;

                    for y_off in 0..height {
                        for x_off in 0..width {
                            let num = format[y_off][x_off];

                            let i = (y + y_off) * GRID_WIDTH + (x + x_off);

                            if num == 2 || num == 3 {
                                goal_index = i;
                            }

                            if let Component::Block { block, state } = &self[i] {
                                if let BlockState::Idle = state {
                                    if num == 1 || num == 2 {
                                        if block.vframe == vframe {
                                            goal += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if goal >= goal_amount {
                        self.cursor.states.push_back(CursorState::MoveSwap {
                            counter: 0,
                            goal: goal_index.to_i2(),
                        });
                    }
                }
            }
        }
    }
	
    pub fn solve_vertically(&mut self, goal_amount: usize, x_start: usize, x_end: usize, y_start: usize, y_end: usize) {
        debug_assert!(goal_amount != 0);

        // TODO(Skytrias): reevaluate smallest distance so that states might be reset in favor

        let mut smallest_distance: Option<i32> = None;
        let mut searched_indexes = Vec::new();
		
        for y in y_start..y_end - goal_amount {
            'skip: for x in x_start..x_end {
                // TODO(Skytrias): modify if more / less vframes exist
				for &vframe in &[3, 4, 5, 6, 7] {
                    // TODO(Skytrias): not great
					let mut indexes = Vec::new();
                    let mut goal_counter = 0;
                    let mut sum_distance = 0;
					
                    for y_off in 0..goal_amount {
                        'inner: for x_off in 0..GRID_WIDTH {
                            let i = (y + y_off) * GRID_WIDTH + x;
                            let j = (y + y_off) * GRID_WIDTH + x_off;

                            if let Component::Empty { .. } = &self[i] {
                                break 'skip;
                            }
							
                            if let Component::Block { block, state } = &self[j] {
								if let BlockState::Idle = state {
                                    if block.vframe == vframe {
                                        goal_counter += 1;
                                        let distance = x as i32 - x_off as i32;
                                        sum_distance += distance.abs();

                                        if i != j {
                                            indexes.push((i, distance));
                                        }
										
                                        break 'inner;
                                    }
                                }
                            }
                        }
                    }

                    if goal_counter == goal_amount {
                        if let Some(distance) = smallest_distance.as_mut() {
                            if *distance > sum_distance {
                                *distance = sum_distance;
                                searched_indexes = indexes;
                            }
                        } else {
                            smallest_distance = Some(sum_distance);
                            searched_indexes = indexes;
                        }
                    }
                }
            }
        }

        if let Some(distance) = smallest_distance {
            for (start_index, distance) in searched_indexes {
				// move to index, dependant on the direction move further
                let goal = if distance > 0 {
                    start_index as i32 - distance
                } else {
                    start_index as i32 - (distance + 1)
                };

                self.cursor.states.push_back(CursorState::MoveTransport {
                    counter: 0,
													 reached: false,
													 swap_end: true,
                    start: start_index.to_i2(),
                    goal: goal.to_i2(),
                });
            }
        }
    }
	
	// checks wether an x path has any empty or clearing block
	pub fn path_has_empty(&self, x_start: usize, x_end: usize, y_axis: usize) -> bool {
		let min = x_start.min(x_end);
		let max = x_start.max(x_end);
		
		for x in min..max {
			let i = y_axis * GRID_WIDTH + x;
			
			if let Component::Empty { .. } = &self[i] {
				if i < GRID_TOTAL - GRID_WIDTH {
			if let Component::Empty { .. } = &self[i + GRID_WIDTH] {
						return true;
				}
				}
			}
			
			if let Component::Block { state, block } = &self[i] {
				match state {
					BlockState::Clear { .. } => return true,
					_ => {}
				}
				}
		}
		
		false
	}
	
    pub fn solve_spawn_vertically(&mut self, y_start: usize) {
        // TODO(Skytrias): reevaluate smallest distance so that states might be reset in favor
		
		// only allow 3x1 pairs
		'search: for x in 0..GRID_WIDTH {
                let i = y_start * GRID_WIDTH + x;
			let mut indexes = Vec::new();
			let mut goal_counter = 0;
			
			if let Component::Block { block, state } = &self[i] {
		if let BlockState::Spawned = state {
								// search for others nearby on the next y positions below
					for y_off in 1..3 {
						// if below is already the same, skip
						if let Component::Block { block: below_block, .. } = &self[(y_start + y_off) * GRID_WIDTH + x] {
							if below_block.vframe == block.vframe {
								continue;
							}
						}
						
						'next: for x_off in 0..GRID_WIDTH {
										let j = (y_start + y_off) * GRID_WIDTH + x_off; 
							
							if let Component::Block { block: below_block, state: below_state } = &self[j] {
											if self.path_has_empty(x_off, x, y_start + y_off) {
												continue;
											}
											
											if below_block.vframe == block.vframe {
												goal_counter += 1;
												indexes.push(((y_start + y_off) * GRID_WIDTH + x, j));
												
												if goal_counter > 1 {
													for (start, end) in &indexes {
														let goal = {
															let pos = end.to_i2();
															
															if pos.x < self.cursor.position.x {
																pos
															} else {
																I2::new(pos.x, pos.y)
															}
														};
														
														self.cursor.states.push_back(CursorState::MoveTransport {
																						 counter: 0,
																						 reached: false,
																						 swap_end: false,
																						 start: start.to_i2(),
																						 goal,
																					 });
													}
													
													break 'search;
												}
												
												break 'next;
											}
									
								}
							}
							}
				}
			} 
        }
		}
	
    pub fn solve_horizontally(&mut self, goal_amount: usize, y_start: usize, y_end: usize) {
        debug_assert!(goal_amount != 0);
		
        // TODO(Skytrias): reevaluate smallest distance so that states might be reset in favor
		
        let mut smallest_distance: Option<i32> = None;
        let mut searched_indexes = Vec::new();
		
		for &vframe in &[3, 4, 5, 6, 7] {
		for y in y_start..y_end {
			for x in 0..GRID_WIDTH - goal_amount {
				// TODO(Skytrias): skip if x axis doesnt contain goal_amount of vframe
				
					
					// TODO(Skytrias): not great
					let mut indexes = Vec::new();
                    let mut goal_counter = 0;
                    let mut sum_distance = 0;
					
					for x_off in 0..GRID_WIDTH {
					let i = y * GRID_WIDTH + x;
						let j = y * GRID_WIDTH + x_off;
						
						// TODO(Skytrias): skip if somewhere below has a empty
						// TODO(Skytrias): skip if x axis has clear?
						
						if let Component::Block { block, state } = &self[j] {
							if let BlockState::Idle = state {
								if block.vframe == vframe {
									goal_counter += 1;
									let distance = x as i32 - x_off as i32;
									sum_distance += distance.abs();
									
									// skip if i is the same as j, the start of the search
									if i != j {
										indexes.push((i, distance));
									}
								}
							}
						}
					}
					
					// if goal reached, check if smaller
                    if goal_counter == goal_amount {
                        if let Some(distance) = smallest_distance.as_mut() {
                            if *distance > sum_distance {
                                *distance = sum_distance;
                                searched_indexes = indexes;
                            }
                        } else {
                            smallest_distance = Some(sum_distance);
                            searched_indexes = indexes;
                        }
                    }
				}
			}
		}
			
        if let Some(distance) = smallest_distance {
            for (start_index, distance) in searched_indexes {
				// move to index, dependant on the direction move further
                let goal = if distance > 0 {
                    start_index as i32 - distance
                } else {
                    start_index as i32 - (distance + 1)
                };
				
                self.cursor.states.push_back(CursorState::MoveTransport {
												 counter: 0,
													 reached: false,
													 swap_end: false,
												 start: start_index.to_i2(),
												 goal: goal.to_i2(),
											 });
            }
        }
    }
	
	// TODO(Skytrias): TAKE AVERAGE 
	/// detects the highest and lowest block y peak and returns true if the difference equals the height_difference 
	pub fn detect_difference(&self, height_difference: i32) -> Option<usize> {
		let mut y_min = GRID_HEIGHT;
		let mut y_max = 0;
		let mut x_axis = 0;
		
		for x in 0..GRID_WIDTH {
				for y in 0..GRID_HEIGHT {
				let i = y * GRID_WIDTH + x;
				
				if let Component::Block { .. } = &self[i] {
					y_max = y_max.max(y);
					
					if y_min > y {
						y_min = y;
						x_axis = x;
					}
					
					break;
				}
			}
		}
		
		if (y_min as i32 - y_max as i32).abs() > height_difference {
			Some(x_axis)
		} else {
			None
		}
	}
	
	pub fn remove_peaks(&mut self, x_axis: usize) {
        for y in 0..GRID_HEIGHT - 1 {
			let i = y * GRID_WIDTH + x_axis;
			
			// TODO(Skytrias): move based on cursor pos
			if let Component::Block { .. } = &mut self[i] {
                let goal = if self.cursor.position.x <= x_axis as i32 {
					i
				} else {
					i
				}.to_i2();
				
				self.cursor.states.push_back(CursorState::MoveSwap {
												 counter: 0,
													 goal,
												 });
				return;
			}
		}
	}
	
	fn nearest_hole(&self, y_axis: usize) -> Option<usize> {
		// TODO(Skytrias): if y == GRID_HEIGHT - 1
		
		for x in 0..GRID_WIDTH {
			let i = y_axis * GRID_WIDTH + x;
			
			if let Component::Empty { .. } = &self[i] {
				if i < GRID_TOTAL - GRID_WIDTH {
					if let Component::Empty { .. } = &self[i + GRID_WIDTH] {
								return Some(i);
					}
				}
			}
		}
		
		None
	}
	
    /// updates all components in the grid and the garbage system
    pub fn update(&mut self, input: &Input, garbage_system: &mut GarbageSystem) {
        debug_assert!(!self.components.is_empty());

        if false {
            // 3ds solvers
            let format = vec![vec![1, 0], vec![1, 0], vec![3, 1]];
            self.solve_format(&format);
            let format = vec![vec![0, 1], vec![0, 1], vec![2, 0]];
            self.solve_format(&format);
            let format = vec![vec![3, 1], vec![1, 0], vec![1, 0]];
            self.solve_format(&format);
            let format = vec![vec![2, 0], vec![0, 1], vec![0, 1]];
            self.solve_format(&format);

            // prioritize 4ths
            let format = vec![vec![0, 1], vec![2, 0], vec![0, 1], vec![0, 1]];
            self.solve_format(&format);
            let format = vec![vec![0, 1], vec![0, 1], vec![2, 0], vec![0, 1]];
            self.solve_format(&format);
            let format = vec![vec![1, 0], vec![3, 1], vec![1, 0], vec![1, 0]];
            self.solve_format(&format);
            let format = vec![vec![1, 0], vec![1, 0], vec![3, 1], vec![1, 0]];
            self.solve_format(&format);

            // generic 3rds
            let format = vec![vec![0, 1], vec![2, 0], vec![0, 1]];
            self.solve_format(&format);
            let format = vec![vec![1, 0], vec![3, 1], vec![1, 0]];
            self.solve_format(&format);

            /*
              // very generic
              let format = vec![
                            vec![0, 1, 0],
                            vec![0, 3, 1],
                            vec![1, 0, 0],
                            ];
              self.solve_format(&format);
            */
        }
		
        self.cursor.update(input, &mut self.components);
		
		// ai update, priority dependant
        if self.id == 1 && !(self.cursor.states.get(0).is_some() || self.cursor.end_delay != 0) {
			// if total block amount is lower than 3 lines of blocks, raise once
			let amt = self.components
				.iter()
				.filter(|c| if let Component::Block { .. } = c { true } else { false })
				.count();
			if amt <= GRID_WIDTH * 4 {
				self.push_raise = true;
			}
			
			// if top has big peaks, solve them by trying vertical solves
			if let Some(x) = self.detect_difference(5) {
				self.remove_peaks(x);
				let x = (x as i32 - 1).max(0) as usize;
				self.solve_vertically(3, x, x + 1, 0, GRID_HEIGHT);
			}
			
			if let Some(y) = garbage_system.lowest_clear(self) {
				// prefer spawn chain preperation
				self.solve_spawn_vertically(y + 1);
			}else if let Some(y) = garbage_system.lowest_idle(self) {
			// solve garbage vertically by 3 if possible, else remove peaks,
				let max = y + 2;
				
				// NOTE(Skytrias): hardcoded offsets
				self.solve_horizontally(3, y + 1, y + 2);
				self.solve_vertically(3, 0, GRID_WIDTH, y, max + 3);
				
				if let Some(start_index) = self.nearest_hole(y + 1) {
				for x in 1..GRID_WIDTH  {
					let i = (y + 1) * GRID_WIDTH + x;
					
					if let Component::Empty { .. } = &self[i] {
						continue;
					}
					
					if let Component::Block { .. } = &self[i] {
						let goal = if self.cursor.position.x <= x as i32 {
							i - 1
						} else {
							i
						}.to_i2();
						
						self.cursor.states.push_back(CursorState::MoveTransport {
																 counter: 0,
																 reached: false,
																 swap_end: true,
																 start: start_index.to_i2(),
																 goal,
															 });
						break;
					}
				}
				} else {
					// solve normally
					self.solve_horizontally(3, 0, GRID_HEIGHT - 1);
					self.solve_vertically(4, 0, GRID_WIDTH, 0, GRID_HEIGHT);
					self.solve_vertically(3, 0, GRID_WIDTH, 0, GRID_HEIGHT);
				}
					
					// else panic and do usual stuff?
					/*
					*/
			} else {
				 // solve normally
				 //self.solve_horizontally(3, 0, GRID_HEIGHT - 1);
				 //self.solve_vertically(4, 0, GRID_WIDTH, 0, GRID_HEIGHT);
				 //self.solve_vertically(3, 0, GRID_WIDTH, 0, GRID_HEIGHT);
				 }
			  }
		
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
		 pub fn push_update(&mut self, garbage_system: &mut GarbageSystem) {
			 // stop pushing if any block is
			 for i in 0..GRID_TOTAL {
				 if let Component::Block { state, .. } = &self[i] {
					 if let BlockState::Clear { .. } = state {
						 self.push_raise = false;
						 return;
					 }
				 }
			 }
	 
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
	 
					 self.cursor.y_offset = -amt;
				 } else {
					 self.push_upwards(garbage_system, false);
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
		 pub fn draw(&mut self, sprites: &mut Sprites, offset: V2, debug: bool) {
			 self.combo_highlight.draw(sprites, offset);
			 self.cursor.draw(sprites, offset);
	 
			 // ai debug draw
		if self.id == 0 {
			let pos = V2::new(self.cursor.position.x as f32, self.cursor.position.y as f32);
			
			/*
			app.push_line(Line {
							 start: pos * ATLAS_SPACING + offset,
							 end: V2::new(6., 4.) * ATLAS_SPACING + offset,
							 ..Default::default()
						  });
		  */
   }
	   
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
	   
				   let position = V2::new(x as f32, y as f32) * ATLAS_SPACING + offset;
				   if let Some(sprite) = self[i].to_sprite(position) {
					  sprites.push(sprite.into());
				   }
				}
			 }
	   
			 // draw some debug info text
			 if debug {
				// draw visual count information
				{
				   let mut sum = 0;
				let mut y_offset = 0;
				   for &vframe in [3, 4, 5, 6, 7].iter() {
					  let mut amt = 0;
					  for i in 0..GRID_TOTAL {
						 if let Component::Block { block, state } = &self[i] {
							if block.vframe == vframe {
							   amt += 1;
						 sum += 1;
					   }
						 }
					  }
	   
					  let position = v2(
						 8. * ATLAS_TILE + offset.x,
						 y_offset as f32 * ATLAS_TILE + offset.y,
					  );
	   
					  sprites.push(Sprite {
						 position: v2(position.x - ATLAS_TILE, position.y),
						 vframe,
						 ..Default::default()
					  });
				  
					  sprites.text(Text {
						 content: &format!("{}", amt),
						 position,
						 ..Default::default()
					  });
					
						 y_offset += 1;
					 }
				 
									let position = v2(
											8. * ATLAS_TILE + offset.x,
											y_offset as f32 * ATLAS_TILE + offset.y,
											);
				
							   sprites.text(Text {
										   content: &format!("{}", sum),
										   position,
										   ..Default::default()
										 });
	}
		
				 // debug info numbers
				 for x in 0..GRID_WIDTH {
					for y in 0..GRID_HEIGHT {
					   let i = y * GRID_WIDTH + x;
		
					   if let Component::Block { block, .. } = &self[i] {
						  let position = v2(
							 x as f32 * ATLAS_TILE + offset.x + 4.,
							 y as f32 * ATLAS_TILE + block.offset.y + offset.y + 8.,
						  );
		
						  let text = &format!("{}", i);
						  //let text = "0";
						
						  sprites.text(Text {
											 content: text,
											 scale: v2(0.3, 0.3),
											 //scale: wgpu_glyph::Scale { x: 20., y: 16. },
						   position,
						   ..Default::default()
						  });
					}
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
			   