use crate::engine::App;
use crate::helpers::*;
use crate::scripts::{BlockState, Component, Grid};
use gilrs::Button;
use std::collections::VecDeque;
use ultraviolet::Lerp;
use miniquad::KeyCode;

/// amount of frames it takes for the fast cursor movement to happen
const FRAME_LIMIT: u32 = 25;
/// amount of frames it takes to animate till the next cursor vframe appears
const ANIMATION_TIME: u32 = 64;
/// amount of frames it takes to lerp from one to the other cursor position
const LERP_TIME: u32 = 8;
/// amount of frames it takes to start an ai step
const START_DELAY_TIME: u32 = 10;
/// amount of frames it takes after the end of an ai step
const END_DELAY_TIME: u32 = 10;

pub enum CursorState {
    Idle,
    MoveSwap {
        counter: u32,
        goal: I2,
    },
    MoveTransport {
        counter: u32,
        reached: bool,
        swap_end: bool,
		start: I2,
        goal: I2,
    },
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
    //pub state: CursorState,
    pub states: VecDeque<CursorState>,

    /// delay the ai for lower speed
    pub start_delay: u32,
    pub end_delay: u32,
}

pub fn move_to(counter: &mut u32, current: &mut I2, goal: I2) -> bool {
    *counter += 1;

    if *counter == 1 || *counter > FRAME_LIMIT {
        if current.y != goal.y {
            if current.y < goal.y {
                current.y += 1;
            } else {
                current.y -= 1;
            }

            return true;
        }

        if current.x != goal.x {
            if current.x < goal.x {
                current.x += 1;
            } else {
                current.x -= 1;
            }

            return true;
        }
    }

    false
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
            states: VecDeque::new(),
            start_delay: 0,
            end_delay: 0,
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
        let left = app.kb_down_frames(KeyCode::Left, Button::DPadLeft);
        let right = app.kb_down_frames(KeyCode::Right, Button::DPadRight);
        let up = app.kb_down_frames(KeyCode::Up, Button::DPadUp);
        let down = app.kb_down_frames(KeyCode::Down, Button::DPadDown);

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

        if app.key_pressed(KeyCode::S)
            || app.button_pressed(Button::South)
            || app.button_pressed(Button::East)
        {
            self.swap_blocks(components);
        }

        // TODO(Skytrias): REMOVE ON RELEASE, only used for debugging faster
        if app.key_pressed(KeyCode::A) {
            let index = self.position.to_index();
            components.swap(index, index - GRID_WIDTH);
        }
    }

    pub fn update_ai(&mut self, app: &App, components: &mut Vec<Component>) {
        if self.end_delay > 0 {
            self.end_delay -= 1;
            return;
        }

        if self.states.is_empty() {
            return;
        }

        if let Some(state) = self.states.get_mut(0) {
            if self.start_delay < START_DELAY_TIME {
                self.start_delay += 1;
                return;
            }

            match state {
                CursorState::Idle => {
                    if self.states.len() == 1 {
                        self.end_delay = END_DELAY_TIME;
                    }

                    self.states.pop_front();

                    self.start_delay = 0;
                    return;
                }

                CursorState::MoveSwap { counter, goal } => {
                    move_to(counter, &mut self.position, *goal);

                    if self.position == *goal {
                        *state = CursorState::Idle;
                        self.swap_blocks(components);
                    }
                }

                CursorState::MoveTransport {
                    counter,
                    reached,
                    start,
                    goal,
					swap_end
                } => {
					// restarts state to idle if any clear occurs
					// TODO(Skytrias): make standalone
					for i in 0..GRID_TOTAL {
						if let Component::Block { state: block_state, .. } = &components[i] {
							if let BlockState::Clear { counter, .. } = block_state {
								if *counter == 0 {
									self.states.clear();
									self.end_delay = END_DELAY_TIME;
									return;
								}
							}
						}
					}
					
					let (x_start, y_start) = (start.x, start.y);

                    if !*reached {
                        if self.position == *goal {
                            *reached = true;
                            *counter = 0;
                            self.start_delay = 0;
                        } else {
                            move_to(counter, &mut self.position, *goal);
                        }
                    } else {
                        if *counter < FRAME_LIMIT {
                            *counter += 1;
                        } else {
                            if self.position == *start {
                                let should_swap = *swap_end;
								*state = CursorState::Idle;
                                
								// only swap at end if wanted
								if should_swap {
								self.swap_blocks(components);
								}
								
									return;
                            }

                            *counter = 0;

                            if self.position.y != y_start {
                                self.swap_blocks(components);

                                if self.position.y < y_start {
                                    self.position.y += 1;
                                } else {
                                    self.position.y -= 1;
                                }

                                return;
                            }

                            if self.position.x != x_start {
                                self.swap_blocks(components);

                                if self.position.x < x_start {
                                    self.position.x += 1;
                                } else {
                                    self.position.x -= 1;
                                }

                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    // draws the cursor sprite into the app
    pub fn draw(&mut self, app: &mut App, offset: V2) {
        self.sprite.position = self.goal_position.lerp(
            self.sprite.position,
            self.goal_counter as f32 / LERP_TIME as f32,
        );
        self.sprite.hframe = (self.counter as f32 / 32.).floor() as u32 * 3;
        self.sprite.offset = offset + V2::new(-16., self.y_offset - ATLAS_TILE / 2.);
        app.push_sprite(self.sprite);

        if self.ai {
            if let Some(state) = self.states.get_mut(0) {
                match state {
                    CursorState::MoveSwap { goal, .. } => {
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

                let text = match state {
                    CursorState::Idle => "I",
                    CursorState::MoveSwap { .. } => "M",
                    CursorState::MoveTransport { .. } => "T",
                };
				
				/*
					app.push_section(Section {
						text,
						screen_position: (
							offset.x + self.sprite.position.x - ATLAS_TILE,
							offset.y + self.sprite.position.y - ATLAS_TILE,
						),
						scale: wgpu_glyph::Scale { x: 40., y: 40. },
						color: [0.1, 0.1, 0.1, 1.0],
						..Default::default()
					});
				*/
	}
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
	