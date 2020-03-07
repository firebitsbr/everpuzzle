use crate::engine::{DEPTH_FORMAT, RENDER_FORMAT, TEXTURE_FORMAT};
use crate::helpers::*;

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

/// converts a line into a valid quad
impl From<Line> for Quad {
    fn from(line: Line) -> Self {
        let model = M4::from_translation(v3(
																						 line.start.x,
																						 line.start.y,
																						 0.,
																						 ));
		
		/*
		  let temp = line.end - line.start;
		  model = model * M4::from_rotation_z(-temp.x.atan2(temp.y) + PI / 2.);
		
		  // always centered
		  model = model * M4::from_translation(v3(
																	 -line.thickness / 2., 
																	 -line.thickness / 2.,
																	 0.,
																	 ));
		
		// TODO(Skytrias): 
		  let dimensions = V2::new(line.start.distance(line.end), line.thickness);
		  model = model * M4::from_nonuniform_scale(v4(dimensions.x, dimensions.y, 1., 1.));
	*/
		
		  Quad {
			  model,
			  tiles: V2::new(1., 1.),
			  hframe: line.hframe as f32,
			  vframe: line.vframe as f32,
			  depth: 0.1,
		  }
	  }
  }
  
  /// quad pipeline to draw any sprite / quad
  pub struct Pipeline {
	  pipeline: wgpu::RenderPipeline,
	  bind_group: wgpu::BindGroup,
	  instances: wgpu::Buffer,
  }
  
  impl Pipeline {
	  /// initializes the quad pipeline properly with a given ubo projection buffer
	  pub fn new(
		  device: &mut wgpu::Device,
		  queue: &mut wgpu::Queue,
		  ubo_projection: &wgpu::Buffer,
	  ) -> Pipeline {
		  let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			  bindings: &[
				  wgpu::BindGroupLayoutBinding {
					  binding: 0,
					  visibility: wgpu::ShaderStage::VERTEX,
					  ty: wgpu::BindingType::UniformBuffer { dynamic: false },
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
  
		  // load our single texture atlas into ubo
		  let texture_view = {
			  // load the texture and its info
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
  
		  let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			  layout: &bind_group_layout,
			  bindings: &[
				  wgpu::Binding {
					  binding: 0,
					  resource: wgpu::BindingResource::Buffer {
						  buffer: &ubo_projection,
						  range: 0..std::mem::size_of::<M4>() as u64,
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
  
		  let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			  bind_group_layouts: &[&bind_group_layout],
		  });
  
		  let vs_module = {
			  let file = load_file!("shaders/quad.vert.spv");
			  device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&file[..])).unwrap())
		  };
  
		  let fs_module = {
			  let file = load_file!("shaders/quad.frag.spv");
			  device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&file[..])).unwrap())
		  };
  
		  let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			  layout: &layout,
			  vertex_stage: wgpu::ProgrammableStageDescriptor {
				  module: &vs_module,
				  entry_point: "main",
			  },
			  fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
				  module: &fs_module,
				  entry_point: "main",
			  }),
			  rasterization_state: Some(wgpu::RasterizationStateDescriptor {
				  front_face: wgpu::FrontFace::Cw,
				  cull_mode: wgpu::CullMode::None,
				  depth_bias: 0,
				  depth_bias_slope_scale: 0.0,
				  depth_bias_clamp: 0.0,
			  }),
			  primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
			  color_states: &[wgpu::ColorStateDescriptor {
				  format: RENDER_FORMAT,
				  color_blend: wgpu::BlendDescriptor {
					  src_factor: wgpu::BlendFactor::SrcAlpha,
					  dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					  operation: wgpu::BlendOperation::Add,
				  },
				  alpha_blend: wgpu::BlendDescriptor {
					  src_factor: wgpu::BlendFactor::One,
					  dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
					  operation: wgpu::BlendOperation::Add,
				  },
				  write_mask: wgpu::ColorWrite::ALL,
			  }],
			  depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
				  format: DEPTH_FORMAT,
				  depth_write_enabled: true,
				  depth_compare: wgpu::CompareFunction::Less,
				  stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
				  stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
				  stencil_read_mask: 0,
				  stencil_write_mask: 0,
			  }),
			  index_format: wgpu::IndexFormat::Uint16,
			  vertex_buffers: &[wgpu::VertexBufferDescriptor {
				  stride: Quad::SIZE as u64,
				  step_mode: wgpu::InputStepMode::Instance,
				  attributes: &[
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 0,
						  format: wgpu::VertexFormat::Float4,
						  offset: 0,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 1,
						  format: wgpu::VertexFormat::Float4,
						  offset: 4 * 4,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 2,
						  format: wgpu::VertexFormat::Float4,
						  offset: 4 * 8,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 3,
						  format: wgpu::VertexFormat::Float4,
						  offset: 4 * 12,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 4,
						  format: wgpu::VertexFormat::Float4,
						  offset: 4 * 16,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 5,
						  format: wgpu::VertexFormat::Float,
						  offset: 4 * 18,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 6,
						  format: wgpu::VertexFormat::Float,
						  offset: 4 * 19,
					  },
					  wgpu::VertexAttributeDescriptor {
						  shader_location: 7,
						  format: wgpu::VertexFormat::Float,
						  offset: 4 * 20,
					  },
				  ],
			  }],
			  sample_count: 1,
			  sample_mask: !0,
			  alpha_to_coverage_enabled: false,
		  });
  
		  let instances = device.create_buffer(&wgpu::BufferDescriptor {
			  size: Quad::SIZE as u64 * Quad::MAX as u64,
			  usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
		  });
  
		  Pipeline {
			  pipeline,
			  bind_group,
			  instances,
		  }
	  }
  
	  /// draws any amount of instances given in a single draw call
	  pub fn draw(
		  &mut self,
		  device: &wgpu::Device,
		  encoder: &mut wgpu::CommandEncoder,
		  depth_texture_view: &wgpu::TextureView,
		  texture_view: &wgpu::TextureView,
		  instances: &[Quad],
	  ) {
		  let total = instances.len();
  
		  // copy instance data into buffer
		  let instance_buffer = device
			  .create_buffer_mapped(total, wgpu::BufferUsage::COPY_SRC)
			  .fill_from_slice(&instances[..total]);
  
		  encoder.copy_buffer_to_buffer(
			  &instance_buffer,
			  0,
			  &self.instances,
			  0,
			  (Quad::SIZE * total) as u64,
		  );
  
		  {
			  let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				  color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
					  attachment: &texture_view,
					  resolve_target: None,
					  load_op: wgpu::LoadOp::Clear,
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
  
			  render_pass.set_pipeline(&self.pipeline);
			  render_pass.set_bind_group(0, &self.bind_group, &[]);
			  render_pass.set_vertex_buffers(0, &[(&self.instances, 0)]);
  
			  // TODO(Skytrias): do clipping
			  /*
			   render_pass.set_scissor_rect(
							 bounds.x,
							 bounds.y,
							 bounds.width,
							 // TODO: Address anti-aliasing adjustments properly
							 bounds.height + 1,
							 );
			  */

            render_pass.draw(0..4, 0..total as u32);
        }
    }
}
