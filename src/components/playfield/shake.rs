#![allow(dead_code)]
use amethyst::ecs::{Component, DenseVecStorage};
use rand::{thread_rng, Rng};
use resources::playfield_resource::PlayfieldResource;

// Handles visual shaking of the playfield
pub struct Shake {
    pub start: bool,
    pub range: f32,
    pub counter: usize,
    pub playfield_old_y: f32,
}

impl Default for Shake {
    fn default() -> Self {
        Shake {
            start: false,
            range: 30.0,
            counter: 0,
            playfield_old_y: 0.0,
        }
    }
}

impl Shake {
    // shakes the playfield y position randomly when shake is initiated
    // the intensity of shake can be modified by shake_amount
    pub fn animate(&mut self, playfield: &mut PlayfieldResource) {
        if self.start {
            if self.counter > 0 {
                playfield.y = thread_rng().gen_range(0.0, self.range) - self.range / 2.0;
                self.counter -= 1;
            } else {
                self.start = false;
                playfield.y = self.playfield_old_y;
            }
        } else {
            self.playfield_old_y = playfield.y;
        }
    }
}

impl Component for Shake {
    type Storage = DenseVecStorage<Self>;
}
