use std::mem;
use bytemuck::{Pod, Zeroable};
use iced::event::Status;
use iced::mouse::Interaction;
use iced::{Color, Rectangle, Renderer, Size};
use iced_graphics::custom::{Cursor, Event, Renderable};
use iced_graphics::{primitive, Transformation};
use wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureView};
use wgpu::util::DeviceExt;

pub struct Program;

#[derive(Default)]
struct StateWrapper(Option<State>);

struct State {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    primitives: Vec<Cube>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
    _padding: [f32; 3],
}

impl Renderable for StateWrapper {
    fn init(&mut self, device: &Device, format: wgpu::TextureFormat) {
        let vertices = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("cubes vertex buffer"),
                size: 0,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let indices = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("cubes index buffer"),
                contents: bytemuck::bytes_of(&CubeIndices::new()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let uniforms = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("cubes uniform buffer"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let uniforms_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cubes uniform layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        mem::size_of::<Uniforms>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("cubes uniform bind group"),
                layout: &uniforms_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &uniforms,
                            offset: 0,
                            size: None,
                        }
                    ),
                }],
            }
        );

        let layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("cubes pipeline layout"),
                bind_group_layouts: &[&uniforms_layout],
                push_constant_ranges: &[],
            }
        );

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("cubes shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader.wgsl")
                )),
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("cubes pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: mem::size_of::<Vertex3D> as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![
                                //position
                                0 => Float32x4,
                                //color
                                1 => Float32x4,
                            ]
                        }
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    .. Default::default()
                },
                depth_stencil: None,
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
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            }
        );

        self.0 = Some(
            State {
                pipeline,
                vertices,
                indices,
                uniforms,
                uniform_bind_group,
                primitives: vec![],
            }
        );
    }

    fn prepare(
        &mut self,
        _device: &Device,
        queue: &Queue,
        _encoder: &mut CommandEncoder,
        scale_factor: f32,
        transformation: Transformation,
    ) {
        if let Some(state) = &self.0 {
            let vertices = state.primitives.iter().fold(vec![], |mut acc, cube| {
                acc.extend(cube.attributes());
                acc
            });

            // for the sake of this simple example, we are just rewriting to all buffers every frame
            queue.write_buffer(&state.vertices, 0, bytemuck::cast_slice(&vertices));
            queue.write_buffer(
                &state.uniforms,
                0,
                bytemuck::bytes_of(&Uniforms {
                    transform: transformation.into(),
                    scale: scale_factor,
                    _padding: [0.0; 3],
                }),
            );
        }
    }

    fn render(
        &self,
        render_pass: &mut RenderPass,
        device: &Device,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clear_color: Option<Color>,
        scale_factor: f32,
        target_size: Size<u32>,
    ) {
        if let Some(state) = &self.0 {
            render_pass.set_pipeline(&state.pipeline);

            render_pass.set_bind_group(0, &state.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, state.vertices.slice(..));
            render_pass.set_index_buffer(
                state.indices.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..36, 0, 0..state.primitives.len() as u32);
        }
    }
}

impl<Message, Theme> iced_graphics::custom::Program<Message, Renderer<Theme>>
    for Program
{
    type State = StateWrapper;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (Status, Option<Message>) {
        todo!()
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &mut Renderer<Theme>,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) {
        //TODO okay I can't do this :-(
        renderer.draw_primitive(primitive::custom(Box::new(state)));
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Interaction {
        todo!()
    }
}

struct Cube {
    origin: [f32; 3],
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
struct Vertex3D {
    pos: [f32; 4],
    color: [f32; 4],
}

impl Vertex3D {
    fn translate(mut self, other: [f32; 3]) -> Self {
        self.pos[0] += other[0];
        self.pos[1] += other[1];
        self.pos[2] += other[2];
        self
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct CubeIndices {
    indices: [u16; 36],
}

unsafe impl bytemuck::Pod for CubeIndices {}
unsafe impl bytemuck::Zeroable for CubeIndices {}

impl CubeIndices {
    fn new() -> Self {
        Self {
            indices: [
                0, 1, 2, 1, 2, 3, //front face
                0, 2, 5, 0, 5, 6, //left face
                4, 5, 6, 4, 6, 7, //back face
                1, 4, 7, 1, 4, 3, //right face
                2, 3, 4, 2, 4, 5, //bottom face
                0, 1, 7, 0, 7, 6, //top face
            ],
        }
    }
}

fn vertex_3d(x: f32, y: f32, z: f32, color: Color) -> Vertex3D {
    Vertex3D {
        pos: [x, y, z, 0.0],
        color: color.into_linear(),
    }
}

impl Cube {
    /// Returns the vertices and indices of the cube.
    fn attributes(&self) -> [Vertex3D; 8] {
        const RED: Color = Color::from_rgb8(255, 0, 0);
        const GREEN: Color = Color::from_rgb8(0, 255, 0);

        [
            vertex_3d(-0.5, -0.5, 0.5, RED).translate(self.origin), //front top left
            vertex_3d(0.5, -0.5, 0.5, RED).translate(self.origin), //front top right
            vertex_3d(-0.5, -0.5, -0.5, RED).translate(self.origin), //front bottom left
            vertex_3d(0.5, -0.5, -0.5, RED).translate(self.origin), //front bottom right
            vertex_3d(0.5, 0.5, -0.5, GREEN).translate(self.origin), //back bottom right
            vertex_3d(-0.5, 0.5, -0.5, GREEN).translate(self.origin), //back bottom left
            vertex_3d(-0.5, 0.5, 0.5, GREEN).translate(self.origin), //back top left
            vertex_3d(0.5, 0.5, 0.5, GREEN).translate(self.origin), //back top right
        ]
    }
}


