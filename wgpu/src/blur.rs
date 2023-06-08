use crate::core::Rectangle;
use wgpu::util::DeviceExt;
use wgpu::BufferSize;
use iced_graphics::Transformation;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    blur_vertices: wgpu::Buffer,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    uniform_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    size: [f32; 2],
    blur: f32,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    transform: [f32; 16],
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        //allocate quad vertices
        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu.blur.vertex_buffer"),
                contents: bytemuck::cast_slice(&[
                    [0.0f32, 0.0], //bottom left
                    [1.0, 0.0], //bottom right
                    [1.0, 1.0], //top right
                    [0.0, 1.0], //top left
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu.blur.index_buffer"),
                contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 3, 0]),
                usage: wgpu::BufferUsages::INDEX,
            });

        //TODO actually add multiple blur instances
        // change layer to have a Blur type primitive maybe so we can group
        let blur_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wgpu.blur.instance_buffer"),
            size: std::mem::size_of::<[Vertex; 4]>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("iced_wpgu.blur.uniforms_buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("iced_wgpu.blur.sampler"),
            ..Default::default()
        });

        let uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("iced_wgpu.blur.uniform.bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(
                                std::mem::size_of::<Uniforms>() as u64,
                            ),
                        },
                        count: None,
                    },
                    // the src texture to sample & blur
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 1,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Texture {
                    //         sample_type: wgpu::TextureSampleType::Float {
                    //             filterable: true,
                    //         },
                    //         view_dimension: wgpu::TextureViewDimension::D2,
                    //         multisampled: false, //TODO sample more than 1 px somehow? idk where to do that
                    //     },
                    //     count: None,
                    // },
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 2,
                    //     visibility: wgpu::ShaderStages::FRAGMENT,
                    //     ty: wgpu::BindingType::Sampler(
                    //         wgpu::SamplerBindingType::Filtering,
                    //     ),
                    //     count: None,
                    // },
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
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<[f32; 2]>()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![
                                //quad position
                                0 => Float32x2,
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![
                                //position
                                1 => Float32x2,
                                //size
                                2 => Float32x2,
                                //blur radius
                                3 => Float32,
                            ],
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self {
            pipeline,
            vertices,
            indices,
            uniforms,
            sampler,
            uniform_layout,
            bind_group: None,
            blur_vertices,
        }
    }

    //TODO try Kawase over Gaussian..? or have an option for different blur types
    //TODO blurred texture caching in image atlas or "blur" atlas.. ?
    pub fn render<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        transform: Transformation,
        // encoder: &mut wgpu::CommandEncoder,
        pass: &mut wgpu::RenderPass<'a>,
        device: &wgpu::Device,
        frame: &wgpu::TextureView,
        // src_texture: &wgpu::TextureView,
        blur: f32,
        bounds: Rectangle,
    ) {
        let vertex = Vertex {
            position: [bounds.x, bounds.y],
            size: [bounds.width, bounds.height],
            blur,
        };

        queue.write_buffer(
            &self.blur_vertices,
            0,
            bytemuck::bytes_of(&[
                vertex,
                vertex,
                vertex,
                vertex,
            ]),
        );

        queue.write_buffer(
            &self.uniforms,
            0,
            bytemuck::bytes_of(&Uniforms {
                transform: transform.into(),
            }),
        );

        self.bind_group =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu.blur.uniform_bind_group"),
                layout: &self.uniform_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            wgpu::BufferBinding {
                                buffer: &self.uniforms,
                                offset: 0,
                                size: BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
                            },
                        ),
                    },
                    // wgpu::BindGroupEntry {
                    //     binding: 1,
                    //     resource: wgpu::BindingResource::TextureView(src_texture),
                    // },
                    // wgpu::BindGroupEntry {
                    //     binding: 2,
                    //     resource: wgpu::BindingResource::Sampler(&self.sampler),
                    // },
                ],
            }));

        if let Some(bind_group) = &self.bind_group {
            // let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            //     label: Some("iced_wgpu.blur.horizontal_pass"),
            //     color_attachments: &[Some(
            //         wgpu::RenderPassColorAttachment {
            //             view: &frame,
            //             resolve_target: None,
            //             ops: wgpu::Operations {
            //                 load: wgpu::LoadOp::Load,
            //                 store: true,
            //             }
            //         }
            //     )],
            //     depth_stencil_attachment: None,
            // });

            pass.set_pipeline(&self.pipeline);
            // pass.set_scissor_rect(
            //     bounds.x as u32,
            //     bounds.y as u32,
            //     bounds.width as u32,
            //     bounds.height as u32,
            // );
            pass.set_bind_group(0, bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_vertex_buffer(1, self.blur_vertices.slice(..));
            pass.set_index_buffer(
                self.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
