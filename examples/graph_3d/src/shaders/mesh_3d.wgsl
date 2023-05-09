struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct Input {
    @location(0) position: vec3<f32>,
}

struct Output {
    @builtin(position) frame_position: vec4<f32>,
    @location(0) vertex_position: vec3<f32>,
}

@vertex
fn vs_main(input: Input) -> Output {
    var output: Output;
    output.frame_position = uniforms.projection * vec4<f32>(input.position, 1.0);
    output.vertex_position = input.position;

    return output;
}

@fragment
fn fs_main(output: Output) -> @location(0) vec4<f32> {
    let radius = 50.0;
    let red = vec3<f32>(1.0, 0.0, 0.0);

    let dist = distance(output.vertex_position, output.frame_position.xyz);
    if (dist > radius) {
        discard;
    }

    let d = dist / radius;
    let color = mix(red, vec3<f32>(0.0), step(1.0 - 0.1, d));

    return vec4<f32>(color, 1.0);
}
