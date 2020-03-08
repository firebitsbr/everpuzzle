use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;
use gilrs::Button;
use miniquad::*;

/// state of the Application, includes drawing, input, generators
pub struct App {
    /// input of the app, contains mouse, key & gamepad with frame times
    input: Input,

    /// sprite rendering state / pipeline
    sprites: Sprites,

    /// multiple grids that exist and interact with each other
    grids: Vec<Grid>,

    /// one garbagesystem that rules over all grids
    garbage_system: GarbageSystem,

    /// debug info turned of by default
    debug: bool,
}

impl App {
    /// initializes the app to default values, sets the grids to have the same start vframes
    pub fn new(ctx: &mut Context) -> Self {
        let vframes = {
            let mut temp_random = oorandom::Rand32::new(5);
            Grid::gen_field(&mut temp_random, 5)
        };

        Self {
            input: Input::default(),
            sprites: Sprites::new(ctx),
            grids: vec![Grid::new(0, 1, &vframes), Grid::new(1, 2, &vframes)],
            garbage_system: GarbageSystem::default(),
            debug: false,
        }
    }
}

impl EventHandler for App {
    /// updates the game based on the windows hz
    fn update(&mut self, ctx: &mut Context) {
        self.input.update_gamepad();

        // quit early
        if self.input.key_pressed(KeyCode::Escape) {
            ctx.quit();
        }

        // toggle debug info
        if self.input.kb_pressed(KeyCode::Tab, Button::Select) {
            self.debug = !self.debug;
        }

        // debug
        if self.input.mouse.left_pressed {
            let pos = I2::new(
                ((self.input.mouse.position.x - 400.) / ATLAS_TILE).floor() as i32,
                ((self.input.mouse.position.y + self.grids[1].push_amount) / ATLAS_TILE).floor()
                    as i32,
            );

            self.grids[1]
                .cursor
                .states
                .push_back(CursorState::MoveSwap {
                    counter: 0,
                    goal: pos,
                });
        }

        if self.input.kb_pressed(KeyCode::A, Button::North) {
            self.grids[1].gen_1d_garbage(&mut self.garbage_system, 6);
        }

        if self.input.kb_pressed(KeyCode::Enter, Button::West) {
            self.grids[1].gen_2d_garbage(&mut self.garbage_system, 2);
        }

        // reset grid
        // TODO(Skytrias): garbage not resetting
        if self.input.kb_pressed(KeyCode::Space, Button::Start) {
            for grid in self.grids.iter_mut() {
                grid.reset();
            }
        }

        // manual raise
        if self.input.key_down(KeyCode::LeftShift)
            || self.input.button_down(Button::LeftTrigger)
            || self.input.button_down(Button::RightTrigger)
        {
            self.grids[0].push_raise = true;
        }

        // update all grids
        let len = self.grids.len();
        for i in 0..len {
            self.grids[i].update(&self.input, &mut self.garbage_system);

            // spawns garbage on other self.grids if a new combo arrives
            for combo_index in 0..self.grids[i].combo_highlight.list.len() {
                // TODO(Skytrias): creates copies, might be bad cuz of performance
                if !self.grids[i].combo_highlight.list[combo_index].sent {
                    let combo_data = self.grids[i].combo_highlight.list[combo_index];

                    for j in 0..len {
                        // skip on the same grid as the goal
                        if i == j {
                            continue;
                        }

                        match combo_data.variant {
                            ComboVariant::Combo => self.grids[j]
                                .gen_1d_garbage(&mut self.garbage_system, combo_data.size as usize),
                            ComboVariant::Chain => self.grids[j]
                                .gen_2d_garbage(&mut self.garbage_system, combo_data.size as usize),
                        }
                    }

                    self.grids[i].combo_highlight.list[combo_index].sent = true;
                }
            }

            self.garbage_system.update(&mut self.grids[i]);
            self.grids[i].push_update(&mut self.garbage_system);
        }

        self.input.update_end();
    }

    /// draws the entire app sprites each frame
    fn draw(&mut self, ctx: &mut Context) {
        self.grids[0].draw(&mut self.sprites, v2(0., 0.), self.debug);
        self.grids[1].draw(&mut self.sprites, v2(400., 0.), self.debug);

        self.sprites.render(ctx);
        ctx.commit_frame();
    }

    /// updates the input internal keys down
    fn key_down_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.input.down_event(keycode);
    }

    /// updates the input internal keys up
    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods) {
        self.input.up_event(keycode);
    }

    /// updates the input internal mouse position
    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.input.mouse.position = v2(x, y);
    }

    /// updates the input internal mouse down to true / release to false
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        match button {
            MouseButton::Left => {
                self.input.mouse.left_down = true;
                self.input.mouse.left_released = false;
            }

            MouseButton::Right => {
                self.input.mouse.right_down = true;
                self.input.mouse.left_released = false;
            }

            _ => {}
        }
    }

    /// updates the input internal mouse down to false / release to true
    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        match button {
            MouseButton::Left => {
                self.input.mouse.left_down = false;
                self.input.mouse.left_released = true;
            }

            MouseButton::Right => {
                self.input.mouse.right_down = false;
                self.input.mouse.left_released = true;
            }

            _ => {}
        }
    }
}
