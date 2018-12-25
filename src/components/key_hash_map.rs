use crate::data::cursor_data::CURSOR_ACTIONS;
use amethyst::{ecs::Read, input::InputHandler};
use std::collections::HashMap;

// A hashmap for storing ticks on inputs
// an input signal will return true the amt of time its held down
// but usually you want to have things like presses or special behaviour
// like having it act like a press but also like a hold after a few frames
#[derive(Clone)]
pub struct KeyHashMap {
    key_presses: HashMap<&'static str, i32>,
}

impl Default for KeyHashMap {
    fn default() -> Self {
        KeyHashMap {
            key_presses: HashMap::new(),
        }
    }
}

impl KeyHashMap {
    pub fn new(id: usize) -> KeyHashMap {
        let mut key_presses: HashMap<&'static str, i32> = HashMap::new();

        // get the actions for the cursor specified by the id
        for action in &CURSOR_ACTIONS[id] {
            key_presses.insert(action, 0);
        }

        KeyHashMap { key_presses }
    }
}

impl KeyHashMap {
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
