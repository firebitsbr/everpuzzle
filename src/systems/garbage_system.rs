use amethyst::ecs::*;
use components::{
    block::Block;
    playfield::{
        stack::Stack,
        garbage::{
            GarbageMaster,
            GarbageChild,
        },
    },
};

// handles all sub garbages
pub struct GarbageSystem;

impl<'a> System<'a> for GarbageSystem {
    type SystemData = (
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
    );

    fn run(&mut self, (mut blocks, stacks): Self::SystemData) {
        
    }
}

