#![allow(unused_variables)]
use crate::{
    block_states::change_state,
    components::{playfield::Stack, Block},
    data::block_data::HOVER_TIME,
};
use amethyst::ecs::WriteStorage;

pub struct Hang;
impl Hang {
    pub fn enter(b: &mut Block) {
        b.counter = HOVER_TIME[b.level];
    }

    pub fn counter_end(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
        change_state(blocks.get_mut(stack[i]).unwrap(), "FALL");
    }
}
