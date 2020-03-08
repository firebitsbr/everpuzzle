use miniquad::*;
use crate::helpers::*;

const VERTEX: &str = r#"#version 100
attribute vec2 v_pos;

attribute mat4 i_model;
attribute vec2 i_tiles;
attribute float i_hframe;
attribute float i_vframe;
attribute float i_depth;

uniform mat4 projection;

varying highp vec2 o_uv;

void main() {
	gl_Position = projection * i_model * vec4(v_pos, i_depth, 1.);
	vec2 i_uv = v_pos * i_tiles;
	o_uv.x = (i_hframe + i_uv.x) * (1. / 26.);
    o_uv.y = (i_vframe + i_uv.y) * (1. / 13.);
}
"#;

const FRAGMENT: &str = r#"#version 100
varying highp vec2 o_uv;

uniform sampler2D texture;

void main() {
	highp vec4 texture_color = texture2D(texture, o_uv);
	
    // discard useless alpha
    if (texture_color.a <= 0.0) {
        discard;
    }
	
	gl_FragColor = texture_color;
}
"#;

const META: ShaderMeta = ShaderMeta {
	images: &["texture"],
	uniforms: UniformBlockLayout {
		uniforms: &[("projection", UniformType::Mat4)],
	},
};

#[repr(C)]
pub struct Uniforms {
	pub projection: M4,
}

pub struct Sprites {
	pipeline: Pipeline,
	bindings: Bindings,
	
	/// data storage for all quads in the frame that you want to draw
    quads: Vec<Quad>,
}

impl Sprites {
	pub fn new(ctx: &mut Context) -> Self {
		#[rustfmt::skip]
			let vertices = [
							v2(0., 0.),
							v2(1., 0.),
							v2(1., 1.),
							v2(0., 1.),
							];
		let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
		
		#[rustfmt::skip]
			let indices: &[i16] = &[
									0, 1, 3,
									1, 2, 3,
									];
		let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, indices);
		
		let instance_buffer = Buffer::stream(
											 ctx,
											 BufferType::VertexBuffer,
											 Quad::MAX * Quad::SIZE,
											 );
		  
		let texture = {
		let data = load_file!("textures/atlas.png");
			let data = std::io::Cursor::new(data);
		 
		 let decoder = png_pong::FrameDecoder::<_, pix::Rgba8>::new(data);
		 let png_pong::Frame { raster, .. } = decoder
			.last()
			.expect("No frames in PNG")
			.expect("PNG parsing error");
		 let width = raster.width();
		 let height = raster.height();
			let texels = raster.as_u8_slice();
   
			Texture::from_rgba8(ctx, width as u16, height as u16, &texels)
		 };
		 texture.set_filter(ctx, FilterMode::Nearest);
		 
		   let bindings = Bindings {
			   vertex_buffers: vec![vertex_buffer, instance_buffer],
			   index_buffer: index_buffer,
			   images: vec![texture],
		   };
		   
		 let shader = Shader::new(ctx, VERTEX, FRAGMENT, META);
		 
		   let pipeline = Pipeline::with_params(
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
										PipelineParams {
										   depth_test: Comparison::Less,
										   depth_write: true,
										   color_blend: Some((
																		  Equation::Add, 
																		  BlendFactor::Value(BlendValue::SourceAlpha),
																		  BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
																		  )),
										   ..Default::default()
										}
							   );
		 
		 Self {
			quads: Vec::with_capacity(Quad::MAX),
			pipeline,
			bindings,
		 }
	  }
	  
	  /// pushes a sprite to the anonymous sprites
	  pub fn push(&mut self, sprite: Sprite) {
		 if self.quads.len() < Quad::MAX {
			   self.quads.push(sprite.into());
		 }
	  }
	  
	  pub fn text(&mut self, text: Text) {
		 let len = text.content.len();
		 
		 if self.quads.len() - len < Quad::MAX {
			   let mut position = text.position;
			
			for c in text.content.chars() {
			   if c.is_whitespace() {
				  position.x += text.step * text.scale.x;
				  continue;
			   }
			   
			   let (hframe, vframe) = {
				  if c.is_digit(10) {
					 (c.to_digit(10).unwrap(), ATLAS_NUMBERS)
				  } else {
					 (c.to_digit(35).unwrap() - 10, ATLAS_ALPHABET)
				  }
			   };
				  
			   self.push(Sprite {
							 position,
							 hframe,
							 vframe,
							 depth: 0.1,
							 scale: text.scale,
							 ..Default::default()
						  });
			   
			   position.x += text.step * text.scale.x;
			}
		 }
	  }
	  
	  pub fn render(&mut self, ctx: &mut Context) {
		 let (width, height) = ctx.screen_size();
		 let projection = ortho(0., width, height, 0., -1., 1.);
		 
		 self.bindings.vertex_buffers[1].update(ctx, &self.quads[..]);
		 
		 ctx.begin_default_pass(PassAction::Clear {
								 color: Some((1., 1., 1., 1.)),
								 depth: Some(1.),
								 stencil: None,
							  });
		 
		 ctx.apply_pipeline(&self.pipeline);
		 ctx.apply_bindings(&self.bindings);
		 ctx.apply_uniforms(&Uniforms { projection });
		 ctx.draw(0, 6, self.quads.len() as i32);
		 ctx.end_render_pass();
		 
		 self.quads.clear();
	  }
   }
   