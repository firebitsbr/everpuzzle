use crate::engine::App;
use crate::scripts::*;
use crate::helpers::*;
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
			}
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
			let index = self.position.to_index().unwrap_or(0);
			let index_right = self.position.to_index().unwrap_or(0) + 1;
			
			if let Components::Normal(b) = &mut grid[index] {
				if b.state.is_real() {
					b.state = BlockStates::Swap(0, SwapDirection::Right, false);
				}
			}
			
			if let Components::Normal(b) = &mut grid[index_right] {
				if b.state.is_real() {
					b.state = BlockStates::Swap(0, SwapDirection::Left, false);
				}
			}
		}
	}
	
	pub fn draw(&mut self, app: &mut App) {
		self.sprite.position = ATLAS_TILE * self.position;
		app.push_sprite(self.sprite);
	}
}



