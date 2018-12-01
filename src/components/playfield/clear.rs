use amethyst::ecs::{Component, DenseVecStorage};

pub struct Clear {
    pub clear_queue: Vec<u32>,
    pub combo_counter: u32,
    pub chain: u32,
    pub last_chain: u32,
}

impl Default for Clear {
    fn default() -> Self {
        Clear {
            clear_queue: Vec::new(),
            combo_counter: 0,
            chain: 1,
            last_chain: 1,
        }
    }
}

impl Component for Clear {
    type Storage = DenseVecStorage<Self>;
}
