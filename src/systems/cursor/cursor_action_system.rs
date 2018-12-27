use crate::{
    block_states::change_state,
    components::{
        playfield::{Push, Stack},
        Block, Cursor, PlayfieldId,
    },
    data::{
        block_data::SWAP_TIME,
        cursor_data::CURSOR_ACTIONS,
        playfield_data::{BLOCKS, COLUMNS},
    },
};
use amethyst::{ecs::*, input::*};

pub struct CursorActionSystem;

impl<'a> System<'a> for CursorActionSystem {
    type SystemData = (
        WriteStorage<'a, Cursor>,
        Read<'a, InputHandler<String, String>>,
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
        WriteStorage<'a, Push>,
        ReadStorage<'a, PlayfieldId>,
    );

    fn run(&mut self, (mut cursors, input, mut blocks, stacks, mut pushes, ids): Self::SystemData) {
        for (stack, push, id) in (&stacks, &mut pushes, &ids).join() {
            let cursor = cursors.get_mut(stack.cursor_entity).unwrap();

            if cursor.keys.press(&input, CURSOR_ACTIONS[**id][4]) {
                swap(cursor.x, cursor.y, &stack, &mut blocks);
            }

            // raise will always be true when the raise key is held down
            push.signal_raise = cursor.keys.down(&input, CURSOR_ACTIONS[**id][5]);
        }
    }
}

fn swap(x: f32, y: f32, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) {
    let i = Stack::coordinates_to_index(x as usize, y as usize);

    let mut can_swap: bool = false;
    {
        let b1 = blocks.get(stack[i]).unwrap();
        let b2 = blocks.get(stack[i + 1]).unwrap();

        // get above blocks if they exist without storing them as mut
        let (b1_above_block, b2_above_block) = {
            if i < BLOCKS - COLUMNS {
                (
                    blocks.get(stack[i + COLUMNS]),
                    blocks.get(stack[i + 1 + COLUMNS]),
                )
            } else {
                (None, None)
            }
        };

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
        let left_block = blocks.get(stack[i]).unwrap().clone();
        let right_block = blocks.get(stack[i + 1]).unwrap().clone();

        blocks
            .get_mut(stack[i + 1])
            .unwrap()
            .set_properties(left_block);

        blocks
            .get_mut(stack[i])
            .unwrap()
            .set_properties(right_block);
    }
}

// swap variables that need to be set on a different direction
fn set_swap_variables(b: &mut Block, dir: f32) {
    b.offset.0 = 16.0 * dir;
    b.counter = SWAP_TIME as u32;
    b.move_dir = dir;
    change_state(b, "SWAP");
}
