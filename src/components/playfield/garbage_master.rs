use amethyst::ecs::prelude::{Component, DenseVecStorage, Entity, WriteStorage};
use components::{
    block::Block,
    playfield::{
        stack::Stack,
    },
    garbage_head::GarbageHead,
};
use data::playfield_data::COLUMNS;

// holds all sub garbages in an array easily acessible
pub struct GarbageMaster {
    pub children: Vec<Entity>,
    pub last_dimensions: (usize, usize),
    pub offset: bool, // wether the next garbage should be ofsett
}

impl Default for GarbageMaster {
    fn default() -> Self {
        GarbageMaster {
            children: Vec::new(),
            last_dimensions: (0, 0),
            offset: false,
        }
    }
}

impl Component for GarbageMaster {
    type Storage = DenseVecStorage<Self>;
}

impl GarbageMaster {
    // spawns a garbage master with the dimensions provided
    // optionally offsets and 1 dimensional garbage when another 1d
    // garbage had spawned before it
    pub fn garbage_spawn(
        &mut self, 
        dimensions: (usize, usize),
        stack: &Stack,
        blocks: &mut WriteStorage<'_, Block>,
    ) -> GarbageHead {
        let mut first_block: Option<Entity> = None; 
        let mut garbage_blocks = Vec::new(); // all garbage blocks
        let mut highest_blocks = Vec::new(); // every block above each garbage block
        let mut lowest_blocks = Vec::new(); // every bottom block below each block
        let mut counter = 0;

        // check wether the block dimensions match the last one
        let last_garbage_matched = 
            self.last_dimensions.1 == 1 && dimensions.1 == 1;

        // offset the upcoming garbage 
        if last_garbage_matched {
            self.offset = !self.offset;
        }

        // go through all blocks in the dimensions specified
        for y in 0..dimensions.1 {
            for x in 0..dimensions.0 {
                // get the entity of each block
                let block_id = {
                    // offset blocks optionally
                    if self.offset && last_garbage_matched {
                        stack[(x + 6 - dimensions.0, y)]
                    }
                    else {
                        stack[(x, y)]
                    }
                };

                // set highest blocks only until 6 wide
                if counter < 6 {
                    counter += 1;
                    highest_blocks.push(block_id);
                }

                // the first block gone through will be the head 
                if first_block == None {
                    first_block = Some(block_id);
                }

                garbage_blocks.push(block_id);
                let b = blocks.get_mut(stack[(x, y)]).unwrap();
                b.is_garbage = true;
                b.garbage_head = first_block;
            }
        }

        // go below all blocks bottom ones
        let size = garbage_blocks.len();  
        for i in (size - COLUMNS)..size {
            if i >= 0 {
                lowest_blocks.push(stack[i]);
            }
        }

        self.last_dimensions = dimensions;
        GarbageHead::new(first_block.unwrap(), garbage_blocks, highest_blocks, lowest_blocks)
    }
}

