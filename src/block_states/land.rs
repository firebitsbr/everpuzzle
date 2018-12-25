#![allow(unused_variables)]
use crate::{
    components::{Block, playfield::Stack},
    data::{
        block_data::LAND_TIME,
        playfield_data::{BLOCKS, COLUMNS},
    },
    block_states::change_state
};
use amethyst::ecs::WriteStorage;

// local animation frames
const LAND_ANIM: [u32; 10] = [2, 2, 2, 3, 3, 3, 4, 4, 4, 0];

// STOPS THE BLOCK FROM BEING CHAINABLE, after animating that is
//
// used for animating the land state
// just sets sprite offset to the current animation frames
pub struct Land;
impl Land {
    // set length of how long the fall will last
    pub fn enter(b: &mut Block) {
        b.counter = LAND_TIME;
        b.anim_counter = LAND_TIME;
    }

    // set anim to 0 for safety, blocks aren't chainable once the land is finished
    // being chainable finally stops here!
    pub fn exit(b: &mut Block) {
        b.anim_offset = 0;
        b.chainable = false;
    }

    // simply animate
    pub fn execute(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
        let b = blocks.get_mut(stack[i]).unwrap();
        b.anim_offset = LAND_ANIM[(LAND_TIME - b.anim_counter - 1) as usize];
    }

    // change to idle on default
    // if above isn't null and hanging, set the counter to the above's counter
    pub fn counter_end(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
        let mut above_hanging: bool = false;
        let mut above_counter: u32 = 0;

        if i < BLOCKS - COLUMNS {
            let above = blocks.get(stack[i + COLUMNS]).unwrap();
            above_hanging = above.state == "HANG";
            above_counter = above.counter;
        }

        let b = blocks.get_mut(stack[i]).unwrap();
        if above_hanging {
            change_state(b, "HANG");
            b.counter = above_counter;
        } else {
            change_state(b, "IDLE");
        }
    }
}
