use crate::{
    block_states::{Clear, Fall, Hang, Idle, Land, Swap},
    components::{
        playfield::{Shake, Stack},
        Block, GarbageHead, PlayfieldId,
    },
    data::playfield_data::{BLOCKS, COLUMNS, ROWS_VISIBLE},
    resources::Playfields,
};
use amethyst::{core::Transform, ecs::*, renderer::*};

// handles everything a block should do itself or based on others
pub struct BlockSystem;
impl<'a> System<'a> for BlockSystem {
    type SystemData = (
        ReadStorage<'a, Stack>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Block>,
        WriteStorage<'a, Hidden>,
        Read<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
        WriteStorage<'a, GarbageHead>,
        WriteStorage<'a, Shake>,
    );

    fn run(
        &mut self,
        (
            stacks,
            mut sprites,
            mut transforms,
            mut blocks,
            mut hiddens,
            playfields,
            ids,
            mut garbage_heads,
            mut shakes,
        ): Self::SystemData,
    ) {
        // run through all existing block stacks
        for (stack, shake) in (&stacks, &mut shakes).join() {
            // run through all states from a block
            for i in 0..BLOCKS {
                // if any block isnt garbage
                if !blocks.get(stack[i]).unwrap().is_garbage {
                    // update the states of all non garbage blocks
                    update_state(i, &stack, &mut blocks, &mut garbage_heads, shake);
                } else {
                    // let head update everything in its order
                    // skip all normal blocks that are only garbage
                    if blocks.get(stack[i]).unwrap().is_garbage_head {
                        {
                            let head = garbage_heads.get_mut(stack[i]).unwrap();
                            head.below_empty(&stack, &blocks);
                            head.below_hang(&stack, &blocks);
                        }

                        update_state(i, &stack, &mut blocks, &mut garbage_heads, shake);
                    }
                }
            }

            // translation
            for (b, transform, id) in (&blocks, &mut transforms, &ids).join() {
                // immutable scale variables
                let (scale_x, scale_y) = {
                    let scale = transform.scale();
                    (scale.x, scale.y)
                };

                transform.set_x((b.x as f32 * 16.0 + b.offset.0 + playfields[**id].x) * scale_x);
                transform.set_y((b.y as f32 * 16.0 + b.offset.1 + playfields[**id].y) * scale_y);
            }

            // rendering
            update_sprites(
                &stack,
                &mut blocks,
                &mut sprites,
                &mut hiddens,
                &mut garbage_heads,
            );
        }
    }
}

// visibility is on when the block's kind isn't -1
// also sets the frame of the sprite by its kind * 9 and an additional
// animation offset used to stay at specific horizontal sprites
fn update_sprites(
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    sprites: &mut WriteStorage<'_, SpriteRender>,
    hiddens: &mut WriteStorage<'_, Hidden>,
    heads: &mut WriteStorage<'_, GarbageHead>,
) {
    for i in 0..BLOCKS {
        let x = blocks.get(stack[i]).unwrap().x;
        let top = blocks
            .get(stack[(x as usize, ROWS_VISIBLE - 1)])
            .unwrap()
            .clone();
        // if the top blocks is a garbage block
        // get its head and check its state, if its idle
        // then its sitting at the top
        let top_garbage_non_idle =
            top.is_garbage && blocks.get(stack[top.garbage_head.unwrap()]).unwrap().state == "IDLE";

        // render sprite with kind if it's not -1
        let visible = {
            // decrease all the time
            let b = blocks.get_mut(stack[i]).unwrap();
            if b.anim_counter > 0 {
                b.anim_counter -= 1;
            }

            b.kind != -1 && !b.clearing
        };

        if visible {
            if hiddens.contains(stack[i]) {
                hiddens.remove(stack[i]);
            }

            let offset: u32;

            // gets the animation offset for normal blocks that are animating (in IDLE)
            if !blocks.get(stack[i]).unwrap().is_garbage {
                let b = blocks.get_mut(stack[i]).unwrap();

                if b.state == "IDLE" {
                    // checks wether the highest block is null
                    // TODO: FIX 1 Frame WHEN GARBAGE IS FALLING
                    if (top.kind != -1 && top.state == "IDLE" && !top.is_garbage)
                        || top_garbage_non_idle
                    {
                        offset = 4;
                    } else if b.y == 0 {
                        offset = 1;
                    } else {
                        offset = 0;
                    }

                    b.anim_offset = offset;
                }
            } else {
                if heads.contains(stack[blocks.get(stack[i]).unwrap().garbage_head.unwrap()]) {
                    offset = get_garbage_offset(i, &stack, &blocks, &heads);
                    blocks.get_mut(stack[i]).unwrap().anim_offset = offset;
                }
            }

            let b = blocks.get(stack[i]).unwrap();
            sprites.get_mut(stack[i]).unwrap().sprite_number =
                b.kind as usize * 8 + b.anim_offset as usize;
        } else {
            if !hiddens.contains(stack[i]) {
                hiddens
                    .insert(stack[i], Hidden::default())
                    .expect("add hide component");
            }
        }
    }
}

// gets the animation offset for garbage blocks
fn get_garbage_offset(
    i: usize,
    stack: &Stack,
    blocks: &WriteStorage<'_, Block>,
    heads: &WriteStorage<'_, GarbageHead>,
) -> u32 {
    // dont change the offset ever
    //b.anim_offset = 0;
    let b = blocks.get(stack[i]).unwrap();
    //println!("is_garbage: {}, head: {:?}", b.is_garbage, b.garbage_head);
    let head_block = blocks.get(stack[b.garbage_head.unwrap()]).unwrap();
    let head = heads.get(stack[b.garbage_head.unwrap()]).unwrap();
    let size = head.dimensions.0 * head.dimensions.1;

    let rel_x = b.x - head_block.x;
    let rel_y = b.y - head_block.y;

    // if garbage is only 1 tall
    // all spritesheet offsets
    if head.dimensions.1 == 1 {
        // each garbage sprite offset with the face included
        let garbage3x1 = [1, 4, 3, 0, 0, 0];
        let garbage4x1 = [1, 5, 6, 3, 0, 0];
        let garbage5x1 = [1, 2, 4, 2, 3, 0];
        let garbage6x1 = [1, 2, 5, 6, 2, 3];
        let all_garbages = [garbage3x1, garbage4x1, garbage5x1, garbage6x1];

        // TODO: corner sprites

        return all_garbages[size - 3][rel_x as usize];
    } else {
        // basic 6x2 of plain sprite offsets without the face yet
        // has a left and right wall in the beginning / end
        let mut garbage6xy = vec![[1, 2, 2, 2, 2, 3], [1, 2, 2, 2, 2, 3]];

        // when dims are larger than 2 insert a new row for each more
        if head.dimensions.1 != 2 {
            for i in 0..head.dimensions.1 - 2 {
                garbage6xy.insert(i + 1, [1, 2, 2, 2, 2, 3]);
            }
        }

        // insert the face
        // if the dims are mod 2 the faces will be in 4 different blocks!
        if head.dimensions.1 % 2 == 0 {
            let middle = head.dimensions.1 / 2;
            garbage6xy[middle][2] = 8;
            garbage6xy[middle][3] = 9;
            garbage6xy[middle - 1][2] = 10;
            garbage6xy[middle - 1][3] = 11;
        }
        // else just go to 2 and 3 and insert the face in 2 blocks
        else {
            garbage6xy[1][2] = 5;
            garbage6xy[1][3] = 6;
        }

        return garbage6xy[rel_y as usize][rel_x as usize];
    }
}

// checks whether the block below is empty or falling, also checks whether this block is empty
impl BlockSystem {
    pub fn check_for_hang(i: usize, stack: &Stack, blocks: &WriteStorage<'_, Block>) -> bool {
        // check if is in vec boundary
        if i > COLUMNS {
            let down = blocks.get(stack[i - COLUMNS]).unwrap();
            let down_empty = down.is_empty() || down.state == "HANG";

            !blocks.get(stack[i]).unwrap().is_empty() && down_empty
        } else {
            false
        }
    }
}

// updates the blocks state machine and triggers transitions to other states
// from withing each state
fn update_state(
    i: usize,
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    garbage_heads: &mut WriteStorage<'_, GarbageHead>,
    shake: &mut Shake,
) {
    // get variables safely for comparisons
    let (mut counter, block_state) = {
        let b = blocks.get(stack[i]).unwrap();
        (b.counter, b.state)
    };

    // decrease the counter if its over 0
    if counter > 0 {
        blocks.get_mut(stack[i]).unwrap().counter -= 1;
        counter -= 1;
    }

    // happens each frame,
    // takes an iterator - to know which block you're looking at right now
    // takes a stack of block entities that you can access
    // takes the whole stack of blocks - get ref or mut out of this
    match block_state {
        "IDLE" => Idle::execute(i, &stack, blocks, garbage_heads),
        "FALL" => Fall::execute(i, &stack, blocks, garbage_heads, shake),
        "LAND" => Land::execute(i, &stack, blocks),
        "CLEAR" => Clear::execute(i, &stack, blocks),
        "SWAP" => Swap::execute(i, &stack, blocks),
        _ => (),
    }

    // gets called once the block's counter runs down to 0
    // mostly used to switch states
    if counter <= 0 {
        match block_state {
            "HANG" => Hang::counter_end(i, &stack, blocks),
            "LAND" => Land::counter_end(i, &stack, blocks),
            "CLEAR" => Clear::counter_end(i, &stack, blocks),
            "SWAP" => Swap::counter_end(i, &stack, blocks),
            _ => (),
        }
    }
}
