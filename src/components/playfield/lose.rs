use amethyst::ecs::{Component, DenseVecStorage};

pub struct Lose {
    pub lost: bool,
    pub counter: u32,
}

impl Default for Lose {
    fn default() -> Lose {
        Lose {
            lost: false,
            counter: 0,
        }
    }
}

impl Component for Lose {
    type Storage = DenseVecStorage<Self>;
}
