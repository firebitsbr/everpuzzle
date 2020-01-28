use crate::helpers::V2;
use glutin::event::MouseButton;
use glutin::event::ElementState;

pub struct Mouse {
	pub left_down: bool,
	pub left_pressed: bool,
	pub left_released: bool,
	last_left_down: bool,
	
	pub right_down: bool,
	pub right_pressed: bool,
	pub right_released: bool,
	last_right_down: bool,
	
	pub position: V2,
}

impl Default for Mouse {
	fn default() -> Self {
		Self {
			left_down: false,
			left_pressed: false,
			left_released: false,
			last_left_down: false,
			
			right_down: false,
			right_pressed: false,
			right_released: false,
			last_right_down: false,
			
			position: V2::zero(),
		}
	}
}

impl Mouse {
	pub fn update_frame(&mut self) {
		self.left_pressed = if self.left_down { self.last_left_down == false } else { false }; 
		self.last_left_down = self.left_down;
		self.right_pressed = if self.right_down { self.last_right_down == false } else { false }; 
		self.last_right_down = self.right_down;
	}
	
	pub fn update_event(&mut self, state: ElementState, button: MouseButton) {
		if button == MouseButton::Left {
			self.left_down = state == ElementState::Pressed;
			self.left_released = state == ElementState::Released;
		}
		
		if button == MouseButton::Right {
			self.right_down = state == ElementState::Pressed;
			self.right_released = state == ElementState::Released;
		}
	}
}