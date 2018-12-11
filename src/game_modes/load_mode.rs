use amethyst::{
    core::{cgmath::Vector3, GlobalTransform, Transform},
    ecs::prelude::Entity,
    prelude::*,
    renderer::*,
};

enum LoadState {
    Loading,
    Loaded,
}

pub struct LoadMode {
    state: LoadState
}

impl<'a, 'b> SimpleState<'a, 'b> for LoadMode {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;


    }
}
