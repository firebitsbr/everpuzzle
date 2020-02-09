use crate::helpers::*;
use num_traits::cast::ToPrimitive;

// attempt at normalizing index usage for the grid vector
// being able to use usize, V2, or any generic tuple is pretty awesome
pub trait BoundIndex {
    fn raw(&self) -> usize; // without bound check
    fn in_bounds(&self) -> bool; // safety check

    // with bound check
    fn to_index(&self) -> Option<usize> {
        if self.in_bounds() {
            Some(self.raw())
        } else {
            None
        }
    }
}

impl BoundIndex for usize {
    // depending on where you get the usize from you're safe to use this
    fn raw(&self) -> usize {
        *self
    }

    // checks upper bounds
    fn in_bounds(&self) -> bool {
        *self < GRID_TOTAL
    }
}

// NOTE(Skytrias): shouldnt use no_bounds for these grid_index implementations
impl BoundIndex for V2 {
    fn raw(&self) -> usize {
        self.y as usize * GRID_WIDTH + self.x as usize
    }

    fn in_bounds(&self) -> bool {
        self.x >= 0.0 && self.x < GRID_WIDTH as f32 && self.y >= 0.0 && self.y < GRID_HEIGHT as f32
    }
}

// TODO(Skytrias): might be really slow, check performance
// NOTE(Skytrias): shouldnt use no_bounds for these grid_index implementations
impl<T, P> BoundIndex for (T, P)
where
    T: ToPrimitive,
    P: ToPrimitive,
{
    fn raw(&self) -> usize {
        self.1.to_usize().unwrap_or(0) * GRID_WIDTH + self.0.to_usize().unwrap_or(0)
    }

    fn in_bounds(&self) -> bool {
        let x_try = self.0.to_usize();
        let y_try = self.1.to_usize();

        if let Some(x) = x_try {
            if let Some(y) = y_try {
                if x < GRID_WIDTH && y < GRID_HEIGHT {
                    return true;
                }
            }
        }

        false
    }
}
