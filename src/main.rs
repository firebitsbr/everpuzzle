extern crate amethyst;
extern crate rand;
#[macro_use]
extern crate serde_derive;

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
mod resources;
mod systems;
use crate::{
    game_modes::GameMode,
    resources::Playfields,
    systems::{
        cursor::{CursorActionSystem, CursorMoveSystem},
        playfield::{ClearSystem, LoseSystem, PushSystem, StatsSystem},
        BlockSystem,
    },
};

fn main() -> amethyst::Result<()> {
    // log only warnings to create less logs
    let mut log = amethyst::LoggerConfig::default();
    log.level_filter = amethyst::LogLevelFilter::Warn;
    amethyst::start_logger(log);

    println!("Change the amount of players in the playfield_config.ron");
    println!("Controller / Keyboard Inputs can be changed in input.ron");

    // necessary to get users path on each separate device
    let app_root = application_root_dir()?;
    // path to display settings
    let display_path = app_root.join("src/configs/display_config.ron");
    let display_config = DisplayConfig::load(&display_path);
    let playfield_path = app_root.join("src/configs/playfield_config.ron");
    let playfield_config = Playfields::load(&playfield_path);

    // start pipeline that clears to white background
    // and lets sprites exist with transparency
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1.0, 1.0, 1.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite),
            )),
    );

    // testing different inputs for keyboard/controller
    let binding_path = app_root.join("src/configs/input.ron");

    // load input settings
    let input_bundle =
        InputBundle::<String, String>::new().with_bindings_from_file(&binding_path)?;

    // build with all bundles and custom systems
    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderBundle::new(pipe, Some(display_config.clone()))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&["transform_system"]),
        )?
        .with_bundle(input_bundle)?
        //.with(FPSSystem, "fps_system", &[])
        .with(BlockSystem {}, "block_system", &[])
        .with(CursorMoveSystem {}, "cursor_move_system", &["input_system"])
        .with(
            CursorActionSystem {},
            "cursor_action_system",
            &["input_system"],
        )
        .with(PushSystem {}, "playfield_push_system", &[])
        .with(ClearSystem {}, "playfield_clear_system", &[])
        .with(LoseSystem {}, "playfield_lose_system", &[])
        .with(StatsSystem {}, "playfield_stats_system", &["input_system"]);

    // set the assets dir where all sprites will be loaded from
    let assets_dir = app_root.join("src/sprites");
    Application::build(assets_dir, GameMode::new())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(1)),
            60,
        )
        .with_resource(display_config)
        .with_resource(playfield_config)
        .build(game_data)?
        .run();

    Ok(())
}
