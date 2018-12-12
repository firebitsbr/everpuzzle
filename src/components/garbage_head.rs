use amethyst::ecs::{Entity, WriteStorage, Component, DenseVecStorage};
use components::{block::Block, playfield::stack::Stack};
use std::cmp::max;

// Head of garbage that stays with the block entity that is its head
// consists of the head and its subparts
pub struct GarbageHead {
    pub can_fall: bool, // saves wether the head is allowed to fall
    pub clearing: bool,
    pub parts: Vec<Entity>, // all garbage blocks in this head
    pub highest_blocks: Vec<Entity>,
    pub lowest_blocks: Vec<Entity>,
    pub marked_clear: bool,
    pub hanged: bool,
    pub new_kinds: Vec<Entity>,
}

impl GarbageHead {
    pub fn new(
        parts: Vec<Entity>,
        highest_blocks: Vec<Entity>,
        lowest_blocks: Vec<Entity>,
    ) -> GarbageHead {
        GarbageHead {
            parts,
            lowest_blocks,
            highest_blocks,
            can_fall: false,
            clearing: false,
            marked_clear: false,
            hanged: false,
            new_kinds: Vec::new(),
        }
    }

    // goes through all lowest blocks and checks wether theyre empty
    // all lowest blocks need to be empty for this to be true!
    pub fn below_empty(&self, blocks: &WriteStorage<'_, Block>) -> bool {
        let mut counter = 0;

        for entity in &self.lowest_blocks {
            // get lower block, can be None
            let down_block = blocks.get(*entity);

            // if not none
            if let Some(down) = down_block {
                if down.state == "IDLE" && down.kind == -1 {
                    counter += 1;
                }
            }
        }

        // return true if the lenght of lowest has been reached
        counter == self.lowest_blocks.len()
    }

    // returns wether this head and its parts can fall and the time the hang will
    // take until it falls down
    pub fn below_hang(&self, blocks: &WriteStorage<'_, Block>) -> (bool, u32) {
        let mut biggest_hang = 0;

        for entity in &self.lowest_blocks {
            // get lower block, can be None
            let down_block = blocks.get(*entity);

            // if not none
            if let Some(down) = down_block {
                // looks at garbage beneat and at its head to see which counter it has
                let down_head = blocks.get(down.garbage_head.unwrap()).unwrap();
                if down.is_garbage && down_head.state == "HANG" {
                    biggest_hang = max(biggest_hang, down_head.counter);
                }
                // look at the hanging block below and max out the counter
                else if down.state == "HANG" {
                    biggest_hang = max(biggest_hang, down.counter);
                }
                // stops all hang wether the block isnt hanging and its a real block
                // or a "dumb" garbage block
                else if down.state != "HANG" && (down.kind != -1 || down.kind == 7) {
                    return (false, 0);
                }
            }
        }

        return (true, biggest_hang);
    }
}

impl Component for GarbageHead {
    type Storage = DenseVecStorage<Self>;
}
