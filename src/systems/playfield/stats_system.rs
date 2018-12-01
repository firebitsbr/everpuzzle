use amethyst::{ecs::*, input::InputHandler};
use components::playfield::stats::Stats;
use resources::playfield_resource::PlayfieldResource;

// all actions we want to count as actions per minute
const CURSOR_ACTIONS: [&'static str; 6] = ["up", "down", "left", "right", "swap", "raise"];

pub struct StatsSystem;

impl<'a> System<'a> for StatsSystem {
    type SystemData = (
        WriteStorage<'a, Stats>,
        Read<'a, InputHandler<String, String>>,
        Write<'a, PlayfieldResource>,
    );

    fn run(&mut self, (mut stats, inputs, mut playfield): Self::SystemData) {
        // increase all stats action counters by looking through all
        // actions possible and adding them up if theyre pressed once
        for stat in (&mut stats).join() {
            for action in &CURSOR_ACTIONS {
                if stat.keys.press(&inputs, action) {
                    stat.action_counter += 1.0;
                }
            }

            // increase stat times and calculate apm
            // TODO: this could crash when run too long <01-12-18, Skytrias> //
            stat.current_time += 1.0;
            stat.actions_per_minute = stat.action_counter / stat.current_time * 3600.0;

            // increase the difficulty of the playfield over time
            // each fifteen seconds the game gets harder
            if stat.current_time % (60.0 * 15.0) == 0.0 {
                // only increase until the max level is reached
                if playfield.level < 9 {
                    playfield.level += 1;
                }
            }
        }
    }
}
