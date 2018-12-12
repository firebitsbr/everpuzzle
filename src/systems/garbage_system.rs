use amethyst::ecs::*; use components::{
    block::Block;
    playfield::{
        stack::Stack,
        garbage_master::GarbageMaster,
        push::Push,
        clear::Clear,
    },
    garbage_head::GarbageHead,
};

// handles all sub garbages
pub struct GarbageSystem;

impl<'a> System<'a> for GarbageSystem {
    type SystemData = (
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
        WriteStorage<'a, GarbageMaster>,
        ReadStorage<'a, Push>,
        ReadStorage<'a, Clear>,
    );

    fn run(&mut self, (
        mut blocks, 
        stacks, 
        mut garbages, 
        pushes,
        clears,
    ): Self::SystemData) {
        for (stack, garbage, push, clears) in (&stacks, &mut garbages, &pushes, &clears).join() {
            if !push.any_clears && !push.any_top_blocks {
                                                                 
            }
        }
    }
}

