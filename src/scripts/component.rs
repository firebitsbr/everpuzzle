use crate::helpers::{Sprite, ATLAS_SPACING, V2};
use crate::scripts::{Block, Child};

// TODO(Skytrias): remove placeholder

/// variants that live in each grid space
pub enum Component {
    /// shows up as nothing in the grid
    Empty,

    /// normal block with unique data
    Block(Block),

    /// garbage child that lives in the grid and is linked up in a higher layer garbage  
    Child(Child),

    /// used to indicate if the component before this was a block that was cleared
    Chainable(usize),

    /// used to differentiate between empty, filler when you dont want anything to happen with the component
    Placeholder,
	}

impl Component {
    /// the vframe of the component, used for drawing
    pub fn hframe(&self) -> u32 {
        match self {
            Component::Block(b) => b.hframe,
            Component::Child(g) => g.hframe,
            _ => 0,
        }
    }

    /// the vframe of the component, used for drawing
    pub fn vframe(&self) -> u32 {
        match self {
            Component::Block(b) => b.vframe,
            Component::Child(g) => g.vframe,

            _ => 0,
        }
    }

    /// returns the offset in the grid, used for drawing
    pub fn offset(&self) -> V2 {
        match self {
            Component::Block(b) => b.offset + ATLAS_SPACING / 2.,
            Component::Child(g) => V2::new(0., g.y_offset) + ATLAS_SPACING / 2.,
            _ => V2::zero(),
        }
    }

    /// returns the scale of the component in the grid, used for drawing
    pub fn scale(&self) -> V2 {
        match self {
            Component::Block(b) => b.scale,
            Component::Child(c) => c.scale,
            _ => V2::zero(),
        }
    }

    /// call updates on the component
    pub fn update(&mut self) {
        match self {
            Component::Block(b) => b.update(),
            _ => {}
        }
    }

    /// call reset on any component
    pub fn reset(&mut self) {
        match self {
            Component::Block(b) => b.reset(),
            _ => {}
        }
    }

    /// returns true if the component is something real,
    pub fn is_block(&self) -> bool {
        match self {
            Component::Block(_) => true,
            _ => false,
        }
    }

    /// returns true if the component is something real,
    pub fn is_garbage(&self) -> bool {
        match self {
            Component::Child(_) => true,
            _ => false,
        }
    }

    /// returns true if the component is something real,
    pub fn is_some(&self) -> bool {
        match self {
            Component::Empty => false,
            Component::Placeholder => false,
            _ => true,
        }
    }

    /// returns true if the component is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Component::Empty => true,
            Component::Chainable(_) => true,
            _ => false,
        }
    }

    /// checks wether the clear state has just started counting
    pub fn clear_started(&self) -> bool {
        match self {
            Component::Block(b) => b.state.clear_started(),
            Component::Child(c) => c.counter == 0 && c.start_time != 0,
            _ => false,
        }
    }

    /// converts the current component into a Component::Child no matter what it was before
    pub fn to_garbage(&mut self, (hframe, vframe): (u32, u32)) {
        *self = Component::Child(Child {
            hframe,
            vframe,
            ..Default::default()
        })
    }

    /// converts the current component into a sprite if it is real, can be turned into a quad
    pub fn to_sprite(&self, position: V2) -> Option<Sprite> {
        if self.is_some() {
            Some(Sprite {
                position,
                hframe: self.hframe(),
                vframe: self.vframe(),
                scale: self.scale(),
                offset: self.offset(),
                centered: true,
                ..Default::default()
            })
        } else {
            None
        }
    }
}

pub trait OptionChainable {
	fn chain() -> bool;
}