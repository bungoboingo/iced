use crate::camera::{Camera, CameraBundle};
use crate::pipeline::mesh_3d::{Mesh3D, Mesh3DBundle};
use crate::util::cube;
use crate::{camera, util};
use glam::{vec2, Vec2, Vec3};
use iced::advanced::graphics::Backend;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, Layout, Widget};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced_graphics::primitive::{CustomPipeline, Renderable};
use iced_graphics::{Primitive, Transformation};
use rand::{thread_rng, Rng};
use std::ops::{Range, RangeInclusive};
use wgpu::{CommandEncoder, Device, Queue, TextureView};

pub struct Graph3D {
    id: u64,
    width: Length,
    height: Length,
}

impl Graph3D {
    pub fn new() -> Self {
        Self {
            id: 0,
            width: Length::Fill,
            height: Length::Fill,
        }
    }
}

pub struct Axis {
    pub range: RangeInclusive<f32>,
    pub step: f32,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            range: -10.0..=10.0,
            step: 0.5,
        }
    }
}

impl Axis {
    pub fn rnd_step(&self) -> f32 {
        let num_steps = ((self.range.end() - self.range.start()) / self.step) as usize;
        let rnd = thread_rng().gen_range(0..=num_steps);
        (rnd as f32) * self.step * if thread_rng().gen_bool(0.5) {
            1.0
        } else {
            -1.0
        }
    }
}

struct State {
    mesh_3d: Mesh3DBundle,
    camera: CameraBundle,
    pipeline: wgpu::RenderPipeline,
    depth_view: wgpu::TextureView,
    x_axis: Axis,
    y_axis: Axis,
    z_axis: Axis,
}

impl State {
    fn init(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Box<dyn Renderable + 'static> {
        let camera_bundle = CameraBundle::new(device);

        let x_axis = Axis::default();
        let y_axis = Axis::default();
        let z_axis = Axis::default();

        let mesh_3d =
            Mesh3DBundle::new(&x_axis, &y_axis, &z_axis, device, target_size);

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

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("graph_3d.pipeline_layout_descriptor"),
                bind_group_layouts: &[
                    &camera_bundle.uniform_layout,
                    &mesh_3d.uniform_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("graph_3d.mesh_3d.shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    include_str!("shaders/mesh_3d.wgsl"),
                )),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("graph3d.mesh_3d.pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vec3>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            //center position of cube
                            0 => Float32x3
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<cube::Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            //position
                            1 => Float32x3,
                            //color
                            2 => Float32x3,
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    .. Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: Default::default(),
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
            mesh_3d,
            camera: camera_bundle,
            pipeline,
            depth_view,
            x_axis,
            y_axis,
            z_axis,
        })
    }
}

impl Renderable for State {
    fn prepare(
        &mut self,
        _device: &Device,
        queue: &Queue,
        _encoder: &mut CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
    ) {
        queue.write_buffer(
            &self.mesh_3d.instances,
            0,
            bytemuck::cast_slice(&self.mesh_3d.mesh_3d.vertices),
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
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("mesh_3d.render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
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
                }
            ),
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera.bind_group, &[]);
        pass.set_bind_group(1, &self.mesh_3d.bind_group, &[]);
        pass.set_index_buffer(
            self.mesh_3d.indices.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        pass.set_vertex_buffer(0, self.mesh_3d.instances.slice(..));
        pass.set_vertex_buffer(1, self.mesh_3d.vertices.slice(..));
        pass.draw_indexed(
            0..cube::INDICES.len() as u32,
            0,
            0..self.mesh_3d.mesh_3d.vertices.len() as u32,
        );
    }
}

impl<B, T, Message> Widget<Message, iced_graphics::Renderer<B, T>> for Graph3D
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

impl<'a, B, Theme, Message> From<Graph3D>
    for Element<'a, Message, iced_graphics::Renderer<B, Theme>>
where
    Message: 'a,
    B: Backend,
{
    fn from(
        graph_3d: Graph3D,
    ) -> Element<'a, Message, iced_graphics::Renderer<B, Theme>> {
        Element::new(graph_3d)
    }
}
