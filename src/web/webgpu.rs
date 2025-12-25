use bytemuck::{Pod, Zeroable};
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

/// Instance data for each entity (32 bytes each)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Instance {
    prev_curr_pos: [f32; 4], // xy = prev_pos, zw = curr_pos
    radius_color: [f32; 4],  // x = radius, yzw = color (rgb)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SimulationUniforms {
    world_size: f32,
    interpolation_factor: f32,
    camera_zoom: f32,
    camera_x: f32,
    camera_y: f32,
    padding1: f32,
    padding2: f32,
    padding3: f32,
}

#[wasm_bindgen]
pub struct WebGpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    num_instances: u32,
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

        // Create surface using raw-window-handle
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

        // Request device
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

        // Create uniforms
        let uniforms = SimulationUniforms {
            world_size: 1000.0, // Default, will be updated
            interpolation_factor: 0.0,
            camera_zoom: 1.0,
            camera_x: 0.0,
            camera_y: 0.0,
            padding1: 0.0,
            padding2: 0.0,
            padding3: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Simulation Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Simulation Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        // Create pipeline layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create render pipeline with instanced rendering
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x4, // prev_curr_pos
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4, // radius_color
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

        // Create instance buffer (pre-allocate for 20000 entities)
        let initial_instances = vec![
            Instance {
                prev_curr_pos: [0.0, 0.0, 0.0, 0.0],
                radius_color: [0.0, 0.0, 0.0, 0.0],
            };
            20000
        ];

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&initial_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(WebGpuRenderer {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            instance_buffer,
            uniform_buffer,
            bind_group,
            num_instances: 0,
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

    pub fn render(
        &mut self,
        entities_ptr: *const f32,
        entity_count: u32,
        world_size: f32,
        interpolation_factor: f32,
        camera_zoom: f32,
        camera_x: f32,
        camera_y: f32,
    ) {
        if entity_count == 0 {
            return;
        }

        // Update uniforms
        let uniforms = SimulationUniforms {
            world_size,
            interpolation_factor,
            camera_zoom,
            camera_x,
            camera_y,
            padding1: 0.0,
            padding2: 0.0,
            padding3: 0.0,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Read entity data (8 floats per entity: prev_x, prev_y, cur_x, cur_y, radius, r, g, b)
        let entity_data =
            unsafe { std::slice::from_raw_parts(entities_ptr, (entity_count * 8) as usize) };

        // Convert to instances (Parallel conversion would be nice but requires a buffer)
        let mut instances = Vec::with_capacity(entity_count as usize);

        for chunk in entity_data.chunks(8) {
            if chunk.len() < 8 {
                break;
            }
            instances.push(Instance {
                prev_curr_pos: [chunk[0], chunk[1], chunk[2], chunk[3]],
                radius_color: [chunk[4], chunk[5], chunk[6], chunk[7]],
            });
        }

        self.num_instances = instances.len() as u32;

        // Update instance buffer
        if !instances.is_empty() {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
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
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..self.num_instances);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
