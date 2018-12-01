use amethyst::ecs::*;
use components::playfield::{lose::Lose, push::Push};
use data::playfield_data::STOP_TIME;
use resources::playfield_resource::PlayfieldResource;
pub struct LoseSystem;

impl<'a> System<'a> for LoseSystem {
    type SystemData = (
        WriteStorage<'a, Lose>,
        ReadStorage<'a, Push>,
        Read<'a, PlayfieldResource>,
    );

    fn run(&mut self, (mut loses, pushes, playfield): Self::SystemData) {
        for (lose, push) in (&mut loses, &pushes).join() {
            if push.any_top_blocks && !push.any_clears {
                if lose.counter > STOP_TIME[playfield.level] {
                    lose.lost = true;
                } else {
                    // count up until stoptime is reached
                    lose.counter += 1;
                }
            }
        }

        // reset lose time frame counting each time a clear happens
        for (lose, push) in (&mut loses, &pushes).join() {
            if !push.any_top_blocks && push.any_clears {
                lose.counter = 0;
            }
        }

        // maybe reset the game for now
        for lose in (&loses).join() {
            if lose.lost {
                println!("You lost the game");
            }
        }
    }
}
