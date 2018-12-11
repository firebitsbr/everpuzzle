use amethyst::{core::Transform, ecs::*, renderer::*};

use block_states::{
    block_state::BlockState, clear::Clear, fall::Fall, hang::Hang, idle::Idle, land::Land,
    swap::Swap,
};
use components::{block::Block, playfield::stack::Stack, playfield_id::PlayfieldId};
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
    );

    fn run(
        &mut self,
        (stacks, mut sprites, mut transforms, mut blocks, mut hiddens, playfields, ids): Self::SystemData,
    ) {
        // run through all existing block stacks
        for stack in (&stacks).join() {
            // run through all states from a block
            for i in 0..BLOCKS {
                let b = *blocks.get(stack[i]).unwrap();

                // decrease the counter if its over 0
                if b.counter > 0 {
                    blocks.get_mut(stack[i]).unwrap().counter -= 1;
                }

                // match all on the blocks state - run all execute functions
                match b.state {
                    "IDLE" => Idle::execute(i, &stack, &mut blocks),
                    "FALL" => Fall::execute(i, &stack, &mut blocks),
                    "LAND" => Land::execute(i, &stack, &mut blocks),
                    "CLEAR" => Clear::execute(i, &stack, &mut blocks),
                    "SWAP" => Swap::execute(i, &stack, &mut blocks),
                    _ => (),
                }

                // if the counter is at 0, call current states counter end function
                if b.counter <= 0 {
                    match b.state {
                        "HANG" => Hang::counter_end(i, &stack, &mut blocks),
                        "FALL" => Fall::counter_end(i, &stack, &mut blocks),
                        "LAND" => Land::counter_end(i, &stack, &mut blocks),
                        "CLEAR" => Clear::counter_end(i, &stack, &mut blocks),
                        "SWAP" => Swap::counter_end(i, &stack, &mut blocks),
                        _ => (),
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
            update_sprites(&stack, &mut blocks, &mut sprites, &mut hiddens);
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
) {
    for i in 0..BLOCKS {
        let x = blocks.get(stack[i]).unwrap().x;
        let top = blocks
            .get(stack[(x as usize, ROWS_VISIBLE - 1)])
            .unwrap()
            .clone();
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
                    if top.kind != -1 && top.state == "IDLE" {
                        b.anim_offset = 4;
                    } else if b.y == 0 {
                        b.anim_offset = 1;
                    }
                }
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
pub fn check_for_hang(i: usize, stack: &Stack, blocks: &mut WriteStorage<'_, Block>) -> bool {
    // condition based on another block in a different lifetime
    let mut down_condition: bool = false;

    // check if is in vec boundary
    if i > COLUMNS {
        let down = blocks.get_mut(stack[i - COLUMNS]).unwrap();
        down_condition = down.is_empty() || down.state == "HANG";
    }

    !blocks.get_mut(stack[i]).unwrap().is_empty() && down_condition
}
