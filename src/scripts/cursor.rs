use crate::engine::App;
use crate::helpers::*;
use crate::scripts::*;
use winit::event::VirtualKeyCode;

const FRAME_LIMIT: u32 = 25;
const ANIMATION_TIME: u32 = 64;
const LERP_TIME: u32 = 8;

pub struct Cursor {
	position: V2,
    last_position: V2,
	sprite: Sprite,
	counter: u32,
    
	goal_position: V2,
	goal_counter: u32,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: v2(2., 5.),
            last_position: V2::zero(),
            goal_position: V2::zero(),
            goal_counter: 0,
			counter: 0,
			sprite: Sprite {
                depth: 0.1,
                position: v2(200., 200.),
                dimensions: v2(ATLAS_TILE * 3., ATLAS_TILE * 2.),
				offset: v2(-16., -16.),
                vframe: ATLAS_CURSOR,
                ..Default::default()
            },
        }
    }
}

impl Cursor {
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
		
        if self.position.y < (GRID_HEIGHT - 1) as f32 {
            if let Some(frame) = down {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.y += 1.;
                }
            }
        }
		
		// cursor lerp animation
		{
		if self.last_position != self.position {
			self.goal_position = self.position * ATLAS_TILE + v2(100., 50.);
			self.goal_counter = LERP_TIME;
		}
		
		if self.goal_counter > 0 {
			self.goal_counter -= 1;
		}
			
			self.last_position = self.position;
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
        self.sprite.position.lerp(self.goal_position, (self.goal_counter as f32 / LERP_TIME as f32));
		self.sprite.hframe = (self.counter as f32 / 32.).floor() * 3.;
        app.push_sprite(self.sprite);
		}
}
