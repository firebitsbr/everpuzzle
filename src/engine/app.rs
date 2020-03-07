use crate::engine::*;
use crate::helpers::*;
use crate::scripts::*;
use gilrs::{
    ev::EventType::{ButtonPressed, ButtonReleased},
    Button,
};
use std::collections::HashMap;
use miniquad::*;

// TODO(Skytrias): set to monitor framerate
const FRAME_AMOUNT: f64 = 120.;
const FPS: u64 = (1. / FRAME_AMOUNT * 1000.) as u64;

/// data that will be sent to the gpu
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Quad {
    /// model matrix that stores position, offset, scale, dimensions, etc
    pub model: M4,
	
    /// how many tiles the quad texture should use
    pub tiles: V2,
	
    /// hframe of the tile in the texture atlas
    pub hframe: f32,
	
    /// vframe of the tile in the texture atlas
    pub vframe: f32,
	
    /// vframe of the tile in the texture atlas
    pub depth: f32,
}

impl Quad {
    /// max number of quads that can be rendered
    pub const MAX: usize = 1000;
	
    /// byte size of the quad struct
    const SIZE: usize = std::mem::size_of::<Quad>();
}

/// converts a sprite into a valid quad
impl From<Sprite> for Quad {
    fn from(sprite: Sprite) -> Self {
		let dimensions = sprite.tiles * ATLAS_SPACING;
		
		let mut model = M4::from_translation(v3(
												sprite.position.x + sprite.offset.x,
												sprite.position.y + sprite.offset.y,
												0.,
												));
		
		model = model * M4::from_nonuniform_scale(v4(sprite.scale.x, sprite.scale.y, 1., 1.));
		model = model * M4::from_nonuniform_scale(v4(dimensions.x, dimensions.y, 1., 1.));
		
		if sprite.centered {
			model = model * M4::from_translation(v3(
													-0.5,
													-0.5,
													0.,
													));
		}
		
        Quad {
            model,
			tiles: sprite.tiles,
            hframe: sprite.hframe as f32,
            vframe: sprite.vframe as f32,
            depth: sprite.depth,
        }
    }
}

/// state of the Application, includes drawing, input, generators
pub struct App {
    /// data storage for each key that was pressed with the frame time
    key_downs: HashMap<KeyCode, u32>,

    /// data storage for each button that was pressed with the frame time
    button_downs: HashMap<Button, u32>,

    /// data storage for all quads in the frame that you want to draw
    quads: Vec<Quad>,

    // mouse handle that which holds left / right button and position info
    //pub mouse: Mouse,
	
	pipeline: Pipeline,
	bindings: Bindings,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        #[rustfmt::skip]
		let vertices = &[
							0., 0.,
							1., 0.,
							1., 1.,
							0., 1.,
							];
		let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, vertices);
		
		#[rustfmt::skip]
		let indices = &[
							0, 1, 3,
							1, 2, 3,
							];
		let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, indices);
		
		let instance_buffer = Buffer::stream(
													 ctx,
													 BufferType::VertexBuffer,
												 Quad::MAX * Quad::SIZE,
													 );
		  
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instance_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };
		  
		let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::META);
		
        let pipeline = Pipeline::new(
									 ctx,
									 &[
									   BufferLayout::default(),
									   BufferLayout {
										   step_func: VertexStep::PerInstance,
										   ..Default::default()
									   },
									   ],
									 &[
									   VertexAttribute::with_buffer("v_pos", VertexFormat::Float2, 0),
									   VertexAttribute::with_buffer("i_model", VertexFormat::Mat4, 1),
									   VertexAttribute::with_buffer("i_tiles", VertexFormat::Float2, 1),
									   VertexAttribute::with_buffer("i_hframe", VertexFormat::Float1, 1),
									   VertexAttribute::with_buffer("i_vframe", VertexFormat::Float1, 1),
									   VertexAttribute::with_buffer("i_depth", VertexFormat::Float1, 1),
									   ],
									 shader,
										 );
		
		Self {
			key_downs: HashMap::new(),
			button_downs: HashMap::new(),
			quads: Vec::new(),
			
			pipeline,
			bindings,
		}
	}
	
	/// returns true if a key is held down
    pub fn key_down(&self, code: KeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v != 0).is_some()
    }

    /// returns true the amount of frames a key has been down for
    pub fn key_down_frames(&self, code: KeyCode) -> Option<u32> {
        self.key_downs.get(&code).filter(|&&v| v != 0).copied()
    }

    /// returns true if a key is pressed for a single frame
    pub fn key_pressed(&self, code: KeyCode) -> bool {
        self.key_downs.get(&code).filter(|&&v| v == 1).is_some()
    }

    /// returns true if a button is held down
    pub fn button_down(&self, button: Button) -> bool {
        self.button_downs
            .get(&button)
            .filter(|&&v| v != 0)
            .is_some()
    }

    /// returns true the amount of frames a button has been down for
    pub fn button_down_frames(&self, button: Button) -> Option<u32> {
        self.button_downs.get(&button).filter(|&&v| v != 0).copied()
    }

    /// returns true if a button is pressed for a single frame
    pub fn button_pressed(&self, button: Button) -> bool {
        self.button_downs
            .get(&button)
            .filter(|&&v| v == 1)
            .is_some()
    }

    /// returns true if a button or a key is held down
    pub fn kb_down(&self, code: KeyCode, button: Button) -> bool {
        self.key_down(code) || self.button_down(button)
    }

    /// returns true the amount of frames a button or a key has been down for
    pub fn kb_down_frames(&self, code: KeyCode, button: Button) -> Option<u32> {
        let mut result = None;

        if let Some(frames) = self.key_down_frames(code) {
            result = Some(frames);
        }

        if let Some(frames) = self.button_down_frames(button) {
            if let Some(old_frames) = result {
                result = Some(old_frames.max(frames));
            } else {
                result = Some(frames);
            }
        }

        result
    }

    /// returns true if a button or a key is pressed for a single frame
    pub fn kb_pressed(&self, code: KeyCode, button: Button) -> bool {
        self.key_pressed(code) || self.button_pressed(button)
    }

    /// pushes a quad to the list of quads to draw
    pub fn push_quad(&mut self, quad: Quad) {
        if self.quads.len() < Quad::MAX {
            self.quads.push(quad);
        }
    }

    /// pushes a line transformed into a quad
    pub fn push_line(&mut self, line: Line) {
        if self.quads.len() < Quad::MAX {
            //self.quads.push(line.into());
        }
    }

    /// pushes a sprite to the anonymous sprites
    pub fn push_sprite(&mut self, sprite: Sprite) {
        if self.quads.len() < Quad::MAX {
            self.quads.push(sprite.into());
        }
    }
	}

impl EventHandler for App {
	fn update(&mut self, ctx: &mut Context) {
		if self.key_pressed(KeyCode::Escape) {
			ctx.quit();
		}
		
		self.push_sprite(Sprite {
							 position: v2(100., 100.),
							 ..Default::default()
						 });
		
		self.push_sprite(Sprite {
							 position: v2(0., 0.),
							 ..Default::default()
						 });
		
		self.push_sprite(Sprite {
							 position: v2(10., 10.),
							 ..Default::default()
						 });
		
		self.bindings.vertex_buffers[1].update(ctx, &self.quads[..]);
		self.quads.clear();
	}
	
	fn draw(&mut self, ctx: &mut Context) {
		let (width, height) = ctx.screen_size();
		let projection = ortho(0., width, 0., height, -1., 1.);
		
		  ctx.begin_default_pass(Default::default());
		
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&shader::Uniforms { projection });
        ctx.draw(0, 6, Quad::MAX as i32);
        ctx.end_render_pass();
		
        ctx.commit_frame();
	}
	
	fn key_down_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
		if let Some(value) = self.key_downs.get_mut(&keycode) {
				if *value == 0 {
				*value = 1;
				}
		} else {
				self.key_downs.insert(keycode, 1);
		}
	}
	
	fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, _: KeyMods) {
		if let Some(value) = self.key_downs.get_mut(&keycode) {
				*value = 0;
		}
	}
}

mod shader {
    use miniquad::*;
	use crate::helpers::M4;
	
    pub const VERTEX: &str = r#"#version 100
		attribute vec2 v_pos;
	
	// wgpu doesnt support VertexFormat Mat4, so i piece them together
	attribute mat4 i_model;
	attribute vec2 i_tiles;
	attribute float i_hframe;
	attribute float i_vframe;
	attribute float i_depth;
	
    uniform mat4 projection;
    
	void main() {
		vec2 test = i_tiles;
		 float test2 = i_hframe;
		 float test4 = i_vframe;
		 float test3 = i_depth;
		
         gl_Position = projection * i_model * vec4(v_pos, 0.5, 1.);
    }
    "#;
		
		pub const FRAGMENT: &str = r#"#version 100
		void main() {
        gl_FragColor = vec4(1., 0., 0., 1.);
    }
    "#;
		
		pub const META: ShaderMeta = ShaderMeta {
        images: &[],
        uniforms: UniformBlockLayout {
            uniforms: &[("projection", UniformType::Mat4)],
        },
    };
	
    #[repr(C)]
		pub struct Uniforms {
        pub projection: M4,
    }
}