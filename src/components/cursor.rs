use amethyst::{
    core::Transform,
    ecs::{Component, DenseVecStorage, Read},
    input::InputHandler,
};
use std::collections::HashMap;

pub struct Cursor {
    pub x: f32,
    pub y: f32,
    pub anim_offset: f32,
    pub offset: (f32, f32),
    pub key_presses: HashMap<&'static str, i32>,
}

impl Default for Cursor {
    fn default() -> Cursor {
        let mut key_presses: HashMap<&'static str, i32> = HashMap::new();
        key_presses.insert("up", 0);
        key_presses.insert("down", 0);
        key_presses.insert("right", 0);
        key_presses.insert("left", 0);
        key_presses.insert("swap", 0);
        key_presses.insert("space", 0);

        Cursor {
            x: 2.0,
            y: 6.0,
            anim_offset: 0.0,
            offset: (0.0, 0.0),
            key_presses,
        }
    }
}

impl Cursor {
    pub fn new(x: f32, y: f32) -> Cursor {
        Cursor {
            x,
            y,
            ..Default::default()
        }
    }

    pub fn set_position(&self, transform: &mut Transform) {
        transform.translation.x = (self.x * 32.0 + self.offset.0) * transform.scale.x;
        transform.translation.y = (self.y * 32.0 + self.offset.1) * transform.scale.y;
    }

    // looks whether an action is held down, good for controller support later
    pub fn hold(&mut self, input: &Read<InputHandler<String, String>>, name: &str) -> bool {
        let ticks: &mut i32 = self.key_presses.get_mut(name).unwrap();

        if input.action_is_down(name).unwrap() {
            if *ticks == 0 || *ticks > 16 {
                *ticks += 1;
                return true;
            }

            *ticks += 1;
            return false;
        }

        *ticks = 0;
        return false;
    }

    // looks whether an action is only pressed once, good for controller support later
    pub fn press(&mut self, input: &Read<InputHandler<String, String>>, name: &str) -> bool {
        let ticks: &mut i32 = self.key_presses.get_mut(name).unwrap();

        if input.action_is_down(name).unwrap() {
            if *ticks == 0 {
                *ticks = 1;
                return true;
            }

            return false;
        }

        *ticks = 0;
        return false;
    }
}

impl Component for Cursor {
    type Storage = DenseVecStorage<Self>;
}
