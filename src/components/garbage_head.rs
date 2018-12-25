use amethyst::ecs::{Component, DenseVecStorage, WriteStorage};
use crate::{
    components::{Block, playfield::Stack},
    data::playfield_data::COLUMNS,
};
use std::cmp::max;

// Head of garbage that stays with the block entity that is its head
// consists of the head and its subparts
#[derive(Clone)]
pub struct GarbageHead {
    pub clearing: bool,
    pub parts: Vec<usize>, // all garbage blocks in this head
    pub highest_blocks: Vec<usize>,
    pub lowest_blocks: Vec<usize>,
    pub can_fall: bool,
    pub can_hang: (bool, u32),
    pub dimensions: (usize, usize),
    pub marked_clear: bool,
    pub hanged: bool,
    pub new_kinds: Vec<usize>,
}

impl GarbageHead {
    pub fn new(
        parts: Vec<usize>,
        highest_blocks: Vec<usize>,
        lowest_blocks: Vec<usize>,
        dimensions: (usize, usize),
    ) -> GarbageHead {
        GarbageHead {
            parts,
            lowest_blocks,
            highest_blocks,
            dimensions,
            can_fall: false,
            can_hang: (false, 0),
            clearing: false,
            marked_clear: false,
            hanged: false,
            new_kinds: Vec::new(),
        }
    }

    // goes through all lowest blocks and checks wether theyre empty
    // all lowest blocks need to be empty for this to be true!
    pub fn below_empty(&mut self, stack: &Stack, blocks: &WriteStorage<'_, Block>) {
        let mut counter = 0;

        // go through all lowest blocks
        for id in &self.lowest_blocks {
            // get his neighbor from this ones id
            let down_block = blocks.get(stack[id - COLUMNS]);

            // look if the downwards block exists
            if let Some(down) = down_block {
                // check it
                if down.state == "IDLE" && down.kind == -1 {
                    counter += 1;
                }
            }
        }

        // return true if the length of lowest has been reached
        self.can_fall = counter == self.lowest_blocks.len();
    }

    // returns wether this head and its parts can fall and the time the hang will
    // take until it falls down
    pub fn below_hang(&self, stack: &Stack, blocks: &WriteStorage<'_, Block>) -> (bool, u32) {
        let mut biggest_hang = 0;

        for id in &self.lowest_blocks {
            // get lower block, can be None
            let down_block = blocks.get(stack[id - COLUMNS]);

            // if not none
            if let Some(down) = down_block {
                if down.state == "HANG" {
                    // looks at garbage beneat and at its head to see which counter it has
                    if down.is_garbage {
                        let down_head = blocks.get(stack[down.garbage_head.unwrap()]).unwrap();
                        biggest_hang = max(biggest_hang, down_head.counter);
                    }
                    // look at the hanging block below and max out the counter
                    else {
                        biggest_hang = max(biggest_hang, down.counter);
                    }
                }
                // stops all hang wether the block isnt hanging and its a real block
                // or a "dumb" garbage block
                else if down.kind != -1 || down.kind == 7 {
                    return (false, 0);
                }
            }
        }
        return (true, biggest_hang);
    }

    // increases all ids a column lower without boundary checking
    pub fn increase_block_ids(&mut self) {
        // normal garbage blocks
        for elem in self.parts.iter_mut() {
            *elem += COLUMNS;
        }

        // lowest i32
        for elem in self.lowest_blocks.iter_mut() {
            *elem += COLUMNS;
        }

        // highest blocks
        for elem in self.highest_blocks.iter_mut() {
            *elem += COLUMNS;
        }
    }

    // same function just for downwards offsetting, im too lazy to
    // convert variables all over just to get a usize to get lower
    // lowers all ids a column lower without boundary checking
    pub fn decrease_block_ids(&mut self) {
        // normal garbage blocks
        for elem in self.parts.iter_mut() {
            *elem -= COLUMNS;
        }

        // lowest i32
        for elem in self.lowest_blocks.iter_mut() {
            *elem -= COLUMNS;
        }

        // highest blocks
        for elem in self.highest_blocks.iter_mut() {
            *elem -= COLUMNS;
        }
    }
}

impl Component for GarbageHead {
    type Storage = DenseVecStorage<Self>;
}
