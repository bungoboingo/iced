use crate::program;
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
use iced_native::Rectangle;

const INITIAL_INSTANCES: usize = 10_000;

#[derive(Debug)]
pub struct Pipeline {
    solid: solid::Program,
    gradient: gradient::Program,
}

#[derive(Debug)]
pub struct Uniforms {
    transform: Transformation,
    transform_location: <glow::Context as HasContext>::UniformLocation,
    scale: f32,
    scale_location: <glow::Context as HasContext>::UniformLocation,
    screen_height: f32,
    screen_height_location: <glow::Context as HasContext>::UniformLocation,
}

impl Uniforms {
    pub fn new(gl: &glow::Context, program: glow::NativeProgram) -> Self {
        let transform = Transformation::identity();
        let scale = 1.0;
        let screen_height = 0.0;

        let transform_location =
            unsafe { gl.get_uniform_location(program, "u_transform") }
                .expect("Get transform location");

        let scale_location =
            unsafe { gl.get_uniform_location(program, "u_scale") }
                .expect("Get scale location");

        let screen_height_location =
            unsafe { gl.get_uniform_location(program, "u_screen_height") }
                .expect("Get target height location");

        unsafe {
            gl.use_program(Some(program));

            gl.uniform_matrix_4_f32_slice(
                Some(&transform_location),
                false,
                transform.as_ref(),
            );

            gl.uniform_1_f32(Some(&scale_location), scale);
            gl.uniform_1_f32(Some(&screen_height_location), screen_height);

            gl.use_program(None);
        }

        Self {
            transform,
            transform_location,
            scale,
            scale_location,
            screen_height,
            screen_height_location,
        }
    }

    pub fn update(
        &mut self,
        gl: &glow::Context,
        new_transform: Transformation,
        new_scale: f32,
        new_screen_height: f32,
    ) {
        if new_transform != self.transform {
            unsafe {
                gl.uniform_matrix_4_f32_slice(
                    Some(&self.transform_location),
                    false,
                    new_transform.as_ref(),
                );

                self.transform = new_transform;
            }
        }

        if new_scale != self.scale {
            unsafe {
                gl.uniform_1_f32(Some(&self.scale_location), new_scale);
            }

            self.scale = new_scale;
        }

        if new_screen_height != self.screen_height {
            unsafe {
                gl.uniform_1_f32(
                    Some(&self.screen_height_location),
                    new_screen_height,
                );
            }

            self.screen_height = new_screen_height;
        }
    }
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

        //draw solid instances
        self.solid.bind(gl);
        self.solid.uniforms.update(
            gl,
            transformation,
            scale,
            target_height as f32,
        );
        draw_instances(gl, instances.solids.as_slice());

        //draw gradient instances
        self.gradient.bind(gl);
        self.gradient.uniforms.update(
            gl,
            transformation,
            scale,
            target_height as f32,
        );
        draw_instances(gl, instances.gradients.as_slice());

        unsafe {
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::SCISSOR_TEST);
        }
    }
}

mod solid {
    use crate::program::{self, Shader};
    use crate::quad::core::{Uniforms, INITIAL_INSTANCES};
    use glow::HasContext;
    use iced_graphics::layer::quad;

    #[derive(Debug)]
    pub struct Program {
        program: glow::Program,
        pub uniforms: Uniforms,
        pub vertex_array: glow::NativeVertexArray,
        pub instances: glow::NativeBuffer,
    }

    impl Program {
        pub fn new(
            gl: &glow::Context,
            shader_version: &program::Version,
        ) -> Self {
            log::info!("GLOW: compiling quad (CORE) solid shaders.");

            let program = unsafe {
                let vertex_shader = Shader::vertex(
                    gl,
                    shader_version,
                    [],
                    include_str!("../shader/quad/core/solid.vert"),
                );
                let fragment_shader = Shader::fragment(
                    gl,
                    shader_version,
                    [
                        include_str!("../shader/includes/quad_distance.vert"),
                        include_str!("../shader/includes/border_radius.vert"),
                    ],
                    include_str!("../shader/quad/core/solid.frag"),
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
                    ],
                )
            };

            let (vertex_array, instances) =
                unsafe { Self::create_instance_buffer(gl, INITIAL_INSTANCES) };

            Self {
                program,
                uniforms: Uniforms::new(gl, program),
                vertex_array,
                instances,
            }
        }

        pub fn bind(&self, gl: &glow::Context) {
            unsafe {
                gl.use_program(Some(self.program));
                gl.bind_vertex_array(Some(self.vertex_array));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instances));
            }
        }

        unsafe fn create_instance_buffer(
            gl: &glow::Context,
            size: usize,
        ) -> (
            <glow::Context as HasContext>::VertexArray,
            <glow::Context as HasContext>::Buffer,
        ) {
            let vertex_array =
                gl.create_vertex_array().expect("Create vertex array");
            let buffer = gl.create_buffer().expect("Create instance buffer");

            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size * std::mem::size_of::<quad::Solid>()) as i32,
                glow::DYNAMIC_DRAW,
            );

            let stride = std::mem::size_of::<quad::Solid>() as i32;

            // Color
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 4, glow::FLOAT, false, stride, 0);
            gl.vertex_attrib_divisor(0, 1);

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
            gl.vertex_attrib_divisor(1, 1);

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
            gl.vertex_attrib_divisor(2, 1);

            // Border color
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(
                3,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2),
            );
            gl.vertex_attrib_divisor(3, 1);

            // Border radii
            gl.enable_vertex_attrib_array(4);
            gl.vertex_attrib_pointer_f32(
                4,
                4,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2 + 4),
            );
            gl.vertex_attrib_divisor(4, 1);

            // Border width
            gl.enable_vertex_attrib_array(5);
            gl.vertex_attrib_pointer_f32(
                5,
                1,
                glow::FLOAT,
                false,
                stride,
                4 * (4 + 2 + 2 + 4 + 4),
            );
            gl.vertex_attrib_divisor(5, 1);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            (vertex_array, buffer)
        }
    }
}

mod gradient {
    use crate::program::{self, Shader};
    use crate::quad::core::{Uniforms, INITIAL_INSTANCES};
    use glow::HasContext;
    use iced_graphics::layer::quad;

    #[derive(Debug)]
    pub struct Program {
        program: glow::Program,
        pub uniforms: Uniforms,
        pub vertex_array: glow::NativeVertexArray,
        pub instances: glow::NativeBuffer,
    }

    impl Program {
        pub fn new(
            gl: &glow::Context,
            shader_version: &program::Version,
        ) -> Self {
            log::info!("GLOW: compiling quad gradient shaders.");

            let program = unsafe {
                let vertex_shader = Shader::vertex(
                    gl,
                    shader_version,
                    [],
                    include_str!("../shader/quad/core/gradient.vert"),
                );
                let fragment_shader = Shader::fragment(
                    gl,
                    shader_version,
                    [
                        include_str!("../shader/includes/gradient.frag"),
                        include_str!("../shader/includes/quad_distance.vert"),
                        include_str!("../shader/includes/border_radius.vert"),
                    ],
                    include_str!("../shader/quad/core/gradient.frag"),
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
                    ],
                )
            };

            let (vertex_array, instances) =
                unsafe { Self::create_instance_buffer(gl, INITIAL_INSTANCES) };

            Self {
                program,
                uniforms: Uniforms::new(gl, program),
                vertex_array,
                instances,
            }
        }

        pub fn bind(&self, gl: &glow::Context) {
            unsafe {
                gl.use_program(Some(self.program));
                gl.bind_vertex_array(Some(self.vertex_array));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instances));
            }
        }

        unsafe fn create_instance_buffer(
            gl: &glow::Context,
            size: usize,
        ) -> (
            <glow::Context as HasContext>::VertexArray,
            <glow::Context as HasContext>::Buffer,
        ) {
            let vertex_array =
                gl.create_vertex_array().expect("Create vertex array");
            let buffer = gl.create_buffer().expect("Create instance buffer");

            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size * std::mem::size_of::<quad::Gradient>()) as i32,
                glow::DYNAMIC_DRAW,
            );

            let stride = std::mem::size_of::<quad::Gradient>() as i32;

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
                gl.vertex_attrib_divisor(i, 1);
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
            gl.vertex_attrib_divisor(8, 1);

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
            gl.vertex_attrib_divisor(9, 1);

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
            gl.vertex_attrib_divisor(10, 1);

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
            gl.vertex_attrib_divisor(11, 1);

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
            gl.vertex_attrib_divisor(12, 1);

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
            gl.vertex_attrib_divisor(13, 1);

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
            gl.vertex_attrib_divisor(14, 1);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            (vertex_array, buffer)
        }
    }
}

fn draw_instances<T>(gl: &glow::Context, instances: &[T])
where
    T: bytemuck::Zeroable + bytemuck::Pod,
{
    for instances_chunk in instances.chunks(INITIAL_INSTANCES) {
        unsafe {
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                bytemuck::cast_slice(instances_chunk),
            );

            gl.draw_arrays_instanced(
                glow::TRIANGLE_STRIP,
                0,
                4,
                instances_chunk.len() as i32,
            );
        }
    }
}
