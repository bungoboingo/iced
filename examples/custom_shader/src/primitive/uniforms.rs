use crate::camera::Camera;
use iced::Point;
use crate::primitive::light::Light;

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    camera_proj: glam::Mat4,
    camera_pos: glam::Vec4,
    light_pos: glam::Vec4,
    light_color: glam::Vec4,
    mouse_pos: [f32; 2],
    _padding: [f32; 2],
}

impl Uniforms {
    pub fn new(camera: &Camera, mouse_position: Point, light: Light) -> Self {
        let light = light.to_raw();
        let camera_proj = camera.build_view_proj_matrix();

        Self {
            camera_proj,
            camera_pos: glam::Vec4::from((camera.eye, 1.0)),
            light_pos: light.position,
            light_color: light.color,
            mouse_pos: [mouse_position.x, mouse_position.y],
            _padding: [0.0; 2],
        }
    }
}
