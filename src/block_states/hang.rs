#![allow(unused_variables)]
use amethyst::ecs::WriteStorage;
use components::{block::Block, playfield::stack::Stack};
use data::block_data::HOVER_TIME;
use systems::block_system::change_state;

pub struct Hang;
impl Hang {
    pub fn enter(b: &mut Block) {
        b.counter = HOVER_TIME[b.level];
    }

    pub fn counter_end(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
        change_state(blocks.get_mut(stack[i]).unwrap(), "FALL");
    }
}
