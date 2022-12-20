//! Draw geometry using meshes of triangles.
use bytemuck::{Pod, Zeroable};

/// A set of [`Vertex2D`] and indices representing a list of triangles.
#[derive(Clone, Debug)]
pub struct Mesh2D<T> {
    /// The vertices of the mesh
    pub vertices: Vec<T>,

    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u32>,
}

/// A vertex which contains 2D position & flattened gradient data.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct GradientVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The flattened vertex data of the gradient.
    pub gradient: [f32; 44],
}

#[allow(unsafe_code)]
unsafe impl Zeroable for GradientVertex2D {}

#[allow(unsafe_code)]
unsafe impl Pod for GradientVertex2D {}

/// A two-dimensional vertex with a color.
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The color of the vertex in __linear__ RGBA.
    pub color: [f32; 4],
}
