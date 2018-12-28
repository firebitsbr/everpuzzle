use amethyst::ecs::*;

use crate::{
    components::{playfield::Shake, PlayfieldId},
    resources::playfield_resource::Playfields,
};

pub struct ShakeSystem;

impl<'a> System<'a> for ShakeSystem {
    type SystemData = (
        WriteStorage<'a, Shake>,
        Write<'a, Playfields>,
        ReadStorage<'a, PlayfieldId>,
    );

    fn run(&mut self, (mut shakes, mut playfields, ids): Self::SystemData) {
        for (shake, id) in (&mut shakes, &ids).join() {
            shake.animate(&mut playfields[**id]);
        }
    }
}
