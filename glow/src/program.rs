use glow::HasContext;

/// The [`Version`] of a `Program`.
pub struct Version {
    vertex: String,
    fragment: String,
}

impl Version {
    pub fn new(gl: &glow::Context) -> Version {
        let version = gl.version();

        let (vertex, fragment) = match (
            version.major,
            version.minor,
            version.is_embedded,
        ) {
            // OpenGL 3.0+
            (3, 0 | 1 | 2, false) => (
                format!("#version 1{}0", version.minor + 3),
                format!(
                    "#version 1{}0\n#define HIGHER_THAN_300 1",
                    version.minor + 3
                ),
            ),
            // OpenGL 3.3+
            (3 | 4, _, false) => (
                format!("#version {}{}0", version.major, version.minor),
                format!(
                    "#version {}{}0\n#define HIGHER_THAN_300 1",
                    version.major, version.minor
                ),
            ),
            // OpenGL ES 3.0+
            (3, _, true) => (
                format!("#version 3{}0 es", version.minor),
                format!(
                    "#version 3{}0 es\n#define HIGHER_THAN_300 1",
                    version.minor
                ),
            ),
            // OpenGL ES 2.0+
            (2, _, true) => (
                String::from(
                    "#version 100\n#define in attribute\n#define out varying",
                ),
                String::from("#version 100\n#define in varying"),
            ),
            // OpenGL 2.1
            (2, _, false) => (
                String::from(
                    "#version 120\n#define in attribute\n#define out varying",
                ),
                String::from("#version 120\n#define in varying"),
            ),
            // OpenGL 1.1+
            _ => panic!("Incompatible context version: {:?}", version),
        };

        log::info!("Shader directive: {}", vertex.lines().next().unwrap());

        Version { vertex, fragment }
    }
}

pub struct Shader(<glow::Context as HasContext>::Shader);

impl Shader {
    fn compile(gl: &glow::Context, stage: u32, content: &str) -> Shader {
        unsafe {
            let shader = gl.create_shader(stage).expect("Cannot create shader");

            gl.shader_source(shader, content);
            gl.compile_shader(shader);

            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }

            Shader(shader)
        }
    }

    /// Creates a vertex [`Shader`].
    pub fn vertex(
        gl: &glow::Context,
        version: &Version,
        includes: impl IntoIterator<Item = &'static str>,
        content: &'static str,
    ) -> Self {
        log::info!("GLOW - Compiling vertex shader.");
        let content = concat_shader(&version.vertex, content, includes);
        Shader::compile(gl, glow::VERTEX_SHADER, &content)
    }

    /// Creates a fragment [`Shader`].
    pub fn fragment(
        gl: &glow::Context,
        version: &Version,
        includes: impl IntoIterator<Item = &'static str>,
        content: &'static str,
    ) -> Self {
        log::info!("GLOW - Compiling fragment shader.");
        let content = concat_shader(&version.fragment, content, includes);
        Shader::compile(gl, glow::FRAGMENT_SHADER, &content)
    }
}

fn concat_shader(
    version: &String,
    content: &'static str,
    includes: impl IntoIterator<Item = &'static str>,
) -> String {
    let includes = includes
        .into_iter()
        .fold(String::new(), |acc, i| acc + "\n" + i);

    if includes.is_empty() {
        format!("{}\n{}", version, content)
    } else {
        format!(
            "{}\n{}",
            version,
            content.replace("// #includes", includes.as_str())
        )
    }
}

pub unsafe fn create(
    gl: &glow::Context,
    shaders: &[Shader],
    attributes: &[(u32, &str)],
) -> <glow::Context as HasContext>::Program {
    let program = gl.create_program().expect("Cannot create program");

    for shader in shaders {
        gl.attach_shader(program, shader.0);
    }

    for (i, name) in attributes {
        gl.bind_attrib_location(program, *i, name);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader.0);
        gl.delete_shader(shader.0);
    }

    program
}
