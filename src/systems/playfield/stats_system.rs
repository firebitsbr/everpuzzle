use amethyst::{ecs::*, input::InputHandler};
use components::{playfield::stats::Stats, playfield_id::PlayfieldId};
use resources::playfield_resource::Playfields;

// all actions we want to count as actions per minute
const CURSOR_ACTIONS: [[&'static str; 6]; 2] = [
    ["up0", "down0", "left0", "right0", "swap0", "raise0"],
    ["up1", "down1", "left1", "right1", "swap1", "raise1"],
];

pub struct StatsSystem;

impl<'a> System<'a> for StatsSystem {
    type SystemData = (
        WriteStorage<'a, Stats>,
        Read<'a, InputHandler<String, String>>,
        Write<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
    );

    fn run(&mut self, (mut stats, inputs, mut playfields, ids): Self::SystemData) {
        // increase all stats action counters by looking through all
        // actions possible and adding them up if theyre pressed once
        for (stat, id) in (&mut stats, &ids).join() {
            for action in &CURSOR_ACTIONS[**id] {
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
                if playfields[**id].level < 9 {
                    playfields[**id].level += 1;
                }
            }
        }
    }
}
