use amethyst::ecs::{Component, DenseVecStorage};

pub struct Stats {
    pub highest_chain: u32,
    pub blocks_cleared: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            highest_chain: 0,
            blocks_cleared: 0,
        }
    }
}

impl Component for Stats {
    type Storage = DenseVecStorage<Self>;
}
