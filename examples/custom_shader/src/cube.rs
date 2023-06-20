mod raw;
mod uniforms;
pub mod vertex_3d;

use crate::camera::Camera;
use crate::cube::uniforms::Uniforms;
use crate::cube::vertex_3d::{v3d, Vertex3D};
use crate::cubes::Cubes;
use crate::pipeline::{Pipeline, Uniforms};
use glam::{vec3, Vec2, Vec3};
use iced::{Color, Rectangle, Size};
use iced_graphics::custom::{self, Storage};
pub use raw::Raw;
use std::iter;
use std::time::Duration;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

/// A single instance of a cube, defined by its size and rotation.
pub struct Cube {
    pub rotation: glam::Quat,
    pub position: Vec3,
    pub size: f32,
}

impl Cube {
    /// Creates a new [`Cube`] with the specified `size`.
    pub fn new(size: usize, origin: Vec3) -> Self {
        Self {
            rotation: glam::Quat::default(),
            position: origin + Vec3::new(0.1, 0.1, 0.1), //todo IDK
            size: size as f32,
        }
    }
}

/// A collection of `Cube`s that can be rendered.
pub struct Primitive {
    cubes: Vec<Cube>,
    uniforms: Uniforms,
}

impl Primitive {
    pub fn new(
        size: usize,
        amount: usize,
        camera: &Camera,
        time: Duration,
    ) -> Self {
        let mut cubes = 0;
        let mut uniforms = Uniforms::new();
        uniforms.update(camera, time);
        let mut origin = Vec3::ZERO;

        Self {
            cubes: Vec::from_iter(iter::from_fn(move || {
                if cubes < amount {
                    cubes += 1;
                    let cube = Some(Cube::new(size, origin));
                    origin += Vec3::new(0.1, 0.1, 0.1); //TODO idk
                    cube
                } else {
                    None
                }
            })),
            uniforms,
        }
    }

    pub fn raw_cubes(&self) -> &[CubeRaw] {
        self.cubes
            .iter()
            .map(Raw::from_cube)
            .collect::<Vec<CubeRaw>>()
            .as_slice()
    }
}

impl custom::Primitive for Primitive {
    fn prepare(
        &self,
        format: TextureFormat,
        device: &Device,
        queue: &Queue,
        target_size: Size<u32>,
        storage: &mut Storage,
    ) {
        //TODO cleanup how this is accessed; always get back a mut pipeline ref here..?
        //first find pipeline
        let pipeline = if let Some(pipeline) = storage.get_mut::<Pipeline>() {
            pipeline
        } else {
            storage.store(Pipeline::new(device, format, target_size));
        };

        //recreate depth texture if size has changed
        pipeline.update_depth_texture(device, target_size);

        // update uniforms
        queue.write_buffer(
            &pipeline.uniforms,
            0,
            bytemuck::bytes_of(&self.uniforms),
        );

        //resize cubes vertex buffer if cubes amt changed
        let new_size = self.cubes.len() * std::mem::size_of::<CubeRaw>();

        //only grow
        if new_size > pipeline.cubes_buffer_size {
            pipeline.vertices = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cubes.vertex_buffer"),
                size: new_size as u64,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        };

        queue.write_buffer(
            &pipeline.cubes_buffer,
            0,
            bytemuck::bytes_of(self.raw_cubes()),
        );
    }

    fn render(
        &self,
        storage: &Storage,
        bounds: Rectangle<u32>,
        target: &TextureView,
        _target_size: Size<u32>,
        encoder: &mut CommandEncoder,
    ) {
        //TODO cleanup
        if let Some(pipeline) = storage.get::<Pipeline>() {
            pipeline.render(target, encoder, bounds, self.cubes.len() as u32)
        }
    }
}
