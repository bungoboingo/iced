use iced::{Color, Rectangle, Size};
use iced_graphics::primitive::{ColoredVertex2D, Renderable};
use iced_graphics::{Backend, Renderer, Transformation};
use wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureView};

pub struct Program {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
}

impl iced_graphics::custom::Program for Program {
    type Pipeline = 0u64;

    fn init(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
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

        Self { pipeline, vertices }
    }

    fn prepare(
        &mut self,
        pipeline: &mut Self::Pipeline,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
        _bounds: Rectangle,
    ) {
        // for the sake of this simple example, we are just rewriting to all buffers every frame
        queue.write_buffer(
            &pipeline.vertices,
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
        pipeline: &mut Self::Pipeline,
        render_pass: &mut wgpu::RenderPass<'b>,
        _device: &wgpu::Device,
        _target: &wgpu::TextureView,
        _clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
        _bounds: Rectangle,
    ) where
        'a: 'b,
    {
        println!("Rendering triangle!");
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_vertex_buffer(0, pipeline.vertices.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}
