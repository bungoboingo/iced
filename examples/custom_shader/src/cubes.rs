use crate::camera::Camera;
use crate::cube::{self, Cube, CubeRaw, CubesPrim, ManyCubes, Vertex3D};
use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};
use iced::advanced::{mouse, Shell};
use iced::event::Status;
use iced::{event, Color, Event, Rectangle, Size};
use iced_graphics::custom::{Program, RenderStatus};
use iced_graphics::Transformation;
use std::time::{Duration, Instant};
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, CommandEncoder, Device, IndexFormat,
    TextureView,
};

#[derive(Default)]
pub struct Cubes {
    // how big cubes are
    pub size: usize,
    // how many cubes to render
    pub amount: usize,
    // duration from app start
    pub time: Duration,
    // current camera transform
    pub camera: Camera,
}

impl<Message> Program<Message> for Cubes {
    type State = ();
    type Primitive = r#mod::Primitive;

    fn update(
        &mut self,
        _state: &mut Self::State,
        _event: Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> Status {
        event::Status::Ignored
    }

    fn draw(&self, _state: &Self::State) -> Self::Primitive {
        cube::Primitive::new(self.size, self.amount, &self.camera, self.time)
    }

    //TODO this is awk
    fn state(&self) -> &Self::State {
        &()
    }
}
