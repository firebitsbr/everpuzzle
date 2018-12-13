use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity, WriteStorage};
use components::{block::Block, garbage_head::GarbageHead, playfield::stack::Stack};
use data::playfield_data::{COLUMNS, ROWS_VISIBLE};

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
        let mut first_block: Option<Entity> = None; // entity ref of the first block
        let mut first_id: Option<(usize, usize)> = None; // stack id position of first block
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
        for y in (ROWS_VISIBLE - dimensions.1 + 1)..ROWS_VISIBLE + 1 {
            for x in 0..dimensions.0 {
                // get the entity of each block
                let pos = {
                    // offset blocks optionally
                    if self.offset && last_garbage_matched {
                        (x + 6 - dimensions.0, y)
                    } else {
                        (x, y)
                    }
                };

                let block_id = stack[pos];

                // set highest blocks only until 6 wide
                if counter < 6 {
                    counter += 1;
                    highest_blocks.push(block_id);
                }

                let b = blocks.get_mut(block_id).unwrap();

                // the first block gone through will be the head
                if first_block == None {
                    first_block = Some(block_id);
                    first_id = Some(pos);
                    b.is_garbage_head = true;
                }

                garbage_blocks.push(block_id);
                b.is_garbage = true;
                b.kind = 7;
                b.garbage_head = first_block;
            }
        }

        // go below all blocks bottom ones
        let size = garbage_blocks.len() as i32;
        for i in (size - COLUMNS as i32)..size {
            if i >= 0 {
                lowest_blocks.push(stack[i as usize]);
            }
        }

        self.last_dimensions = dimensions;

        let id = stack[first_id.unwrap()];
        if !garbage_heads.contains(id) {
            garbage_heads
                .insert(
                    id,
                    GarbageHead::new(garbage_blocks, highest_blocks, lowest_blocks),
                )
                .expect("garbage head should be added");
        }
    }
}

impl Component for GarbageMaster {
    type Storage = DenseVecStorage<Self>;
}
