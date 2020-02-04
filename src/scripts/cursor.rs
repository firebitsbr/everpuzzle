use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use winit::event::VirtualKeyCode;

const FRAME_LIMIT: u32 = 25;

pub struct Cursor {
    position: V2,
    sprite: Sprite,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: v2(2., 5.),
            sprite: Sprite {
                depth: 0.1,
                position: v2(200., 200.),
                dimensions: v2(ATLAS_TILE * 2., ATLAS_TILE),
                vframe: ATLAS_CURSOR,
                ..Default::default()
            },
        }
    }
}

impl Cursor {
    pub fn update(&mut self, app: &App, grid: &mut Grid) {
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

        if self.position.y < (GRID_HEIGHT - 1) as f32 {
            if let Some(frame) = down {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.y += 1.;
                }
            }
        }

        if app.key_pressed(VirtualKeyCode::S) {
            // safe for no_bounds since the cursor is limited to the grid indexes
            let i = self.position.raw();

            // look for valid state or check if the spot is empty
            // TODO(Skytrias): make grid.block_real() -> bool?
            let left_state = grid.block_state(i).filter(|s| s.is_real()).is_some();
            let left_empty = grid.block(i).is_none();
            let right_state = grid.block_state(i + 1).filter(|s| s.is_real()).is_some();
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

    pub fn draw(&mut self, app: &mut App) {
        self.sprite.position = ATLAS_TILE * self.position;
        app.push_sprite(self.sprite);
    }
}
