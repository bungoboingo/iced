mod buffer;
mod light;
mod raw;
mod uniforms;
pub mod vertex;

use crate::camera::Camera;
use crate::pipeline::Pipeline;
use glam::Vec3;
use iced::{Point, Rectangle, Size};
use iced_graphics::custom::{self, Storage};
use iced_graphics::Transformation;
use rand::{thread_rng, Rng};

pub use crate::primitive::vertex::Vertex;
pub use buffer::Buffer;
pub use light::Light;
pub use raw::RawCube;
pub use uniforms::Uniforms;

/// A single instance of a cube.
#[derive(Debug, Clone)]
pub struct Cube {
    pub rotation: glam::Quat,
    pub position: Vec3,
    pub size: f32,
    rotation_dir: f32,
    rotation_axis: glam::Vec3,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            rotation: glam::Quat::IDENTITY,
            position: glam::Vec3::ZERO,
            size: 0.1,
            rotation_dir: 1.0,
            rotation_axis: glam::Vec3::Y,
        }
    }
}

impl Cube {
    pub fn new(size: f32, origin: Vec3) -> Self {
        let rnd = thread_rng().gen_range(0.0..=1.0f32);

        Self {
            rotation: glam::Quat::IDENTITY,
            position: origin + Vec3::new(0.1, 0.1, 0.1),
            size,
            rotation_dir: if rnd <= 0.5 { -1.0 } else { 1.0 },
            rotation_axis: if rnd <= 0.33 {
                glam::Vec3::Y
            } else if rnd <= 0.66 {
                glam::Vec3::X
            } else {
                glam::Vec3::Z
            },
        }
    }

    pub fn update(&mut self, size: f32, time: f32) {
        self.rotation = glam::Quat::from_axis_angle(
            self.rotation_axis,
            time / 2.0 * self.rotation_dir,
        );
        self.size = size;
    }
}

/// A collection of `Cube`s that can be rendered.
#[derive(Debug)]
pub struct Primitive {
    cubes: Vec<RawCube>,
    uniforms: Uniforms,
    show_depth_buffer: bool,
}

impl Primitive {
    pub fn new(
        cubes: &[Cube],
        camera: &Camera,
        show_depth_buffer: bool,
        mouse_pos: Point,
        light: Light,
    ) -> Self {
        let uniforms = Uniforms::new(camera, mouse_pos, light);

        Self {
            cubes: cubes
                .iter()
                .map(RawCube::from_cube)
                .collect::<Vec<RawCube>>(),
            uniforms,
            show_depth_buffer,
        }
    }
}

impl custom::Primitive for Primitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        _scale_factor: f32,
        _transform: Transformation,
        storage: &mut custom::Storage,
    ) {
        //check if we have already initialized our pipeline in storage
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, queue, format, target_size))
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        //upload data to GPU
        pipeline.update(
            device,
            queue,
            target_size,
            &self.uniforms,
            self.cubes.len(),
            &self.cubes,
        );
    }

    fn render(
        &self,
        storage: &Storage,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        _target_size: Size<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        //at this point our pipeline should always be initialized
        let pipeline = storage.get::<Pipeline>().unwrap();

        //render primitive
        pipeline.render(
            target,
            encoder,
            bounds,
            self.cubes.len() as u32,
            self.show_depth_buffer,
        )
    }
}
