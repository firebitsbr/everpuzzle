#![allow(unused_variables)]
use crate::{
    block_states::change_state,
    components::{playfield::Stack, Block, GarbageHead},
    systems::BlockSystem,
};
use amethyst::ecs::WriteStorage;

// only detects if this block can fall and sets the state to hang
// resets chainable to false if this block can't fall
pub struct Idle;
impl Idle {
    pub fn execute(
        i: usize,
        stack: &Stack,
        blocks: &mut WriteStorage<'_, Block>,
        heads: &mut WriteStorage<'_, GarbageHead>,
    ) {
        if !blocks.get(stack[i]).unwrap().is_garbage {
            let block_can_hang = BlockSystem::check_for_hang(i, stack, blocks);
            let b = blocks.get_mut(stack[i]).unwrap();

            // change the block to state if it isn't empty and the block below is empty or falling
            if block_can_hang {
                change_state(b, "HANG");
            } else {
                b.chainable = false;
            }
        } else {
            let head = heads.get_mut(stack[i]).unwrap();
            let b = blocks.get_mut(stack[i]).unwrap();

            // when all garbage can fall -> change head to hang
            if head.can_fall {
                change_state(b, "HANG");
            }
            // when anything is hanging below, set hang to it
            // counter is set to the biggest counter
            else if head.can_hang.0 {
                head.hanged = true;
                b.state = "HANG";
                b.counter = head.can_hang.1;
            }
        }
    }
}
