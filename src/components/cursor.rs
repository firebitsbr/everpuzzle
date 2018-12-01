use amethyst::{
    core::Transform,
    ecs::{Component, DenseVecStorage, Read},
    input::InputHandler,
};
use std::collections::HashMap;

// cursor that saves all key_presses ticks
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
        key_presses.insert("raise", 0);

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

    // reset everything but the keypresses
    pub fn reset(&mut self) {
        *self = Cursor {
            key_presses: self.key_presses.clone(),
            ..Default::default()
        };
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

    // looks wether in action is just pressed down and just counts up
    pub fn down(&mut self, input: &Read<InputHandler<String, String>>, name: &str) -> bool {
        let ticks: &mut i32 = self.key_presses.get_mut(name).unwrap();

        if input.action_is_down(name).unwrap() {
            if *ticks == 0 {
                *ticks = 1;
            }

            return *ticks == 1;
        }

        *ticks = 0;
        return false;
    }
}

impl Component for Cursor {
    type Storage = DenseVecStorage<Self>;
}
