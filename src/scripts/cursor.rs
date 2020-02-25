use crate::engine::{App, State, EmptyChain};
use crate::helpers::*;
use crate::scripts::{Grid, SwapDirection};
use winit::event::VirtualKeyCode;
use hecs::{Entity, World};

/// amount of frames it takes for the fast cursor movement to happen
const FRAME_LIMIT: u32 = 25;
/// amount of frames it takes to animate till the next cursor vframe appears
const ANIMATION_TIME: u32 = 64;
/// amount of frames it takes to lerp from one to the other cursor position
const LERP_TIME: u32 = 8;

/// the player controls the cursor, holds sprite and position data
pub struct Cursor {
    sprite: Sprite,

    pub y_offset: f32,
    pub position: I2,
    pub last_position: I2,
    counter: u32,

    pub goal_position: V2,
    goal_counter: u32,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: I2::new(2, 5),
            goal_position: V2::zero(),
            last_position: I2::zero(),
            goal_counter: 0,
            counter: 0,
            sprite: Sprite {
                tiles: V2::new(3., 2.),
                offset: V2::new(-16., -16.),
                vframe: ATLAS_CURSOR,
                depth: 0.1,
                ..Default::default()
            },
            y_offset: 0.,
        }
    }
}

impl Cursor {
    /// input update which controls the movement of the cursor and also swapping of blocks in the grid
    pub fn update(&mut self, app: &App, grid: &mut Vec<Entity>, world: &mut World) {
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

        if self.position.x > 0 {
            if let Some(frame) = left {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.x -= 1;
                }
            }
        }

        if self.position.x < (GRID_WIDTH - 2) as i32 {
            if let Some(frame) = right {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.x += 1;
                }
            }
        }

        if self.position.y > 0 {
            if let Some(frame) = up {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.y -= 1;
                }
            }
        }

        if self.position.y < (GRID_HEIGHT - 2) as i32 {
            if let Some(frame) = down {
                if frame == 1 || frame > FRAME_LIMIT {
                    self.position.y += 1;
                }
            }
        }

        // cursor lerp animation
        {
            if self.last_position != self.position {
                self.goal_position.x = self.position.x as f32 * ATLAS_TILE;
                self.goal_position.y = self.position.y as f32 * ATLAS_TILE;
                self.goal_counter = LERP_TIME;
            }

            if self.goal_counter > 0 {
                self.goal_counter -= 1;
            }

            self.last_position = self.position;
        }

        if app.key_pressed(VirtualKeyCode::S) {
            self.swap_blocks(grid, world);
        }

        // TODO(Skytrias): REMOVE ON RELEASE, only used for debugging faster
        if app.key_pressed(VirtualKeyCode::A) {
            if let Some(index) = self.position.to_index() {
				grid.swap(index, index - GRID_WIDTH);
			}
			}
    }

    // draws the cursor sprite into the app
    pub fn draw(&mut self, app: &mut App) {
        //self.sprite.position = self.goal_position;
        self.sprite.position = V2::lerp(
            self.goal_position,
            self.sprite.position,
            self.goal_counter as f32 / LERP_TIME as f32,
        );
        self.sprite.hframe = (self.counter as f32 / 32.).floor() as u32 * 3;
        self.sprite.offset.y = self.y_offset - ATLAS_TILE / 2.;
        app.push_sprite(self.sprite);
    }

    pub fn swap_blocks(&self, grid: &Vec<Entity>, world: &mut World) {
        if let Some(index) = self.position.to_index() {
				
			
			/*
			// NOTE(Skytrias): feel of swap - might disable this to allow tricks, wont allow swap if any of the above left / right have hang
			{
				if i as i32 - 1 - GRID_WIDTH as i32 > 0 {
					if grid.block_state_check(i - 1 - GRID_WIDTH, |s| s.is_hang()) {
						return;
					}
				}
				
				if i as i32 + 1 - GRID_WIDTH as i32 > 0 {
					if grid.block_state_check(i + 1 - GRID_WIDTH, |s| s.is_hang()) {
						return;
					}
				}
			}
			*/
			
			// TODO(Skytrias): refactor to one call 
			
				let (left_swap, right_swap) = {
					let left_state = world.get::<State>(grid[index]);
					let left_empty = world.get::<EmptyChain>(grid[index]);
					let right_state = world.get::<State>(grid[index + 1]);
				let right_empty = world.get::<EmptyChain>(grid[index + 1]);
					
					let mut result_left = false;
				let mut result_right = false;
					
					if let Ok(state) = left_state {
				if let State::Idle = *state {
						if let Ok(right) = right_state {
							if let State::Idle = *right {
								result_left = true;
						}
						}
						
						if let Ok(_) = right_empty {
							result_left = true;
						}
						
						}
				}
				
				let left_state = world.get::<State>(grid[index]);
				let left_empty = world.get::<EmptyChain>(grid[index]);
				let right_state = world.get::<State>(grid[index + 1]);
				let right_empty = world.get::<EmptyChain>(grid[index + 1]);
				
				if let Ok(state) = right_state {
					if let State::Idle = *state {
						if let Ok(left) = left_state {
							if let State::Idle = *left {
								result_right = true;
							}
						}
						
						if let Ok(_) = left_empty {
							 result_right = true;
						}
						
					}
				}
				
				(result_left, result_right)
				};
					
				if left_swap {
					let left_state = world.get_mut::<State>(grid[index]);
					
					if let Ok(mut state) = left_state {
					*state = State::Swap { 
						counter: 0, 
						direction: 1,
						x_offset: 0.,
					};
				}
				}
			
				if right_swap {
				let right_state = world.get_mut::<State>(grid[index + 1]);
				
				if let Ok(mut state) = right_state {
					*state = State::Swap { 
						counter: 0, 
						direction: -1,
						x_offset: 0.,
					};
				}
				}
		}
		}
}
