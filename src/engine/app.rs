use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::desktop::EventLoopExtDesktop;
use glutin::window::WindowBuilder;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::ptr;

// TODO(Skytrias): set to monitor framerate
const FRAME_AMOUNT: f64 = 120.;
const FPS: u64 = (1. / FRAME_AMOUNT * 1000.) as u64;

// state of the Application, includes drawing, input, generators
pub struct App {
    ubo_projection: u32,
    ubo_sprite: u32,
    ubo_grid: u32,
    ubo_text: u32,
    shaders: HashMap<String, u32>,
	
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
    pub fn draw_grid(&mut self, data: &[V4]) {
        unsafe {
            gl::UseProgram(self.shaders["grid"]);
            update_ubo(self.ubo_grid, data, 2);
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, GRID_TOTAL as i32);
        }
    }
	
    // pushes a sprite to the anonymous sprites
    pub fn push_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }
	
    // draws all acquired sprites and clears the sprites again
    fn draw_sprites(&mut self) {
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
    }
	
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
    }
	
    // draws a number at a specified position
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
}

// drop gl allocs
impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.ubo_projection);
            gl::DeleteBuffers(1, &self.ubo_sprite);
            gl::DeleteBuffers(1, &self.ubo_grid);
            gl::DeleteBuffers(1, &self.ubo_text);
			
            for (_, program) in &self.shaders {
                gl::DeleteProgram(*program);
            }
        }
    }
}

// print the opengl version that was loaded
fn print_gl_version() {
    use std::ffi::CStr;
    let version = unsafe {
        let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };
    println!("{}", version);
}

// sets null data to a specific ubo position in all shaders
fn init_ubo(size: usize, index: u32) -> u32 {
    unsafe {
        let mut ubo: u32 = 0;
        gl::GenBuffers(1, &mut ubo);
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
        gl::BufferData(
					   gl::UNIFORM_BUFFER,
					   size as isize,
					   ptr::null(),
					   gl::STATIC_DRAW,
					   );
        gl::BindBufferBase(gl::UNIFORM_BUFFER, index, ubo);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        ubo
    }
}

// updates data to a specific ubo position in all shaders
pub fn update_ubo<T>(ubo: u32, data: &[T], index: u32) {
    unsafe {
        gl::BindBufferBase(gl::UNIFORM_BUFFER, index, ubo);
        gl::BufferSubData(
						  gl::UNIFORM_BUFFER,
						  0,
						  mem::size_of_val(data) as isize,
						  data.as_ptr() as *const c_void,
						  );
    }
}

// main loop of the game
// loads the window && gl && all script objects
pub fn run(width: f32, height: f32, title: &'static str) {
    let mut event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(glutin::dpi::LogicalSize::new(width, height));
	
    let window = {
        unsafe {
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_srgb(true)
                .with_gl_debug_flag(true)
                .build_windowed(wb, &event_loop)
                .expect("building window failed")
                .make_current()
                .unwrap()
        }
    };
	
    gl::load_with(|s| window.context().get_proc_address(s) as *const _);
	
    // load projection
    let _scale = 1.0;
    let projection = ortho(0., width / _scale, height / _scale, 0., -1., 1.);
    let mut ubo_projection: u32 = 0;
    unsafe {
        gl::GenBuffers(1, &mut ubo_projection);
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo_projection);
        gl::BufferData(
					   gl::UNIFORM_BUFFER,
					   mem::size_of::<Matrix>() as isize,
					   projection.as_ptr() as *const c_void,
					   gl::STATIC_DRAW,
					   );
        gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, ubo_projection);
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }
	
    // other ubos
    let ubo_sprite = init_ubo(mem::size_of::<Sprite>() * SPRITE_AMOUNT, 1);
    let ubo_grid = init_ubo(mem::size_of::<V4>() * GRID_TOTAL * 2, 2);
    let ubo_text = init_ubo(mem::size_of::<V4>() * TEXT_AMOUNT, 3);
	
    // load all shaders
    let shaders = {
        let mut shaders = HashMap::new();
        shaders.insert("sprite".to_string(), load_shader("sprite"));
        shaders.insert("grid".to_string(), load_shader("grid"));
        shaders.insert("text".to_string(), load_shader("text"));
        shaders
    };
	
    // TODO(Skytrias): use env_dir?
    let _ = Texture::new("textures/atlas.png");
	
    // enable options
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::DepthFunc(gl::LESS);
		
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }
	
    let mut app = App {
        ubo_sprite,
        ubo_grid,
        ubo_projection,
        ubo_text,
        shaders,
		
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
										  WindowEvent::Resized(physical_size) => window.resize(physical_size),
										  
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
			
            unsafe {
                gl::ClearColor(1., 1., 1., 1.);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
			
            grid.draw(&mut app);
            cursor.draw(&mut app);
            app.draw_sprites();
			
            window.swap_buffers().unwrap();
        }
		
        std::thread::sleep(std::time::Duration::from_millis(FPS));
    }
}
