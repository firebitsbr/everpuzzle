use crate::helpers::*;
use crate::scripts::*;
use Components::*;

// NOTE(Skytrias): could make bottom ones different

pub enum Components {
    Empty,
    Normal(Block),
    Placeholder, // used to differentiate between empty
}

impl Components {
    // returns 1 if the component is currently visible, used for drawing
    pub fn visible(&self) -> f32 {
        match self {
            Empty => -1.,
            _ => 1.,
        }
    }

    // the vframe of the component, used for drawing
    pub fn vframe(&self) -> f32 {
        match self {
            Normal(b) => b.vframe,
            _ => -1.,
        }
    }

    // returns the x_offset in the grid, used for drawing
    pub fn x_offset(&self) -> f32 {
        match self {
            Normal(b) => b.offset.x,
            _ => 0.,
        }
    }

    // returns the x_offset in the grid, used for drawing
    pub fn y_offset(&self) -> f32 {
        match self {
            Normal(b) => b.offset.y,
            _ => 0.,
        }
    }

    // returns the scale in the grid, used for drawing
    pub fn scale(&self) -> f32 {
        match self {
            Normal(b) => b.scale,
            _ => 0.,
        }
    }

    // call updates on the component
    pub fn update(&mut self) {
        match self {
            Normal(b) => b.update(),
            _ => {}
        }
    }

    // call reset on any component
    pub fn reset(&mut self) {
        match self {
            Normal(b) => b.reset(),
            _ => {}
        }
    }

    // returns true if component is a placeholder
    pub fn is_placeholder(&self) -> bool {
        match self {
            Placeholder => true,
            _ => false,
        }
    }

    // returns true if the component is something real,
    pub fn is_some(&self) -> bool {
        match self {
            Empty => false,
            Placeholder => false,
            _ => true,
        }
    }

    // returns true if the component is nothing
    pub fn is_none(&self) -> bool {
        match self {
            Empty => true,
            // NOTE(Skytrias): include placeholder?
            _ => false,
        }
    }
}
