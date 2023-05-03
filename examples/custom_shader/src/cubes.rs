use crate::camera::Camera;
use bytemuck::{Pod, Zeroable};
use glam::{vec3, vec4, Vec3};
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, Layout, Widget};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced_graphics::custom::{Program, RenderState};
use iced_graphics::primitive::CustomPipeline;
use iced_graphics::{Backend, Primitive, Transformation};
use std::collections::hash_map::DefaultHasher;
use std::convert::Into;
use std::hash::{Hash, Hasher};
use std::ops::Neg;
use std::time::Duration;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, CommandEncoder, Device, IndexFormat,
    TextureView,
};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    curr_uniform: Uniforms,
    camera: Camera,
    depth_view: wgpu::TextureView,
}

#[derive(Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct Vertex3D {
    position: glam::Vec4,
    color: [f32; 4],
}

fn v3d(pos: glam::Vec3, color: Color) -> Vertex3D {
    Vertex3D {
        position: glam::vec4(pos.x, pos.y, pos.z, 1.0),
        color: color.into_linear(),
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
            0, 1, 2, 0, 3, 2, //front
            0, 4, 1, 1, 5, 4, //left
            4, 5, 6, 6, 7, 4, //back
            7, 3, 2, 2, 6, 7, //right
            0, 3, 7, 7, 4, 0, //bottom
            1, 2, 6, 6, 5, 1, //top
        ])
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    projection: glam::Mat4,
    time: f32,
    _padding: [f32; 3],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            projection: glam::Mat4::IDENTITY,
            time: 0.0,
            _padding: [0.0; 3],
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.projection = camera.build_view_proj_matrix();
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
struct Cube {
    vertices: [Vertex3D; 8],
    dir: Vec3,
    _padding: f32,
}

impl Cube {
    fn new(dir: Vec3) -> Self {
        Self {
            vertices: [
                //front vertices
                v3d(
                    vec3(-1.0, -1.0, 1.0),
                    Color::from_rgba8(75, 118, 156, 0.8),
                ),
                v3d(
                    vec3(-1.0, 1.0, 1.0),
                    Color::from_rgba8(179, 245, 255, 0.8),
                ),
                v3d(vec3(1.0, 1.0, 1.0), Color::from_rgba8(179, 245, 255, 0.8)),
                v3d(vec3(1.0, -1.0, 1.0), Color::from_rgba8(75, 118, 156, 0.8)),
                //back vertices
                v3d(
                    vec3(-1.0, -1.0, -1.0),
                    Color::from_rgba8(48, 86, 120, 0.8),
                ),
                v3d(
                    vec3(-1.0, 1.0, -1.0),
                    Color::from_rgba8(115, 208, 222, 0.8),
                ),
                v3d(
                    vec3(1.0, 1.0, -1.0),
                    Color::from_rgba8(115, 208, 222, 0.8),
                ),
                v3d(vec3(1.0, -1.0, -1.0), Color::from_rgba8(48, 86, 120, 0.8)),
            ],
            dir,
            _padding: 0.0,
        }
    }

    fn scale(&mut self, scale: f32) {
        for v in self.vertices.iter_mut() {
            v.position.x *= scale;
            v.position.y *= scale;
            v.position.z *= scale;
        }
    }
}

impl Pipeline {
    pub fn init(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Box<dyn Program + 'static> {
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cubes vertex buffer"),
            size: std::mem::size_of::<[Vertex3D; 8]>() as u64, //allocate enough space for 100 cubes
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indices = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("cubes index buffer"),
            contents: bytemuck::bytes_of(&Indices::new()),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera = Camera::default();

        let mut u = Uniforms::new();
        u.update(&camera);

        let uniforms = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("cubes uniform buffer"),
            contents: bytemuck::bytes_of(&u),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cubes uniform bind group layout"),
                entries: &[BindGroupLayoutEntry {
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
                entries: &[BindGroupEntry {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex3D>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            //position
                            0 => Float32x4,
                            //color
                            1 => Float32x4,
                        ],
                    }],
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
                    depth_compare: wgpu::CompareFunction::Less,
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
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Src,
                                dst_factor: wgpu::BlendFactor::Dst,
                                operation: wgpu::BlendOperation::Subtract,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Box::new(Self {
            pipeline,
            uniforms,
            uniform_bind_group,
            vertices,
            indices,
            curr_uniform: u,
            camera,
            depth_view,
        })
    }
}

impl Program for Pipeline {
    fn update(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
        time: Duration,
    ) {
        let mut cube = Cube::new(Vec3::Z);
        cube.scale(0.5);

        self.curr_uniform.time = time.as_secs_f32();

        queue.write_buffer(
            &self.uniforms,
            0,
            bytemuck::bytes_of(&self.curr_uniform),
        );

        // for the sake of this simple example, we are just rewriting to all buffers every frame
        queue.write_buffer(
            &self.vertices,
            0,
            bytemuck::bytes_of(&cube.vertices),
        );
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        _device: &Device,
        target: &TextureView,
        clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
    ) -> RenderState {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cubes render_pass)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some(background_color) => wgpu::LoadOp::Clear({
                                let [r, g, b, a] =
                                    background_color.into_linear();

                                wgpu::Color {
                                    r: f64::from(r),
                                    g: f64::from(g),
                                    b: f64::from(b),
                                    a: f64::from(a),
                                }
                            }),
                            None => wgpu::LoadOp::Load,
                        },
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass
            .set_index_buffer(self.indices.slice(..), IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.draw_indexed(0..36, 0, 0..1);

        RenderState::Dirty
    }
}
