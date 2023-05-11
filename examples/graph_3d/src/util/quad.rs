use glam::{Vec2, vec2, Vec3};

pub const VERTICES: [Vec2; 4] = [
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5),
];

pub const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];