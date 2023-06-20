use glam::vec4;
use iced::{Color, mouse, Point};

#[derive(Copy, Clone, Debug)]
pub struct Light {
    _position: Point,
    color: Color,
}

impl Light {
    pub fn new(mouse: mouse::Cursor, color: Color) -> Self {
        Self {
            _position: mouse.position().unwrap_or(Point::new(-1.0, -1.0)),
            color,
        }
    }

    pub fn to_raw(self) -> Raw {
        Raw {
            position: vec4(0.0, 3.0, 3.0, 0.0),
            color: glam::Vec4::from(self.color.into_linear()),
        }
    }
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Raw {
    pub position: glam::Vec4,
    pub color: glam::Vec4,
}