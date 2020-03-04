use crate::helpers::{Sprite, ATLAS_SPACING, V2};
use crate::scripts::{Block, BlockState, Child};

/// variants that live in each grid space
pub enum Component {
    /// shows up as nothing in the grid
    Empty { size: usize, alive: bool },

    /// normal block with unique data
    Block { block: Block, state: BlockState },

    /// garbage child that lives in the grid and is linked up in a higher layer garbage  
    Child(Child),
}

impl Component {
    pub fn spawn(opt_vframe: Option<u32>) -> Self {
        match opt_vframe {
            Some(vframe) => Component::Block {
                block: Block {
                    vframe,
                    ..Default::default()
                },
                state: BlockState::Idle,
            },
            _ => Component::Empty {
                size: 0,
                alive: false,
            },
        }
    }

    /// converts the current component into a sprite if it is real, can be turned into a quad
    pub fn to_sprite(&self, position: V2) -> Option<Sprite> {
        let mut sprite = Sprite {
            position,
            centered: true,
            ..Default::default()
        };

        match self {
            Component::Block { block, .. } => {
                sprite.scale = block.scale;
                sprite.vframe = block.vframe;
                sprite.hframe = block.hframe;
                sprite.offset = block.offset + ATLAS_SPACING / 2.;
                Some(sprite)
            }

            Component::Child(child) => {
                sprite.scale = child.scale;
                sprite.vframe = child.vframe;
                sprite.hframe = child.hframe;
                sprite.offset = V2::new(0., child.y_offset) + ATLAS_SPACING / 2.;
                Some(sprite)
            }

            _ => None,
        }
    }
}
