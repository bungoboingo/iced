use crate::camera::Camera;
use std::time::Duration;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Uniforms {
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

    pub fn update(&mut self, camera: &Camera, time: Duration) {
        self.camera_projection = camera.build_view_proj_matrix();
        self.time = time.as_secs_f32();
    }
}
