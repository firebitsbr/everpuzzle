use crate::engine::App;
use crate::helpers::{Sprite, ATLAS_SPACING, V2};
use std::collections::VecDeque;
use wgpu_glyph::{HorizontalAlign, Layout, Scale, Section, VerticalAlign};

const COMBO_APPEAR_TIME: u32 = 5;
const COMBO_DISAPPEAR_START: u32 = 10;
const COMBO_DISAPPEAR_TIME: u32 = 500;

enum ComboVariant {
    /// wether a combo is the type
    Combo,

    /// wether a combo is a result of the chain
    Chain,
}

struct ComboData {
    size: u32,
    counter: u32,
    variant: ComboVariant,
}

pub struct ComboHighlight {
    list: VecDeque<ComboData>,
    dimensions: V2,
    y_offset: u32,
}

impl Default for ComboHighlight {
    fn default() -> Self {
        Self {
            list: VecDeque::new(),
            dimensions: V2::new(50., 25.),
            y_offset: 0,
        }
    }
}

impl ComboHighlight {
    pub fn push_chain(&mut self, chain_size: u32) {
        self.list.push_front(ComboData {
            size: chain_size,
            counter: 0,
            variant: ComboVariant::Chain,
        });
        self.y_offset = COMBO_APPEAR_TIME;
    }

    pub fn push_combo(&mut self, combo_size: u32) {
        self.list.push_front(ComboData {
            size: combo_size,
            counter: 0,
            variant: ComboVariant::Combo,
        });
        self.y_offset = COMBO_APPEAR_TIME;
    }

    pub fn draw(&mut self, app: &mut App) {
        let mut position = V2::new(200., 300.);
        for combo in self.list.iter_mut() {
            let offset_position = V2::new(
                position.x,
                position.y + self.dimensions.y * (self.y_offset as f32 / COMBO_APPEAR_TIME as f32),
            );

            let death_counter = (combo.counter as i32
                - (COMBO_DISAPPEAR_TIME - COMBO_DISAPPEAR_START) as i32)
                .max(0);
            let stupid_scale = 1. - (death_counter as f32 / COMBO_DISAPPEAR_START as f32);

            let hframe = match combo.variant {
                ComboVariant::Combo => 5,
                ComboVariant::Chain => 6,
            };

            app.push_sprite(Sprite {
                position: offset_position,
                offset: self.dimensions / 2.,
                hframe,
                scale: self.dimensions / ATLAS_SPACING * stupid_scale,
                centered: true,
                ..Default::default()
            });

            let text = match combo.variant {
                ComboVariant::Combo => format!("{}", combo.size),
                ComboVariant::Chain => format!("x{}", combo.size),
            };

            // TODO(Skytrias): implement from(f32, f32) for V2
            app.push_section(Section {
                text: &text,
                screen_position: (
                    offset_position.x + self.dimensions.x / 2.,
                    offset_position.y + self.dimensions.y / 2.,
                ),
                layout: Layout::default()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
                scale: Scale::uniform(16. * stupid_scale),
                ..Default::default()
            });

            position.y -= self.dimensions.y;

            if combo.counter < COMBO_DISAPPEAR_TIME {
                combo.counter += 1;
            } else {
                self.list.pop_back();
                break;
            }
        }

        if self.y_offset != 0 {
            self.y_offset -= 1;
        }
    }
}
