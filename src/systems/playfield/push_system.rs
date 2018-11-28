use amethyst::ecs::*;

use components::{
    block::Block,
    cursor::Cursor,
    playfield::{kind_generator::KindGenerator, push::Push, stack::Stack},
};
use data::block_data::*;

const RAISE_TIME: f32 = 0.025;

// handles the entire pushing system which offsets all blocks and cursor
// each complete grid offset the entire blocks get copied and move up one row
// in the stack entities.
pub struct PushSystem;

impl<'a> System<'a> for PushSystem {
    type SystemData = (
        WriteStorage<'a, Push>,
        ReadStorage<'a, Stack>,
        WriteStorage<'a, Block>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, KindGenerator>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (mut pushes, stacks, mut blocks, mut cursors, mut kind_gens, entities): Self::SystemData,
    ) {
        // playfield push info / push animation WIP
        for (entity, stack) in (&entities, &stacks).join() {
            {
                // store info in push
                let mut push = pushes.get_mut(entity).unwrap();
                push.any_clears = check_blocks_clearing(&stack, &blocks);
                push.any_top_blocks = check_blocks_at_top(&stack, &blocks);
            }

            {
                // actually offset things based on time
                visual_offset(
                    pushes.get_mut(entity).unwrap(),
                    &stack,
                    &mut blocks,
                    cursors.get_mut(stack.cursor_entity).unwrap(),
                    kind_gens.get_mut(entity).unwrap(),
                );
            }
        }
    }
}

// offsets the playfield each frame by an amount, the amount can be increased by holding raise
// swaps all blocks one upwards each time the offset hits the size of the blocks themselves
// also resets the hold signal for fast raising when any tops are at the top / being cleared
fn visual_offset(
    push: &mut Push,
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    cursor: &mut Cursor,
    generator: &mut KindGenerator,
) {
    // if any cursor signal comes through do smooth increase thats faster and stops
    if push.signal_raise {
        push.smooth_raise = true;
    }

    // stop any raise, even smooth call
    if push.any_clears || push.any_top_blocks {
        push.smooth_raise = false; // deletes all smooth_raise signals
        return;
    }

    // if anything blocks raise by setting its time all raise stops until it counts down
    // used to block the amount of time it takes until another raise triggers
    if push.raised_blocked_counter > 0 {
        push.raised_blocked_counter -= 1;
        push.smooth_raise = false; // deletes all smooth_raise signals
        return;
    }

    // until counter is at 16 (the block sprite size)
    if push.offset_counter > 16.0 {
        // reset all offsets and reset smoothing
        push.offset_counter = 0.0;
        set_visual_offsets(0.0, &stack, blocks, cursor);
        push.smooth_raise = false;
        push.raised_blocked_counter = 5; // TODO: GET TIME FROM FILE
        push_blocks(&stack, blocks, cursor, generator);
    } else {
        // if smooth - increase faster
        if push.smooth_raise {
            push.offset_counter += 4.0;
        }
        // else slowly increase
        else {
            push.offset_counter += RAISE_TIME; // TODO: TIMES LEVEL DEPENDANT
        }

        set_visual_offsets(push.offset_counter, stack, blocks, cursor);
    }
}

// swaps all blocks frmo top to bottom, making everything in the grid move one up
// spawns new data in the lowest row via the stacks generator
fn push_blocks(
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    cursor: &mut Cursor,
    generator: &mut KindGenerator,
) {
    // have a block and store its down neighbors values
    // go down the grid to not copy the same data
    for i in 0..BLOCKS - COLS {
        // TODO: Fix ceiling with upcoming data
        // since for i doesnt work backwards we do this
        let reverse = BLOCKS - i - 1;

        let down: Block = *blocks.get(stack[reverse - COLS]).unwrap();

        let b = blocks.get_mut(stack[reverse]).unwrap();

        b.set_properties(down);
        b.anim_offset = 0;
    }

    // generate lowest row since its now empty!
    let new_row = generator.create_rows((6, 1));
    for i in 0..COLS {
        blocks.get_mut(stack[i]).unwrap().kind = new_row[i];
    }

    // move up the y position, since blocks move 1 up the cursor would stick to the same place
    if cursor.y < ROWS as f32 {
        cursor.y += 1.0;
    }
}

// sets y offsets on each stack's cursor and blocks
// rounds them to make it appear pixel perfect - not true when scaling everything up tho
fn set_visual_offsets(
    value: f32,
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    cursor: &mut Cursor,
) {
    // round values to go "pixel perfect"
    let rounded_value = value.round();

    for i in 0..BLOCKS {
        blocks.get_mut(stack[i]).unwrap().offset.1 = rounded_value;
    }

    cursor.offset.1 = rounded_value * 2.0;
}

// returns true when any block was found that is currently in clear state
fn check_blocks_clearing(stack: &Stack, blocks: &WriteStorage<'_, Block>) -> bool {
    for i in 0..BLOCKS {
        let b = blocks.get(stack[i]).unwrap();

        if b.state == "CLEAR" {
            // or garbage clear
            return true;
        }
    }

    return false;
}

// returns true if any "real" block is at the top of the grid
fn check_blocks_at_top(stack: &Stack, blocks: &WriteStorage<'_, Block>) -> bool {
    for x in 0..COLS {
        let b = blocks.get(stack[(x, ROWS - 1)]).unwrap();

        if b.kind != -1 && b.state == "IDLE" {
            // or garbage
            return true;
        }
    }

    return false;
}
