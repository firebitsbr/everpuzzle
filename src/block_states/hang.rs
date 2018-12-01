#![allow(unused_variables)]
use amethyst::ecs::prelude::WriteStorage;
use block_states::block_state::{change_state, BlockState};
use components::block::Block;
use components::playfield::stack::Stack;
use data::block_data::HOVER_TIME;

pub struct Hang;
impl BlockState for Hang {
    fn enter(b: &mut Block) {
        b.counter = HOVER_TIME[b.level];
    }

    fn exit(b: &mut Block) {}
    fn execute(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {}

    fn counter_end(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
        change_state(blocks.get_mut(stack[i]).unwrap(), "FALL");
    }
}
