use crate::simulation::Simulation;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
    center: [f32; 2], // Center position of the ball
    radius: f32,
}

pub struct State {
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl State {
    pub async fn new(
        surface: &wgpu::Surface<'_>,
        adapter: &wgpu::Adapter,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate, // Use Immediate for better performance
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1, // Reduce latency
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress
                                + std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2, // Changed to Float32x2 for center
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress
                                + std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress
                                + std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling for transparent objects
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create initial vertex buffer with reasonable size
        let initial_vertices = vec![
            Vertex {
                position: [0.0, 0.0],
                color: [0.0, 0.0, 0.0],
                center: [0.0, 0.0],
                radius: 0.0,
            };
            2000
        ]; // Pre-allocate space for 1000 entities (6 vertices per entity for quads)

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&initial_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices: 0,
        }
    }

    pub fn resize(&mut self, surface: &wgpu::Surface<'_>, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            surface.configure(&self.device, &self.config);
        }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, simulation: &Simulation, world_size: f32) {
        let entities = simulation.get_entities();
        self.update_with_entities(entities, world_size);
    }

    pub fn update_interpolated(
        &mut self,
        simulation: &Simulation,
        world_size: f32,
        interpolation_factor: f32,
    ) {
        let entities = simulation.get_interpolated_entities(interpolation_factor);
        self.update_with_entities(entities, world_size);
    }

    fn update_with_entities(
        &mut self,
        entities: Vec<(f32, f32, f32, f32, f32, f32)>,
        world_size: f32,
    ) {
        // Convert entities to vertices (triangles for circles)
        let mut vertices = Vec::new();

        // Draw all entities without sampling to prevent flickering
        for (x, y, radius, r, g, b) in entities {
            // Convert world coordinates to normalized device coordinates (-1 to 1)
            // Ensure proper centering and scaling
            let screen_x = (x + world_size / 2.0) / world_size * 2.0 - 1.0;
            let screen_y = -((y + world_size / 2.0) / world_size * 2.0 - 1.0); // Flip Y for screen coordinates
            let screen_radius = (radius / world_size * 2.0 / 10.0).min(0.015); // Scale radius - made 10x smaller

            // Create a larger quad to accommodate the glow effect
            // The glow extends beyond the actual radius, so we need extra space
            let glow_extension = screen_radius * 0.5; // Extra space for glow
            let quad_size = screen_radius + glow_extension;

            // Create a quad for each entity (will be rendered as a glowing ball)
            let color = [r, g, b];

            // Quad vertices (two triangles to form a square)
            // Triangle 1
            vertices.push(Vertex {
                position: [screen_x - quad_size, screen_y - quad_size],
                color,
                center: [screen_x, screen_y],
                radius: screen_radius,
            });
            vertices.push(Vertex {
                position: [screen_x + quad_size, screen_y - quad_size],
                color,
                center: [screen_x, screen_y],
                radius: screen_radius,
            });
            vertices.push(Vertex {
                position: [screen_x - quad_size, screen_y + quad_size],
                color,
                center: [screen_x, screen_y],
                radius: screen_radius,
            });

            // Triangle 2
            vertices.push(Vertex {
                position: [screen_x + quad_size, screen_y - quad_size],
                color,
                center: [screen_x, screen_y],
                radius: screen_radius,
            });
            vertices.push(Vertex {
                position: [screen_x + quad_size, screen_y + quad_size],
                color,
                center: [screen_x, screen_y],
                radius: screen_radius,
            });
            vertices.push(Vertex {
                position: [screen_x - quad_size, screen_y + quad_size],
                color,
                center: [screen_x, screen_y],
                radius: screen_radius,
            });
        }

        self.num_vertices = vertices.len() as u32;

        // Only recreate vertex buffer if size changed significantly or if it's empty
        if !vertices.is_empty() {
            // Use a larger buffer size to avoid frequent recreations
            let buffer_size = (vertices.len() * std::mem::size_of::<Vertex>()).max(2 * 1024 * 1024);

            if self.vertex_buffer.size() < buffer_size as u64 {
                // Recreate buffer if it's too small
                self.vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });
            } else {
                // Update existing buffer
                self.queue
                    .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            }
        }
    }

    pub fn render(&mut self, surface: &wgpu::Surface<'_>) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
