//struct Line {
//    origin: vec3<f32>,
//    end: vec3<f32>,
//}
//
//struct LineOut {
//    @builtin(position) clip_position: vec4<f32>,
//    end: vec4<f32>,
//}
//
//@vertex
//fn vs_main(@location(0) line: Line) -> LineOut {
//    var line_out = LineOut;
//
//    line_out.position = vec4<f32>(line.origin, 1.0);
//    line_out.end = vec4<f32>(line.end, 1.0);
//
//    return line_out;
//}
//
//@fragment
//fn fs_main(line: LineOut) -> @location(0) vec4<f32> {
//    let start = line.position;
//    let end = line.end;
//
//    let d = (end.x - start.x) * (start.x - )
//
//    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
//}
