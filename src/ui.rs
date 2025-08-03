use crate::simulation::Simulation;
use bytemuck::{Pod, Zeroable};
use std::time::{Duration, Instant};

use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl State {
    async fn new(
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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
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
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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

        // Create initial empty vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
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

    fn resize(&mut self, surface: &wgpu::Surface<'_>, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self, simulation: &Simulation, world_size: f32) {
        let entities = simulation.get_entities();

        // Convert entities to vertices (triangles for circles)
        let mut vertices = Vec::new();
        let max_entities_to_draw = 1000;
        let entities_to_draw = if entities.len() > max_entities_to_draw {
            let step = entities.len() / max_entities_to_draw;
            entities
                .iter()
                .step_by(step)
                .take(max_entities_to_draw)
                .collect::<Vec<_>>()
        } else {
            entities.iter().collect::<Vec<_>>()
        };

        for (x, y, radius, r, g, b) in entities_to_draw {
            // Convert world coordinates to normalized device coordinates (-1 to 1)
            let screen_x = (*x + world_size / 2.0) / world_size * 2.0 - 1.0;
            let screen_y = -((*y + world_size / 2.0) / world_size * 2.0 - 1.0); // Flip Y
            let screen_radius = (*radius / world_size * 2.0).min(0.1); // Scale radius

            // Create a simple triangle for each entity (simplified circle representation)
            let color = [*r, *g, *b];

            // Triangle 1
            vertices.push(Vertex {
                position: [screen_x, screen_y + screen_radius],
                color,
            });
            vertices.push(Vertex {
                position: [screen_x - screen_radius, screen_y - screen_radius],
                color,
            });
            vertices.push(Vertex {
                position: [screen_x + screen_radius, screen_y - screen_radius],
                color,
            });
        }

        self.num_vertices = vertices.len() as u32;

        // Update vertex buffer
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
    }

    fn render(&mut self, surface: &wgpu::Surface<'_>) -> Result<(), wgpu::SurfaceError> {
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
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
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

pub fn run(world_size: f32) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Evolution Simulation - Press ESC to exit")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);
    let window_for_event = window.clone();

    // Request the first redraw to start the animation loop
    window_for_event.request_redraw();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let surface = instance.create_surface(&window).unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let mut state = pollster::block_on(State::new(&surface, &adapter, window.inner_size()));
    let mut simulation = Simulation::new(world_size);
    let mut frame_count = 0;
    let window_id = window.id();

    println!("Evolution simulation window created! You should see colored triangles representing entities.");

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id: event_window_id,
                } if event_window_id == window_for_event.id() => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(&surface, *physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            // Update simulation
                            simulation.update();
                            frame_count += 1;

                            // Update rendering every frame for smooth animation
                            state.update(&simulation, world_size);

                            // Debug: Print frame info
                            if frame_count % 60 == 0 {
                                let entities = simulation.get_entities();
                                println!("Frame {}: {} entities", frame_count, entities.len());
                            }

                            // Debug: Print first few entity positions
                            if frame_count % 120 == 0 {
                                let entities = simulation.get_entities();
                                if !entities.is_empty() {
                                    let (x, y, _, _, _, _) = entities[0];
                                    println!("First entity position: ({:.2}, {:.2})", x, y);
                                }
                            }

                            match state.render(&surface) {
                                Ok(_) => {
                                    // Request next frame for continuous animation
                                    // This will be handled by the AboutToWait event
                                }
                                Err(wgpu::SurfaceError::Lost) => state.resize(&surface, state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    window_for_event.request_redraw();
                    elwt.set_control_flow(ControlFlow::Poll);
                }
                _ => {}
            }
        })
        .unwrap();
}
