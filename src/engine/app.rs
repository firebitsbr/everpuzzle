use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;
use std::collections::HashMap;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, VerticalAlign};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::WindowBuilder,
};

/// wgpu depth const
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

// TODO(Skytrias): set to monitor framerate
const FRAME_AMOUNT: f64 = 120.;
const FPS: u64 = (1. / FRAME_AMOUNT * 1000.) as u64;

/// helper for recreating a depth texture
fn create_depth_texture(
    device: &wgpu::Device,
    swapchain_desc: &wgpu::SwapChainDescriptor,
) -> wgpu::Texture {
    let desc = wgpu::TextureDescriptor {
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        ..swapchain_desc.to_texture_desc()
    };
    device.create_texture(&desc)
}

/// state of the Application, includes drawing, input, generators
pub struct App {
    /// wgpu device
    device: wgpu::Device,

    /// projection ubo buffer, resends ortho on resize
    ubo_projection: wgpu::Buffer,

    /// pipeline for rendering sprites
    quad_pipeline: quad::Pipeline,

    /// queues draw commands
    queue: wgpu::Queue,

    /// description of the swapchain containing width / height
    swapchain_desc: wgpu::SwapChainDescriptor,

    /// depth texture access for depth checking
    depth_texture_view: wgpu::TextureView,

    /// data storage to draw texts
    glyph_brush: GlyphBrush<'static, ()>,

    /// data storage for each key that was pressed with the frame time
    key_downs: HashMap<VirtualKeyCode, u32>,

    /// data storage for all quads in the frame that you want to draw
    quads: Vec<quad::Quad>,

    /// mouse handle that which holds left / right button and position info
    pub mouse: Mouse,

    /// random number generator
    pub gen: oorandom::Rand32,
}

impl App {
    /// returns true if a key is held down
    pub fn key_down(&self, code: VirtualKeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v != 0).is_some()
    }

    /// returns true if a key is held down
    pub fn key_down_frames(&self, code: VirtualKeyCode) -> Option<u32> {
        self.key_downs.get(&code).filter(|&&v| v != 0).copied()
    }

    /// returns true if a key is pressed for a single frame
    pub fn key_pressed(&self, code: VirtualKeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v == 1).is_some()
    }

    /// returns an integer in the range wanted
    #[inline]
    pub fn rand_int(&mut self, range: u32) -> i32 {
        (self.gen.rand_float() * range as f32).round() as i32
    }

    /// pushes a sprite to the anonymous sprites
    pub fn push_sprite(&mut self, sprite: Sprite) {
        if self.quads.len() < quad::Quad::MAX {
            self.quads.push(sprite.into());
        }
    }

    /// draws all acquired sprites and clears the sprites again
    fn draw_sprites(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        // dont draw anything if sprites havent been set
        if self.quads.len() == 0 {
            return;
        }

        self.quad_pipeline.draw(
            &mut self.device,
            encoder,
            &self.depth_texture_view,
            view,
            &self.quads,
        );

        self.quads.clear();
    }

    /// pushes a section of text to be rendered this frame
    pub fn push_section(&mut self, section: Section) {
        self.glyph_brush.queue(section);
    }

    /// pushes a specified text aligned to a rectangle at the specified position with dimensions
    pub fn push_text_sprite<'a>(
        &mut self,
        position: V2,
        dimensions: V2,
        text: &'a str,
        hframe: u32,
    ) {
        self.push_sprite(Sprite {
            position,
            hframe,
            scale: dimensions / ATLAS_SPACING,
            ..Default::default()
        });

        // TODO(Skytrias): implement from(f32, f32) for V2
        self.push_section(Section {
            text,
            screen_position: (
                position.x + dimensions.x / 2.,
                position.y + dimensions.y / 2.,
            ),
            layout: Layout::default()
                .h_align(HorizontalAlign::Center)
                .v_align(VerticalAlign::Center),
            ..Default::default()
        });
    }
}

/// main loop of the game, loads the window && all script objects
pub fn run(width: f32, height: f32, title: &'static str) {
    // winit init
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    // wgpu init
    let surface = wgpu::Surface::create(&window);
    let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        // NOTE(Skytrias): use vulkan by default
        backends: wgpu::BackendBit::VULKAN,
    })
    .unwrap();
    let (mut device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    // projection matrix ubo
    let ubo_projection = {
        let projection = ortho(0., width, 0., height, -1., 1.);
        device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[projection])
    };

    // swapchain
    let (mut swap_chain, swapchain_desc) = {
        let desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: width as u32,
            height: height as u32,
            present_mode: wgpu::PresentMode::Vsync,
        };

        (device.create_swap_chain(&surface, &desc), desc)
    };

    // depth texture
    let mut depth_texture = create_depth_texture(&device, &swapchain_desc);
    let depth_texture_view = depth_texture.create_default_view();

    // first pipeline
    let quad_pipeline = quad::Pipeline::new(&mut device, &mut queue, &ubo_projection);

    let font = load_file!("fonts/JetBrainsMono-Regular.ttf");
    let glyph_brush = GlyphBrushBuilder::using_font_bytes(font).build(&mut device, RENDER_FORMAT);

    // initializse the app itself
    let mut app = App {
        device,

        ubo_projection,
        quad_pipeline,
        glyph_brush,

        queue,
        swapchain_desc,
        depth_texture_view,

        key_downs: HashMap::new(),
        quads: Vec::new(),

        mouse: Default::default(),
        gen: oorandom::Rand32::new(0),
    };

    // scripts
    let mut cursor = Cursor::default();
    let mut grid = Grid::new(&mut app);
    let mut garbage_system = GarbageSystem::default();
    let mut ui_context = UiContext::default();

    // main loop
    let mut quit = false;
    let mut fixedstep = fixedstep::FixedStep::start(FRAME_AMOUNT);
    while !quit {
        // update scope
        while fixedstep.update() {
            // quit on escape
            if app.key_pressed(VirtualKeyCode::Escape) {
                quit = true;
            }

            /*
            if app.key_pressed(VirtualKeyCode::A) {
                let offset = (app.rand_int(1) * 3) as usize;
                grid.gen_1d_garbage(&mut garbage_system, 3, offset);
                }
            */

            if app.key_pressed(VirtualKeyCode::Return) {
                grid.gen_2d_garbage(&mut garbage_system, 2);
            }

            if let Some(frames) = app.key_down_frames(VirtualKeyCode::Space) {
                if frames == 1 || frames % 50 == 0 {
                    grid.push_upwards(&mut app, &mut garbage_system, &mut cursor);
                }
            }

            cursor.update(&app, &mut grid);
            grid.update(&mut garbage_system);
            garbage_system.update(&mut app, &mut grid);
            grid.push_update(&mut app, &mut garbage_system, &mut cursor);

            // clearing key / mouse input
            {
                // increase the frame times on the keys
                for (_, value) in app.key_downs.iter_mut() {
                    if *value != 0 {
                        *value += 1;
                    }
                }

                // enable mouse pressing
                app.mouse.update_frame();
            }
        }

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::MainEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if size.width != 0 && size.height != 0 {
                            // recreate swapchain
                            app.swapchain_desc.width = size.width;
                            app.swapchain_desc.height = size.height;
                            swap_chain =
                                app.device.create_swap_chain(&surface, &app.swapchain_desc);

                            depth_texture = create_depth_texture(&app.device, &app.swapchain_desc);
                            app.depth_texture_view = depth_texture.create_default_view();

                            // upload new projection
                            let projection =
                                ortho(0., size.width as f32, 0., size.height as f32, -1., 1.);
                            let temp_buffer = app
                                .device
                                .create_buffer_mapped(1, wgpu::BufferUsage::COPY_SRC)
                                .fill_from_slice(&[projection]);

                            let mut init_encoder = app.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor { todo: 0 },
                            );
                            init_encoder.copy_buffer_to_buffer(
                                &temp_buffer,
                                0,
                                &app.ubo_projection,
                                0,
                                std::mem::size_of::<M4>() as u64,
                            );
                            init_encoder.finish();
                        }
                    }

                    WindowEvent::CloseRequested => {
                        quit = true;
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        app.mouse.position = V2::new(position.x as f32, position.y as f32);
                    }

                    WindowEvent::MouseInput { state, button, .. } => {
                        app.mouse.update_event(state, button);
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(code),
                                state,
                                ..
                            },
                        ..
                    } => {
                        // add new codes, modify code that is 0 back to 1 if pressed
                        if state == ElementState::Pressed {
                            if let Some(value) = app.key_downs.get_mut(&code) {
                                if *value == 0 {
                                    *value = 1;
                                }
                            } else {
                                app.key_downs.insert(code, 1);
                            }
                        }

                        if state == ElementState::Released {
                            if let Some(value) = app.key_downs.get_mut(&code) {
                                *value = 0;
                            }
                        }
                    }

                    _ => (),
                },

                _ => (),
            }
        });

        // render scope
        {
            let _delta = fixedstep.render_delta();

            grid.draw(&mut app);
            cursor.draw(&mut app);
            //garbage_system.draw(&mut app, &mut grid);

            let frame = swap_chain.get_next_texture();

            let mut encoder = app
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

            // enable mouse pressing
            app.mouse.update_frame();

            let width = app.swapchain_desc.width as f32;
            let height = app.swapchain_desc.height as f32;
            UiBuilder::new(
                &mut app,
                &mut ui_context,
                R4::new(width - 200., 0., 200., height),
                4,
            )
            .push_button("reset", |app| {
                grid = Grid::new(app);
            })
            .push_button("spawn 1d", |app| {
                let offset = (app.rand_int(1) * 3) as usize;
                grid.gen_1d_garbage(&mut garbage_system, 3, offset);
            })
            .push_text("-----")
            .push_button("spawn 2d", |_app| {
                grid.gen_2d_garbage(&mut garbage_system, 2);
            });

            app.draw_sprites(&frame.view, &mut encoder);

            // draws all sections sent into glyph_brush
            app.glyph_brush
                .draw_queued(
                    &mut app.device,
                    &mut encoder,
                    &frame.view,
                    app.swapchain_desc.width,
                    app.swapchain_desc.height,
                )
                .expect("GLYPH BRUSH: failed to render text");

            app.queue.submit(&[encoder.finish()]);
        }

        std::thread::sleep(std::time::Duration::from_millis(FPS));
    }
}
