use amethyst::ecs::{Component, DenseVecStorage};
use components::key_hash_map::KeyHashMap;

pub struct Stats {
    pub highest_chain: u32,
    pub blocks_cleared: u32,
    pub actions_per_minute: f32,
    pub action_counter: f32,
    pub keys: KeyHashMap,
    pub current_time: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            highest_chain: 1,
            blocks_cleared: 0,
            actions_per_minute: 0.0,
            action_counter: 0.0,
            keys: KeyHashMap::default(),
            current_time: 0.0,
        }
    }
}

impl Component for Stats {
    type Storage = DenseVecStorage<Self>;
}
