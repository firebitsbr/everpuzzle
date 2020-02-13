use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::ptr;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::WindowBuilder,
};

// size_of's
const FLOAT_SIZE: usize = 4;
const PROJECTION_LEN: usize = 16; // 16 floats?
pub const PROJECTION_SIZE: u64 = (PROJECTION_LEN * FLOAT_SIZE) as u64;
const GRID_LEN: usize = GRID_TOTAL;
const GRID_SIZE: u64 = (GRID_LEN * std::mem::size_of::<GridBlock>()) as u64;
const SPRITE_SIZE: u64 = (SPRITE_LEN * std::mem::size_of::<Sprite>()) as u64;
const TEXT_SIZE: u64 = std::mem::size_of::<TextData>() as u64;

// text character limitations
const TEXT_CHARACTERS: usize = 20;
type TextData = [V4; TEXT_CHARACTERS];

// wgpu consts
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

// TODO(Skytrias): set to monitor framerate
const FRAME_AMOUNT: f64 = 120.;
const FPS: u64 = (1. / FRAME_AMOUNT * 1000.) as u64;

// helper for recreating a depth texture
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

// i hate this but it makes life easier, queue is the only mut so thats why i had to split it up from App
// issues a copy to a buffer from a generic slice with its size and length, and draws it with the given bind_groupd and render pipeline
fn copy_and_draw<T: Copy + 'static>(
									load_op: wgpu::LoadOp, // wether to load or clear the last draw
									device: &wgpu::Device,
									queue: &mut wgpu::Queue,
									depth_texture_view: &wgpu::TextureView,
									frame: &wgpu::SwapChainOutput<'_>,
									bind_group: &wgpu::BindGroup,
									pipeline: &wgpu::RenderPipeline,
									buffer: &wgpu::Buffer,
									data: &[T],
									len: usize,
									size: usize,
									) {
    // map ubo data into gpu
    {
        let temp_buffer = device
            .create_buffer_mapped(len, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(data);
		
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
		
        encoder.copy_buffer_to_buffer(&temp_buffer, 0, buffer, 0, size as u64);
		
        queue.submit(&[encoder.finish()]);
    }
	
    // render call
    {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
		
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
														  color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
																				   attachment: &frame.view,
																				   resolve_target: None,
																				   load_op,
																				   store_op: wgpu::StoreOp::Store,
																				   clear_color: wgpu::Color::WHITE,
																			   }],
														  depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
																							 attachment: &depth_texture_view,
																							 depth_load_op: wgpu::LoadOp::Clear,
																							 depth_store_op: wgpu::StoreOp::Store,
																							 clear_depth: 1.0,
																							 stencil_load_op: wgpu::LoadOp::Clear,
																							 stencil_store_op: wgpu::StoreOp::Store,
																							 clear_stencil: 0,
																						 }),
													  });
			
            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, bind_group, &[]);
            rpass.draw(0..4, 0..len as u32);
        }
		
        queue.submit(&[encoder.finish()]);
    }
}

// state of the Application, includes drawing, input, generators
pub struct App {
    device: wgpu::Device,
	
    ubo_projection: wgpu::Buffer,
    ubo_grid: wgpu::Buffer,
    sbo_sprite: wgpu::Buffer,
    sbo_text: wgpu::Buffer,
	
    grid_group: wgpu::BindGroup,
    grid_pipeline: wgpu::RenderPipeline,
    sprite_group: wgpu::BindGroup,
    sprite_pipeline: wgpu::RenderPipeline,
    text_group: wgpu::BindGroup,
    text_pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
    swapchain_desc: wgpu::SwapChainDescriptor,
    depth_texture_view: wgpu::TextureView,
	
    key_downs: HashMap<VirtualKeyCode, u32>,
    sprites: Vec<Sprite>,
    texts: Vec<TextData>,
	
    pub frame_counter: u32,
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
        self.key_downs.get(&code).filter(|&&v| v != 0).copied()
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
    pub fn draw_grid(&mut self, data: &[GridBlock], frame: &wgpu::SwapChainOutput<'_>) {
        copy_and_draw(
					  wgpu::LoadOp::Clear,
					  &self.device,
					  &mut self.queue,
					  &self.depth_texture_view,
					  frame,
					  &self.grid_group,
					  &self.grid_pipeline,
					  &self.ubo_grid,
					  data,
					  GRID_LEN,
					  GRID_SIZE as usize,
					  );
    }
	
    // TODO(Skytrias): remove sprite.visible?
    // pushes a sprite to the anonymous sprites
    pub fn push_sprite(&mut self, sprite: Sprite) {
        if self.sprites.len() < SPRITE_LEN && sprite.visible == 1. {
            self.sprites.push(sprite);
        }
    }
	
    // draws all acquired sprites and clears the sprites again
    fn draw_sprites(&mut self, frame: &wgpu::SwapChainOutput<'_>) {
        let len = self.sprites.len();
		
        // dont draw anything if sprites havent been set
        if len == 0 {
            return;
        }
		
        copy_and_draw(
					  wgpu::LoadOp::Load,
					  &self.device,
					  &mut self.queue,
					  &self.depth_texture_view,
					  frame,
					  &self.sprite_group,
					  &self.sprite_pipeline,
					  &self.sbo_sprite,
					  &self.sprites,
					  len,
					  SPRITE_SIZE as usize,
					  );
		
        self.sprites.clear();
    }
	
    // TODO(Skytrias): optimize
    // pushes a string text to the anonymous sprites
    pub fn push_text(&mut self, text: Text) {
        if self.texts.len() > TEXT_LEN {
            return;
        }
		
        let mut data = [v4(-1., 0., 0., 0.); TEXT_CHARACTERS];
		
        match text.variant {
            TextVariant::Value(value) => {
                let mut length = 0;
                if value == 0.0 {
                    data[0].x = 0.0;
                    data[0].y = text.position.x;
                    data[0].z = text.position.y;
                    length += 1;
                } else {
                    let mut count = value as u32;
                    while count != 0 && length < TEXT_CHARACTERS {
                        data[length].x = (count % 10) as f32;
                        data[length].y = text.position.x;
                        data[length].z = text.position.y;
                        count /= 10;
                        length += 1;
                    }
                }
				
                data[0].w = ATLAS_NUMBERS; // atlas vframe
                data[1].w = length as f32;
            }
			
            TextVariant::Characters(chars) => {
                // set data to digits
                for (i, r) in chars.chars().enumerate() {
                    if r != ' ' {
                        let value = r.to_digit(35);
						
                        if let Some(num) = value {
                            data[i].x = num as f32 - 10.0;
                        }
						
                        // add to length
                        data[1].w += 1.0;
                    }
					
                    // add position per index
                    data[i].y = text.position.x;
                    data[i].z = text.position.y;
                }
				
                data[0].w = ATLAS_ALPHABET;
            }
        }
		
        data[2].w = if text.centered { 1. } else { 0. };
        data[3].w = text.dimensions.x;
        data[4].w = text.dimensions.y;
		
        self.texts.push(data);
    }
	
    // draws any text at the position specified, each character can have a different position if wanted
    pub fn draw_texts(&mut self, frame: &wgpu::SwapChainOutput<'_>) {
        let len = self.texts.len();
		
        if len == 0 {
            return;
        }
		
        for text in self.texts.iter() {
            copy_and_draw(
						  wgpu::LoadOp::Load,
						  &self.device,
						  &mut self.queue,
						  &self.depth_texture_view,
						  frame,
						  &self.text_group,
						  &self.text_pipeline,
						  &self.sbo_text,
						  text,
						  TEXT_CHARACTERS,
						  TEXT_SIZE as usize,
						  );
        }
		
        self.texts.clear();
    }
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
										 })
		.unwrap();
	
    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
														 extensions: wgpu::Extensions {
															 anisotropic_filtering: false,
														 },
														 limits: wgpu::Limits::default(),
													 });
	
    // projection matrix ubo
    let ubo_projection = {
        let projection = ortho(0., width, 0., height, -1., 1.);
        device
            .create_buffer_mapped(
								  PROJECTION_LEN,
								  wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
								  )
            .fill_from_slice(&projection)
    };
	
    // TODO(Skytrias): not do empty fill?
    // empty fill to the grid
    let ubo_grid = {
        let data = [GridBlock::default(); GRID_LEN];
        device
            .create_buffer_mapped(
								  GRID_LEN,
								  wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
								  )
            .fill_from_slice(&data)
    };
	
    let sbo_sprite = {
        // TODO(Skytrias): not do empty fill?
        // empty fill to the grid
        let temp_sprites = [Sprite::empty(); SPRITE_LEN];
		
        device
            .create_buffer_mapped(
								  SPRITE_LEN,
								  wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
								  )
            .fill_from_slice(&temp_sprites)
    };
	
    // TODO(Skytrias): not do empty fill?
    // empty fill to the grid
    let sbo_text = {
        let texts = [V4::zero(); TEXT_CHARACTERS];
		
        device
            .create_buffer_mapped(
								  TEXT_CHARACTERS,
								  wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
								  )
            .fill_from_slice(&texts)
    };
	
    // load our single texture atlas into ubo
    let texture_view = {
        // load the texture and its info
        let data = std::fs::read("textures/atlas.png").expect("Failed to open PNG");
        let data = std::io::Cursor::new(data);
        let decoder = png_pong::FrameDecoder::<_, pix::Rgba8>::new(data);
        let png_pong::Frame { raster, .. } = decoder
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
		
        // copy texels into into gpu
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
	
    // shader info for easier uniform / storage calls
    // NOTE(Skytrias): shouldnt these references go out of scope before app is initialized?
    let shader_info = ShaderInfo {
        device: &device,
        ubo_projection: &ubo_projection,
        texture_view: &texture_view,
        sampler: &sampler,
    };
	
    // create bind_groups and pipelines per shader / ubo / sbo
    let (grid_group, grid_pipeline) = shader_info.uniform("grid", &ubo_grid, GRID_SIZE);
    let (sprite_group, sprite_pipeline) = shader_info.storage("sprite", &sbo_sprite, SPRITE_SIZE);
    let (text_group, text_pipeline) = shader_info.storage("text", &sbo_text, TEXT_SIZE);
	
    let mut app = App {
        device,
		
        ubo_projection,
        ubo_grid,
        sbo_sprite,
        sbo_text,
		
        grid_group,
        grid_pipeline,
        sprite_group,
        sprite_pipeline,
        text_group,
        text_pipeline,
        queue,
        swapchain_desc,
        depth_texture_view,
		
        key_downs: HashMap::new(),
        sprites: Vec::new(),
        texts: Vec::new(),
		
        frame_counter: 0,
        mouse: Default::default(),
        gen: oorandom::Rand32::new(0),
    };
	
    let mut cursor = Cursor::default();
    let mut grid = Grid::new(&mut app);
	let mut garbage_system = GarbageSystem::default();
	
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
                //grid = Grid::new(&mut app);
                for x in 0..GRID_WIDTH {
                    grid.components[(x, 0).raw()] = Components::Normal(Block::random(&mut app));
                }
            }
			
            // reload grid on space
            if app.key_pressed(VirtualKeyCode::Return) {
                grid.gen_2d_garbage(&mut garbage_system, 2);
            }
			
            if app.key_pressed(VirtualKeyCode::A) {
                let offset = (app.rand_int(1) * 3) as usize;
				println!("{}", offset);
				grid.gen_1d_garbage(&mut garbage_system, 3, offset);
            }
				
			cursor.update(&app, &mut grid);
			grid.update(&mut app, &mut garbage_system);
			garbage_system.update(&mut app, &mut grid);
				
            if app.key_pressed(VirtualKeyCode::D) {
				app.frame_counter += 1;
			}
				
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
													  ortho(0., size.width as f32, size.height as f32, 0., -1., 1.);
												  let temp_buffer = app
													  .device
													  .create_buffer_mapped(PROJECTION_LEN, wgpu::BufferUsage::COPY_SRC)
													  .fill_from_slice(&projection);
												  
												  let mut init_encoder = app.device.create_command_encoder(
																										   &wgpu::CommandEncoderDescriptor { todo: 0 },
																										   );
												  init_encoder.copy_buffer_to_buffer(
																					 &temp_buffer,
																					 0,
																					 &app.ubo_projection,
																					 0,
																					 PROJECTION_SIZE,
																					 );
												  init_encoder.finish();
											  }
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
			
            let frame = swap_chain.get_next_texture();
            grid.draw(&mut app, &frame);
            cursor.draw(&mut app);
            app.draw_sprites(&frame);
			
			   app.push_text(Text {
							  variant: app.frame_counter.into(),
							  position: v2(10., 10.),
							  dimensions: V2::both(25.),
							  ..Default::default()
						  });
			
            app.draw_texts(&frame);
        }
		
        std::thread::sleep(std::time::Duration::from_millis(FPS));
    }
}
