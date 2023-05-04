struct Uniforms {
    projection: mat4x4<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

const LIGHT_POS: vec3<f32> = vec3<f32>(0.0, 2.0, 3.0); //light dir is negative this since we're always pointing at zero

struct Cube {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec4<f32>,
}

struct CubeInstance {
    @location(5) matrix_0: vec4<f32>,
    @location(6) matrix_1: vec4<f32>,
    @location(7) matrix_2: vec4<f32>,
    @location(8) matrix_3: vec4<f32>,
}

struct CubeOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
}

fn rotate_y(angle: f32) -> mat3x3<f32> {
    let c: f32 = cos(angle);
    let s: f32 = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c, 0.0, s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-s, 0.0, c),
    );
}

@vertex
fn vs_main(cube: Cube, cube_instance: CubeInstance) -> CubeOut {
    let cube_matrix = mat4x4<f32>(
        cube_instance.matrix_0,
        cube_instance.matrix_1,
        cube_instance.matrix_2,
        cube_instance.matrix_3,
    );

    var out: CubeOut;
    out.position = uniforms.projection * cube_matrix * vec4<f32>(cube.position * rotate_y(uniforms.time), 1.0);
    out.normal = cube.normal;
    out.color = cube.color;
    return out;
}

@fragment
fn fs_main(cube: CubeOut) -> @location(0) vec4<f32> {
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let diffuse_str = max(dot(cube.normal, LIGHT_POS * -1.0), 0.0);

    return cube.color;
}
