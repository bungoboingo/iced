//! Draw meshes of triangles.
mod msaa;

use crate::Transformation;
use crate::{buffer, settings};

use iced_graphics::layer::mesh::{self, Mesh};
use iced_graphics::triangle::ColoredVertex2D;
use iced_graphics::Size;
#[cfg(feature = "tracing")]
use tracing::info_span;

const INITIAL_BUFFER_COUNT: usize = 10_000;

#[derive(Debug)]
pub struct Pipeline {
    blit: Option<msaa::Blit>,
    index_buffer: buffer::Static<u32>,
    index_strides: Vec<u32>,
    solid: solid::Pipeline,
    gradient: gradient::Pipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Pipeline {
        Pipeline {
            blit: antialiasing.map(|a| msaa::Blit::new(device, format, a)),
            index_buffer: buffer::Static::new(
                device,
                "iced_wgpu::triangle vertex buffer",
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                INITIAL_BUFFER_COUNT,
            ),
            index_strides: Vec::new(),
            solid: solid::Pipeline::new(device, format, antialiasing),
            gradient: gradient::Pipeline::new(device, format, antialiasing),
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
        transformation: Transformation,
        scale_factor: f32,
        meshes: &[Mesh<'_>],
    ) {
        #[cfg(feature = "tracing")]
        let _ = info_span!("Wgpu::Triangle", "DRAW").entered();

        // Count the total amount of vertices & indices we need to handle
        let count = mesh::attribute_count_of(meshes);

        // Then we ensure the current attribute buffers are big enough, resizing if necessary.
        // We are not currently using the return value of these functions as we have no system in
        // place to calculate mesh diff, or to know whether or not that would be more performant for
        // the majority of use cases. Therefore we will write GPU data every frame (for now).
        let _ = self.index_buffer.resize(device, count.indices);
        let _ = self.solid.vertices.resize(device, count.solid_vertices);
        let _ = self
            .gradient
            .vertices
            .resize(device, count.gradient_vertices);

        self.index_strides.clear();
        self.solid.vertices.clear();
        self.gradient.vertices.clear();
        // Prepare dynamic buffers & data store for writing
        self.solid.uniforms.clear();
        self.gradient.uniforms.clear();

        let mut solid_vertex_offset = 0;
        let mut gradient_vertex_offset = 0;
        let mut index_offset = 0;

        for mesh in meshes {
            let origin = mesh.origin();
            let indices = mesh.indices();

            let transform =
                transformation * Transformation::translate(origin.x, origin.y);

            let new_index_offset = self.index_buffer.write(
                device,
                staging_belt,
                encoder,
                index_offset,
                indices,
            );

            index_offset += new_index_offset;
            self.index_strides.push(indices.len() as u32);

            //push uniform data to CPU buffers
            match mesh {
                Mesh::Solid { buffers, .. } => {
                    self.solid.uniforms.push(&Uniforms::new(transform));

                    let written_bytes = self.solid.vertices.write(
                        device,
                        staging_belt,
                        encoder,
                        solid_vertex_offset,
                        &buffers.vertices,
                    );

                    solid_vertex_offset += written_bytes;
                }
                Mesh::Gradient { buffers, .. } => {
                    self.gradient.uniforms.push(&Uniforms {
                        transform: transform.into(),
                    });

                    let written_bytes = self.gradient.vertices.write(
                        device,
                        staging_belt,
                        encoder,
                        gradient_vertex_offset,
                        &buffers.vertices,
                    );

                    gradient_vertex_offset += written_bytes;
                }
            }
        }

        // Write uniform data to GPU
        if count.solid_vertices > 0 {
            let uniforms_resized = self.solid.uniforms.resize(device);

            if uniforms_resized {
                self.solid.bind_group = solid::Pipeline::bind_group(
                    device,
                    self.solid.uniforms.raw(),
                    &self.solid.bind_group_layout,
                )
            }

            self.solid.uniforms.write(device, staging_belt, encoder);
        }

        if count.gradient_vertices > 0 {
            // Resize buffers if needed
            let uniforms_resized = self.gradient.uniforms.resize(device);

            if uniforms_resized {
                self.gradient.bind_group = gradient::Pipeline::bind_group(
                    device,
                    self.gradient.uniforms.raw(),
                    &self.gradient.bind_group_layout,
                );
            }

            // Write to GPU
            self.gradient.uniforms.write(device, staging_belt, encoder);
        }

        // Configure render pass
        {
            let (attachment, resolve_target, load) = if let Some(blit) =
                &mut self.blit
            {
                let (attachment, resolve_target) =
                    blit.targets(device, target_size.width, target_size.height);

                (
                    attachment,
                    Some(resolve_target),
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                )
            } else {
                (target, None, wgpu::LoadOp::Load)
            };

            #[cfg(feature = "tracing")]
            let _ = info_span!("Wgpu::Triangle", "BEGIN_RENDER_PASS").enter();

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("iced_wgpu::triangle render pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: attachment,
                            resolve_target,
                            ops: wgpu::Operations { load, store: true },
                        },
                    )],
                    depth_stencil_attachment: None,
                });

            let mut num_solids = 0;
            let mut num_gradients = 0;
            let mut last_is_solid = None;

            for (index, mesh) in meshes.iter().enumerate() {
                let clip_bounds = (mesh.clip_bounds() * scale_factor).snap();

                render_pass.set_scissor_rect(
                    clip_bounds.x,
                    clip_bounds.y,
                    clip_bounds.width,
                    clip_bounds.height,
                );

                match mesh {
                    Mesh::Solid { .. } => {
                        if !last_is_solid.unwrap_or(false) {
                            render_pass.set_pipeline(&self.solid.pipeline);

                            last_is_solid = Some(true);
                        }

                        render_pass.set_bind_group(
                            0,
                            &self.solid.bind_group,
                            &[self.solid.uniforms.offset_at_index(num_solids)],
                        );

                        render_pass.set_vertex_buffer(
                            0,
                            self.solid.vertices.slice_from_index(num_solids),
                        );

                        num_solids += 1;
                    }
                    Mesh::Gradient { .. } => {
                        if last_is_solid.unwrap_or(true) {
                            render_pass.set_pipeline(&self.gradient.pipeline);

                            last_is_solid = Some(false);
                        }

                        render_pass.set_bind_group(
                            0,
                            &self.gradient.bind_group,
                            &[self
                                .gradient
                                .uniforms
                                .offset_at_index(num_gradients)],
                        );

                        render_pass.set_vertex_buffer(
                            0,
                            self.gradient
                                .vertices
                                .slice_from_index(num_gradients),
                        );

                        num_gradients += 1;
                    }
                };

                render_pass.set_index_buffer(
                    self.index_buffer.slice_from_index(index),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(0..self.index_strides[index], 0, 0..1);
            }
        }

        self.index_buffer.clear();

        if let Some(blit) = &mut self.blit {
            blit.draw(encoder, target);
        }
    }
}

fn fragment_target(
    texture_format: wgpu::TextureFormat,
) -> Option<wgpu::ColorTargetState> {
    Some(wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    })
}

fn primitive_state() -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        front_face: wgpu::FrontFace::Cw,
        ..Default::default()
    }
}

fn multisample_state(
    antialiasing: Option<settings::Antialiasing>,
) -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: antialiasing.map(|a| a.sample_count()).unwrap_or(1),
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}

#[derive(Debug, Clone, Copy, encase::ShaderType)]
pub struct Uniforms {
    transform: glam::Mat4,
}

impl Uniforms {
    pub fn new(transform: Transformation) -> Self {
        Self {
            transform: transform.into(),
        }
    }
}

mod solid {
    use crate::triangle;
    use crate::triangle::{Uniforms, INITIAL_BUFFER_COUNT};
    use crate::{buffer, settings};
    use encase::ShaderType;

    #[derive(Debug)]
    pub struct Pipeline {
        pub pipeline: wgpu::RenderPipeline,
        pub vertices: buffer::Static<triangle::ColoredVertex2D>,
        pub uniforms: buffer::DynamicUniform<Uniforms>,
        pub bind_group_layout: wgpu::BindGroupLayout,
        pub bind_group: wgpu::BindGroup,
    }

    impl Pipeline {
        /// Creates a new [SolidPipeline] using `triangle/solid.wgsl` shader.
        pub fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            antialiasing: Option<settings::Antialiasing>,
        ) -> Self {
            let vertices = buffer::Static::new(
                device,
                "iced_wgpu::triangle::solid vertex buffer",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                INITIAL_BUFFER_COUNT,
            );

            let uniforms = buffer::DynamicUniform::new(
                device,
                "iced_wgpu::triangle::solid uniforms",
            );

            let bind_group_layout = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("iced_wgpu::triangle::solid bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(Uniforms::min_size()),
                        },
                        count: None,
                    }],
                },
            );

            let bind_group =
                Self::bind_group(device, uniforms.raw(), &bind_group_layout);

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("iced_wgpu::triangle::solid pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(
                        "iced_wgpu::triangle::solid create shader module",
                    ),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(include_str!(
                            "shader/triangle/solid.wgsl"
                        )),
                    ),
                });

            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu::triangle::solid pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<
                                triangle::ColoredVertex2D,
                            >()
                                as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array!(
                                // Position
                                0 => Float32x2,
                                // Color
                                1 => Float32x4,
                            ),
                        }],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[triangle::fragment_target(format)],
                    }),
                    primitive: triangle::primitive_state(),
                    depth_stencil: None,
                    multisample: triangle::multisample_state(antialiasing),
                    multiview: None,
                },
            );

            Self {
                pipeline,
                vertices,
                uniforms,
                bind_group_layout,
                bind_group,
            }
        }

        pub fn bind_group(
            device: &wgpu::Device,
            buffer: &wgpu::Buffer,
            layout: &wgpu::BindGroupLayout,
        ) -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::triangle::solid bind group"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer,
                            offset: 0,
                            size: Some(Uniforms::min_size()),
                        },
                    ),
                }],
            })
        }
    }
}

mod gradient {
    use crate::triangle;
    use crate::{buffer, settings};

    use crate::triangle::{Uniforms, INITIAL_BUFFER_COUNT};
    use encase::ShaderType;
    use iced_graphics::triangle::GradientVertex2D;

    #[derive(Debug)]
    pub struct Pipeline {
        pub pipeline: wgpu::RenderPipeline,
        pub vertices: buffer::Static<GradientVertex2D>,
        pub uniforms: buffer::DynamicUniform<Uniforms>,
        pub bind_group_layout: wgpu::BindGroupLayout,
        pub bind_group: wgpu::BindGroup,
    }

    impl Pipeline {
        /// Creates a new [GradientPipeline] using `triangle/gradient.wgsl` shader.
        pub(super) fn new(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            antialiasing: Option<settings::Antialiasing>,
        ) -> Self {
            let vertices = buffer::Static::new(
                device,
                "iced_wgpu::triangle::gradient vertex buffer",
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                INITIAL_BUFFER_COUNT,
            );

            let uniforms = buffer::DynamicUniform::new(
                device,
                "iced_wgpu::triangle::gradient uniforms",
            );

            let bind_group_layout = device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some(
                        "iced_wgpu::triangle::gradient bind group layout",
                    ),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(Uniforms::min_size()),
                        },
                        count: None,
                    }],
                },
            );

            let bind_group = Pipeline::bind_group(
                device,
                uniforms.raw(),
                &bind_group_layout,
            );

            let layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some(
                        "iced_wgpu::triangle::gradient pipeline layout",
                    ),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                },
            );

            let shader =
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(
                        "iced_wgpu::triangle::gradient create shader module",
                    ),
                    source: wgpu::ShaderSource::Wgsl(
                        std::borrow::Cow::Borrowed(include_str!(
                            "shader/triangle/gradient.wgsl"
                        )),
                    ),
                });

            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("iced_wgpu::triangle::gradient pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<GradientVertex2D>(
                            ) as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array!(
                                // Position
                                0 => Float32x2,
                                // Color 1
                                1 => Float32x4,
                                // Color 2
                                2 => Float32x4,
                                // Color 3
                                3 => Float32x4,
                                // Color 4
                                4 => Float32x4,
                                // Color 5
                                5 => Float32x4,
                                // Color 6
                                6 => Float32x4,
                                // Color 7
                                7 => Float32x4,
                                // Color 8
                                8 => Float32x4,
                                // Offsets 1-4
                                9 => Float32x4,
                                // Offsets 5-8
                                10 => Float32x4,
                                // Direction
                                11 => Float32x4
                            ),
                        }],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[triangle::fragment_target(format)],
                    }),
                    primitive: triangle::primitive_state(),
                    depth_stencil: None,
                    multisample: triangle::multisample_state(antialiasing),
                    multiview: None,
                },
            );

            Self {
                pipeline,
                vertices,
                uniforms,
                bind_group_layout,
                bind_group,
            }
        }

        pub fn bind_group(
            device: &wgpu::Device,
            uniform_buffer: &wgpu::Buffer,
            layout: &wgpu::BindGroupLayout,
        ) -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("iced_wgpu::triangle::gradient bind group"),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: uniform_buffer,
                            offset: 0,
                            size: Some(Uniforms::min_size()),
                        },
                    ),
                }],
            })
        }
    }
}
