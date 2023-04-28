use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, Layout, Widget};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced_graphics::primitive::{ColoredVertex2D, CustomPipeline, Renderable};
use iced_graphics::{Backend, Primitive, Transformation};
use std::collections::hash_map::DefaultHasher;
use std::convert::Into;
use std::hash::{Hash, Hasher};

pub struct Triangle {
    height: Length,
    width: Length,
    id: u64,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            height: Length::Shrink,
            width: Length::Shrink,
            id: 0,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn id(mut self, id: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        self.id = hasher.finish();
        self
    }
}

impl<'a, B, Theme, Message> From<Triangle>
    for Element<'a, Message, iced_graphics::Renderer<B, Theme>>
where
    Message: 'a,
    B: Backend,
{
    fn from(
        triangle: Triangle,
    ) -> Element<'a, Message, iced_graphics::Renderer<B, Theme>> {
        Element::new(triangle)
    }
}

pub struct State {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
}

impl State {
    fn init(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Box<dyn Renderable + 'static> {
        println!("Initializing pipeline..");
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("triangle vertex buffer"),
            size: std::mem::size_of::<[ColoredVertex2D; 3]>() as u64 * 100, //allocate enough space for 100 triangles
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("triangle pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("triangle shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shader.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("triangle pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ColoredVertex2D>()
                            as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            //position
                            0 => Float32x2,
                            //color
                            1 => Float32x4,
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
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
            });

        Box::new(Self { pipeline, vertices })
    }
}

impl Renderable for State {
    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
    ) {
        // for the sake of this simple example, we are just rewriting to all buffers every frame
        queue.write_buffer(
            &self.vertices,
            0,
            bytemuck::cast_slice(&[
                ColoredVertex2D {
                    position: [0.0, 0.5],
                    color: Color::from_rgb8(255, 0, 0).into_linear(),
                },
                ColoredVertex2D {
                    position: [0.5, -0.5],
                    color: Color::from_rgb8(0, 255, 0).into_linear(),
                },
                ColoredVertex2D {
                    position: [-0.5, -0.5],
                    color: Color::from_rgb8(0, 0, 255).into_linear(),
                },
            ]),
        );
    }

    fn render<'a, 'b>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        _device: &wgpu::Device,
        _target: &wgpu::TextureView,
        _clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
    ) where
        'a: 'b,
    {
        println!("Rendering triangle!");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}

impl<B, T, Message> Widget<Message, iced_graphics::Renderer<B, T>> for Triangle
where
    B: Backend,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &iced_graphics::Renderer<B, T>,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut iced_graphics::Renderer<B, T>,
        _theme: &T,
        _style: &Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        renderer.draw_primitive(Primitive::Custom {
            bounds,
            pipeline: CustomPipeline {
                id: self.id,
                init: State::init,
            },
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
}
