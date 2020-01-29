use crate::helpers::*;
use num_traits::cast::ToPrimitive;

// attempt at normalizing index usage for the grid vector
// being able to use usize, V2, or any generic tuple is pretty awesome
pub trait BoundIndex {
	fn no_bounds(self) -> usize; // without bound check
	fn to_index(self) -> Option<usize>; // with bound check
}

impl BoundIndex for usize {
	// depending on where you get the usize from you're safe to use this
	fn no_bounds(self) -> usize {
		self
	}
	
	// checks upper bounds
	fn to_index(self) -> Option<usize> {
		if self < GRID_TOTAL { 
			Some(self.no_bounds())
		} else { 
			None
		}
	}
}

// NOTE(Skytrias): shouldnt use no_bounds for these grid_index implementations
impl BoundIndex for V2 {
	fn no_bounds(self) -> usize {
		self.y as usize * GRID_WIDTH + self.x as usize
	}
	
	fn to_index(self) -> Option<usize> {
		if self.x >= 0.0 && self.x < GRID_WIDTH as f32 && self.y >= 0.0 && self.y < GRID_HEIGHT as f32 {
			Some(self.no_bounds())
		} else {
			None
		}
	}
}

// TODO(Skytrias): might be really slow, check performance
// NOTE(Skytrias): shouldnt use no_bounds for these grid_index implementations
impl<T, P> BoundIndex for (T, P) 
where T: ToPrimitive, P: ToPrimitive
{
	fn no_bounds(self) -> usize {
		self.1.to_usize().unwrap_or(0) * GRID_WIDTH + self.0.to_usize().unwrap_or(0)
	}
	
	fn to_index(self) -> Option<usize> {
		let x_try = self.0.to_usize();
		let y_try = self.1.to_usize();
		
		if let Some(x) = x_try {
			if let Some(y) = y_try {
				if x < GRID_WIDTH && y < GRID_HEIGHT {
					return Some(y * GRID_WIDTH + x);
				}
			}
		} 
		
		None
	}
}