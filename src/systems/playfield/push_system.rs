use amethyst::ecs::*;

use crate::{
    components::{
        playfield::{KindGenerator, Push, Stack},
        Block, Cursor, GarbageHead, PlayfieldId,
    },
    data::playfield_data::{BLOCKS, COLUMNS, RAISE_BLOCKED_TIME, RAISE_TIME, ROWS_VISIBLE},
    resources::Playfields,
};

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
        Read<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
        WriteStorage<'a, GarbageHead>,
    );

    fn run(
        &mut self,
        (
            mut pushes,
            stacks,
            mut blocks,
            mut cursors,
            mut kind_gens,
            entities,
            playfields,
            ids,
            mut garbage_heads,
        ): Self::SystemData,
    ) {
        // playfield push info / push animation WIP
        for (entity, stack, id) in (&entities, &stacks, &ids).join() {
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
                    &mut garbage_heads,
                    cursors.get_mut(stack.cursor_entity).unwrap(),
                    kind_gens.get_mut(entity).unwrap(),
                    playfields[**id].level,
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
    heads: &mut WriteStorage<'_, GarbageHead>,
    cursor: &mut Cursor,
    generator: &mut KindGenerator,
    level: usize,
) {
    // if any cursor signal comes through do smooth increase which is faster and stops
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
        push.raised_blocked_counter = RAISE_BLOCKED_TIME;
        push_blocks(&stack, blocks, heads, cursor, generator);
    } else {
        // if smooth - increase faster
        if push.smooth_raise {
            push.offset_counter += 4.0;
        }
        // else slowly increase
        else {
            push.offset_counter += RAISE_TIME[level];
        }

        set_visual_offsets(push.offset_counter, stack, blocks, cursor);
    }
}

// swaps all blocks from top to bottom, making everything in the grid move one up
// spawns new data in the lowest row via the stack's generator
fn push_blocks(
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    heads: &mut WriteStorage<'_, GarbageHead>,
    cursor: &mut Cursor,
    generator: &mut KindGenerator,
) {
    // have a block and store its down neighbor's values
    // go down the grid to not copy the same data
    for i in 0..BLOCKS - COLUMNS {
        // since for i doesn't work backwards we do this
        let reverse = BLOCKS - i - 1;

        // if theres a garbage head, move its inner ids up and offset its
        // component one up too
        if heads.contains(stack[reverse - COLUMNS]) {
            // offsets all garbage entities since ids move one up too
            heads
                .get_mut(stack[reverse - COLUMNS])
                .unwrap()
                .increase_block_ids();

            let down_head_data = heads.get(stack[reverse - COLUMNS]).unwrap().clone();
            heads.remove(stack[reverse - COLUMNS]);
            heads.insert(stack[reverse], down_head_data).expect("head to be pushed");
        }

        let down = blocks.get(stack[reverse - COLUMNS]).unwrap().clone();
        blocks
            .get_mut(stack[reverse]).unwrap()
            .set_properties(down)
            .set_garbage_head(1)
            .anim_offset = 0;
    }

    // generate lowest row since it's now empty!
    let new_row = generator.create_rows((6, 1));
    for i in 0..COLUMNS {
        blocks.get_mut(stack[i]).unwrap().kind = new_row[i];
    }

    // move up the y position, since blocks move 1 up the cursor would stick to the same place
    if cursor.y < ROWS_VISIBLE as f32 {
        cursor.y += 1.0;
    }
}

// sets y offsets on each stack's cursor and blocks
// rounds them to make it appear pixel perfect - not true when scaling everything up though
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
    for x in 0..COLUMNS {
        let b = blocks.get(stack[(x, ROWS_VISIBLE - 1)]).unwrap();

        if b.kind != -1 && b.state == "IDLE" {
            // or garbage
            return true;
        }
    }

    return false;
}
