use amethyst::ecs::{Component, DenseVecStorage};
use std::ops::Deref;

// used to store the playfields id that each enitity needs to know
pub struct PlayfieldId {
    id: usize,
}

impl PlayfieldId {
    pub fn new(id: usize) -> PlayfieldId {
        PlayfieldId { id }
    }
}

impl Component for PlayfieldId {
    type Storage = DenseVecStorage<Self>;
}

impl Deref for PlayfieldId {
    type Target = usize;

    fn deref(&self) -> &usize {
        &self.id
    }
}
