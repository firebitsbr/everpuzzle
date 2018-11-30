use amethyst::{core::Transform, ecs::*, input::InputHandler, renderer::SpriteRender};

use block_states::block_state::change_state;
use components::cursor::Cursor;
use data::playfield_data::{BLOCKS, COLUMNS, ROWS_VISIBLE};

pub struct CursorMoveSystem;

impl<'a> System<'a> for CursorMoveSystem {
    type SystemData = (
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Cursor>,
        Read<'a, InputHandler<String, String>>,
    );

    fn run(&mut self, (mut sprites, mut transforms, mut cursors, input): Self::SystemData) {
        for cursor in (&mut cursors).join() {
            if cursor.hold(&input, "up") {
                if cursor.y < (ROWS_VISIBLE - 1) as f32 {
                    cursor.y += 1.0;
                }
            }

            if cursor.hold(&input, "down") {
                if cursor.y > 1.0 {
                    cursor.y -= 1.0;
                }
            }

            if cursor.hold(&input, "left") {
                if cursor.x > 0.0 {
                    cursor.x -= 1.0;
                }
            }

            if cursor.hold(&input, "right") {
                if cursor.x < (COLUMNS - 2) as f32 {
                    cursor.x += 1.0;
                }
            }
        }

        for (sprite, transform, cursor) in (&mut sprites, &mut transforms, &mut cursors).join() {
            cursor.set_position(transform);

            sprite.sprite_number = cursor.anim_offset as usize;
            if cursor.anim_offset < 7.0 {
                cursor.anim_offset += 1.0 / 4.0;
            } else {
                cursor.anim_offset = 0.0;
            }
        }
    }
}
