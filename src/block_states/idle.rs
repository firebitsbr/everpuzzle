#![allow(unused_variables)]
use amethyst::ecs::WriteStorage;
use components::{block::Block, garbage_head::GarbageHead, playfield::stack::Stack};
use systems::block_system::{change_state, check_for_hang};

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
            let block_can_hang = check_for_hang(i, stack, blocks);
            let b = blocks.get_mut(stack[i]).unwrap();

            // change the block to state if it isn't empty and the block below is empty or falling
            if block_can_hang {
                change_state(b, "HANG");
            } else {
                b.chainable = false;
            }
        } else {
            let head = heads.get_mut(stack[i]).unwrap();
            let garbage_can_hang = head.below_hang(&stack, &blocks);

            let b = blocks.get_mut(stack[i]).unwrap();
            // when all garbage can fall -> change head to hang
            if head.can_fall {
                change_state(b, "HANG");
            }
            // when anything is hanging below, set hang to it
            // counter is set to the biggest counter
            else if garbage_can_hang.0 {
                head.hanged = true;
                b.state = "HANG";
                b.counter = garbage_can_hang.1;
            }
        }
    }
}
