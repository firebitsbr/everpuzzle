use crate::engine::*;
use crate::helpers::*;
use crate::scripts::{Cursor, ComboHighlight};
use std::collections::HashMap;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, HorizontalAlign, Layout, Section, VerticalAlign};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::WindowBuilder,
};
use hecs::*;

pub struct EmptyChain {
	size: usize, 
	alive: bool,
}
#[derive(Debug)]
pub enum State {
	Idle,
	Hang { counter: u32 },
	Fall,
	Swap { counter: u32, direction: i32, x_offset: f32 },
	Land { counter: u32 },
	 Clear { counter: u32, start_time: u32, end_time: u32 }
}

pub struct Block {
	/// hframe horizontal position in the texture atlas
    pub hframe: u32,
	
    /// vframe vertical position in the texture atlas
    pub vframe: u32,
	
	pub y_offset: f32,
	
    /// visual sprite scale
    pub scale: V2,
	
	pub saved_chain: Option<usize>,
}

/// wgpu depth const
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

// TODO(Skytrias): set to monitor framerate
const FRAME_AMOUNT: f64 = 120.;
const FPS: u64 = (1. / FRAME_AMOUNT * 1000.) as u64;

fn block_update_states(grid: &mut Vec<Entity>, world: &mut World, combo_highlight: &mut ComboHighlight) {
	// update counters
	for y in 0..GRID_HEIGHT {
	for x in 0..GRID_WIDTH {
			let i = y * GRID_WIDTH + x;
			
			if let Ok(mut state) = world.get_mut::<State>(grid[i]) {
	if let Ok(mut block) = world.get_mut::<Block>(grid[i]) {
					if i >= GRID_TOTAL - GRID_WIDTH * 2 {
								println!("i {} state {:?}", i, *state);
					}
					
					match &mut *state {
						State::Idle => {
							//block.saved_chain = None;
							
							if i >= GRID_TOTAL - GRID_WIDTH * 1 {
								println!("asdasd");
								block.hframe = 2;
							} else {
								block.hframe = 0;
							}
						}
						
						State::Hang { counter, .. } => *counter += 1,
					State::Land { counter, .. } => {
						block.hframe = 3 + ((*counter as f32 / LAND_TIME as f32) * 3.).floor() as u32;
						//block.saved_chain = None;
						*counter += 1;
					}
						
					State::Swap { counter, direction, x_offset } => {
						*x_offset = match *direction {
							 -1 => -(*counter as f32) / (SWAP_TIME as f32 / 2.) * ATLAS_TILE,
							 1 => (*counter as f32) / (SWAP_TIME as f32 / 2.) * ATLAS_TILE,
							_ => 0.
						};
						
						*counter += 1;
				}
					
					State::Clear { counter, start_time, .. } => {
							if *counter > *start_time {
								if (*counter - *start_time) < CLEAR_TIME - 1 {
									let amt = 1. - ((*counter - *start_time) as f32) / (CLEAR_TIME as f32);
									block.scale = V2::broadcast(amt);
								} else {
								block.scale = V2::zero();
								}
							}
							
							block.hframe = 1;
							*counter += 1;
					}
					
					_ => {}
		}
	}
	}
	}
}
	
	block_resolve_swap(grid, world);
	block_resolve_land(grid, world);
	block_resolve_fall(grid, world);
	block_resolve_hang(grid, world);//garbage_system);
	block_detect_hang(grid, world);
	block_detect_clear(grid, world, combo_highlight);
	block_resolve_clear(grid, world);
}

fn block_resolve_land(grid: &Vec<Entity>, world: &mut World) {
	for x in (0..GRID_WIDTH).rev() {
		for y in (0..GRID_HEIGHT).rev() {
			let i = y * GRID_WIDTH + x;
			
			if let Ok(mut state) = world.get_mut::<State>(grid[i]) {
				if let State::Land { counter } = *state {
					if counter >= LAND_TIME - 1 {
								*state = State::Idle;
					}
				}
			}
		}
	}
}

fn block_resolve_swap(grid: &mut Vec<Entity>, world: &mut World) {
	for i in 0..GRID_TOTAL - 1 {
		let mut swapped = None;
		
		if let Ok(state) = world.get::<State>(grid[i]) {
			if let State::Swap { counter, direction, .. } = *state {
				if counter >= SWAP_TIME - 1 {
					grid.swap(i, (i as i32 + direction) as usize);
					swapped = Some(direction);
				}
			}
		}
		
		if let Some(direction) = swapped {
			if let Ok(mut state) = world.get_mut::<State>(grid[i]) {
				*state = State::Idle;
			}
			
			if let Ok(mut state) = world.get_mut::<State>(grid[(i as i32 + direction) as usize]) {
				*state = State::Idle;
			}
		}
	}
}
	
fn block_resolve_fall(grid: &mut Vec<Entity>, world: &mut World) {
	for x in (0..GRID_WIDTH).rev() {
		for y in (0..GRID_HEIGHT - 1).rev() {
			let i = y * GRID_WIDTH + x;
			
			if let Ok(mut state) = world.get_mut::<State>(grid[i]) {
			if let Ok(mut block) = world.get_mut::<Block>(grid[i]) {
		if let State::Fall = *state {
			if let Ok(mut chain) = world.get_mut::<EmptyChain>(grid[i + GRID_WIDTH]) {
							block.saved_chain = Some(chain.size);
							
							if chain.alive {
								chain.alive = false;
							}
							
							grid.swap(i, i + GRID_WIDTH);
			} else  {
				*state = State::Land { counter: 0 };
			}
		}
		}
	}
}
	}
	}

fn block_resolve_hang(grid: &Vec<Entity>, world: &mut World) {
	let mut above_fall = false;
	for x in (0..GRID_WIDTH).rev() {
		for y in (0..GRID_HEIGHT).rev() {
			let i = y * GRID_WIDTH + x;
			
				if let Ok(mut state) = world.get_mut::<State>(grid[i]) {
				match *state {
					State::Hang { counter } => {
						if counter >= HANG_TIME - 1 {
							above_fall = true;
							*state = State::Fall;
						}
					},
					
					State::Idle => {
						if above_fall {
							*state = State::Fall;
						}
					}
					
					_ => above_fall = false
				}
				}
				
				// garbage
		}
	}
}

fn block_detect_hang(grid: &Vec<Entity>, world: &mut World) {
for x in (0..GRID_WIDTH).rev() {
		for y in (0..(GRID_HEIGHT - 1)).rev() {
			let i = y * GRID_WIDTH + x;
			//println!("x {} y {}, i {}", x, y, i);
		
		if let Ok(mut state) = world.get_mut::<State>(grid[i]) {
			if let State::Idle = *state {
				if let Ok(_) = world.get::<EmptyChain>(grid[i + GRID_WIDTH]) {
					*state = State::Hang { counter: 0 };
				}
			}
		}
	}
}
}

fn block_detect_clear(grid: &Vec<Entity>, world: &mut World, combo_highlight: &mut ComboHighlight) {
	// NOTE(Skytrias): consider pushing to grid variables?
	let mut list = Vec::new();
	
	// get all vframes, otherwhise 99
	let vframes: Vec<u32> = (0..GRID_TOTAL)
		.map(|i| {
				 let mut result = 99;
				 
				 if let Ok(block) = world.get::<Block>(grid[i]) {
				 if let Ok(state) = world.get::<State>(grid[i]) {
						 match *state {
							 State::Idle => result = block.vframe,
							 State::Swap { counter, .. } => {
								 if counter == 0 {
									 result = block.vframe;
								 }
							 }
							 _ => {}
						 }
					 }
					 }
				 
				 result
			 }).collect();
				 
	// loop through vframes and match horizontal or vertical matches, append them to list
	for x in 0..GRID_WIDTH {
		for y in 0..GRID_HEIGHT {
			let i = y * GRID_WIDTH + x;
			let hv0 = vframes[i];
			
			if x > 1 {
				let h1 = vframes[i - 1];
				let h2 = vframes[i - 2];
				
				if hv0 != 99 && hv0 == h1 && hv0 == h2 {
					let mut temp = vec![i, i - 1, i - 2];
					list.append(&mut temp);
				}
			}
			
			if y > 1 {
				let v1 = vframes[i - GRID_WIDTH];
				let v2 = vframes[i - GRID_WIDTH * 2];
				
				if hv0 != 99 && hv0 == v1 && hv0 == v2 {
					let mut temp = vec![i, i - GRID_WIDTH, i - GRID_WIDTH * 2];
					list.append(&mut temp);
				}
			}
		}
	}
	
	if list.len() != 0 {
		// clear duplicates and sort
		list.sort();
		list.dedup();
		let length = list.len();
		
		let end_time = (length * CLEAR_TIME as usize) as u32;
		
		let mut had_chainable = None;
		for (i, index) in list.iter().enumerate() {
			if let Ok(mut state) = world.get_mut::<State>(grid[*index]) {
				if let State::Idle = *state {
					*state = State::Clear {
					counter: 0,
					start_time: (i * CLEAR_TIME as usize) as u32,
						end_time
				};
					}
				
				if let Ok(block) = world.get_mut::<Block>(grid[*index]) {
					if let Some(size) = block.saved_chain {
						had_chainable = Some(size);
					}
				}
				}
		}
		
		// push chainable even if count was 3
		if let Some(size) = had_chainable {
			//if size != 0 {
			combo_highlight.push_chain(size as u32 + 1);
		//}
		}
		
		// always send combo info
		combo_highlight.push_combo(length as u32);
	}
}

/// clear the component if clear state is finished
fn block_resolve_clear(grid: &Vec<Entity>, world: &mut World) {
	for i in 0..GRID_TOTAL {
		let mut saved_chain = None;
		
		if let Ok(state) = world.get::<State>(grid[i]) {
			if let State::Clear { counter, end_time, .. } = *state {
		if let Ok(block) = world.get::<Block>(grid[i]) {
				if counter >= end_time - 1 {
						saved_chain = block.saved_chain;
					}
				}
			}
		}
		
		if let Some(size) = saved_chain {
			world.remove::<(State, Block)>(grid[i]).expect("remove block and state");
			world.insert(grid[i], (EmptyChain { size: size + 1, alive: true }, )).expect("insert empty chain with set size and alive");
		}
	}
	}

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
    
	/*
	let mut garbage_system = GarbageSystem::default();
*/
    
	let mut combo_highlight = ComboHighlight::default();
	let mut ui_context = UiContext::default();
	let mut world = World::new();
	
	let mut grid = Vec::with_capacity(GRID_TOTAL);
	for y in 0..GRID_HEIGHT {
		for x in 0..GRID_WIDTH {
			grid.push(if app.rand_int(1) != 0 {
			world.spawn((
															 State::Idle,
														   Block {
															   vframe: (app.rand_int(5) + 3) as u32,
															   hframe: 0,
															   y_offset: 0.,
															   scale: V2::one(),
															   saved_chain: None,
														   },
																		   ))
				} else {
							  world.spawn((EmptyChain { size: 0, alive: false }, ))
								}
							);
			}
	}
	
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
			
			block_update_states(&mut grid, &mut world, &mut combo_highlight);
			
            if app.key_pressed(VirtualKeyCode::Return) {
                //grid.gen_2d_garbage(&mut garbage_system, 2);
            }

            if let Some(frames) = app.key_down_frames(VirtualKeyCode::Space) {
                if frames == 1 || frames % 50 == 0 {
                    //grid.push_upwards(&mut app, &mut garbage_system, &mut cursor);
                }
            }
			
            cursor.update(&app, &mut grid, &mut world);
			
			/*
						grid.update(&mut garbage_system);
						garbage_system.update(&mut app, &mut grid);
						grid.push_update(&mut app, &mut garbage_system, &mut cursor);
			   */
			
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
			
			for i in 0..GRID_TOTAL {
				if let Ok(block) = world.get::<Block>(grid[i]) {
				if let Ok(state) = world.get::<State>(grid[i]) {
						let x_offset = if let State::Swap { x_offset, .. } = *state {
							x_offset
						} else {
							0.
						};
						
						app.push_sprite(Sprite {
											position: V2::new(
															  (i % GRID_WIDTH) as f32 * ATLAS_TILE,
															  (i / GRID_WIDTH) as f32 * ATLAS_TILE,
																  ),
											hframe: block.hframe,
											vframe: block.vframe,
											scale: block.scale,
												offset: V2::new(
																	x_offset,
																	block.y_offset,
																	) + ATLAS_SPACING / 2.,
											centered: true,
											..Default::default()
										});
				}
				}
				}
			
            //grid.draw(&mut app);
            combo_highlight.draw(&mut app);
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
                //grid = Grid::new(app);
            })
            .push_button("spawn 1d", |app| {
                let offset = (app.rand_int(1) * 3) as usize;
                //grid.gen_1d_garbage(&mut garbage_system, 3, offset);
            })
            .push_text("-----")
            .push_button("spawn 2d", |_app| {
                //grid.gen_2d_garbage(&mut garbage_system, 2);
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
