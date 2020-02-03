use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::ptr;
use winit::{
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
	event::*,
	platform::desktop::EventLoopExtDesktop,
};
use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;

// TODO(Skytrias): set to monitor framerate
const FRAME_AMOUNT: f64 = 120.;
const FPS: u64 = (1. / FRAME_AMOUNT * 1000.) as u64;

// state of the Application, includes drawing, input, generators
pub struct App {
    ubo_projection: wgpu::Buffer,
    //ubo_grid: u32,
    //ubo_sprite: u32,
    //ubo_text: u32,
    //shaders: HashMap<String, u32>,
	
    key_downs: HashMap<VirtualKeyCode, u32>,
    sprites: Vec<Sprite>,
	
    pub mouse: Mouse,
    pub gen: oorandom::Rand32,
}

impl App {
    // returns true if a key is held down
    pub fn key_down(&self, code: VirtualKeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v != 0).is_some()
    }
	
	// returns true if a key is held down
    pub fn key_down_frames(&self, code: VirtualKeyCode) -> Option<u32> {
        self.key_downs.get(&code).filter(|&&v| v != 0).map(|v| *v)
    }
	
    // returns true if a key is pressed for a single frame
    pub fn key_pressed(&self, code: VirtualKeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v == 1).is_some()
    }
	
    // returns an integer in the range wanted
    #[inline]
		pub fn rand_int(&mut self, range: u32) -> i32 {
        (self.gen.rand_float() * range as f32).round() as i32
    }
	
    // draws the grid with all v4 info
    pub fn draw_grid(&mut self, _data: &[V4]) {
        /*
		unsafe {
            gl::UseProgram(self.shaders["grid"]);
            update_ubo(self.ubo_grid, data, 2);
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, GRID_TOTAL as i32);
        }
    */
	}
	
    // pushes a sprite to the anonymous sprites
    pub fn push_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }
	
    // draws all acquired sprites and clears the sprites again
    fn draw_sprites(&mut self) {
		/*
		// dont draw anything if sprites havent been set
        if self.sprites.len() == 0 {
            return;
        }
		
        unsafe {
            gl::UseProgram(self.shaders["sprite"]);
            update_ubo(self.ubo_sprite, self.sprites.as_slice(), 1);
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, SPRITE_AMOUNT as i32);
        }
		
        self.sprites = Vec::with_capacity(SPRITE_AMOUNT);
    */
	}
	
	/*
    // draws any text at the position specified, each character can have a different position if wanted
    pub fn draw_string(&self, text: &'static str, position: V2, centered: bool) {
        unsafe {
            gl::UseProgram(self.shaders["text"]);
        }
		
        let mut data = vec![V4::zero(); TEXT_AMOUNT];
		
        // null data
        for num in &mut data {
            num.x = -1.0;
        }
		
        // set data to digits
        let mut length = 0.0;
        for (i, r) in text.chars().enumerate() {
            if r != ' ' {
                let value = r.to_digit(35);
				
                if let Some(num) = value {
                    data[i].x = num as f32 - 10.0;
                }
				
                data[i].y = position.x;
                data[i].z = position.y;
                length += 1.0;
            }
        }
		
        data[0].w = ATLAS_ALPHABET; // atlas vframe
        data[1].w = length;
        data[2].w = if centered { 1.0 } else { 0.0 };
		
        update_ubo(self.ubo_text, &data[..], 3);
		
        unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, TEXT_AMOUNT as i32);
        }
    }*/
	
    // draws a number at a specified position
    /*
	pub fn draw_number(
					   &self,
					   value: f32,
					   position: V2,
					   //centered: bool,
					   ) {
        unsafe {
            gl::UseProgram(self.shaders["text"]);
        }
		
        let mut data = vec![V4::zero(); TEXT_AMOUNT];
		
        // null data
        for num in &mut data {
            num.x = -1.0;
        }
		
        // set data to digits
        let mut length = 0;
        // manual 0 add
        if value == 0.0 {
            data[length].x = 0.0;
            data[length].y = position.x;
            data[length].z = position.y;
            length += 1;
        } else {
            let mut count = value as u32;
            while count != 0 && length < TEXT_AMOUNT {
                data[length].x = (count % 10) as f32;
                data[length].y = position.x;
                data[length].z = position.y;
                count /= 10;
                length += 1;
            }
        }
		
        data[0].w = ATLAS_NUMBERS; // atlas vframe
        data[1].w = length as f32;
        //data[2].w = if centered { 1.0 } else { 0.0 };
		
        update_ubo(self.ubo_text, &data[..], 3);
		
        unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, TEXT_AMOUNT as i32);
        }
    }
	*/
}

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

// opens a shader from a file lives next to src directory
fn load_shader(device: &wgpu::Device, name: &'static str) -> wgpu::ShaderModule {
	let file = std::fs::File::open(name).expect("FS: file open failed");
	device.create_shader_module(&wgpu::read_spirv(file).unwrap())
}

// main loop of the game
// loads the window && all script objects
pub fn run(width: f32, height: f32, title: &'static str) {
	// winit
	let mut event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title(title)
		.with_inner_size(winit::dpi::LogicalSize::new(width, height))
		.build(&event_loop)
		.unwrap();
	
	// wgpu
	let surface = wgpu::Surface::create(&window);
	
	let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
											 power_preference: wgpu::PowerPreference::Default,
											 // NOTE(Skytrias): use vulkan by default
											 backends: wgpu::BackendBit::VULKAN,
										 }).unwrap();
	
	let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
														 extensions: wgpu::Extensions {
															 anisotropic_filtering: false,
														 },
														 limits: wgpu::Limits::default(),
													 });
	
	// shaders
	let vs_module = load_shader(&device, "shaders/grid.vert.spv");
	let fs_module = load_shader(&device, "shaders/grid.frag.spv");
	
	// projection matrix ubo
	let (ubo_projection, matrix_size) = {
		let projection = ortho(0., width, 0., height, -1., 1.);
		let ubo_projection = device.create_buffer_mapped(
														 16, // 16 bits?
														 wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST
														 ).fill_from_slice(&projection);
		
		(ubo_projection, std::mem::size_of::<Matrix>())
	};
	
	// load our single texture atlas into ubo
	let texture_view = {
		let data = std::fs::read("textures/atlas.png").expect("Failed to open PNG");
		let data = std::io::Cursor::new(data);
		let decoder = png_pong::FrameDecoder::<_, pix::Rgba8>::new(data);
		let png_pong::Frame { raster, delay: _ } = decoder
			.last()
			.expect("No frames in PNG")
			.expect("PNG parsing error");
		let width = raster.width();
		let height = raster.height();
		let texels = raster.as_u8_slice();
		
		let texture_extent = wgpu::Extent3d {
			width: width as u32,
			height: height as u32,
			depth: 1,
		};
		let texture = device.create_texture(&wgpu::TextureDescriptor {
												size: texture_extent,
												array_layer_count: 1,
												mip_level_count: 1,
												sample_count: 1,
												dimension: wgpu::TextureDimension::D2,
												format: TEXTURE_FORMAT,
												usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
											});
		let texture_view = texture.create_default_view();
		let temp_buf = device
			.create_buffer_mapped(texels.len(), wgpu::BufferUsage::COPY_SRC)
			.fill_from_slice(texels);
		
		{
			let mut init_encoder =
				device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
			
			init_encoder.copy_buffer_to_texture(
												wgpu::BufferCopyView {
													buffer: &temp_buf,
													offset: 0,
													row_pitch: 4 * width,
													image_height: height,
												},
												wgpu::TextureCopyView {
													texture: &texture,
													mip_level: 0,
													array_layer: 0,
													origin: wgpu::Origin3d {
														x: 0.,
														y: 0.,
														z: 0.,
													},
												},
												texture_extent,
												);
			
			queue.submit(&[init_encoder.finish()]);
		}
		
		texture_view
	};
	
    // other ubos
    //let ubo_sprite = init_ubo(mem::size_of::<Sprite>() * SPRITE_AMOUNT, 1);
    //let ubo_grid = init_ubo(mem::size_of::<V4>() * GRID_TOTAL * 2, 2);
    //let ubo_text = init_ubo(mem::size_of::<V4>() * TEXT_AMOUNT, 3);
	
	// sampler used for textures
	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
											address_mode_u: wgpu::AddressMode::ClampToEdge,
											address_mode_v: wgpu::AddressMode::ClampToEdge,
											address_mode_w: wgpu::AddressMode::ClampToEdge,
											mag_filter: wgpu::FilterMode::Nearest,
											min_filter: wgpu::FilterMode::Nearest,
											mipmap_filter: wgpu::FilterMode::Nearest,
											lod_min_clamp: -100.0,
											lod_max_clamp: 100.0,
											compare_function: wgpu::CompareFunction::Always,
										});
	
	// order of shader
	let (bind_group, render_pipeline) = {
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
																	bindings: &[
																				wgpu::BindGroupLayoutBinding {
																					binding: 0,
																					visibility: wgpu::ShaderStage::VERTEX,
																					ty: wgpu::BindingType::UniformBuffer {
																						dynamic: false,
																					},
																				},
																				
																				wgpu::BindGroupLayoutBinding {
																					binding: 1,
																					visibility: wgpu::ShaderStage::FRAGMENT,
																					ty: wgpu::BindingType::SampledTexture {
																						multisampled: false,
																						dimension: wgpu::TextureViewDimension::D2,
																					},
																				},
																				
																				wgpu::BindGroupLayoutBinding {
																					binding: 2,
																					visibility: wgpu::ShaderStage::FRAGMENT,
																					ty: wgpu::BindingType::Sampler,
																				},
																				],
																});
		
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
													  layout: &bind_group_layout,
													  bindings: &[
																  wgpu::Binding {
																	  binding: 0,
																	  resource: wgpu::BindingResource::Buffer {
																		  buffer: &ubo_projection,
																		  range: 0..matrix_size as wgpu::BufferAddress,
																	  },
																  },
																  
																  wgpu::Binding {
																	  binding: 1,
																	  resource: wgpu::BindingResource::TextureView(&texture_view),
																  },
																  
																  wgpu::Binding {
																	  binding: 2,
																	  resource: wgpu::BindingResource::Sampler(&sampler),
																  },
																  ],
												  });
		
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
																bind_group_layouts: &[&bind_group_layout],
															});
		
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
																layout: &pipeline_layout,
																vertex_stage: wgpu::ProgrammableStageDescriptor {
																	module: &vs_module,
																	entry_point: "main",
																},
																fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
																						 module: &fs_module,
																						 entry_point: "main",
																					 }),
																rasterization_state: Some(wgpu::RasterizationStateDescriptor {
																							  front_face: wgpu::FrontFace::Ccw,
																							  cull_mode: wgpu::CullMode::None,
																							  depth_bias: 0,
																							  depth_bias_slope_scale: 0.0,
																							  depth_bias_clamp: 0.0,
																						  }),
																primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
																color_states: &[wgpu::ColorStateDescriptor {
																					format: wgpu::TextureFormat::Bgra8UnormSrgb,
																					color_blend: wgpu::BlendDescriptor::REPLACE,
																					alpha_blend: wgpu::BlendDescriptor::REPLACE,
																					write_mask: wgpu::ColorWrite::ALL,
																				}],
																depth_stencil_state: None,
																index_format: wgpu::IndexFormat::Uint16,
																vertex_buffers: &[],
																sample_count: 1,
																sample_mask: !0,
																alpha_to_coverage_enabled: false,
															});
		
		(bind_group, render_pipeline)
	};
	
	let mut swap_chain = device.create_swap_chain(
												  &surface,
												  &wgpu::SwapChainDescriptor {
													  usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
													  format: wgpu::TextureFormat::Bgra8UnormSrgb,
													  width: width as u32,
													  height: height as u32,
													  present_mode: wgpu::PresentMode::Vsync,
												  },
												  );
	
    // load all shaders
	/*
	let shaders = {
        let mut shaders = HashMap::new();
        shaders.insert("sprite".to_string(), load_shader("sprite"));
        shaders.insert("grid".to_string(), load_shader("grid"));
        shaders.insert("text".to_string(), load_shader("text"));
        shaders
    };
	*/
	
    let mut app = App {
        ubo_projection,
        //ubo_sprite,
        //ubo_grid,
        //ubo_text,
        //shaders,
		
        key_downs: HashMap::new(),
        sprites: Vec::with_capacity(SPRITE_AMOUNT),
		
        mouse: Default::default(),
        gen: oorandom::Rand32::new(0),
    };
	
    let mut cursor = Cursor::default();
    let mut grid = Grid::new(&mut app);
	
    let mut quit = false;
    let mut fixedstep = fixedstep::FixedStep::start(FRAME_AMOUNT);
    while !quit {
        // update scope
        while fixedstep.update() {
            // quit on escape
            if app.key_pressed(VirtualKeyCode::Escape) {
                quit = true;
            }
			
            // reload grid on space
            if app.key_pressed(VirtualKeyCode::Space) {
                grid = Grid::new(&mut app);
            }
			
            // reload grid on space
            if app.key_pressed(VirtualKeyCode::Return) {
				let children: Vec<usize> = (0..GRID_WIDTH).collect();
				
				for x in 0..GRID_WIDTH {
					grid.components[(x, 0).raw()] = {
						if x == 0 {
							Components::GarbageParent(Garbage::new(children.clone()))
						} else {
							Components::GarbageChild(0)
						}
					};
				}
			}
			
            grid.update(&mut app);
            cursor.update(&app, &mut grid);
			
            // clearing
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
										  WindowEvent::Resized(_physical_size) => {
											  //window.resize(physical_size),
											  // TODO(Skytrias): resize
										  }
										  
										  WindowEvent::CloseRequested => {
											  quit = true;
										  }
										  
										  WindowEvent::CursorMoved { position, .. } => {
											  // NOTE(Skytrias): convert into?
											  app.mouse.position = v2(position.x as f32, position.y as f32);
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
			
            //grid.draw(&mut app);
            //cursor.draw(&mut app);
            //app.draw_sprites();
			
            //window.swap_buffers().unwrap();
			let frame = swap_chain.get_next_texture();
			let mut encoder =
				device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
			{
				let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
															  color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
																					   attachment: &frame.view,
																					   resolve_target: None,
																					   load_op: wgpu::LoadOp::Clear,
																					   store_op: wgpu::StoreOp::Store,
																					   clear_color: wgpu::Color::WHITE,
																				   }],
															  depth_stencil_attachment: None,
														  });
				rpass.set_pipeline(&render_pipeline);
				rpass.set_bind_group(0, &bind_group, &[]);
				rpass.draw(0 .. 4, 0 .. 1);
			}
			
			queue.submit(&[encoder.finish()]);
		}
		
        std::thread::sleep(std::time::Duration::from_millis(FPS));
    }
}
