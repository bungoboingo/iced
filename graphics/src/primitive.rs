use iced_core::alignment;
use iced_core::image;
use iced_core::svg;
use iced_core::{Background, Color, Font, Gradient, Rectangle, Size, Vector};
use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use crate::custom::Program;
use crate::Transformation;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

#[cfg(feature = "wgpu")]
pub trait Renderable {
    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
    );

    fn render<'a, 'b>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        _device: &wgpu::Device,
        _target: &wgpu::TextureView,
        _clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
    ) where
        'a: 'b;
}

/// A rendering primitive.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Primitive {
    /// A text primitive
    Text {
        /// The contents of the text
        content: String,
        /// The bounds of the text
        bounds: Rectangle,
        /// The color of the text
        color: Color,
        /// The size of the text
        size: f32,
        /// The font of the text
        font: Font,
        /// The horizontal alignment of the text
        horizontal_alignment: alignment::Horizontal,
        /// The vertical alignment of the text
        vertical_alignment: alignment::Vertical,
    },
    /// A quad primitive
    Quad {
        /// The bounds of the quad
        bounds: Rectangle,
        /// The background of the quad
        background: Background,
        /// The border radius of the quad
        border_radius: [f32; 4],
        /// The border width of the quad
        border_width: f32,
        /// The border color of the quad
        border_color: Color,
    },
    /// An image primitive
    Image {
        /// The handle of the image
        handle: image::Handle,
        /// The bounds of the image
        bounds: Rectangle,
    },
    /// An SVG primitive
    Svg {
        /// The path of the SVG file
        handle: svg::Handle,

        /// The [`Color`] filter
        color: Option<Color>,

        /// The bounds of the viewport
        bounds: Rectangle,
    },
    /// A low-level primitive to render a mesh of triangles with a solid color.
    ///
    /// It can be used to render many kinds of geometry freely.
    SolidMesh {
        /// The vertices and indices of the mesh.
        buffers: Mesh2D<ColoredVertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,
    },
    /// A low-level primitive to render a mesh of triangles with a gradient.
    ///
    /// It can be used to render many kinds of geometry freely.
    GradientMesh {
        /// The vertices and indices of the mesh.
        buffers: Mesh2D<Vertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,

        /// The [`Gradient`] to apply to the mesh.
        gradient: Gradient,
    },
    #[cfg(feature = "tiny-skia")]
    Fill {
        path: tiny_skia::Path,
        paint: tiny_skia::Paint<'static>,
        rule: tiny_skia::FillRule,
        transform: tiny_skia::Transform,
    },
    #[cfg(feature = "tiny-skia")]
    Stroke {
        path: tiny_skia::Path,
        paint: tiny_skia::Paint<'static>,
        stroke: tiny_skia::Stroke,
        transform: tiny_skia::Transform,
    },
    /// A group of primitives
    Group {
        /// The primitives of the group
        primitives: Vec<Primitive>,
    },
    /// A clip primitive
    Clip {
        /// The bounds of the clip
        bounds: Rectangle,
        /// The content of the clip
        content: Box<Primitive>,
    },
    /// A primitive that applies a translation
    Translate {
        /// The translation vector
        translation: Vector,

        /// The primitive to translate
        content: Box<Primitive>,
    },
    /// A cached primitive.
    ///
    /// This can be useful if you are implementing a widget where primitive
    /// generation is expensive.
    Cache {
        /// The cached primitive
        content: Arc<Primitive>,
    },
    #[cfg(feature = "wgpu")]
    Custom {
        bounds: Rectangle,
        pipeline: CustomPipeline,
    },
}

impl Debug for CustomPipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomPipeline id: {:?}", self.id)
    }
}

impl Primitive {
    pub fn group(primitives: Vec<Self>) -> Self {
        Self::Group { primitives }
    }

    pub fn clip(self, bounds: Rectangle) -> Self {
        Self::Clip {
            bounds,
            content: Box::new(self),
        }
    }

    pub fn translate(self, translation: Vector) -> Self {
        Self::Translate {
            translation,
            content: Box::new(self),
        }
    }

    pub fn bounds(&self) -> Rectangle {
        match self {
            Self::Text {
                bounds,
                horizontal_alignment,
                vertical_alignment,
                ..
            } => {
                let mut bounds = *bounds;

                bounds.x = match horizontal_alignment {
                    alignment::Horizontal::Left => bounds.x,
                    alignment::Horizontal::Center => {
                        bounds.x - bounds.width / 2.0
                    }
                    alignment::Horizontal::Right => bounds.x - bounds.width,
                };

                bounds.y = match vertical_alignment {
                    alignment::Vertical::Top => bounds.y,
                    alignment::Vertical::Center => {
                        bounds.y - bounds.height / 2.0
                    }
                    alignment::Vertical::Bottom => bounds.y - bounds.height,
                };

                bounds.expand(1.5)
            }
            Self::Quad { bounds, .. }
            | Self::Image { bounds, .. }
            | Self::Svg { bounds, .. }
            | Self::Custom { bounds, .. } => bounds.expand(1.0),
            Self::Clip { bounds, .. } => bounds.expand(1.0),
            Self::SolidMesh { size, .. } | Self::GradientMesh { size, .. } => {
                Rectangle::with_size(*size)
            }
            #[cfg(feature = "tiny-skia")]
            Self::Fill { path, .. } | Self::Stroke { path, .. } => {
                let bounds = path.bounds();

                Rectangle {
                    x: bounds.x(),
                    y: bounds.y(),
                    width: bounds.width(),
                    height: bounds.height(),
                }
                .expand(1.0)
            }
            Self::Group { primitives } => primitives
                .iter()
                .map(Self::bounds)
                .fold(Rectangle::with_size(Size::ZERO), |a, b| {
                    Rectangle::union(&a, &b)
                }),
            Self::Translate {
                translation,
                content,
            } => content.bounds() + *translation,
            Self::Cache { content } => content.bounds(),
        }
    }

    #[cfg(feature = "wgpu")]
    pub fn custom(bounds: Rectangle, pipeline: CustomPipeline) -> Self {
        Self::Custom { bounds, pipeline }
    }
}

/// A set of [`Vertex2D`] and indices representing a list of triangles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mesh2D<T> {
    /// The vertices of the mesh
    pub vertices: Vec<T>,

    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u32>,
}

/// A two-dimensional vertex.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],
}

/// A two-dimensional vertex with a color.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The color of the vertex in __linear__ RGBA.
    pub color: [f32; 4],
}

impl From<()> for Primitive {
    fn from(_: ()) -> Self {
        Self::Group { primitives: vec![] }
    }
}

/// An identifier for a custom pipeline.
#[derive(Clone)]
pub struct CustomPipeline {
    pub id: u64,
    pub pipeline:
        fn(device: &wgpu::Device, format: wgpu::TextureFormat) -> Box<dyn Any>, //returns the "State" of the pipeline
    pub prepare: fn(
        state: &mut Box<dyn Any + 'static>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scale_factor: f32,
        transformation: Transformation,
    ),
    pub render: fn(
        state: Box<dyn Any + 'static>,
        render_pass: &mut wgpu::RenderPass<'_>,
        device: &wgpu::Device,
        target: &wgpu::TextureView,
        clear_color: Option<Color>,
        scale_factor: f32,
        target_size: Size<u32>,
    ),
}

impl PartialEq for CustomPipeline {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}

