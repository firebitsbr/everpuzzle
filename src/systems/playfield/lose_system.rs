use amethyst::ecs::*;
use components::playfield::{lose::Lose, push::Push};

pub struct LoseSystem;

const STOP_TIME: u32 = 121;

impl<'a> System<'a> for LoseSystem {
    type SystemData = (
		WriteStorage<'a, Lose>,
		WriteStorage<'a, Push>,
	);

    fn run(&mut self, (mut loses, mut pushes): Self::SystemData) {
		for (lose, push) in (&mut loses, &mut pushes).join() {
			if push.any_top_blocks && !push.any_clears {
				if lose.counter > STOP_TIME {
					lose.lost = true;
				}
			}

			lose.counter += 1;
		}

		// reset lose time frame counting each time a clear happens
		for (lose, push) in (&mut loses, &mut pushes).join() {
			if !push.any_top_blocks && push.any_clears {
				lose.counter = 0;
			}
		}

		// maybe reset the game for now
		for (lose, push) in (&mut loses, &mut pushes).join() {
			if lose.lost {
				println!("You lost the game");
			}
    	}
    }
}
