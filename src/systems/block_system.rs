use amethyst::{core::Transform, ecs::*, renderer::*};

use block_states::{clear::Clear, fall::Fall, hang::Hang, idle::Idle, land::Land, swap::Swap};
use components::{
    block::Block, garbage_head::GarbageHead, playfield::stack::Stack, playfield_id::PlayfieldId,
};
use data::playfield_data::{BLOCKS, COLUMNS, ROWS_VISIBLE};
use resources::playfield_resource::Playfields;

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
        ): Self::SystemData,
    ) {
        // run through all existing block stacks
        for stack in (&stacks).join() {
            // run through all states from a block
            for i in 0..BLOCKS {
                // if any block isnt garbage
                if !blocks.get(stack[i]).unwrap().is_garbage {
                    // update the states of all non garbage blocks
                    update_state(i, &stack, &mut blocks, &mut garbage_heads);
                } else {
                    // let head update everything in its order
                    // skip all normal blocks that are only garbage
                    if blocks.get(stack[i]).unwrap().is_garbage_head {
                        garbage_heads
                            .get_mut(stack[i])
                            .unwrap()
                            .below_empty(&stack, &blocks);

                        update_state(i, &stack, &mut blocks, &mut garbage_heads);
                        println!("{}", blocks.get(stack[i]).unwrap().state);
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
        let b = blocks.get_mut(stack[i]).unwrap();

        // decrease all the time
        if b.anim_counter > 0 {
            b.anim_counter -= 1;
        }

        // render sprite with kind if it's not -1
        if b.kind != -1 && !b.clearing {
            if hiddens.contains(stack[i]) {
                hiddens.remove(stack[i]);
            }

            if !b.is_garbage {
                if b.state == "IDLE" {
                    // checks wether the highest block is null
                    if (top.kind != -1 && top.state == "IDLE" && !top.is_garbage)
                        || top_garbage_non_idle
                    {
                        b.anim_offset = 4;
                    } else if b.y == 0 {
                        b.anim_offset = 1;
                    } else {
                        b.anim_offset = 0;
                    }
                }
            } else {
                // dont change the offset ever
                b.anim_offset = 0;
            }

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

// checks whether the block below is empty or falling, also checks whether this block is empty
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

// updates the blocks state machine and triggers transitions to other states
// from withing each state
fn update_state(
    i: usize,
    stack: &Stack,
    blocks: &mut WriteStorage<'_, Block>,
    garbage_heads: &mut WriteStorage<'_, GarbageHead>,
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
        "FALL" => Fall::execute(i, &stack, blocks, garbage_heads),
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

// changes the current block's state to a new one
pub fn change_state(b: &mut Block, new_state: &'static str) {
    if b.state == new_state {
        return;
    }

    // call the current block's state's exit function
    match b.state {
        "LAND" => Land::exit(b),
        "CLEAR" => Clear::exit(b),
        _ => (),
    }

    b.state = new_state;

    // call the current block's state's enter function
    match b.state {
        "HANG" => Hang::enter(b),
        "LAND" => Land::enter(b),
        "CLEAR" => Clear::enter(b),
        _ => (),
    }
}
