use amethyst::ecs::*;
use components::{
    block::Block,
    cursor::Cursor,
    playfield::{
        clear::Clear, kind_generator::KindGenerator, lose::Lose, push::Push, stack::Stack,
        stats::Stats,
    },
};
use data::playfield_data::{BLOCKS, STOP_TIME};
use resources::playfield_resource::PlayfieldResource;

// handles the losing mechanics of the game
// counts the time up until you lose when at the top of the game
// also resets everything after lose and prints out all stats in cmd
pub struct LoseSystem;

impl<'a> System<'a> for LoseSystem {
    type SystemData = (
        WriteStorage<'a, Lose>,
        WriteStorage<'a, Push>,
        Write<'a, PlayfieldResource>,
        WriteStorage<'a, Stats>,
        WriteStorage<'a, Clear>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, Block>,
        WriteStorage<'a, KindGenerator>,
        ReadStorage<'a, Stack>,
    );

    fn run(
        &mut self,
        (
            mut loses,
            mut pushes,
            mut playfield,
            mut stats,
            mut clears,
            mut cursors,
            mut blocks,
            mut kind_gens,
            stacks,
        ): Self::SystemData,
    ) {
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
        for (lose, stat) in (&mut loses, &mut stats).join() {
            if lose.lost {
                println!("--------------------------------------");
                println!("You lost the game, here are your stats");
                println!("--------------------------------------");
                println!("Highest Chain: {}", stat.highest_chain);
                println!("Total Blocks Cleared: {}", stat.blocks_cleared);
                println!("Actions Per Minute: {}", stat.actions_per_minute.floor());
                println!(
                    "Time played: {} minutes and {} seconds",
                    // get frame counter through minutes / seconds, then mod them to only go to 60
                    (stat.current_time / 3600.0).floor() as i32 % 60,
                    (stat.current_time / 60.0).floor() as i32 % 60
                );
                println!(
                    "Start Difficulty: {}, End Difficulty: {}",
                    playfield.start_level, playfield.level
                );
                println!("--------------------------------------");

                // reset everything, same used in cursor space
                for (stack, push, clear, kind_gen) in
                    (&stacks, &mut pushes, &mut clears, &mut kind_gens).join()
                {
                    let kinds = kind_gen.create_stack(5, 8);

                    for i in 0..BLOCKS {
                        let b = blocks.get_mut(stack[i]).unwrap();
                        b.reset();
                        b.kind = kinds[i];
                    }

                    *push = Default::default();
                    *clear = Default::default();
                    *lose = Default::default();
                    *stat = Default::default();
                    playfield.level = playfield.start_level;
                    cursors.get_mut(stack.cursor_entity).unwrap().reset();
                }
            }
        }
    }
}
