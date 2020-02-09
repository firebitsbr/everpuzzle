use crate::scripts::*;
use crate::helpers::*;
use Components::*;

// NOTE(Skytrias): could make bottom ones different

/*
pub struct Child {
	pub parent: usize,
}
*/

pub enum Components {
    Empty,
    Normal(Block),
    GarbageParent(Garbage),
    GarbageChild(usize), // index to parent
    Placeholder,         // used to differentiate between empty
}

impl Components {
    // returns 1 if the component is currently visible, used for drawing
    pub fn visible(&self) -> i32 {
        match self {
            Empty => -1,
            _ => 1,
        }
    }
	
    // the vframe of the component, used for drawing
    pub fn hframe(&self) -> u32 {
        match self {
            Normal(b) => b.hframe,
            GarbageParent(g) => g.hframe,
			_ => 0,
        }
    }
	
    // the vframe of the component, used for drawing
    pub fn vframe(&self) -> u32 {
        match self {
            Normal(b) => b.vframe,
            GarbageParent(g) => g.vframe,
			
            // TODO(Skytrias): depends on the parent!
            GarbageChild(_) => ATLAS_GARBAGE as u32,
			
            _ => 0,
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
            GarbageParent(_) => 1.,
            GarbageChild(_) => 1.,
            _ => 0.,
        }
    }
	
    // call updates on the component
    pub fn update(&mut self) {
        match self {
            Normal(b) => b.update(),
            GarbageParent(g) => g.update(),
            _ => {}
        }
    }
	
    // call reset on any component
    pub fn reset(&mut self) {
        match self {
            Normal(b) => b.reset(),
            GarbageParent(g) => g.reset(),
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
	
    // returns true if component is a garbage parent
    pub fn is_garbage(&self) -> bool {
        match self {
            GarbageParent(_) => true,
            _ => false,
        }
    }
	
    // returns true if component is a garbage child
    pub fn is_garbage_child(&self) -> bool {
        match self {
            GarbageChild(_) => true,
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
        !self.is_some()
    }
	
    // returns true if the component is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Empty => true,
            _ => false,
        }
    }
	
    // data used to send to gpu
    pub fn to_grid_block(&self) -> GridBlock {
        GridBlock {
            hframe: self.hframe(),
            vframe: self.vframe(),
            visible: self.visible(),
            scale: self.scale(),
            x_offset: self.x_offset(),
            y_offset: self.y_offset(),
            ..Default::default()
        }
    }
}
