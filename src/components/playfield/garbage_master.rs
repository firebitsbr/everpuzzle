use crate::{
    components::{block::Block, garbage_head::GarbageHead, playfield::stack::Stack},
    data::playfield_data::ROWS,
};
use amethyst::ecs::prelude::{Component, DenseVecStorage, WriteStorage};

// Deals with Garbage Spawns, and keeps info on general garbage
// holds all sub garbages in an array easily acessible
pub struct GarbageMaster {
    pub last_dimensions: (usize, usize),
    pub offset: bool, // wether the next garbage should be ofsett
}

impl Default for GarbageMaster {
    fn default() -> Self {
        GarbageMaster {
            last_dimensions: (0, 0),
            offset: false,
        }
    }
}

impl GarbageMaster {
    // spawns a garbage master with the dimensions provided
    // optionally offsets and 1 dimensional garbage when another 1d
    // garbage had spawned before it
    pub fn spawn(
        &mut self,
        dimensions: (usize, usize),
        stack: &Stack,
        blocks: &mut WriteStorage<'_, Block>,
        garbage_heads: &mut WriteStorage<'_, GarbageHead>,
    ) {
        let mut first_block: Option<usize> = None; // entity ref of the first block
        let mut garbage_blocks = Vec::new(); // all garbage blocks
        let mut highest_blocks = Vec::new(); // every block above each garbage block
        let mut lowest_blocks = Vec::new(); // every bottom block below each block
        let mut counter = 0;

        // check wether the block dimensions match the last one
        let last_garbage_matched = self.last_dimensions.1 == 1 && dimensions.1 == 1;

        // offset the upcoming garbage
        if last_garbage_matched {
            self.offset = !self.offset;
        }

        // go through all blocks in the dimensions specified
        for y in (ROWS - dimensions.1)..ROWS {
            for x in 0..dimensions.0 {
                // get the entity of each block
                let index = {
                    // offset blocks optionally
                    if self.offset && last_garbage_matched {
                        Stack::coordinates_to_index(x + 6 - dimensions.0, y)
                    } else {
                        Stack::coordinates_to_index(x, y)
                    }
                };

                // set first_lowest blocks only until 6 wide
                if counter < 6 {
                    counter += 1;
                    lowest_blocks.push(index);
                }

                let b = blocks.get_mut(stack[index]).unwrap();

                // the first block gone through will be the head
                if first_block == None {
                    first_block = Some(index);
                    b.is_garbage_head = true;
                }

                garbage_blocks.push(index);
                b.is_garbage = true;
                b.kind = 7;
                b.garbage_head = first_block;
            }
        }

        // clone the garbage_blocks contents, reverse order it and go for the new first
        // column of it
        let mut temp_blocks = garbage_blocks.clone();
        temp_blocks.reverse();
        for i in 0..temp_blocks.len() {
            if i < 6 {
                highest_blocks.push(temp_blocks[i]);
            }
        }

        self.last_dimensions = dimensions;

        // add a garbage component to the block entity
        let id = stack[first_block.unwrap()];
        if !garbage_heads.contains(id) {
            garbage_heads
                .insert(
                    id,
                    GarbageHead::new(garbage_blocks, highest_blocks, lowest_blocks, dimensions),
                )
                .expect("garbage head should be added");
        }
    }
}

impl Component for GarbageMaster {
    type Storage = DenseVecStorage<Self>;
}
