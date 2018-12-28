#![allow(unused_variables)]
use crate::{
    block_states::change_state,
    components::{
        playfield::{Shake, Stack},
        Block, GarbageHead,
    },
    data::playfield_data::{COLUMNS, SHAKE_CHAIN_TIME, SHAKE_COMBO_TIME},
};
use amethyst::ecs::WriteStorage;

// falls to one block below IN 1 FRAME
// sets the block below to this current one
// resets this blocks data to default
pub struct Fall;
impl Fall {
    pub fn execute(
        i: usize,
        stack: &Stack,
        blocks: &mut WriteStorage<'_, Block>,
        heads: &mut WriteStorage<'_, GarbageHead>,
        shake: &mut Shake,
    ) {
        // when the block isnt garbage, do normal fall
        if !blocks.get(stack[i]).unwrap().is_garbage {
            // if crossed the border, return and set state to idle
            if i < COLUMNS {
                blocks.get_mut(stack[i]).unwrap().state = "IDLE";
                return;
            }

            // get all info required for fall determining
            let (down_empty, down_state, down_counter) = {
                let b = blocks.get(stack[i - COLUMNS]).unwrap();
                (b.is_empty(), b.state, b.counter)
            };

            if down_empty {
                // store data from the current to a temp
                let temp_block = blocks.get(stack[i]).unwrap().clone();

                // store data into the down block
                blocks
                    .get_mut(stack[i - COLUMNS])
                    .unwrap()
                    .set_properties(temp_block);

                // reset data in the current one to default
                blocks.get_mut(stack[i]).unwrap().reset();
            } else if down_state == "HANG" {
                let b = blocks.get_mut(stack[i]).unwrap();
                b.state = "HANG";
                b.counter = down_counter;
            } else {
                change_state(blocks.get_mut(stack[i]).unwrap(), "LAND");
            }
        }
        // when the block is the head
        else {
            // if the head and its children can all fall
            if heads.get(stack[i]).unwrap().can_fall {
                // save vec to iter through
                let head_parts = heads.get(stack[i]).unwrap().parts.clone();
                let size = head_parts.len();

                for iterator in 0..size {
                    let id = head_parts[iterator];
                    // store data from the current to a temp
                    let temp_block = blocks.get(stack[id]).unwrap().clone();

                    // store data into the down block
                    blocks
                        .get_mut(stack[id - COLUMNS])
                        .unwrap()
                        .set_properties(temp_block)
                        .set_garbage_head(-1);

                    // reset data in the current one to default
                    blocks.get_mut(stack[id]).unwrap().reset();
                }

                heads.get_mut(stack[i]).unwrap().decrease_block_ids();

                // remove this head and push it one lower if theres no head yet
                let removed_head = heads.remove(stack[i]).unwrap();
                if !heads.contains(stack[i - COLUMNS]) {
                    heads
                        .insert(stack[i - COLUMNS], removed_head)
                        .expect("head to be inserted");
                }
            } else {
                let head = heads.get_mut(stack[i]).unwrap();

                if !head.hanged {
                    let (x, y) = head.dimensions;

                    // set shake frame times for combos or chains
                    if y == 1 {
                        if y < 4 {
                            shake.counter = SHAKE_COMBO_TIME[x - 3];
                        } else {
                            shake.counter = SHAKE_COMBO_TIME[SHAKE_COMBO_TIME.len() - 1];
                        }
                    } else {
                        if y < 4 {
                            shake.counter = SHAKE_CHAIN_TIME[y - 2];
                        } else {
                            shake.counter = SHAKE_CHAIN_TIME[SHAKE_CHAIN_TIME.len() - 1];
                        }
                    }

                    shake.start = true;
                    head.hanged = false;
                }

                // go through all blocks in garbage head and set all to idle
                for id in &head.parts {
                    blocks.get_mut(stack[*id]).unwrap().state = "IDLE";
                }
            }
        }
    }
}
