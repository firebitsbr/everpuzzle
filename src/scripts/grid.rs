use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use std::cmp::max;
use std::ops::{Index, IndexMut};

// like shader
#[derive(Copy, Clone, Debug)]
pub struct GridBlock {
    pub first: V4,
    pub second: V4,
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

    pub fn garbage_can_hang<I: Copy + BoundIndex>(&self, i: I) -> Option<u32> {
        // get copy of indexes
        let child_indexes: Option<Vec<usize>> = self.garbage(i).map(|g| g.lowest());

        let mut can_hang = true;
        let mut hang_counter = 0;

        // loop through children, look below each and check
        if let Some(indexes) = child_indexes {
            for child_index in indexes.iter() {
                if let Some(ib) = (child_index + GRID_WIDTH).to_index() {
                    let below_component_empty = self[ib].is_empty();
                    let below_block_state =
                        self.block_state(ib).unwrap_or(&BlockStates::Idle).clone();
                    let below_garbage_state = self
                        .garbage_state(ib)
                        .unwrap_or(&GarbageStates::Idle)
                        .clone();

                    if !below_component_empty {
                        if let BlockStates::Hang { counter, .. } = below_block_state {
                            hang_counter = max(hang_counter, counter);
                        } else {
                            if let GarbageStates::Hang { counter, .. } = below_garbage_state {
                                hang_counter = max(hang_counter, counter);
                            //println!("got garbage hang {}, {}", i.raw(), counter);
                            } else {
                                can_hang = false;
                            }
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

    pub fn update(&mut self, app: &mut App) {
        assert!(self.components.len() != 0);

        // garbage hang detection
        for (_, _, i) in iter_yx_rev() {
            // skip none garbage
            if self.garbage(i).is_none() {
                continue;
            }

            if let Some(counter) = self.garbage_can_hang(i) {
                if let Some(state) = self.garbage_state_mut(i) {
                    if state.is_idle() {
                        state.to_hang(counter);
                    }
                }
            }
        }

        // garbage clear detection
        for (_, _, i) in iter_yx_rev() {
            // skip none garbage and also only allow idle to be detected
            if self.garbage(i).is_none() {
                continue;
            }

            let g = self.garbage(i).unwrap();

            if !g.state.is_idle() {
                continue;
            }

            let mut clear_found = false;
            for index in g.children.iter() {
                if let Some(ib) = (index + GRID_WIDTH).to_index() {
                    if let Some(s) = self.block_state(ib) {
                        if s.clear_started() {
                            clear_found = true;
                            println!("found!");
                            break;
                        }
                    }
                }
            }

            if clear_found {
                if let Some(g) = self.garbage_mut(i) {
                    g.state.to_clear();
                }
            }
        }

        for (_, _, i) in iter_xy() {
            if let Some(g) = self.garbage_mut(i) {
                if g.state.clear_finished() {
                    for index in g.children.clone().iter() {
                        self.components[*index] = Components::Normal(Block::random(app));
                    }
                }
            }
        }

        // block check for swap state
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
                    &mut self[i].reset();
                    &mut self[offset].reset();
                }
            }
        }

        // block set bottom row to bottom state
        for x in 0..GRID_WIDTH {
            if let Some(b) = self.block_mut((x, GRID_HEIGHT - 1)) {
                if b.state.is_swap() || b.state.is_clear() {
                    continue;
                }

                b.state = BlockStates::Bottom;
            }
        }

        // block hang setting
        for (x, y, i) in iter_yx_rev() {
            // only allow idle
            if self.block_state(i).filter(|s| s.is_idle()).is_none() {
                continue;
            }

            if let Some(ib) = (x, y + 1).to_index() {
                let below_empty = self[ib].is_empty();
                let below_state = self.block_state(ib).unwrap_or(&BlockStates::Idle).clone();

                if let Some(state) = self.block_state_mut(i) {
                    if below_empty {
                        if state.is_idle() {
                            state.to_hang();
                        }
                    } else {
                        if below_state.is_hang() {
                            *state = below_state;
                        }
                    }
                }
            }
        }

        // block hang finished execution
        for (x, y, i) in iter_yx_rev() {
            if let Some(state) = self.block_state(i) {
                if !state.hang_finished() {
                    continue;
                }
            } else {
                continue;
            }

            let index_below = (x, y + 1).to_index();

            if let Some(ib) = index_below {
                self.components.swap(i, ib);

                let index_below_below = (x, y + 2).to_index();

                if let Some(ibb) = index_below_below {
                    if !self[ibb].is_empty() {
                        &mut self[i].reset();
                        &mut self[ib].reset();
                    }
                }
            }
        }

        // garbage hang finish
        for (_, _, head_index) in iter_yx_rev() {
            // skip non garbage hang state
            if let Some(state) = self.garbage_state(head_index) {
                if !state.hang_finished() {
                    continue;
                }
            } else {
                continue;
            }

            // get copy of indexes
            let child_indexes: Option<Vec<usize>> =
                self.garbage(head_index).map(|g| g.children.clone());

            // move all children including the parent down
            if let Some(indexes) = child_indexes {
                for &index in indexes.iter() {
                    let index_below = (index + GRID_WIDTH).to_index();

                    if let Some(ib) = index_below {
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

        // block flood fill check for the current color of the block for any other near colors
        for (x, y, i) in iter_xy() {
            let frame = self
                .block(i)
                .filter(|b| b.state.is_real())
                .map(|b| b.vframe);

            if let Some(vframe) = frame {
                self.flood_check(x, y, vframe as f32, FloodDirection::Horizontal);
                self.flood_check(x, y, vframe as f32, FloodDirection::Vertical);

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

        // block clear the component if clear state is finished
        for (_, _, i) in iter_xy() {
            let finished = self
                .block_state(i)
                .map(|s| s.is_clear() && s.clear_finished())
                .unwrap_or(false);

            if finished {
                self.components[i] = Components::Empty;
            }
        }

        // update all components
        for c in self.components.iter_mut() {
            c.update();
        }
    }

    fn flood_check(&mut self, x: usize, y: usize, vframe: f32, direction: FloodDirection) {
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
        let data: Vec<GridBlock> = self
            .components
            .iter()
            .map(|c| GridBlock {
                first: v4(c.hframe(), c.vframe(), c.visible(), 1.),
                second: v4(c.x_offset(), c.y_offset(), 0., 0.),
            })
            .collect();

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
