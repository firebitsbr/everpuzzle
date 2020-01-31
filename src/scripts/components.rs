use crate::scripts::*;
use Components::*;

// NOTE(Skytrias): could make bottom ones different

pub enum Components {
    Empty,
    Normal(Block),
    GarbageParent(Garbage),
	GarbageChild(usize), // index to parent
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
    pub fn hframe(&self) -> f32 {
        match self {
            Normal(b) => b.hframe,
            GarbageParent(g) => g.hframe,
			
			// TODO(Skytrias): depends on the parent!
			//GarbageChild(_) => 0.,
            
            _ => 0.,
        }
    }
	
    // the vframe of the component, used for drawing
    pub fn vframe(&self) -> f32 {
        match self {
            Normal(b) => b.vframe,
            GarbageParent(g) => g.vframe,
            
			// TODO(Skytrias): depends on the parent!
			GarbageChild(_) => 10.,
            
			_ => 0.,
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
}
