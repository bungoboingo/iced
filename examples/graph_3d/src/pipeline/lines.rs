// use glam::Vec3;
// use wgpu::{BufferAddress, BufferSize};
// use iced::Size;
//
// pub struct Lines {
//     pipeline: wgpu::RenderPipeline,
//     lines: wgpu::Buffer,
// }
//
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// #[repr(C)]
// pub struct Line {
//     origin: Vec3,
//     direction: Vec3,
//     length: f32,
// }
//
// impl Lines {
//     fn init(
//         device: &wgpu::Device,
//         format: wgpu::TextureFormat,
//         target_size: Size<u32>,
//     ) -> Self {
//         let lines = device.create_buffer(&wgpu::BufferDescriptor {
//             label: Some("graph_3d.lines.buffer"),
//             size: std::mem::size_of::<Line>() as BufferAddress,
//             usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//             mapped_at_creation: false,
//         });
//
//         let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//             label: Some("graph_3d.lines.shader"),
//             source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
//                 "../shaders/lines.wgsl"
//             ))),
//         });
//
//         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("graph_3d.lines.pipeline"),
//             layout: None,
//             vertex: wgpu::VertexState {
//                 module: &shader,
//                 entry_point: "vs_main",
//                 buffers: &[wgpu::VertexBufferLayout {
//                     array_stride: std::mem::size_of::<Vec3>() as u64,
//                     step_mode: wgpu::VertexStepMode::Vertex,
//                     attributes: &wgpu::vertex_attr_array![
//                         //origin
//                         0 => Float32x3,
//                         //direction
//                         1 => Float32x3,
//                         //length
//                         2 => Float32
//                     ],
//                 }],
//             },
//             primitive: Default::default(),
//             depth_stencil: None,
//             multisample: Default::default(),
//             fragment: None,
//             multiview: None,
//         });
//     }
// }