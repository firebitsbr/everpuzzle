use crate::engine::App;
use crate::helpers::*;
use crate::scripts::{Grid, SwapDirection};
use winit::event::VirtualKeyCode;

/// amount of frames it takes for the fast cursor movement to happen
const FRAME_LIMIT: u32 = 25;
/// amount of frames it takes to animate till the next cursor vframe appears
const ANIMATION_TIME: u32 = 64;
/// amount of frames it takes to lerp from one to the other cursor position
const LERP_TIME: u32 = 8;

/// the player controls the cursor, holds sprite and position data
pub struct Cursor {
    sprite: Sprite,

    pub position: V2,
    last_position: V2,
    counter: u32,

    goal_position: V2,
    goal_counter: u32,
}

impl Default for Cursor {
    fn default() -> Self {
        let start_position = V2::new(2., 5.);

        Self {
            position: start_position,
            goal_position: V2::zero(),
            last_position: V2::zero(),
            goal_counter: 0,
            counter: 0,
            sprite: Sprite {
                tiles: V2::new(3., 2.),
                offset: V2::new(-16., -16.),
                vframe: ATLAS_CURSOR,
                depth: 0.1,
                ..Default::default()
            },
        }
    }
}

impl Cursor {
    /// input update which controls the movement of the cursor and also swapping of blocks in the grid
    pub fn update(&mut self, app: &App, grid: &mut Grid) {
        if self.counter < ANIMATION_TIME - 1 {
            self.counter += 1;
        } else {
            self.counter = 0;
        }

        let left = app.key_down_frames(VirtualKeyCode::Left);
        let right = app.key_down_frames(VirtualKeyCode::Right);
        let up = app.key_down_frames(VirtualKeyCode::Up);
        let down = app.key_down_frames(VirtualKeyCode::Down);

        // movement dependant on how long a key down has been held for in frames

        if self.position.x > 0. {
            if let Some(frame) = left {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.x -= 1.;
                }
            }
        }

        if self.position.x < (GRID_WIDTH - 2) as f32 {
            if let Some(frame) = right {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.x += 1.;
                }
            }
        }

        if self.position.y > 0. {
            if let Some(frame) = up {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.y -= 1.;
                }
            }
        }

        if self.position.y < (GRID_HEIGHT - 2) as f32 {
            if let Some(frame) = down {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.y += 1.;
                }
            }
        }

        // cursor lerp animation
        {
            if self.last_position != self.position {
                self.goal_position = self.position * ATLAS_TILE;
                self.goal_counter = LERP_TIME;
            }

            if self.goal_counter > 0 {
                self.goal_counter -= 1;
            }

            self.last_position = self.position;
        }

        if app.key_pressed(VirtualKeyCode::S) {
            self.swap_blocks(grid);
        }
    }

    // draws the cursor sprite into the app
    pub fn draw(&mut self, app: &mut App) {
        self.sprite.position = V2::lerp(
            self.goal_position,
            self.sprite.position,
            self.goal_counter as f32 / LERP_TIME as f32,
        );
        self.sprite.hframe = (self.counter as f32 / 32.).floor() as u32 * 3;
        app.push_sprite(self.sprite);
    }

    pub fn swap_blocks(&self, grid: &mut Grid) {
        // safe for no_bounds since the cursor is limited to the grid indexes
        let i = self.position.raw();

        // look for valid state or check if the spot is empty
        let left_state = grid.block_state_check(i, |s| s.is_real());
        let left_empty = grid.block(i).is_none();
        let right_state = grid.block_state_check(i + 1, |s| s.is_real());
        let right_empty = grid.block(i + 1).is_none();

        if let Some(state) = grid.block_state_mut(i) {
            if left_state && (right_state || right_empty) {
                state.to_swap(SwapDirection::Right);
            }
        }

        if let Some(state) = grid.block_state_mut(i + 1) {
            if right_state && (left_state || left_empty) {
                state.to_swap(SwapDirection::Left);
            }
        }
    }
}
