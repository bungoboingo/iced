use crate::camera::Camera;
use crate::cube::{Cube, CubeRaw, Vertex3D};
use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};
use iced::{Color, Size};
use iced_graphics::custom::{Program, RenderStatus};
use iced_graphics::Transformation;
use std::time::Duration;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, CommandEncoder, Device, IndexFormat,
    TextureView,
};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vec3 = vec3(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    cubes: Vec<Cube>,
    cubes_buffer: wgpu::Buffer,

    uniforms: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    curr_uniform: Uniforms,
    camera: Camera,
    depth_view: wgpu::TextureView,
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

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    camera_projection: glam::Mat4,
    time: f32,
    _padding: [f32; 3],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            camera_projection: glam::Mat4::IDENTITY,
            time: 0.0,
            _padding: [0.0; 3],
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.camera_projection = camera.build_view_proj_matrix();
    }
}

impl Pipeline {
    pub fn init(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Box<dyn Program + 'static> {
        let cubes = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position =
                        vec3(x as f32, 0.0, z as f32) - INSTANCE_DISPLACEMENT;
                    let rotation = if position == Vec3::ZERO {
                        glam::Quat::from_axis_angle(Vec3::Z, 0.0)
                    } else {
                        glam::Quat::from_axis_angle(position.normalize(), 45.0)
                    };

                    Cube {
                        rotation,
                        position,
                        _padding: 0.0,
                    }
                })
            })
            .collect::<Vec<_>>();

        let raw_cubes = cubes.iter().map(Cube::to_raw).collect::<Vec<_>>();

        let cubes_buffer =
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("cubes instance buffer"),
                contents: bytemuck::cast_slice(&raw_cubes),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cubes vertex buffer"),
            size: std::mem::size_of::<[Vertex3D; 24]>() as u64, //allocate enough space for 100 cubes
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

        //TODO resize with window size
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
                    buffers: &[Vertex3D::desc(), CubeRaw::desc()],
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

        Box::new(Self {
            pipeline,
            cubes,
            cubes_buffer,
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
            bytemuck::bytes_of(&Cube::vertices(0.2)),
        );
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        _device: &Device,
        target: &TextureView,
        _clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
    ) -> RenderStatus {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cubes render_pass)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear({
                            let [r, g, b, a] =
                                Color::from_rgb8(35, 70, 120).into_linear();

                            wgpu::Color {
                                r: f64::from(r),
                                g: f64::from(g),
                                b: f64::from(b),
                                a: f64::from(a),
                            }
                        }),
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
        render_pass.set_vertex_buffer(1, self.cubes_buffer.slice(..));
        render_pass.draw_indexed(0..36, 0, 0..self.cubes.len() as _);

        RenderStatus::RequestRedraw
    }
}
