#![allow(unused_variables)]
use amethyst::ecs::prelude::WriteStorage;
use block_states::block_state::{change_state, BlockState};
use components::block::Block;
use components::playfield::stack::Stack;
use data::playfield_data::COLUMNS;

// falls to one block below IN 1 FRAME
// sets the block below to this current one
// resets this blocks data to default
pub struct Fall;
impl BlockState for Fall {
    fn enter(b: &mut Block) {}
    fn exit(b: &mut Block) {}

    fn execute(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
        // get a block back while boundary checking it
        // returns an unreferenced block
        let down_block = {
            if i > COLUMNS {
                Some(*blocks.get(stack[i - COLUMNS]).unwrap())
            } else {
                None
            }
        };

        if let Some(down) = down_block {
            if down.is_empty() {
                // store data from the current to a temp
                let temp_block = *blocks.get(stack[i]).unwrap();

                // store data into the down block
                blocks
                    .get_mut(stack[i - COLUMNS])
                    .unwrap()
                    .set_properties(temp_block);

                // reset data in the current one to default
                blocks.get_mut(stack[i]).unwrap().reset();
            } else if down.state == "HANG" {
                let b = blocks.get_mut(stack[i]).unwrap();
                b.state = "HANG";
                b.counter = down.counter;
            } else {
                change_state(blocks.get_mut(stack[i]).unwrap(), "LAND");
            }
        } else {
            blocks.get_mut(stack[i]).unwrap().state = "IDLE";
        }
    }

    fn counter_end(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {}
}
