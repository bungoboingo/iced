struct Uniforms {
    transform: mat4x4<f32>,
}

/// The rendered texture of the layer
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var src_texture: texture_2d<f32>;
@group(0) @binding(2) var src_sampler: sampler;

struct VertexInput {
    @location(0) v_pos: vec2<f32>, // quad position
    @location(1) pos: vec2<f32>, //position within bounds,
    @location(2) size: vec2<f32>, //actual size within bounds
    @location(3) blur: f32, //blur radius
}

// TODO could switch to @builtin(vertex_index) maybe
// or just move the uniform information to the vertex attribute data
@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(input.size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, input.size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(input.pos, 0.0, 1.0),
    );

    return uniforms.transform * transform * vec4<f32>(input.v_pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let src_color = textureSample(
        src_texture,
        src_sampler,
        position.xy,
    );

    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}