struct Camera {
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(0)
var<uniform> resolution: vec2<f32>;

struct Input {
    @location(0) cube_center: vec3<f32>,
    @location(1) cube_vertex: vec3<f32>,
    @location(2) cube_color: vec3<f32>,
}

struct Output {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec3<f32>,
}

/// Calculates the length of the ray from the origin (the camera position) to the sphere surface, or
/// -1.0 in the case of a miss.
fn sphere_intersect(ray_origin: vec3<f32>, ray_dir: vec3<f32>, sph: vec4<f32>) -> f32 {
    let oc = ray_origin - sph.xyz;
    let b = dot(oc, ray_dir);
    let c = dot(oc, oc) - sph.w * sph.w;
    var h = b * b - c;
    if (h < 0.0) {
        //miss
        return -1.0;
    }
    //hit!
    h = sqrt(h);
    return -b -h;
}

@vertex
fn vs_main(input: Input) -> Output {
    var output: Output;

    output.clip_position = camera.projection * vec4<f32>(input.cube_vertex * 0.5 + input.cube_center.xyz, 1.0);
    output.color = input.cube_color;

// * sphere_intersect(
//        camera.position, //ray origin
//        normalize(output.clip_position.xyz / output.clip_position.w), //ray direction
//        vec4<f32>(input.cube_center, 1.0),
//    );

    return output;
}

@fragment
fn fs_main(output: Output) -> @location(0) vec4<f32> {
    return vec4<f32>(output.color, 1.0);
}
