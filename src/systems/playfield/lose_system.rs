use amethyst::ecs::*;
use components::{
    block::Block,
    cursor::Cursor,
    playfield::{
        clear::Clear,
        kind_generator::{generate_random_seed, KindGenerator},
        lose::Lose,
        push::Push,
        stack::Stack,
        stats::Stats,
    },
    playfield_id::PlayfieldId,
};
use data::playfield_data::{BLOCKS, STOP_TIME};
use resources::playfield_resource::Playfields;

// handles the losing mechanics of the game
// counts the time up until you lose when at the top of the game
// also resets everything after lose and prints out all stats in cmd
pub struct LoseSystem;

impl<'a> System<'a> for LoseSystem {
    type SystemData = (
        WriteStorage<'a, Lose>,
        WriteStorage<'a, Push>,
        Write<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
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
            mut playfields,
            ids,
            mut stats,
            mut clears,
            mut cursors,
            mut blocks,
            mut kind_gens,
            stacks,
        ): Self::SystemData,
    ) {
        for (lose, push, id) in (&mut loses, &pushes, &ids).join() {
            if push.any_top_blocks && !push.any_clears {
                if lose.counter > STOP_TIME[playfields[**id].level] {
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
        for (lose, stat, id) in (&mut loses, &mut stats, &ids).join() {
            if lose.lost {
                println!("You lost the game, here are your stats");
                println!("------------------------------------------");
                println!("You can increase the difficulty by editing");
                println!("the playfield_config.ron file .level");
                println!("------------------------------------------");
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
                    playfields[**id].start_level, playfields[**id].level
                );
                println!("------------------------------------------");

                // reset everything
                let random_seed = generate_random_seed();
                for (stack, push, clear, kind_gen) in
                    (&stacks, &mut pushes, &mut clears, &mut kind_gens).join()
                {
                    kind_gen.new_rng(random_seed);
                    let kinds = kind_gen.create_stack(5, 8);

                    // reset al blocks and set their kinds completely new
                    for i in 0..BLOCKS {
                        let b = blocks.get_mut(stack[i]).unwrap();
                        b.reset();
                        b.kind = kinds[i];
                    }

                    *push = Default::default();
                    *clear = Default::default();
                    *lose = Default::default();
                    stat.reset();

                    for i in 0..playfields.len() {
                        playfields[i].level = playfields[i].start_level;
                    }

                    cursors.get_mut(stack.cursor_entity).unwrap().reset();
                }
            }
        }
    }
}
