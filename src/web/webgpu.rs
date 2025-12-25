use bytemuck::{Pod, Zeroable};
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
    center: [f32; 2],
    radius: f32,
}

#[wasm_bindgen]
pub struct WebGpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl WebGpuRenderer {
    pub async fn create(canvas: web_sys::HtmlCanvasElement) -> Result<WebGpuRenderer, JsValue> {
        let width = canvas.width();
        let height = canvas.height();

        // Create wgpu instance with WebGPU backend
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        });

        // Create surface using raw-window-handle (matching galacto)
        let canvas_handle = unsafe {
            raw_window_handle::WebCanvasWindowHandle::new(std::ptr::NonNull::new_unchecked(
                &canvas as *const _ as *mut std::ffi::c_void,
            ))
        };
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: raw_window_handle::RawDisplayHandle::Web(
                        raw_window_handle::WebDisplayHandle::new(),
                    ),
                    raw_window_handle: raw_window_handle::RawWindowHandle::WebCanvas(canvas_handle),
                })
                .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {:?}", e)))?
        };

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find adapter")?;

        // Request device - use default to avoid browser-incompatible limits
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to get device: {:?}", e)))?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        // Create pipeline
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
                entry_point: Some("vs_main"),
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
                            format: wgpu::VertexFormat::Float32x2,
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
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            cache: None,
        });

        // Create initial vertex buffer
        let initial_vertices = vec![
            Vertex {
                position: [0.0, 0.0],
                color: [0.0, 0.0, 0.0],
                center: [0.0, 0.0],
                radius: 0.0,
            };
            60000 // Pre-allocate for 10000 entities (6 vertices each)
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&initial_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(WebGpuRenderer {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            vertex_buffer,
            num_vertices: 0,
            width,
            height,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, entities_ptr: *const f32, entity_count: u32) {
        if entity_count == 0 {
            return;
        }

        // Read entity data from pointer (6 floats per entity: x, y, radius, r, g, b)
        let entity_data =
            unsafe { std::slice::from_raw_parts(entities_ptr, (entity_count * 6) as usize) };

        // Convert to vertices
        let mut vertices = Vec::with_capacity(entity_count as usize * 6);
        let world_size = self.width.max(self.height) as f32;

        for chunk in entity_data.chunks(6) {
            if chunk.len() < 6 {
                break;
            }
            let (x, y, radius, r, g, b) =
                (chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]);

            let screen_x = (x + world_size / 2.0) / world_size * 2.0 - 1.0;
            let screen_y = -((y + world_size / 2.0) / world_size * 2.0 - 1.0);
            let screen_radius = (radius / world_size * 2.0 / 10.0).min(0.015);

            let glow_extension = screen_radius * 0.5;
            let quad_size = screen_radius + glow_extension;
            let color = [r, g, b];

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

        // Update vertex buffer
        if !vertices.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }

        // Render
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };

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
    }
}
