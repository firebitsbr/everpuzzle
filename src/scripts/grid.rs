use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::cmp::max;
use std::ops::{Index, IndexMut};

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
                let parent_index = indexes[0];

                // TODO(Skytrias): look for garbage child and its parent_index
                if let Some(state) = self.garbage_state(parent_index + GRID_WIDTH) {
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
                            self[i - GRID_WIDTH].clear_started(),
                            self[i + 1].clear_started(),
                            self[i - 1].clear_started(),
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

    // detects wether the block can currently hang, switches the state to hang with below counter
    pub fn block_detect_hang(&mut self) {
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

    // detects if the garbage can hang and sets it to the hang state
    pub fn garbage_detect_hang(&mut self) {
        for (_, _, i) in iter_yx_rev() {
            if self.garbage(i).is_some() {
                if let Some(counter) = self.garbage_can_hang(i) {
                    if let Some(state) = self.garbage_state_mut(i) {
                        if state.is_idle() {
                            state.to_hang(counter);
                        }
                    }
                }
            }
        }
    }

    // once block / garbage hang is done it swaps index components out
    // 2 types of hang have to happen in the same yx_rev loop,
    pub fn block_garbage_resolve_hang(&mut self) {
        for (x, y, i) in iter_yx_rev() {
            // block hang finish
            if self.block_state(i).filter(|s| s.hang_finished()).is_some() {
                let index_below = (x, y + 1).to_index();

                // below in range && below still empty
                if let Some(ib) = index_below {
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

            // garbage hang finish
            {
                // skip non garbage hang state
                let head_index = i;
                if let Some(state) = self.garbage_state(head_index) {
                    if !state.hang_finished() {
                        continue;
                    }
                } else {
                    continue;
                }

                // TODO(Skytrias): check again after hang is done if we can still go downwards

                // get copy of indexes, sorted so that the higher indexes (the lowest ones in the grid) appear first so it behaves like xy_rev
                let child_indexes: Option<Vec<usize>> = self.garbage(head_index).map(|g| {
                    let mut clone = g.children.clone();
                    clone.sort_by(|a, b| b.cmp(a));
                    clone
                });

                // move all children including the parent down
                if let Some(indexes) = child_indexes {
                    for &index in indexes.iter() {
                        if let Some(ib) = (index + GRID_WIDTH).to_index() {
                            self.components.swap(index, ib);

                            if let Components::GarbageChild(parent_index) = &mut self[ib] {
                                *parent_index = head_index + GRID_WIDTH;
                            }
                        }
                    }
                }

                let head_index = head_index + GRID_WIDTH;

                // shift all children indexes one down from the new head index position
                if let Some(g) = self.garbage_mut(head_index) {
                    for child_index in g.children.iter_mut() {
                        // TODO(Skytrias): bad if garbage sits at bottom
                        *child_index += GRID_WIDTH;
                    }
                }

                let hang_counter = self.garbage_can_hang(head_index);
                if let Some(g) = self.garbage_mut(head_index) {
                    if hang_counter.is_none() {
                        g.reset();
                    }
                }
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

                if self.flood_horizontal_count > 2 {
                    // TODO(Skytrias): bad to clone!
                    for clear_index in self.flood_horizontal_history.clone().iter() {
                        if let Some(state) = self.block_state_mut(*clear_index) {
                            state.to_clear();
                        }
                    }
                }

                if self.flood_vertical_count > 2 {
                    // TODO(Skytrias): bad to clone!
                    for clear_index in self.flood_vertical_history.clone().iter() {
                        if let Some(state) = self.block_state_mut(*clear_index) {
                            state.to_clear();
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

    pub fn update(&mut self, app: &mut App) {
        assert!(!self.components.is_empty());

        // NOTE(Skytrias): resolves might need to happen before detects, so there is 1 frame delay?
        self.garbage_detect_clears();
        self.garbage_resolve_clears(app);
        self.block_resolve_swap();
        self.block_detect_bottom();
        self.block_detect_hang();
        self.garbage_detect_hang();
        self.block_garbage_resolve_hang();
        self.block_detect_clear();
        self.block_resolve_clear();

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
