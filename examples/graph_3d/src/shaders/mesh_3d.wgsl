struct In {
    @location(0) position: vec3<f32>,
}

struct Out {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: In) -> Out {
    var out: Out;
    out.clip_position = vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn fs_main(out: Out) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
