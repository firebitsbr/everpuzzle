use amethyst::{
    ecs::*,
    core::{Transform, cgmath::Vector3},
    renderer::*,
    input::*
};

use basics::{
    block::Block,
    cursor::Cursor,
    rng_resource::RngResource
};

use data::{
    block_data::*, 
    helpers::tuple2i
};

use std::collections::HashMap;
use rand::prelude::*;

pub struct CursorSystem {
    key_presses: HashMap<String, i32>
}

// everything the player controls should happen here
// all actions should happen here
impl CursorSystem {
    pub fn new() -> CursorSystem {
        let mut key_presses: HashMap<String, i32> = HashMap::new();
        key_presses.insert(String::from("up"), 0);
        key_presses.insert(String::from("down"), 0);
        key_presses.insert(String::from("right"), 0);
        key_presses.insert(String::from("left"), 0);
        key_presses.insert(String::from("swap"), 0);
        key_presses.insert(String::from("space"), 0);

        CursorSystem {
            key_presses
        }
    }

    // looks wether an action is held down, good for controller support later
    pub fn hold(&mut self, input: &mut Read<InputHandler<String, String>>, name: &str) -> bool {
        if input.action_is_down(name).unwrap() {
            let result = *self.key_presses.get(name).unwrap();

            // special, detects at frame 0 and later on returns true all the 
            // time like in the real game
            if result == 0 || result > 8 {
                *self.key_presses.get_mut(name).unwrap() += 1;
                return true;
            }

            *self.key_presses.get_mut(name).unwrap() += 1;
        }
        else {
            *self.key_presses.get_mut(name).unwrap() = 0;
        }

        return false;
    }

    // looks wether an action is only pressed once, good for controller support later
    pub fn press(&mut self, input: &mut Read<InputHandler<String, String>>, name: &str) -> bool {
        if input.action_is_down(name).unwrap() {
            if *self.key_presses.get(name).unwrap() == 0 {
                *self.key_presses.get_mut(name).unwrap() = 1;
                return true;
            }
        }
        else {
            *self.key_presses.get_mut(name).unwrap() = 0;
        }

        return false;
    }
}

impl<'a> System<'a> for CursorSystem {
    type SystemData = (
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Cursor>,
        Read<'a, InputHandler<String, String>>,
        Write<'a, RngResource>,
        WriteStorage<'a, Block>,
    );

    fn run(&mut self, (
            mut sprites, 
            mut transforms, 
            mut cursors, 
            mut input,
            mut generator,
            mut blocks): Self::SystemData) 
    {
        if self.hold(&mut input, "up") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.1 < (ROWS - 1) as f32 {
                    cursor.pos.1 += 1.0;
                }
            }
        }

        if self.hold(&mut input, "down") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.1 > 0.0 {
                    cursor.pos.1 -= 1.0;
                }
            }
        }

        if self.hold(&mut input, "left") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.0 > 0.0 {
                    cursor.pos.0 -= 1.0;
                }
            }
        }

        if self.hold(&mut input, "right") {
            for cursor in (&mut cursors).join() {
                if cursor.pos.0 < (COLS - 2) as f32 {
                    cursor.pos.0 += 1.0;
                }
            }
        }

        // reset all block colors to a random value
        if self.press(&mut input, "space") {
            for block in (&mut blocks).join() {
                let num = generator.rng.gen_range(0, 7);

                if num == 6 {
                    block.kind = None;
                }
                else {
                    block.kind = Some(num);
                }
            }
        }

        // swaps block kinds around, gets all blocks, searches through creation id,
        // id matches cursor pos conversion, swapping from one block to another block
        if self.press(&mut input, "swap") {
            for cursor in (cursors).join() {
                let mut search_blocks = (&mut blocks).join();
                let mut pos = tuple2i(cursor.pos);

                let b1 = search_blocks.get_unchecked(pos as u32).unwrap();
                let b2 = search_blocks.get_unchecked((pos + 1) as u32).unwrap();

                let mut temp_kind: Option<i32> = b1.kind;
                b1.kind = b2.kind;
                b2.kind = temp_kind;
            }
        }

        for (sprite, transform, cursor) in (&mut sprites, &mut transforms, &mut cursors).join() {
            transform.translation = Vector3::new(
                cursor.pos.0 * 32.0,
                cursor.pos.1 * 32.0,
                0.0
            );
            sprite.sprite_number = cursor.anim_offset as usize;

            if cursor.anim_offset < 7.0 {
                cursor.anim_offset += 1.0 / 2.0;
            }
            else {
                cursor.anim_offset = 0.0;
            }
        }
    }
}