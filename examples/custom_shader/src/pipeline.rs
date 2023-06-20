use crate::camera::Camera;
use crate::cube;
use crate::cube::vertex_3d::Vertex3D;
use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};
use iced::{Rectangle, Size};
use std::time::Duration;
use wgpu::util::DeviceExt;
use wgpu::TextureView;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vec3 = vec3(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    pub cubes_buffer: wgpu::Buffer,
    pub cubes_buffer_size: usize,
    pub curr_uniforms: Uniforms,
    pub uniforms: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    pub vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    pub depth_texture_size: Size<u32>,
    depth_view: wgpu::TextureView,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Self {
        // Buffer for a single cube's vertices; can re-use between cubes
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cubes vertex buffer"),
            size: std::mem::size_of::<[Vertex3D; 24]>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("cubes index buffer"),
                contents: bytemuck::bytes_of(&Indices::new()),
                usage: wgpu::BufferUsages::INDEX,
            });

        let cubes_buffer_size = std::mem::size_of::<cube::Raw>();

        // cube instance buffer
        let cubes_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cubes instance buffer"),
            size: cubes_buffer_size as u64,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // camera projection & time for rotation
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cubes uniform buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cubes uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("cubes uniform bind group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                }],
            });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cubes depth texture"),
            size: wgpu::Extent3d {
                width: target_size.width,
                height: target_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });

        let depth_view =
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cubes pipeline layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("cubes shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("cubes pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex3D::desc(), cube::Raw::desc()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
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
            cubes_buffer,
            cubes_buffer_size,
            curr_uniforms: u,
            uniforms,
            uniform_bind_group,
            vertices,
            indices,
            depth_texture_size: target_size,
            depth_view,
        }
    }

    pub fn update_depth_texture(
        &mut self,
        device: &wgpu::Device,
        size: Size<u32>,
    ) {
        if self.depth_texture_size.height != size.height
            || self.depth_texture_size.width != size.width
        {
            let text = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("cubes depth texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[wgpu::TextureFormat::Depth32Float],
            });

            self.depth_view =
                text.create_view(&wgpu::TextureViewDescriptor::default());
            self.depth_texture_size = size;
        }
    }

    pub fn render(
        &self,
        target: &TextureView,
        encoder: &mut wgpu::CommandEncoder,
        bounds: Rectangle<u32>,
        num_cubes: u32,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cubes.pipeline.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                },
            ),
        });

        pass.set_scissor_rect(bounds.x, bounds.y, bounds.width, bounds.height);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_index_buffer(
            self.indices.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_vertex_buffer(1, self.cubes_buffer.slice(..));
        pass.draw_indexed(0..36, 0, 0..num_cubes);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Indices([u16; 36]);

unsafe impl Pod for Indices {}
unsafe impl Zeroable for Indices {}

impl Indices {
    fn new() -> Self {
        Self([
            0, 1, 2, 2, 3, 0, //front
            4, 5, 6, 6, 7, 4, //left
            8, 9, 10, 10, 11, 8, //back
            12, 13, 14, 14, 15, 12, //right
            16, 17, 18, 18, 19, 16, //bottom
            20, 21, 22, 22, 23, 20, //top
        ])
    }
}
