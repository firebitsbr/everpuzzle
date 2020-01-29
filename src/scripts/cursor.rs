use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use glutin::event::VirtualKeyCode;

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
        let left = app.key_pressed(VirtualKeyCode::Left);
        let right = app.key_pressed(VirtualKeyCode::Right);
        let up = app.key_pressed(VirtualKeyCode::Up);
        let down = app.key_pressed(VirtualKeyCode::Down);
		
        if left && self.position.x > 0. {
            self.position.x -= 1.;
        }
		
        if right && self.position.x < (GRID_WIDTH - 2) as f32 {
            self.position.x += 1.;
        }
		
        if up && self.position.y > 0. {
            self.position.y -= 1.;
        }
		
        if down && self.position.y < (GRID_HEIGHT - 1) as f32 {
            self.position.y += 1.;
        }
		
        if app.key_pressed(VirtualKeyCode::S) {
            // safe for no_bounds since the cursor is limited to the grid indexes
			let i = self.position.no_bounds();
			
			// look for valid state or check if the spot is empty
			// TODO(Skytrias): make grid.block_real() -> bool?
			let left_state = grid.block_state(i).filter(|s| s.is_real()).is_some();
			let left_empty = grid.block(i).is_none(); 
			let right_state = grid.block_state(i + 1).filter(|s| s.is_real()).is_some();
			let right_empty = grid.block(i + 1).is_none(); 
			
			if let Some(state) = grid.block_state_mut(i) {
				if left_state && (right_state || right_empty) {
					*state = BlockStates::Swap(SwapState::new(SwapDirection::Right));
				}
			}
			
			if let Some(state) = grid.block_state_mut(i + 1) {
				if right_state && (left_state || left_empty) {
					*state = BlockStates::Swap(SwapState::new(SwapDirection::Left));
				}
			}
		}
    }
	
    pub fn draw(&mut self, app: &mut App) {
        self.sprite.position = ATLAS_TILE * self.position;
        app.push_sprite(self.sprite);
    }
}
