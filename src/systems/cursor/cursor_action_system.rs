use amethyst::{ecs::*, input::*};

use block_states::block_state::change_state;
use components::{
    block::Block,
    cursor::Cursor,
    playfield::{
        clear::Clear, kind_generator::KindGenerator, lose::Lose, push::Push, stack::Stack,
        stats::Stats,
    },
};
use data::block_data::SWAP_TIME;
use data::playfield_data::{BLOCKS, COLUMNS};

pub struct CursorActionSystem;

impl<'a> System<'a> for CursorActionSystem {
    type SystemData = (
        WriteStorage<'a, Cursor>,
        Read<'a, InputHandler<String, String>>,
        WriteStorage<'a, KindGenerator>,
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
        WriteStorage<'a, Push>,
        WriteStorage<'a, Clear>,
        WriteStorage<'a, Lose>,
        WriteStorage<'a, Stats>,
    );

    fn run(
        &mut self,
        (
            mut cursors,
            input,
            mut kind_gens,
            mut blocks,
            stacks,
            mut pushes,
            mut clears,
            mut loses,
            mut stats,
        ): Self::SystemData,
    ) {
        for cursor in (&mut cursors).join() {
            // reset all block colors to a random value
            if cursor.keys.press(&input, "space") {
                for (stack, push, clear, kind_gen, lose, stat) in (
                    &stacks,
                    &mut pushes,
                    &mut clears,
                    &mut kind_gens,
                    &mut loses,
                    &mut stats,
                )
                    .join()
                {
                    let kinds = kind_gen.create_stack(5, 8);

                    for i in 0..BLOCKS {
                        let b = blocks.get_mut(stack[i]).unwrap();
                        b.reset();
                        b.kind = kinds[i];
                    }

                    *push = Default::default();
                    *clear = Default::default();
                    *lose = Default::default();
                    *stat = Default::default();
                    cursor.reset();
                }
            }

            // swaps block kinds around, gets all blocks, searches through creation id,
            // id matches cursor pos conversion, swapping from one block to another block
            if cursor.keys.press(&input, "swap") {
                for stack in (&stacks).join() {
                    swap(cursor.x, cursor.y, &stack, &mut blocks);
                }
            }
        }

        // raise will always be true when the raise key is held down
        for (stack, push) in (&stacks, &mut pushes).join() {
            let cursor = cursors.get_mut(stack.cursor_entity).unwrap();
            push.signal_raise = cursor.keys.down(&input, "raise");
        }
    }
}

fn swap(x: f32, y: f32, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
    let i = Stack::coordinates_to_index(x as usize, y as usize);

    let mut can_swap: bool = false;
    {
        let b1 = blocks.get(stack[i]).unwrap();
        let b2 = blocks.get(stack[i + 1]).unwrap();

        let mut b1_above_block: Option<&Block> = None;
        let mut b2_above_block: Option<&Block> = None;

        if i < BLOCKS - COLUMNS {
            b1_above_block = blocks.get(stack[i + COLUMNS]);
            b2_above_block = blocks.get(stack[i + 1 + COLUMNS]);
        }

        if b1.is_swappable(b2, b1_above_block) && b2.is_swappable(b1, b2_above_block) {
            if b1.is_empty() && b2.is_empty() {
                return;
            }

            can_swap = true;
        }
    }

    if can_swap {
        // set variables
        set_swap_variables(blocks.get_mut(stack[i]).unwrap(), 1.0);
        set_swap_variables(blocks.get_mut(stack[i + 1]).unwrap(), -1.0);

        // set default stack blocks
        let left_block = *blocks.get(stack[i]).unwrap();
        let right_block = *blocks.get(stack[i + 1]).unwrap();

        {
            blocks
                .get_mut(stack[i + 1])
                .unwrap()
                .set_properties(left_block);
        }

        {
            blocks
                .get_mut(stack[i])
                .unwrap()
                .set_properties(right_block);
        }
    }
}

// swap variables that need to be set on a different direction
fn set_swap_variables(b: &mut Block, dir: f32) {
    b.offset.0 = 16.0 * dir;
    b.counter = SWAP_TIME as u32;
    b.move_dir = dir;
    change_state(b, "SWAP");
}
