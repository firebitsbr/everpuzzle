#![allow(dead_code)]

use amethyst::assets::*;
use amethyst::prelude::*;
use amethyst::renderer::*;

fn load_texture<N>(name: N, world: &World) -> TextureHandle
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

pub fn load_blocks_sprite_sheet(world: &mut World) -> SpriteRender {
    let loader = world.read_resource::<Loader>();

    const SPRITESHEET_SIZE: (u32, u32) = (128, 144);

    // Create the sprite for the paddles.
    //
    // Texture coordinates are expressed as a proportion of the sprite sheet's dimensions between
    // 0.0 and 1.0, so they must be divided by the width or height.
    //
    // In addition, on the Y axis, texture coordinates are 0.0 at the bottom of the sprite sheet and
    // 1.0 at the top, which is the opposite direction of pixel coordinates, so we have to invert
    // the value by subtracting the pixel proportion from 1.0.
    let mut sprites: Vec<Sprite> = Vec::with_capacity(9 * 8);
    for y in 0..9 {
        for x in 0..8 {
            sprites.push(Sprite::from_pixel_values(
                SPRITESHEET_SIZE.0,
                SPRITESHEET_SIZE.1,
                16,
                16,
                x * 16,
                y * 16,
                [-8.0, -8.0],
            ));
        }
    }

    // Collate the sprite layout information into a sprite sheet
    let sprite_sheet = SpriteSheet {
        texture: load_texture("blocks_orig.png", &world),
        sprites,
    };

    let sprite_sheet_handle = {
        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load_from_data(sprite_sheet, (), &sprite_sheet_store)
    };

    let sprite_render_block = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };

    sprite_render_block
}

// loads a spritesheet from a ron file
pub fn load_sprite_sheet(world: &mut World, name: &str, filename: &str) -> SpriteSheetHandle {
    let loader = world.read_resource::<Loader>();

    // spritesheet_handle return
    let sprite_sheethandle = {
        loader.load(
            filename,
            SpriteSheetFormat,
            load_texture(name, &world),
            (),
            &world.read_resource::<AssetStorage<SpriteSheet>>(),
        )
    };

    sprite_sheethandle
}
