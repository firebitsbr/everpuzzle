use amethyst::ecs::{Component, Entity, DenseVecStorage};

// individual parts that will lay in the master
// consists of the head and its subparts
pub struct GarbageHead {
    pub head: Entity,
    pub can_fall: bool,
    pub clearing: bool,
    pub parts: Vec<Entity>,
    pub highest_blocks: Vec<Entity>,
    pub lowest_blocks: Vec<Entity>,
    pub marked_clear: bool,
    pub hanged: bool,
    pub new_kinds: Vec<Entity>
}

impl GarbageHead {
    pub fn new(
        head: Entity, 
        parts: Vec<Entity>, 
        highest_blocks: Vec<Entity>, 
        lowest_blocks: Vec<Entity>, 
    ) -> GarbageHead {
        GarbageHead {
            head,
            parts,
            lowest_blocks,
            highest_blocks,
            can_fall: false,
            clearing: false,
            marked_clear: false,
            hanged: false,
            new_kinds: Vec::new(),
        }
    }
}

impl Component for GarbageHead {
    type Storage = DenseVecStorage<Self>;
}
