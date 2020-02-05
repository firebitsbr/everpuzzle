use crate::engine::app::{DEPTH_FORMAT, PROJECTION_SIZE};
use crate::helpers::*;

// temp holder of all info so you dont have to give 4 extra arguments per call making life easier
pub struct ShaderInfo<'a> {
    pub device: &'a wgpu::Device,
    pub ubo_projection: &'a wgpu::Buffer,
    pub texture_view: &'a wgpu::TextureView,
    pub sampler: &'a wgpu::Sampler,
}

impl<'a> ShaderInfo<'a> {
    fn pipeline(
        &self,
        bind_group_layout: &wgpu::BindGroupLayout,
        file_name: &'static str,
    ) -> wgpu::RenderPipeline {
        // load all shaders
        let vs_module = {
            let name = format!("shaders/{}.vert.spv", file_name);
            let file = std::fs::File::open(name).expect("FS: file open failed");
            self.device
                .create_shader_module(&wgpu::read_spirv(file).unwrap())
        };

        let fs_module = {
            let name = format!("shaders/{}.frag.spv", file_name);
            let file = std::fs::File::open(name).expect("FS: file open failed");
            self.device
                .create_shader_module(&wgpu::read_spirv(file).unwrap())
        };

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&bind_group_layout],
            });

        // pipeline is always the same with depth
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    front_face: wgpu::FrontFace::Cw,
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
                vertex_buffers: &[],
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            })
    }

    // bind a buffer type with its info and return the layout and itself
    fn bind(
        &self,
        ty: wgpu::BindingType,
        ubo: &wgpu::Buffer,
        data_size: u64,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        wgpu::BindGroupLayoutBinding {
                            binding: 3,
                            visibility: wgpu::ShaderStage::VERTEX,
                            ty,
                        },
                    ],
                });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &self.ubo_projection,
                        range: 0..PROJECTION_SIZE,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: ubo,
                        range: 0..data_size,
                    },
                },
            ],
        });

        (bind_group, bind_group_layout)
    }

    // returns a new bind_group and its pipeline created from the shader and buffer info
    pub fn uniform(
        &self,
        file_name: &'static str,
        ubo: &wgpu::Buffer,
        data_size: u64,
    ) -> (wgpu::BindGroup, wgpu::RenderPipeline) {
        let (bind_group, bind_group_layout) = self.bind(
            wgpu::BindingType::UniformBuffer { dynamic: false },
            ubo,
            data_size,
        );

        (bind_group, self.pipeline(&bind_group_layout, file_name))
    }

    // returns a new bind_group and its pipeline created from the shader and buffer info
    pub fn storage(
        &self,
        file_name: &'static str,
        sbo: &wgpu::Buffer,
        data_size: u64,
    ) -> (wgpu::BindGroup, wgpu::RenderPipeline) {
        let (bind_group, bind_group_layout) = self.bind(
            wgpu::BindingType::StorageBuffer {
                dynamic: false,
                readonly: false,
            },
            sbo,
            data_size,
        );

        (bind_group, self.pipeline(&bind_group_layout, file_name))
    }
}
