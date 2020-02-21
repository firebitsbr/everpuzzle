use crate::engine::App;
use crate::helpers::{Sprite, ATLAS_SPACING, R4, V2};
use wgpu_glyph::{HorizontalAlign, Layout, Section};
use UiItem::*;

// TODO(Skytrias): helper for drawing text and sprite on top of each other

pub struct UiBuilder<'a> {
    offset: f32,
    base: R4,
    app: &'a mut App,
    ui_context: &'a mut UiContext,
}

impl<'a> UiBuilder<'a> {
    pub fn new(
        app: &'a mut App,
        ui_context: &'a mut UiContext,
        rect: R4,
        amount: u32,
    ) -> UiBuilder<'a> {
        let mut base = rect;

        let position = base.position();
        let dimensions: V2 = base.extent().into();
        app.push_sprite(Sprite {
            position,
            hframe: 1,
            depth: 0.99,
            scale: dimensions / ATLAS_SPACING,
            ..Default::default()
        });

        app.push_section(Section {
            text: "header",
            layout: Layout::default().h_align(HorizontalAlign::Center),
            screen_position: (position.x + dimensions.x / 2., position.y),
            ..Default::default()
        });

        let offset = 20.;
        base.x += offset;
        base.y += offset;
        base.w -= offset;
        base.h -= offset;

        let offset = (base.h) / amount as f32;
        base.h = offset;

        Self {
            offset,
            base,
            app,
            ui_context,
        }
    }

    /// pushes a button to the layout, increases the base rect y position
    pub fn push_button<P: FnOnce(&mut App)>(
        &mut self,
        text: &'static str,
        predicate: P,
    ) -> &'a mut UiBuilder {
        if self.ui_context.draw_button(self.app, text, self.base) {
            predicate(self.app);
        }

        self.base.y += self.offset;
        self
    }

    pub fn push_text(&mut self, text: &'static str) -> &'a mut UiBuilder {
        self.ui_context.draw_text(self.app, text, self.base);
        self.base.y += self.offset;
        self
    }
}

/// all ui that exist and can be drawn
#[derive(Copy, Clone, PartialEq)]
pub enum UiItem {
    Empty,
    Button,
}

/// stores the entire state of the ui
pub struct UiContext {
    hot: UiItem,
    active: UiItem,
}

impl Default for UiContext {
    fn default() -> Self {
        Self {
            hot: Empty,
            active: Empty,
        }
    }
}

impl UiContext {
    /// draws a button and has state logic returning true on press
    /// thanks casey muratori for the inspiration!
    pub fn draw_button(&mut self, app: &mut App, text: &'static str, rect: R4) -> bool {
        let mut result = false;

        let intersection = rect.contains_point(app.mouse.position);
        if intersection {
            if self.active == Button {
                if app.mouse.left_released {
                    if self.hot == Button {
                        result = true;
                    }

                    self.active = Empty;
                }
            } else if self.hot == Button {
                if app.mouse.left_pressed {
                    self.active = Button;
                }
            }

            self.hot = Button;
        }

        let hframe = {
            if intersection {
                if app.mouse.left_down {
                    4
                } else {
                    3
                }
            } else {
                2
            }
        };

        app.push_text_sprite(rect.position(), rect.extent().into(), text, hframe);

        result
    }

    /// only draws text with a rectangle at the specified rect
    pub fn draw_text(&mut self, app: &mut App, text: &'static str, rect: R4) {
        app.push_text_sprite(rect.position(), rect.extent().into(), text, 2);
    }
}
