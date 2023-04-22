use iced::event::Status;
use iced::mouse::Interaction;
use iced::{Color, Rectangle, Renderer};
use iced_graphics::custom::{Cursor, Event};
use iced_graphics::primitive;

pub struct Program;

//TODO this has to be Default somehow
struct State {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    uniform_layout: wgpu::BindGroupLayout,
}

impl<Message, Theme> iced_graphics::custom::Program<Message, Renderer<Theme>>
    for Program
{
    type State = Option<State>;

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
        renderer.draw_primitive(primitive::custom(|render_pass, state| {
            if let Some(state) = state.downcast_ref::<Self::State>() {
                render_pass.set_pipeline(state.pipeline);
            }
        }))
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced_graphics::custom::Cursor,
    ) -> Interaction {
        todo!()
    }
}

struct Cube {
    scale: f32,
    color: Color,
}

struct Vertex3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Vertex3D {
    /// Scalar multiplication of a [`Vertex3D]`.
    fn scale(mut self, scale: f32) -> Self {
        self.x *= scale;
        self.y *= scale;
        self.x *= scale;
        self
    }
}

fn vertex_3d(x: f32, y: f32, z: f32) -> Vertex3D {
    Vertex3D { x, y, z }
}

impl Cube {
    /// Returns the vertices and indices of the cube.
    fn attributes(&self) -> ([Vertex3D; 8], Vec<u32>) {
        (
            [
                vertex_3d(0.0, 0.0, 1.0).scale(self.scale), //front top left
                vertex_3d(1.0, 0.0, 1.0).scale(self.scale), //front top right
                vertex_3d(0.0, 0.0, 0.0).scale(self.scale), //front bottom left
                vertex_3d(1.0, 0.0, 0.0).scale(self.scale), //front bottom right
                vertex_3d(1.0, 1.0, 0.0).scale(self.scale), //back bottom right
                vertex_3d(0.0, 1.0, 0.0).scale(self.scale), //back bottom left
                vertex_3d(0.0, 1.0, 1.0).scale(self.scale), //back top left
                vertex_3d(1.0, 1.0, 1.0).scale(self.scale), //back top right
            ],
            vec![
                0, 1, 2, 1, 2, 3, //front face
                0, 2, 5, 0, 5, 6, //left face
                4, 5, 6, 4, 6, 7, //back face
                1, 4, 7, 1, 4, 3, //right face
                2, 3, 4, 2, 4, 5, //bottom face
                0, 1, 7, 0, 7, 6,
            ],
        )
    }
}
