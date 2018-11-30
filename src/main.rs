extern crate amethyst;
extern crate rand;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, TransformBundle},
    input::InputBundle,
    prelude::*,
    renderer::*,
    utils::application_root_dir,
};
use std::time::Duration;

mod block_states;
mod components;
mod data;
mod game_modes;
mod systems;
use game_modes::game_mode::GameMode;
use systems::{
    block_system::BlockSystem,
    cursor::{cursor_action_system::CursorActionSystem, cursor_move_system::CursorMoveSystem},
    playfield::{clear_system::ClearSystem, lose_system::LoseSystem, push_system::PushSystem},
};

// static seed for rand crate that can be used to have the same rand seed - good for debugging
const SOME_SEED: [u8; 16] = [0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

fn main() -> amethyst::Result<()> {
    // log only warnings to create less logs
    let mut log = amethyst::LoggerConfig::default();
    log.level_filter = amethyst::LogLevelFilter::Warn;
    amethyst::start_logger(log);

    // necessary to get users path on each separate device
    let app_root = application_root_dir();
    // path to display settings
    let path = format!("{}/src/configs/display_config.ron", app_root);
    let display_config = DisplayConfig::load(&path);

    // start pipeline that clears to white background
    // and lets sprites exist with transparency
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1.0, 1.0, 1.0, 1.0], 1.0)
            .with_pass(DrawSprite::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite),
            )),
    );

    // create some randomized seed to be shared
    let mut rand_seed: [u8; 16] = [0; 16];
    for x in &mut rand_seed {
        *x = rand::random::<u8>();
    }

    // testing different inputs for keyboard/controller
    let binding_path = {
        if cfg!(feature = "sdl_controller") {
            format!("{}/src/configs/input_controller.ron", app_root)
        } else {
            format!("{}/src/configs/input.ron", app_root)
        }
    };

    // load input settings
    let input_bundle =
        InputBundle::<String, String>::new().with_bindings_from_file(&binding_path)?;

    // build with all bundles and custom systems
    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderBundle::new(pipe, Some(display_config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&["transform_system"]),
        )?.with_bundle(input_bundle)?
        //.with(FPSSystem, "fps_system", &[])
        .with(BlockSystem {}, "block_system", &[])
        .with(CursorMoveSystem {}, "cursor_move_system", &["input_system"])
        .with(
            CursorActionSystem {},
            "cursor_action_system",
            &["input_system"],
        ).with(PushSystem {}, "playfield_push_system", &[])
        .with(ClearSystem {}, "playfield_clear_system", &[])
        .with(LoseSystem {}, "playfield_lose_system", &[]);

    // set the assets dir where all sprites will be loaded from
    let assets_dir = format!("{}/src/sprites/", app_root);
    let display_resource = DisplayConfig::load(&path);
    Application::build(assets_dir, GameMode::new(SOME_SEED, display_resource))?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(1)),
            60,
        ).build(game_data)?
        .run();

    Ok(())
}
