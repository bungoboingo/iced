use crate::program;
use glow::HasContext;
use iced_graphics::{layer, Rectangle, Transformation};

// Only change `MAX_QUADS`, otherwise you could cause problems
// by splitting a triangle into different render passes.
const MAX_QUADS: usize = 100_000;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;

#[derive(Debug)]
pub struct Pipeline {
    solid: solid::Program,
    gradient: gradient::Program,
}

impl Pipeline {
    pub fn new(
        gl: &glow::Context,
        shader_version: &program::Version,
    ) -> Pipeline {
        Pipeline {
            solid: solid::Program::new(gl, shader_version),
            gradient: gradient::Program::new(gl, shader_version),
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        target_height: u32,
        instances: &layer::Quads,
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
    ) {
        unsafe {
            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(
                bounds.x as i32,
                (target_height - (bounds.y + bounds.height)) as i32,
                bounds.width as i32,
                bounds.height as i32,
            );
        }

        self.solid.bind(gl);
        self.solid.uniforms.update(
            gl,
            transformation,
            scale,
            target_height as f32,
        );
        draw(gl, instances.solids.as_slice());

        self.gradient.bind(gl);
        self.gradient.uniforms.update(
            gl,
            transformation,
            scale,
            target_height as f32,
        );
        draw(gl, instances.gradients.as_slice());

        unsafe {
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::SCISSOR_TEST);
        }
    }
}

mod solid {
    use crate::program::{self, Shader};
    use crate::quad;
    use crate::quad::compatibility::{CompatVertex, MAX_VERTICES};
    use glow::{Context, HasContext};
    use iced_graphics::layer;
    use layer::quad::Solid;

    #[derive(Debug)]
    pub struct Program {
        program: glow::Program,
        pub uniforms: quad::core::Uniforms,
        pub vertex_array: <Context as HasContext>::VertexArray,
        pub vertex_buffer: <Context as HasContext>::Buffer,
        pub index_buffer: <Context as HasContext>::Buffer,
    }

    impl Program {
        pub fn new(gl: &Context, shader_version: &program::Version) -> Self {
            log::info!("GLOW: compiling quad (COMPATIBILITY) solid shaders.");

            let program = unsafe {
                let vertex_shader = Shader::vertex(
                    gl,
                    shader_version,
                    include_str!("../shader/quad/compatibility/solid.vert"),
                );
                let fragment_shader = Shader::fragment(
                    gl,
                    shader_version,
                    include_str!("../shader/quad/compatibility/solid.frag"),
                );

                program::create(
                    gl,
                    &[vertex_shader, fragment_shader],
                    &[
                        (0, "i_color"),
                        (1, "i_position"),
                        (2, "i_size"),
                        (3, "i_border_color"),
                        (4, "i_border_radius"),
                        (5, "i_border_width"),
                        (6, "i_quad_position"),
                    ],
                )
            };

            let (vertex_array, vertex_buffer, index_buffer) =
                unsafe { Self::create_buffers(gl, MAX_VERTICES) };

            Self {
                program,
                uniforms: quad::core::Uniforms::new(gl, program),
                vertex_array,
                vertex_buffer,
                index_buffer,
            }
        }

        pub fn bind(&self, gl: &glow::Context) {
            unsafe {
                gl.use_program(Some(self.program));
                gl.bind_vertex_array(Some(self.vertex_array));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
                gl.bind_buffer(
                    glow::ELEMENT_ARRAY_BUFFER,
                    Some(self.index_buffer),
                );
            }
        }

        unsafe fn create_buffers(
            gl: &Context,
            size: usize,
        ) -> (
            <Context as HasContext>::VertexArray,
            <Context as HasContext>::Buffer,
            <Context as HasContext>::Buffer,
        ) {
            let vertex_array =
                gl.create_vertex_array().expect("Create solid vertex array");
            let vertex_buffer =
                gl.create_buffer().expect("Create solid vertex buffer");
            let index_buffer =
                gl.create_buffer().expect("Create solid index buffer");

            gl.bind_vertex_array(Some(vertex_array));

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                12 * size as i32,
                glow::DYNAMIC_DRAW,
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size * CompatVertex::<Solid>::SIZE) as i32,
                glow::DYNAMIC_DRAW,
            );

            let stride = CompatVertex::<Solid>::SIZE as i32;

            // Color
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, stride, 0);

            // Position
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                4 * 4,
            );

            // Size
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(
                2,
                2,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2),
            );

            // Border Color
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(
                3,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2),
            );

            // Border Radii
            gl.enable_vertex_attrib_array(4);
            gl.vertex_attrib_pointer_f32(
                4,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2 + 4),
            );

            // Border Width
            gl.enable_vertex_attrib_array(5);
            gl.vertex_attrib_pointer_f32(
                5,
                1,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2 + 4 + 4),
            );

            // Quad Position
            gl.enable_vertex_attrib_array(6);
            gl.vertex_attrib_pointer_f32(
                6,
                2,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2 + 4 + 4 + 1),
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            (vertex_array, vertex_buffer, index_buffer)
        }
    }
}

mod gradient {
    use crate::program::{self, Shader};
    use crate::quad;
    use crate::quad::compatibility::{CompatVertex, MAX_VERTICES};
    use glow::{Context, HasContext};
    use iced_graphics::layer;
    use layer::quad::Gradient;

    #[derive(Debug)]
    pub struct Program {
        program: glow::Program,
        pub uniforms: quad::core::Uniforms,
        pub vertex_array: <Context as HasContext>::VertexArray,
        pub vertex_buffer: <Context as HasContext>::Buffer,
        pub index_buffer: <Context as HasContext>::Buffer,
    }

    impl Program {
        pub fn new(gl: &Context, shader_version: &program::Version) -> Self {
            log::info!(
                "GLOW: compiling quad (COMPATIBILITY) gradient shaders."
            );

            let program = unsafe {
                let vertex_shader = Shader::vertex(
                    gl,
                    shader_version,
                    include_str!("../shader/quad/compatibility/gradient.vert"),
                );
                let fragment_shader = Shader::fragment(
                    gl,
                    shader_version,
                    include_str!("../shader/quad/compatibility/gradient.frag"),
                );

                program::create(
                    gl,
                    &[vertex_shader, fragment_shader],
                    &[
                        (0, "i_colors_1"),
                        (1, "i_colors_2"),
                        (2, "i_colors_3"),
                        (3, "i_colors_4"),
                        (4, "i_colors_5"),
                        (5, "i_colors_6"),
                        (6, "i_colors_7"),
                        (7, "i_colors_8"),
                        (8, "i_offsets_1"),
                        (9, "i_offsets_2"),
                        (10, "i_direction"),
                        (11, "i_position_and_size"),
                        (12, "i_border_color"),
                        (13, "i_border_radius"),
                        (14, "i_border_width"),
                        (15, "i_quad_position"),
                    ],
                )
            };

            let (vertex_array, vertex_buffer, index_buffer) =
                unsafe { Self::create_buffers(gl, MAX_VERTICES) };

            Self {
                program,
                uniforms: quad::core::Uniforms::new(gl, program),
                vertex_array,
                vertex_buffer,
                index_buffer,
            }
        }

        pub fn bind(&self, gl: &glow::Context) {
            unsafe {
                gl.use_program(Some(self.program));
                gl.bind_vertex_array(Some(self.vertex_array));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
                gl.bind_buffer(
                    glow::ELEMENT_ARRAY_BUFFER,
                    Some(self.index_buffer),
                );
            }
        }

        unsafe fn create_buffers(
            gl: &Context,
            size: usize,
        ) -> (
            <Context as HasContext>::VertexArray,
            <Context as HasContext>::Buffer,
            <Context as HasContext>::Buffer,
        ) {
            let vertex_array = gl
                .create_vertex_array()
                .expect("Create gradient vertex array");
            let vertex_buffer =
                gl.create_buffer().expect("Create gradient vertex buffer");
            let index_buffer =
                gl.create_buffer().expect("Create gradient index buffer");

            gl.bind_vertex_array(Some(vertex_array));

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                12 * size as i32,
                glow::DYNAMIC_DRAW,
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size * CompatVertex::<Gradient>::SIZE) as i32,
                glow::DYNAMIC_DRAW,
            );

            let stride = CompatVertex::<Gradient>::SIZE as i32;

            // Colors vec4 array[8] (indices 1-8)
            for i in 0..=7 {
                gl.enable_vertex_attrib_array(i);
                gl.vertex_attrib_pointer_f32(
                    i,
                    4,
                    glow::FLOAT,
                    false,
                    stride,
                    (4 * i * 4) as i32,
                );
            }

            // Offsets 1-4
            gl.enable_vertex_attrib_array(8);
            gl.vertex_attrib_pointer_f32(
                8,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * 32,
            );

            // Offsets 5-8
            gl.enable_vertex_attrib_array(9);
            gl.vertex_attrib_pointer_f32(
                9,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4),
            );

            // Direction
            gl.enable_vertex_attrib_array(10);
            gl.vertex_attrib_pointer_f32(
                10,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4 + 4),
            );

            // Position & Scale
            gl.enable_vertex_attrib_array(11);
            gl.vertex_attrib_pointer_f32(
                11,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4 + 4 + 4),
            );

            // Border Color
            gl.enable_vertex_attrib_array(12);
            gl.vertex_attrib_pointer_f32(
                12,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4 + 4 + 4 + 4),
            );

            // Border Radii
            gl.enable_vertex_attrib_array(13);
            gl.vertex_attrib_pointer_f32(
                13,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4 + 4 + 4 + 4 + 4),
            );

            // Border Width
            gl.enable_vertex_attrib_array(14);
            gl.vertex_attrib_pointer_f32(
                14,
                1,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4 + 4 + 4 + 4 + 4 + 4),
            );

            // Quad Position
            gl.enable_vertex_attrib_array(15);
            gl.vertex_attrib_pointer_f32(
                15,
                2,
                glow::FLOAT,
                false,
                stride,
                4 * (32 + 4 + 4 + 4 + 4 + 4 + 4 + 1),
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            (vertex_array, vertex_buffer, index_buffer)
        }
    }
}

/// The vertex of a colored rectangle with a border.
///
/// This type can be directly uploaded to GPU memory.
//TODO how can we remove this extra wrapper to avoid an additional allocation?
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CompatVertex<T: Copy + bytemuck::Zeroable + bytemuck::Pod> {
    /// THe __quad__ of the [`Vertex`].
    quad: T,

    /// The __quad__ position of the [`Vertex`].
    quad_position: [f32; 2],
}

unsafe impl<T: Copy + bytemuck::Pod + bytemuck::Zeroable> bytemuck::Pod
    for CompatVertex<T>
{
}
unsafe impl<T: Copy + bytemuck::Zeroable + bytemuck::Pod> bytemuck::Zeroable
    for CompatVertex<T>
{
}

impl<T: bytemuck::Zeroable + bytemuck::Pod> CompatVertex<T> {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn from_quad(quad: &T) -> [Self; 4] {
        let base = Self {
            quad: *quad,
            quad_position: [0.0, 0.0],
        };

        [
            base,
            Self {
                quad_position: [0.0, 1.0],
                ..base
            },
            Self {
                quad_position: [1.0, 0.0],
                ..base
            },
            Self {
                quad_position: [1.0, 1.0],
                ..base
            },
        ]
    }
}

fn draw<T: bytemuck::Zeroable + bytemuck::Pod>(
    gl: &glow::Context,
    instances: &[T],
) {
    // TODO: Remove this allocation (probably by changing the shader and removing the need of two `position`)
    let vertices: Vec<CompatVertex<T>> =
        instances.iter().flat_map(CompatVertex::from_quad).collect();

    // TODO: Remove this allocation (or allocate only when needed)
    let indices: Vec<i32> = (0..instances.len().min(MAX_QUADS) as i32)
        .flat_map(|i| {
            [i * 4, 1 + i * 4, 2 + i * 4, 2 + i * 4, 1 + i * 4, 3 + i * 4]
        })
        .cycle()
        .take(instances.len() * 6)
        .collect();

    let passes = vertices
        .chunks(MAX_VERTICES)
        .zip(indices.chunks(MAX_INDICES));

    for (vertices, indices) in passes {
        unsafe {
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(vertices),
            );

            gl.buffer_sub_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(indices),
            );

            gl.draw_elements(
                glow::TRIANGLES,
                indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }
}
