use crate::core::{Rectangle, Size};
use iced_graphics::Transformation;
use wgpu::util::DeviceExt;
use wgpu::{BufferSize, Extent3d};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    src_texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    uniform_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    transform: [f32; 16],
    //position of the layer
    position: [f32; 2],
    //size of the layer
    size: [f32; 2],
    //amount to be blurred
    blur: f32,
    //for shader use, 1.0 = vertical pass, 0.0 = horizontal pass
    dir: f32,
    _padding: [f32; 2],
}

impl Pipeline {
    pub fn resize_texture(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        new_size: Size<u32>,
    ) -> wgpu::TextureView {
        if new_size.width != self.src_texture.width()
            || new_size.height != self.src_texture.height()
        {
            // screen changed, recreate texture
            self.src_texture =
                Self::create_src_texture(device, format, new_size);
        }

        self.src_texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_src_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: Size<u32>,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("iced_wgpu.blur.source_texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        //allocate quad vertices
        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu.blur.vertex_buffer"),
                contents: bytemuck::cast_slice(&[
                    Vertex {
                        pos: [0.0, 0.0],
                        uv: [0.0, 1.0],
                    },
                    Vertex {
                        pos: [1.0, 0.0],
                        uv: [1.0, 1.0],
                    },
                    Vertex {
                        pos: [1.0, 1.0],
                        uv: [1.0, 0.0],
                    },
                    Vertex {
                        pos: [0.0, 1.0],
                        uv: [0.0, 0.0],
                    },
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu.blur.index_buffer"),
                contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 3, 0]),
                usage: wgpu::BufferUsages::INDEX,
            });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wpgu.blur.uniforms_buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.blur.sampler"),
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu.blur.uniform.bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("iced_wgpu.blur.shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader/blur.wgsl"),
                )),
            });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("iced_wgpu.blur.pipeline_layout"),
                bind_group_layouts: &[&uniform_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("iced_wgpu.blur.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            //quad position
                            0 => Float32x2,
                            //uv
                            1 => Float32x2,
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let src_texture =
            Self::create_src_texture(device, format, Size::new(1, 1));

        Self {
            pipeline,
            vertices,
            indices,
            uniforms,
            sampler,
            uniform_layout,
            bind_group: None,
            src_texture,
        }
    }

    //TODO try Kawase over Gaussian..? or have an option for different blur types..? box filter?
    //TODO need to write all textures to a single texture, making two each pass is no beuno
    //TODO can get rid of nearly all these uniform values; consolidate!
    pub fn render<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        transform: Transformation,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        frame: &wgpu::TextureView,
        src_texture: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        blur: f32,
        bounds: Rectangle,
    ) {
        // Create the texture to render the vertical pass to
        let horizontal_texture =
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("iced_wgpu.horizontal.blur.texture"),
                size: Extent3d {
                    width: bounds.width as u32,
                    height: bounds.height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let horizontal_view = horizontal_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        //first do a vertical render pass to the horizontal texture
        {
            self.bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu.blur.uniform_bind_group.vertical"),
                    layout: &self.uniform_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.uniforms.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                src_texture, //sample src texture
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(
                                &self.sampler,
                            ),
                        },
                    ],
                }));

            // Write uniforms with vertical pass indicator, direction = 1.0
            queue.write_buffer(
                &self.uniforms,
                0,
                bytemuck::bytes_of(&Uniforms {
                    transform: transform.into(),
                    position: [0.0, 0.0],
                    size: [bounds.width, bounds.height],
                    blur,
                    dir: 1.0, //vertical pass
                    _padding: [0.0; 2],
                }),
            );

            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu.blur.vertical_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &horizontal_view, //attach the horizontal texture view
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_index_buffer(
                self.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            pass.draw_indexed(0..6, 0, 0..1);
        }

        //now we render the horizontal pass to the frame, sampling the vertical pass texture just rendered to
        {
            //rewrite uniforms indicating that we are now doing a horizontal pass, direction = 0.0
            queue.write_buffer(
                &self.uniforms,
                0,
                bytemuck::bytes_of(&Uniforms {
                    transform: transform.into(),
                    position: [bounds.x, bounds.y],
                    size: [bounds.width, bounds.height],
                    blur,
                    dir: 0.0, //horizontal
                    _padding: [0.0; 2],
                }),
            );

            // recreate bind group with new horizontal texture to sample
            self.bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("iced_wgpu.blur.uniform_bind_group.vertical"),
                    layout: &self.uniform_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.uniforms.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                &horizontal_view, //sample horizontal texture
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(
                                &self.sampler,
                            ),
                        },
                    ],
                }));

            //create new pass targeting the frame's surface
            let mut pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu.blur.vertical_pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &frame,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            pass.set_pipeline(&self.pipeline);
            pass.set_scissor_rect(
                bounds.x as u32,
                bounds.y as u32,
                bounds.width as u32,
                bounds.height as u32,
            );
            pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_index_buffer(
                self.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
