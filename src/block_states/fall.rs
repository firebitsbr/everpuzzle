#![allow(unused_variables)]
use amethyst::ecs::WriteStorage;
use components::{block::Block, playfield::stack::Stack, garbage_head::GarbageHead};
use data::playfield_data::COLUMNS;
use systems::block_system::change_state;

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

                for entity in &head_parts {
                    // store data from the current to a temp
                    let temp_block = blocks.get(*entity).unwrap().clone();
                    let temp_id = temp_block.id;

                    // store data into the down block
                    blocks
                        .get_mut(stack[temp_block.id as usize - COLUMNS])
                        .unwrap()
                        .set_properties(temp_block);

                    // reset data in the current one to default
                    blocks.get_mut(*entity).unwrap().reset();
                    let removed_head = heads.remove(stack[i]).unwrap();
                    if !heads.contains(stack[temp_id as usize - COLUMNS]) {
                        heads.insert(stack[temp_id as usize - COLUMNS], removed_head);
                        println!("moved the head one down");
                    }
                }
            }
        }
    }
}
