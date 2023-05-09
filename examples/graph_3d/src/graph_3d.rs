use crate::{camera, util};
use crate::camera::Camera;
use crate::pipeline::mesh_3d::Mesh3D;
use iced::advanced::graphics::Backend;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, Layout, Widget};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced_graphics::primitive::{CustomPipeline, Renderable};
use iced_graphics::{Primitive, Transformation};
use std::ops::{Range, RangeInclusive};
use glam::vec2;
use rand::{Rng, thread_rng};
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
            range: 0.0..=10.0,
            step: 1.0,
        }
    }
}

impl Axis {
    pub fn rnd_step(&self) -> f32 {
        let num_steps = (self.range.end() / self.step) as usize;
        let rnd = thread_rng().gen_range(0..=num_steps);
        (rnd as f32) * self.step
    }
}

struct State {
    mesh_3d: Mesh3D,
    camera: Camera,
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
        let camera = Camera::default();
        let mut camera_uniforms = camera::Uniforms::new();
        camera_uniforms.update(&camera);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("graph_3d.pipeline_layout_descriptor"),
            bind_group_layouts: &[&camera_uniforms.],
            push_constant_ranges: &[],
        });

        let x_axis = Axis {
            range: 0.0..=5.0,
            step: 0.5,
        };

        let y_axis = Axis {
            range: 0.0..=5.0,
            step: 0.5,
        };

        let z_axis = Axis {
            range: 0.0..=1.0,
            step: 0.1,
        };

        Box::new(Self {
            mesh_3d: Mesh3D::gen_rnd(&x_axis, &y_axis, &z_axis, device, format, &layout),
            camera: Default::default(),
            uniforms,
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
        self.mesh_3d.prepare(queue);
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
            color_attachments: &[Some(
                wgpu::RenderPassColorAttachment {
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
                }
            )],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(&self.mesh_3d.pipeline);
        pass.set_bind_group(0, &self.camera_uniforms_raw.bind_group, &[]);
        pass.set_vertex_buffer(0, self.mesh_3d.vertex_buffer.slice(..));
        pass.draw(0..100, 0..1);
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
