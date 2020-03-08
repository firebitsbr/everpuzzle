use crate::helpers::*;
use gilrs::{
    ev::EventType::{ButtonPressed, ButtonReleased},
    Button,
};
use miniquad::KeyCode;
use std::collections::HashMap;

pub struct Input {
    /// data storage for each key that was pressed with the frame time
    key_downs: HashMap<KeyCode, u32>,

    /// data storage for each button that was pressed with the frame time
    button_downs: HashMap<Button, u32>,

    gilrs: gilrs::Gilrs,

    /// mouse handle that which holds left / right button and position info
    pub mouse: Mouse,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            key_downs: HashMap::new(),
            button_downs: HashMap::new(),

            // gamepad
            gilrs: match gilrs::GilrsBuilder::new().set_update_state(false).build() {
                Ok(g) => g,
                Err(gilrs::Error::NotImplemented(g)) => {
                    eprintln!("Current platform is not supported");

                    g
                }
                Err(e) => {
                    eprintln!("Failed to create gilrs context: {}", e);
                    std::process::exit(-1);
                }
            },

            mouse: Mouse::default(),
        }
    }
}

/// mouse that holds all info of the click states and its position
pub struct Mouse {
    pub left_down: bool,
    pub left_pressed: bool,
    pub left_released: bool,
    last_left_down: bool,

    pub right_down: bool,
    pub right_pressed: bool,
    pub right_released: bool,
    last_right_down: bool,

    pub position: V2,
}

impl Default for Mouse {
    fn default() -> Self {
        Self {
            left_down: false,
            left_pressed: false,
            left_released: false,
            last_left_down: false,

            right_down: false,
            right_pressed: false,
            right_released: false,
            last_right_down: false,

            position: V2::zero(),
        }
    }
}

impl Input {
    pub fn down_event(&mut self, keycode: KeyCode) {
        if let Some(value) = self.key_downs.get_mut(&keycode) {
            if *value == 0 {
                *value = 1;
            }
        } else {
            self.key_downs.insert(keycode, 1);
        }
    }

    pub fn up_event(&mut self, keycode: KeyCode) {
        if let Some(value) = self.key_downs.get_mut(&keycode) {
            *value = 0;
        }
    }

    /// returns true if a key is held down
    pub fn key_down(&self, code: KeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v != 0).is_some()
    }

    /// returns true the amount of frames a key has been down for
    pub fn key_down_frames(&self, code: KeyCode) -> Option<u32> {
        self.key_downs.get(&code).filter(|&&v| v != 0).copied()
    }

    /// returns true if a key is pressed for a single frame
    pub fn key_pressed(&self, code: KeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v == 1).is_some()
    }

    /// returns true if a button is held down
    pub fn button_down(&self, button: Button) -> bool {
        self.button_downs
            .get(&button)
            .filter(|&&v| v != 0)
            .is_some()
    }

    /// returns true the amount of frames a button has been down for
    pub fn button_down_frames(&self, button: Button) -> Option<u32> {
        self.button_downs.get(&button).filter(|&&v| v != 0).copied()
    }

    /// returns true if a button is pressed for a single frame
    pub fn button_pressed(&self, button: Button) -> bool {
        self.button_downs
            .get(&button)
            .filter(|&&v| v == 1)
            .is_some()
    }

    /// returns true if a button or a key is held down
    pub fn kb_down(&self, code: KeyCode, button: Button) -> bool {
        self.key_down(code) || self.button_down(button)
    }

    /// returns true the amount of frames a button or a key has been down for
    pub fn kb_down_frames(&self, code: KeyCode, button: Button) -> Option<u32> {
        let mut result = None;

        if let Some(frames) = self.key_down_frames(code) {
            result = Some(frames);
        }

        if let Some(frames) = self.button_down_frames(button) {
            if let Some(old_frames) = result {
                result = Some(old_frames.max(frames));
            } else {
                result = Some(frames);
            }
        }

        result
    }

    /// returns true if a button or a key is pressed for a single frame
    pub fn kb_pressed(&self, code: KeyCode, button: Button) -> bool {
        self.key_pressed(code) || self.button_pressed(button)
    }

    pub fn update_gamepad(&mut self) {
        while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
            match event {
                ButtonPressed(btn, _) => {
                    if let Some(value) = self.button_downs.get_mut(&btn) {
                        if *value == 0 {
                            *value = 1;
                        }
                    } else {
                        self.button_downs.insert(btn, 1);
                    }
                }

                ButtonReleased(btn, _) => {
                    if let Some(value) = self.button_downs.get_mut(&btn) {
                        *value = 0;
                    }
                }

                _ => {}
            }
        }
    }

    pub fn update_end(&mut self) {
        // increase the frame times on the keys
        for (_, value) in self.key_downs.iter_mut() {
            if *value != 0 {
                *value += 1;
            }
        }

        // increase the frame times on the keys
        for (_, value) in self.button_downs.iter_mut() {
            if *value != 0 {
                *value += 1;
            }
        }

        self.mouse.left_pressed = if self.mouse.left_down {
            !self.mouse.last_left_down
        } else {
            false
        };
        self.mouse.last_left_down = self.mouse.left_down;
        self.mouse.right_pressed = if self.mouse.right_down {
            !self.mouse.last_right_down
        } else {
            false
        };
        self.mouse.last_right_down = self.mouse.right_down;
    }
}
