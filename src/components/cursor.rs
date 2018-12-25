use crate::components::key_hash_map::KeyHashMap;
use amethyst::ecs::{Component, DenseVecStorage};

// cursor that saves all key_presses ticks
pub struct Cursor {
    pub x: f32,
    pub y: f32,
    pub anim_offset: f32,
    pub offset: (f32, f32),
    pub keys: KeyHashMap,
}

impl Default for Cursor {
    fn default() -> Cursor {
        Cursor {
            x: 2.0,
            y: 6.0,
            anim_offset: 0.0,
            offset: (0.0, 0.0),
            keys: KeyHashMap::default(),
        }
    }
}

impl Cursor {
    pub fn new(p_id: usize, x: f32, y: f32) -> Cursor {
        Cursor {
            x,
            y,
            keys: KeyHashMap::new(p_id),
            ..Default::default()
        }
    }

    // reset everything but the keypresses
    pub fn reset(&mut self) {
        *self = Cursor {
            keys: self.keys.clone(),
            ..Default::default()
        };
    }
}

impl Component for Cursor {
    type Storage = DenseVecStorage<Self>;
}
