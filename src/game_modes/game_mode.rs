use amethyst::{
    core::{nalgebra::Vector3, GlobalTransform, Transform},
    ecs::prelude::Entity,
    prelude::*,
    renderer::*,
};
use components::{
    block::Block,
    cursor::Cursor,
    playfield::{
        clear::Clear,
        kind_generator::{generate_random_seed, KindGenerator},
        lose::Lose,
        push::Push,
        stack::Stack,
        stats::Stats,
    },
    playfield_id::PlayfieldId,
    spritesheet_loader::{load_blocks_sprite_sheet, load_sprite_sheet},
};
use data::playfield_data::BLOCKS;
use resources::playfield_resource::Playfields;

pub struct GameMode;

impl GameMode {
    pub fn new() -> GameMode {
        GameMode {}
    }

    // creates a cursor entity contains
    // a spritesheet set by a .ron file
    // a transform to position the sprite in the world
    // its cursor component data
    // a transparent component since the spritesheet has alpha
    fn create_cursor(&mut self, p_id: usize, world: &mut World) -> Entity {
        // load the cursor sprite and attach its data component
        let sprite_sheet = SpriteRender {
            sprite_sheet: load_sprite_sheet(world, "cursor.png", "cursor_spritesheet.ron"),
            sprite_number: 0,
        };

        // cursor scale should be half of the blocks, since its twice as big
        let scale: (f32, f32) = {
            let temp_scale = world.read_resource::<Playfields>().scale.unwrap();
            (temp_scale.0 / 2.0, temp_scale.1 / 2.0)
        };

        // cursor transform
        let mut trans = Transform::default();
        trans.set_scale(scale.0, scale.1, 1.0);

        let cursor = Cursor::new(p_id, 2.0, 5.0);

        // generate a cursor entity
        world.register::<Cursor>();
        let cursor_entity = world
            .create_entity()
            .with(sprite_sheet)
            .with(Transparent::default())
            .with(cursor)
            .with(GlobalTransform::default())
            .with(trans)
            .with(PlayfieldId::new(p_id))
            .build();

        cursor_entity
    }

    // creates the playfield with all blocks and its cursor
    // also links all entities to the stack so that they can be
    // accessed via the playfield easily
    //
    // the goal of this function is to be able to repeat this
    // while gathering playfield amt etc from a .ron file
    fn create_playfield(
        &mut self,
        p_id: usize,
        kind_gen: KindGenerator,
        block_entities: Vec<Entity>,
        world: &mut World,
    ) {
        // generate other entities
        let cursor_entity = self.create_cursor(p_id, world);

        // Create a Playfield with a stack, clear, push, lose and kind generator
        // STACK gives access to blocks and cursor dependant on the general storages
        world.register::<PlayfieldId>();
        world.register::<Clear>();
        world.register::<Push>();
        world.register::<Lose>();
        world.register::<KindGenerator>();
        world.register::<Stats>();
        world
            .create_entity()
            .with(Clear::default())
            .with(Push::default())
            .with(Lose::default())
            .with(Stack::new(p_id, block_entities, cursor_entity))
            .with(kind_gen)
            .with(Stats::new(p_id))
            .with(PlayfieldId::new(p_id))
            .build();
    }

    // create a camera that should have the same dimensions as the
    // display_config.ron. TODO: use the dimensions
    fn initialise_camera(&mut self, world: &mut World) {
        let mut transform = Transform::default();
        transform.translate_z(1.0);

        // get dimensions from main.rs display config
        let dimensions = {
            let config = &world.read_resource::<DisplayConfig>();
            config.dimensions.unwrap()
        };

        world
            .create_entity()
            .with(Camera::from(Projection::orthographic(
                0.0,
                dimensions.0 as f32,
                0.0,
                dimensions.1 as f32,
            )))
            .with(transform)
            .build();
    }
}

impl SimpleState for GameMode {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        // create some randomized seed to be shared
        let random_seed = generate_random_seed();
        let mut block_entities: Vec<Entity> = Vec::new();
        let mut kind_generators: Vec<KindGenerator> = Vec::new();
        let amt = world.read_resource::<Playfields>().len();
        let block_sprite = load_blocks_sprite_sheet(world);

        // create all block entities first to distribute them
        for id in 0..amt {
            kind_generators.push(KindGenerator::new(random_seed));
            block_entities.append(&mut create_blocks(
                &block_sprite,
                id,
                world,
                kind_generators[id].create_stack(5, 8),
            ));
        }

        // go through ids and generate playfields and cursors, also set the camera
        for id in 0..amt {
            self.create_playfield(
                id,
                kind_generators[id].clone(),
                block_entities.clone(),
                world,
            );

            // save the level by the playfield_resource.ron into its struct so it can be reset to it
            {
                let mut playfields = world.write_resource::<Playfields>();
                playfields[id].start_level = playfields[id].level;
            }
        }

        self.initialise_camera(world);
    }
}

// creates the block entity,
// contains the whole spritesheet with different sprites all set manually
// transform positions the sprite
// takes a vec of i32's that will be used to init all the kinds
// returns a vec of all block entities that should be stored in a playfield stack
pub fn create_blocks(
    sprite: &SpriteRender,
    p_id: usize,
    world: &mut World,
    kinds: Vec<i32>,
) -> Vec<Entity> {
    world.register::<Block>();
    let mut block_entities: Vec<Entity> = Vec::new();

    let (level, scale): (usize, (f32, f32)) = {
        let playfield = world.read_resource::<Playfields>();
        (playfield[p_id].level, playfield.scale.unwrap())
    };

    for i in 0..BLOCKS {
        let mut trans = Transform::default();
        trans.set_scale(scale.0, scale.1, 1.0);

        // set position instantly so no weird spawn flash happens
        let (x, y) = Stack::index_to_coordinates(i);
        let mut b = Block::new(i as u32, kinds[i], x as i32, y as i32, level);

        block_entities.push(
            world
                .create_entity()
                .with(sprite.clone())
                .with(b)
                .with(GlobalTransform::default())
                .with(trans)
                .with(PlayfieldId::new(p_id))
                .build(),
        );
    }

    block_entities
}
