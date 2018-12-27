use crate::{
    block_states::change_state,
    components::{
        playfield::{Clear, GarbageMaster, Stack, Stats},
        Block, GarbageHead, PlayfieldId,
    },
    data::{
        block_data::{FACE_TIME, FLASH_TIME, POP_TIME},
        playfield_data::{COLUMNS, ROWS_VISIBLE},
    },
    resources::Playfields,
};
use amethyst::ecs::*;
use std::cmp::max;

pub struct ClearSystem;

impl<'a> System<'a> for ClearSystem {
    type SystemData = (
        WriteStorage<'a, Clear>,
        WriteStorage<'a, Block>,
        ReadStorage<'a, Stack>,
        WriteStorage<'a, Stats>,
        Read<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
        WriteStorage<'a, GarbageMaster>,
        WriteStorage<'a, GarbageHead>,
    );

    fn run(
        &mut self,
        (
            mut clears,
            mut blocks,
            stacks,
            mut stats,
            playfields,
            ids,
            mut garbages,
            mut garbage_heads,
        ): Self::SystemData,
    ) {
        // block clear detection
        // counts the amount of clears each frame, passes them uniquely to an array holding their ids
        // sets a lot of playfield_clear values and then sets the blocks to animate with given times
        for (clear, stack, stats, id, garbage) in
            (&mut clears, &stacks, &mut stats, &ids, &mut garbages).join()
        {
            // add all blocks to a vec that are clearing right now,
            // each block can only exist once in this vec
            for x in 0..COLUMNS {
                for y in 0..ROWS_VISIBLE {
                    for clear_block_id in check_clear(x, y, &stack, &blocks) {
                        if !clear.clear_queue.contains(&clear_block_id) {
                            clear.clear_queue.push(clear_block_id);
                        }
                    }
                }
            }

            // if no clears were found, don't go through all of them
            let clear_size = clear.clear_queue.len() as u32;
            if clear_size != 0 {
                clear.combo_counter = 0;

                // animation times, TODO: get playfield level dependant times
                let flash: u32 = FLASH_TIME[playfields[**id].level];
                let face: u32 = FACE_TIME[playfields[**id].level];
                let pop: u32 = POP_TIME[playfields[**id].level];

                let all_time: u32 = flash + face + pop * clear_size;

                let had_chainable: bool = any_chainable_exists(&clear.clear_queue, stack, &blocks);

                // max the chain and save data in a last chain
                if had_chainable {
                    clear.chain += 1;
                    clear.last_chain = max(clear.chain, clear.last_chain);
                    stats.highest_chain = max(clear.chain, stats.highest_chain);
                }
                // otherwise reset the chain
                else {
                    clear.chain = 1;
                }

                // set all animation times and general time it will take all blocks that are
                // comboing to finish their animation
                for id in &clear.clear_queue {
                    let b = blocks.get_mut(stack[*id as usize]).unwrap();
                    let set_time = flash + face + pop * clear.combo_counter;
                    b.clear_time = set_time as i32;
                    clear.combo_counter += 1;

                    b.counter = all_time;
                    b.clear_start_counter = all_time as i32;
                    change_state(b, "CLEAR");
                }

                // if a combo is bigger than 3 or a chain greater than 1 is there
                if clear.combo_counter > 3 || clear.chain != 1 {
                    // either by combo or by chain
                    let pos = {
                        // if chain == 1 use combo - 1 as x
                        if clear.chain == 1 {
                            (clear.combo_counter as usize - 1, 1)
                        }
                        // if chain is bigger than 1, use x as 6 and go by chain - 1 = y
                        else {
                            (6, clear.chain as usize - 1)
                        }
                    };

                    // spawn the head and its sub parts
                    garbage.spawn(pos, &stack, &mut blocks, &mut garbage_heads);
                }

                // clear the clear_queue if its not empty
                stats.blocks_cleared += clear.combo_counter;
                clear.clear_queue.clear();
            }
        }
    }
}

// checks through each block's right, right_right and up, up_up to see if they are performing a combo
// returns an array of block ids to identify them
fn check_clear(x: usize, y: usize, stack: &Stack, blocks: &WriteStorage<'_, Block>) -> Vec<u32> {
    let mut checks: Vec<u32> = Vec::new();

    let r_rr = check_similar_block(x, y, 1, 0, stack, blocks);
    let u_uu = check_similar_block(x, y, 0, 1, stack, blocks);

    if let Some(mut right_vec) = r_rr {
        checks.append(&mut right_vec);
    }

    if let Some(mut up_vec) = u_uu {
        checks.append(&mut up_vec);
    }

    checks
}

// checks for similar blocks from the current block to 2 others
// checks if they all exist, are comboable, and also if their kinds match with the first
// returns an array of u32 ids of the blocks that are comboable or nothing
// to save on cpu -> not creating empty vecs
fn check_similar_block(
    x: usize,
    y: usize,
    x_offset: usize,
    y_offset: usize,
    stack: &Stack,
    blocks: &WriteStorage<'_, Block>,
) -> Option<Vec<u32>> {
    let b1 = blocks.get(stack[(x, y)]).unwrap();

    let check_boundary = |x: usize, y: usize| -> Option<&Block> {
        if x < COLUMNS && y < ROWS_VISIBLE {
            blocks.get(stack[(x, y)])
        } else {
            None
        }
    };

    let b2 = check_boundary(x + x_offset, y + y_offset);
    let b3 = check_boundary(x + x_offset * 2, y + y_offset * 2);

    if b1.is_comboable() {
        if let Some(block2) = b2 {
            if let Some(block3) = b3 {
                if block2.is_comboable_with(b1.kind) && block3.is_comboable_with(b1.kind) {
                    return Some(vec![b1.id as u32, block2.id as u32, block3.id as u32]);
                }
            }
        }
    }

    // just return nothing to save up on cpu
    // we could just return an empty vec but since this happens around 72 * 2 times it's expensive to do so
    None
}

// checks wether any current block is inside a chain
fn any_chainable_exists(
    clear_ids: &[u32],
    stack: &Stack,
    blocks: &WriteStorage<'_, Block>,
) -> bool {
    for id in clear_ids {
        if blocks.get(stack[*id as usize]).unwrap().chainable {
            return true;
        }
    }

    return false;
}
