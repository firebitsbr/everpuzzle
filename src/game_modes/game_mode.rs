use amethyst::{
    core::{cgmath::Vector3, GlobalTransform, Transform},
    ecs::prelude::Entity,
    prelude::*,
    renderer::*,
    utils::fps_counter::FPSCounter,
};
use rand::prelude::*;

use components::{
    block::Block,
    cursor::Cursor,
    playfield::{
        clear::Clear, kind_generator::KindGenerator, lose::Lose, push::Push, stack::Stack,
        stats::Stats,
    },
    spritesheet_loader::{load_sprite_sheet, SpriteSheetLoader},
};
use data::playfield_data::BLOCKS;
use resources::playfield_resource::PlayfieldResource;

pub struct GameMode;

impl GameMode {
    // creates the block entity,
    // contains the whole spritesheet with different sprites all set manually
    // transform positions the sprite
    // takes a vec of i32's that will be used to init all the kinds
    // returns a vec of all block entities that should be stored in a playfield stack
    pub fn create_blocks(&mut self, world: &mut World, kinds: Vec<i32>) -> Vec<Entity> {
        world.register::<Block>();
        let mut block_entities: Vec<Entity> = Vec::new();

        let level = world.read_resource::<PlayfieldResource>().level;

        for i in 0..BLOCKS {
            let mut trans = Transform::default();
            trans.scale = Vector3::new(4.0, 4.0, 4.0);

            // set position instantly so no weird spawn flash happens
            let (x, y) = Stack::index_to_coordinates(i);
            let mut b = Block::new(i as u32, kinds[i], x as i32, y as i32, level);

            let sprite_render_block = SpriteRender {
                sprite_sheet: SpriteSheetLoader::load_blocks_sprite_sheet(world),
                sprite_number: 0,
                flip_horizontal: false,
                flip_vertical: false,
            };

            block_entities.push(
                world
                    .create_entity()
                    .with(sprite_render_block)
                    .with(b)
                    .with(GlobalTransform::default())
                    .with(trans)
                    .build(),
            );
        }

        block_entities
    }

    // creates a cursor entity contains
    // a spritesheet set by a .ron file
    // a transform to position the sprite in the world
    // its cursor component data
    // a transparent component since the spritesheet has alpha
    fn create_cursor(&mut self, world: &mut World) -> Entity {
        // load the cursor sprite and attach its data component
        let sprite_sheet = SpriteRender {
            sprite_sheet: load_sprite_sheet(world, "cursor.png", "cursor_spritesheet.ron"),
            sprite_number: 0,
            flip_horizontal: false,
            flip_vertical: false,
        };

        // cursor transform
        let mut trans = Transform::default();
        trans.scale = Vector3::new(2.0, 2.0, 2.0);

        let cursor = Cursor::new(2.0, 5.0);

        // generate a cursor entity
        world.register::<Cursor>();
        let cursor_entity = world
            .create_entity()
            .with(sprite_sheet)
            .with(Transparent::default())
            .with(cursor)
            .with(GlobalTransform::default())
            .with(trans)
            .build();

        cursor_entity
    }

    // creates the playfield with all blocks and its cursor
    // also links all entities to the stack so that they can be
    // accessed via the playfield easily
    //
    // the goal of this function is to be able to repeat this
    // while gathering playfield amt etc from a .ron file
    fn create_playfield(&mut self, world: &mut World) {
        // create some randomized seed to be shared
        let mut rand_seed: [u8; 16] = [0; 16];
        for x in &mut rand_seed {
            *x = rand::random::<u8>();
        }

        // create random generator for random seeded numbers
        let mut kind_gen = KindGenerator {
            rng: SmallRng::from_seed(rand_seed),
        };
        let kinds = kind_gen.create_stack(5, 8);

        // generate other entities
        let block_entities = self.create_blocks(world, kinds);
        let cursor_entity = self.create_cursor(world);

        // Create a Playfield with a stack, clear, push, lose and kind generator
        // STACK gives access to blocks and cursor dependant on the general storages
        world.register::<Stack>();
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
            .with(Stack::new(block_entities, cursor_entity))
            .with(kind_gen)
            .with(Stats::default())
            .build();
    }

    // create a camera that should have the same dimensions as the
    // display_config.ron. TODO: use the dimensions
    fn initialise_camera(&mut self, world: &mut World) {
        let mut transform = Transform::default();
        transform.translation.z = 1.0;

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
                dimensions.1 as f32,
                0.0,
            ))).with(transform)
            .build();
    }
}

impl<'a, 'b> SimpleState<'a, 'b> for GameMode {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        self.create_playfield(world);
        //world.add_resource::<FPSCounter>(Default::default());
        self.initialise_camera(world);
    }
}
