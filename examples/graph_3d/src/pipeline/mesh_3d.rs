use glam::{Vec3, vec3};
use rand::Rng;
use wgpu::BufferAddress;
use iced::Size;

pub struct Mesh3D {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub vertices: [Vec3; 100], //hard-coded 100 for now
}

impl Mesh3D {
    pub fn gen_rnd(device: &wgpu::Device, format: wgpu::TextureFormat, layout: &wgpu::PipelineLayout) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph_3d.mesh_3d.vertices"),
            size: std::mem::size_of::<Mesh3D>() as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("graph_3d.mesh_3d.shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/mesh_3d.wgsl"))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("graph3d.mesh_3d.pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vec3>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3
                        ],
                    }
                ],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: Default::default(),
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Point,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(
                wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }
            ),
            multiview: None,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertices: std::array::from_fn(|index| {
                vec3(
                    rand::thread_rng().gen_range(0.0..=1.0),
                    rand::thread_rng().gen_range(0.0..=1.0),
                    rand::thread_rng().gen_range(0.0..=1.0),
                )
            })
        }
    }

    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
    ) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }
}