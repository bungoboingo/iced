struct Uniforms {
    transform: mat4x4<f32>,
    pos: vec2<f32>,
    scale: vec2<f32>,
    blur: f32,
    dir: f32,
}

/// The rendered texture of the layer
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var src_texture: texture_2d<f32>;
@group(0) @binding(2) var src_sampler: sampler;

struct VertexInput {
    @location(0) v_pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(uniforms.scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, uniforms.scale.y + 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(uniforms.pos - vec2<f32>(0.5, 0.5), 0.0, 1.0),
    );

    var out: VertexOutput;
    out.clip_pos = uniforms.transform * transform * vec4<f32>(input.v_pos, 0.0, 1.0);
    out.uv = input.uv;

    return out;
}

fn gaussian(x: f32, e: f32) -> f32 {
    return exp(-pow(x, 2.0)/e);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // the kernel size == the blur radius
    var color = vec4<f32>(0.0);
    let rad = uniforms.blur;

    var total = 0.0;
    let kernel_w = abs(u32(rad) + 1u); //rad == 15, this == 16

    for (var x = 0u; x < kernel_w; x++) {
        let x_2 = 1.0 * (f32(x) - rad); // -15

        var offset: vec2<f32>;

        if (uniforms.dir >= 0.5) {
            //vertical
            offset = vec2<f32>(0.0, x_2 / uniforms.scale.y); //vec2(0, -0.01), vertical
        } else {
            //horizontal
            offset = vec2<f32>(x_2 / uniforms.scale.x, 0.0); //vec2(-0.01, 0), horizontal
        }

        let i_pos = input.uv + offset;
        let g = gaussian(x_2, rad * rad);
        color += g * textureSample(src_texture, src_sampler, i_pos);
        total += g;
    }

    return color / total;
}
