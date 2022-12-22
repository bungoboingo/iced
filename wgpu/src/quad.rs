use crate::{buffer, Transformation};
use iced_graphics::layer;
use iced_native::Rectangle;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages,
    ShaderStages,
};

use wgpu::util::DeviceExt;

#[cfg(feature = "tracing")]
use tracing::info_span;

#[derive(Debug)]
pub struct Pipeline {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    uniforms: buffer::Static<Uniforms>,
    uniforms_bind_group: wgpu::BindGroup,
    solid: solid::Pipeline,
    gradient: gradient::Pipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipeline {
        let vertices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTS),
                usage: BufferUsages::VERTEX,
            });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("iced_wgpu::quad index buffer"),
                contents: bytemuck::cast_slice(&QUAD_INDICES),
                usage: BufferUsages::INDEX,
            });

        let uniforms = buffer::Static::new(
            device,
            "wgpu::quad uniforms buffer",
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            1,
        );

        let uniforms_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("wgpu::quad uniforms bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<Uniforms>()
                                as wgpu::BufferAddress,
                        ),
                    },
                    count: None,
                }],
            });

        let uniforms_bind_group =
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("wgpu::quad uniform bind group"),
                layout: &uniforms_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniforms.raw().as_entire_binding(),
                }],
            });

        let solid = solid::Pipeline::new(device, format, &uniforms_layout);
        let gradient =
            gradient::Pipeline::new(device, format, &uniforms_layout);

        Pipeline {
            vertices,
            indices,
            uniforms,
            uniforms_bind_group,
            solid,
            gradient,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        instances: &layer::Quads,
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
    ) {
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Quad", "DRAW").entered();

        // write uniforms to GPU
        let _ = self.uniforms.write(
            device,
            staging_belt,
            encoder,
            0,
            &[Uniforms::new(transformation, scale)],
        );

        // resize buffers if necessary
        let _ = self.solid.instances.resize(device, instances.solids.len());
        let _ = self
            .gradient
            .instances
            .resize(device, instances.gradients.len());

        // write instances to solid/gradient pipelines
        let _ = self.solid.instances.write(
            device,
            staging_belt,
            encoder,
            0,
            instances.solids.as_slice(),
        );

        let _ = self.gradient.instances.write(
            device,
            staging_belt,
            encoder,
            0,
            instances.gradients.as_slice(),
        );

        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Quad", "BEGIN_RENDER_PASS").enter();

        //done writing, begin render pass
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iced_wgpu::quad render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(
            self.indices.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.set_scissor_rect(
            bounds.x,
            bounds.y,
            bounds.width,
            // TODO: Address anti-aliasing adjustments properly
            bounds.height,
        );

        // draw solid quads
        if !instances.solids.is_empty() {
            render_pass.set_pipeline(&self.solid.pipeline);
            render_pass
                .set_vertex_buffer(1, self.solid.instances.raw().slice(..));
            render_pass.draw_indexed(
                0..QUAD_INDICES.len() as u32,
                0,
                0..instances.solids.len() as u32,
            );
        }

        //draw gradient quads
        if !instances.gradients.is_empty() {
            render_pass.set_pipeline(&self.gradient.pipeline);
            render_pass
                .set_vertex_buffer(1, self.gradient.instances.raw().slice(..));
            render_pass.draw_indexed(
                0..QUAD_INDICES.len() as u32,
                0,
                0..instances.gradients.len() as u32,
            );
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    _position: [f32; 2],
}

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const QUAD_VERTS: [QuadVertex; 4] = [
    QuadVertex {
        _position: [0.0, 0.0],
    },
    QuadVertex {
        _position: [1.0, 0.0],
    },
    QuadVertex {
        _position: [1.0, 1.0],
    },
    QuadVertex {
        _position: [0.0, 1.0],
    },
];

const INITIAL_INSTANCES: usize = 10_000;

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
    // Uniforms must be aligned to their largest member,
    // this uses a mat4x4<f32> which aligns to 16, so align to that
    _padding: [f32; 3],
}

impl Uniforms {
    fn new(transformation: Transformation, scale: f32) -> Uniforms {
        Self {
            transform: *transformation.as_ref(),
            scale,
            _padding: [0.0; 3],
        }
    }
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            scale: 1.0,
            _padding: [0.0; 3],
        }
    }
}

mod solid {
    use crate::buffer;
    use crate::quad::{QuadVertex, INITIAL_INSTANCES};
    use iced_graphics::layer::quad;
    use std::mem;

    #[derive(Debug)]
    pub struct Pipeline {
        pub pipeline: wgpu::RenderPipeline,
        pub instances: buffer::Static<quad::Solid>,
    }

    impl Pipeline {
        pub fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            uniforms_layout: &wgpu::BindGroupLayout,
        ) -> Self {
            let instances = buffer::Static::new(
                device,
                "iced_wgpu::quad::solid instance buffer",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                INITIAL_INSTANCES,
            );

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu::quad::solid pipeline layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[uniforms_layout],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("iced_wgpu::quad::solid shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(include_str!(
                            "shader/quad.wgsl"
                        )),
                    ),
                });

            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu::quad::solid pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "solid_vs_main",
                        buffers: &[
                            wgpu::VertexBufferLayout {
                                array_stride: mem::size_of::<QuadVertex>()
                                    as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    shader_location: 0,
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: 0,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: mem::size_of::<quad::Solid>()
                                    as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: &wgpu::vertex_attr_array!(
                                    // Color
                                    1 => Float32x4,
                                    // Position
                                    2 => Float32x2,
                                    // Size
                                    3 => Float32x2,
                                    // Border color
                                    4 => Float32x4,
                                    // Border radius
                                    5 => Float32x4,
                                    // Border width
                                    6 => Float32,
                                ),
                            },
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "solid_fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor:
                                        wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor:
                                        wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Cw,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                },
            );

            Self {
                pipeline,
                instances,
            }
        }
    }
}

mod gradient {
    use crate::buffer;
    use crate::quad::{QuadVertex, INITIAL_INSTANCES};
    use iced_graphics::layer::quad;
    use std::mem;

    #[derive(Debug)]
    pub struct Pipeline {
        pub pipeline: wgpu::RenderPipeline,
        pub instances: buffer::Static<quad::Gradient>,
    }

    impl Pipeline {
        pub fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            uniforms_layout: &wgpu::BindGroupLayout,
        ) -> Self {
            let instances = buffer::Static::new(
                device,
                "iced_wgpu::quad::gradient instance buffer",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                INITIAL_INSTANCES,
            );

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu::quad::gradient pipeline layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts: &[uniforms_layout],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("iced_wgpu::quad::gradient shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(include_str!(
                            "shader/quad.wgsl"
                        )),
                    ),
                });

            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu::quad::gradient pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "gradient_vs_main",
                        buffers: &[
                            wgpu::VertexBufferLayout {
                                array_stride: mem::size_of::<QuadVertex>()
                                    as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[wgpu::VertexAttribute {
                                    shader_location: 0,
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: 0,
                                }],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: mem::size_of::<quad::Gradient>()
                                    as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: &wgpu::vertex_attr_array!(
                                    // Color 1
                                    1 => Float32x4,
                                    // Color 2
                                    2 => Float32x4,
                                    // Color 3
                                    3 => Float32x4,
                                    // Color 4
                                    4 => Float32x4,
                                    // Color 5
                                    5 => Float32x4,
                                    // Color 6
                                    6 => Float32x4,
                                    // Color 7
                                    7 => Float32x4,
                                    // Color 8
                                    8 => Float32x4,
                                    // Offsets 1-4
                                    9 => Float32x4,
                                    // Offsets 5-8
                                    10 => Float32x4,
                                    // Direction
                                    11 => Float32x4,
                                    // Position & Scale
                                    12 => Float32x4,
                                    // Border color
                                    13 => Float32x4,
                                    // Border radius
                                    14 => Float32x4,
                                    // Border width
                                    15 => Float32
                                ),
                            },
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "gradient_fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor:
                                        wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor:
                                        wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Cw,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                },
            );

            Self {
                pipeline,
                instances,
            }
        }
    }
}
