use crate::engine::App;
use crate::helpers::*;
use crate::scripts::{BlockState, Component, Grid};
use gilrs::Button;
use winit::event::VirtualKeyCode;
use wgpu_glyph::Section;

/// amount of frames it takes for the fast cursor movement to happen
const FRAME_LIMIT: u32 = 25;
/// amount of frames it takes to animate till the next cursor vframe appears
const ANIMATION_TIME: u32 = 64;
/// amount of frames it takes to lerp from one to the other cursor position
const LERP_TIME: u32 = 8;
/// amount of frames it takes to make an ai step
const DELAY_TIME: u32 = 20;

pub enum CursorState {
	Idle,
	Move {
		counter: u32, 
		goal: I2,
	},
	Swap,
}

/// the player controls the cursor, holds sprite and position data
pub struct Cursor {
    sprite: Sprite,

    pub y_offset: f32,
    pub position: I2,
    pub last_position: I2,
    counter: u32,

    pub goal_position: V2,
    goal_counter: u32,
	
	pub ai: bool,
	
	/// ai state
	pub state: CursorState,
	
	/// delay the ai for lower speed
	pub delay: u32,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            position: I2::new(2, 7),
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
			
			// ai
			state: CursorState::Idle,
			delay: 0,
			ai: false,
		}
    }
}

impl Cursor {
    pub fn reset(&mut self) {
		self.position = I2::new(2, 7);
		}
	
	pub fn new(ai: bool) -> Self {
		Self {
			ai,
			..Default::default()
		}
	}
	
	/// input update which controls the movement of the cursor and also swapping of blocks in the grid
    pub fn update(&mut self, app: &App, components: &mut Vec<Component>) {
        if self.counter < ANIMATION_TIME - 1 {
            self.counter += 1;
        } else {
            self.counter = 0;
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
		
		match self.ai {
			true => self.update_ai(app, components),
			 false => self.update_player(app, components),
		}
	}
	
	fn update_player(&mut self, app: &App, components: &mut Vec<Component>) {
        let left = app.kb_down_frames(VirtualKeyCode::Left, Button::DPadLeft);
        let right = app.kb_down_frames(VirtualKeyCode::Right, Button::DPadRight);
        let up = app.kb_down_frames(VirtualKeyCode::Up, Button::DPadUp);
        let down = app.kb_down_frames(VirtualKeyCode::Down, Button::DPadDown);

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

        if app.key_pressed(VirtualKeyCode::S)
            || app.button_pressed(Button::South)
            || app.button_pressed(Button::East)
        {
            self.swap_blocks(components);
        }

        // TODO(Skytrias): REMOVE ON RELEASE, only used for debugging faster
        if app.key_pressed(VirtualKeyCode::A) {
            let index = self.position.to_index();
            components.swap(index, index - GRID_WIDTH);
        }
    }
	
	pub fn update_ai(&mut self, app: &App, components: &mut Vec<Component>) {
		if self.delay < DELAY_TIME {
				self.delay += 1;
			return;
		}
			
			self.delay = 0;
				
				match self.state {
				CursorState::Idle => {
					//println!("idle");
				}
				
				CursorState::Move { mut counter, goal } => {
					//println!("move");
				counter += 1;
				
				if counter == 1 || counter > FRAME_LIMIT {
						if self.position.y != goal.y {
							if self.position.y < goal.y {
								self.position.y += 1;
							} else {
								self.position.y -= 1;
							}
							
							return;
						}
						
						if self.position.x != goal.x {
							if self.position.x < goal.x {
								self.position.x += 1;
							} else {
								self.position.x -= 1;
							}
							
							return;
						}
						
						if self.position == goal {
							self.state = CursorState::Swap;
							//self.state = CursorState::Idle;
							return;
						}
					}
				}
				
				CursorState::Swap => {
					//println!("swap");
					self.swap_blocks(components);
					self.state = CursorState::Idle;
				}
				}
	}
	
    // draws the cursor sprite into the app
    pub fn draw(&mut self, app: &mut App, offset: V2) {
        self.sprite.position = V2::lerp(
            self.goal_position,
            self.sprite.position,
            self.goal_counter as f32 / LERP_TIME as f32,
        );
        self.sprite.hframe = (self.counter as f32 / 32.).floor() as u32 * 3;
        self.sprite.offset = offset + V2::new(-16., self.y_offset - ATLAS_TILE / 2.);
        app.push_sprite(self.sprite);
		
		if self.ai {
			match self.state {
				CursorState::Move { goal, .. } => {
					let goal = V2::new(goal.x as f32, goal.y as f32);
					
					app.push_line(Line {
										  start: self.sprite.position + offset,
										  end: goal * ATLAS_SPACING + offset + ATLAS_SPACING / 2.,
										  thickness: 15.,
										  hframe: 8,
										  ..Default::default()
									  });
				}
				
				_ => {}
			}
			
			let text = match self.state {
				CursorState::Idle => "I",
				CursorState::Move { .. } => "M",
				CursorState::Swap => "S",
			};
		
		app.push_section(Section {
									 text,
									 screen_position: (offset.x + self.sprite.position.x - ATLAS_TILE, offset.y + self.sprite.position.y - ATLAS_TILE),
								 scale: wgpu_glyph::Scale { x: 40., y: 40. },
								 color: [0.1, 0.1, 0.1, 1.0],
								 ..Default::default()
							 });
		}
		}
	
    pub fn swap_blocks(&self, components: &mut Vec<Component>) {
        let i = self.position.to_index();
		
        let right = can_swap(components, i + 1);
        let left = can_swap(components, i);

        if right {
            if let Component::Block { state, .. } = &mut components[i] {
                if let BlockState::Idle = state {
                    *state = BlockState::Swap {
                        counter: 0,
                        direction: 1,
                    };
                }
            }
        }

        if left {
            if let Component::Block { state, .. } = &mut components[i + 1] {
                if let BlockState::Idle = state {
                    *state = BlockState::Swap {
                        counter: 0,
                        direction: -1,
                    };
                }
            }
        }
    }
}

/// helper to detect if a block is currently swappable - in idle state or empty
fn can_swap(components: &Vec<Component>, index: usize) -> bool {
    match &components[index] {
        Component::Block { state, .. } => {
            if let BlockState::Idle = state {
                return true;
            }
        }

        Component::Empty { .. } => {
            return true;
        }

        _ => return false,
    }

    false
}
