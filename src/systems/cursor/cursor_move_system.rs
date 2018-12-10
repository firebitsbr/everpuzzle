use amethyst::{core::Transform, ecs::*, input::InputHandler, renderer::SpriteRender};

use components::{cursor::Cursor, playfield_id::PlayfieldId};
use data::{
    cursor_data::CURSOR_ACTIONS,
    playfield_data::{COLUMNS, ROWS_VISIBLE},
};
use resources::playfield_resource::Playfields;

pub struct CursorMoveSystem;

impl<'a> System<'a> for CursorMoveSystem {
    type SystemData = (
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Cursor>,
        Read<'a, InputHandler<String, String>>,
        Read<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
    );

    fn run(
        &mut self,
        (mut sprites, mut transforms, mut cursors, input, playfields, ids): Self::SystemData,
    ) {
        for (cursor, id) in (&mut cursors, &ids).join() {
            if cursor.keys.hold(&input, CURSOR_ACTIONS[**id][0]) {
                if cursor.y < (ROWS_VISIBLE - 1) as f32 {
                    cursor.y += 1.0;
                }
            }

            if cursor.keys.hold(&input, CURSOR_ACTIONS[**id][1]) {
                if cursor.y > 1.0 {
                    cursor.y -= 1.0;
                }
            }

            if cursor.keys.hold(&input, CURSOR_ACTIONS[**id][2]) {
                if cursor.x > 0.0 {
                    cursor.x -= 1.0;
                }
            }

            if cursor.keys.hold(&input, CURSOR_ACTIONS[**id][3]) {
                if cursor.x < (COLUMNS - 2) as f32 {
                    cursor.x += 1.0;
                }
            }
        }

        for (sprite, transform, cursor, id) in
            (&mut sprites, &mut transforms, &mut cursors, &ids).join()
        {
            set_position(cursor, transform, &playfields, **id);

            sprite.sprite_number = cursor.anim_offset as usize;
            if cursor.anim_offset < 7.0 {
                cursor.anim_offset += 1.0 / 4.0;
            } else {
                cursor.anim_offset = 0.0;
            }
        }
    }
}

// sets the cursors position scaled on a grid with an offset by the playfield
fn set_position(
    cursor: &Cursor,
    transform: &mut Transform,
    playfields: &Read<'_, Playfields>,
    id: usize,
) {
    // immutable scale variables
    let (scale_x, scale_y) = {
        let scale = transform.scale();
        (scale.x, scale.y)
    };

    transform.set_x((cursor.x * 32.0 + cursor.offset.0 + playfields[id].x * 2.0) * scale_x);
    transform.set_y((cursor.y * 32.0 + cursor.offset.1 + playfields[id].y * 2.0) * scale_y);
}
