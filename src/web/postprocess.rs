//! HDR scene target + bloom post-processing (ported from the galacto sandbox).
//!
//! Creatures are rendered additively into an HDR (`rgba16float`) scene texture so
//! bright regions can exceed 1.0. Bloom is then extracted (bright-pass), blurred
//! separably at reduced resolution, and added back during a tonemapped composite
//! into the swapchain. See `src/post.wgsl`.

use wgpu::util::DeviceExt;

/// Format of the offscreen scene and bloom targets. The particle render pipeline
/// targets this format (not the surface format); only the composite writes the
/// surface. `rgba16float` is a renderable, blendable, filterable WebGPU format.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Bloom buffers are this fraction of the scene resolution — cheaper and wider.
const BLOOM_DIV: u32 = 4;

/// Size-dependent views and bind groups, rebuilt on resize.
struct Targets {
    scene_view: wgpu::TextureView,
    bloom_a_view: wgpu::TextureView,
    bloom_b_view: wgpu::TextureView,
    scene_bg: wgpu::BindGroup,
    bloom_a_bg: wgpu::BindGroup,
    bloom_b_bg: wgpu::BindGroup,
    bloom_a_tex_bg: wgpu::BindGroup,
}

pub struct PostProcess {
    blur_layout: wgpu::BindGroupLayout,
    bloom_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    fade_pipeline: wgpu::RenderPipeline,
    bright_pipeline: wgpu::RenderPipeline,
    blur_h_pipeline: wgpu::RenderPipeline,
    blur_v_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    /// Live post params `[bloom, trail_persistence, exposure, _pad]`, bound at
    /// group 3 of the fade + composite passes. Updated by the Glow/Trails/
    /// Brightness sliders via `set_params`.
    post_buffer: wgpu::Buffer,
    post_bg: wgpu::BindGroup,
    targets: Targets,
}

impl PostProcess {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../post.wgsl").into()),
        });

        let blur_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Blur Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bloom_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Bloom Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        // Live post params, bound at group 3 so the fade and composite passes
        // share one value without colliding with the blur passes' group 0/1
        // textures. Defaults match the cinematic look: glow 0.72, trails 0.93,
        // exposure 1.2.
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Params Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let post_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Post Params Buffer"),
            contents: bytemuck::cast_slice(&[[0.72f32, 0.93, 1.2, 0.0]]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let post_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post Params BG"),
            layout: &post_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: post_buffer.as_entire_binding(),
            }],
        });

        let blur_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Blur Pipeline Layout"),
            bind_group_layouts: &[Some(&blur_layout)],
            immediate_size: 0,
        });
        // Composite reads group 0 (scene), group 1 (bloom), group 3 (post params).
        let composite_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Composite Pipeline Layout"),
            bind_group_layouts: &[
                Some(&blur_layout),
                Some(&bloom_layout),
                None,
                Some(&post_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = |entry: &str, layout: &wgpu::PipelineLayout, format: wgpu::TextureFormat| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("fs_vert"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };

        let bright_pipeline = pipeline("bright_pass", &blur_pl, HDR_FORMAT);
        let blur_h_pipeline = pipeline("blur_h", &blur_pl, HDR_FORMAT);
        let blur_v_pipeline = pipeline("blur_v", &blur_pl, HDR_FORMAT);
        let composite_pipeline = pipeline("composite", &composite_pl, surface_format);

        // Trail-fade pipeline: a fullscreen black quad with alpha blending that
        // decays the HDR scene each frame before particles are redrawn on top.
        // It samples nothing, so its layout is empty.
        // Fade reads only the post params (group 3); groups 0–2 are unused.
        let fade_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Fade Pipeline Layout"),
            bind_group_layouts: &[None, None, None, Some(&post_layout)],
            immediate_size: 0,
        });
        let fade_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fade"),
            layout: Some(&fade_pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("fs_vert"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fade"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Post Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let targets = Self::make_targets(device, &blur_layout, &bloom_layout, &sampler, size);

        Self {
            blur_layout,
            bloom_layout,
            sampler,
            fade_pipeline,
            bright_pipeline,
            blur_h_pipeline,
            blur_v_pipeline,
            composite_pipeline,
            post_buffer,
            post_bg,
            targets,
        }
    }

    /// Update the live post params from the Glow/Trails/Brightness sliders.
    pub fn set_params(
        &self,
        queue: &wgpu::Queue,
        bloom: f32,
        trail_persistence: f32,
        exposure: f32,
    ) {
        queue.write_buffer(
            &self.post_buffer,
            0,
            bytemuck::cast_slice(&[[bloom, trail_persistence, exposure, 0.0f32]]),
        );
    }

    fn make_targets(
        device: &wgpu::Device,
        blur_layout: &wgpu::BindGroupLayout,
        bloom_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: (u32, u32),
    ) -> Targets {
        let target = |w: u32, h: u32, label: &str| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: w.max(1),
                        height: h.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        let scene_view = target(size.0, size.1, "Scene HDR");
        let bloom_a_view = target(size.0 / BLOOM_DIV, size.1 / BLOOM_DIV, "Bloom A");
        let bloom_b_view = target(size.0 / BLOOM_DIV, size.1 / BLOOM_DIV, "Bloom B");

        let blur_bg = |view: &wgpu::TextureView, label: &str| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: blur_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            })
        };

        let bloom_a_tex_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom A Texture BG"),
            layout: bloom_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bloom_a_view),
            }],
        });

        Targets {
            scene_bg: blur_bg(&scene_view, "Scene BG"),
            bloom_a_bg: blur_bg(&bloom_a_view, "Bloom A BG"),
            bloom_b_bg: blur_bg(&bloom_b_view, "Bloom B BG"),
            bloom_a_tex_bg,
            scene_view,
            bloom_a_view,
            bloom_b_view,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.targets = Self::make_targets(
            device,
            &self.blur_layout,
            &self.bloom_layout,
            &self.sampler,
            size,
        );
    }

    /// The HDR target the particle pass renders into.
    pub fn scene_view(&self) -> &wgpu::TextureView {
        &self.targets.scene_view
    }

    /// Decay the persisted HDR scene by `TRAIL_PERSISTENCE` (a fullscreen
    /// alpha-blended black quad), leaving motion trails for the next particle
    /// pass to build on. Must run BEFORE the particle pass, which loads (does not
    /// clear) this same target. Trails are screen-space, so panning/zooming the
    /// camera smears them — an accepted trade-off for the default near-static
    /// view. The renderer draws interpolated positions at 60fps, so trails stay
    /// smooth even though the sim ticks far slower.
    pub fn fade_scene(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Trail Fade"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.targets.scene_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rp.set_pipeline(&self.fade_pipeline);
        rp.set_bind_group(3, &self.post_bg, &[]);
        rp.draw(0..3, 0..1);
    }

    /// Run bright-pass → blur (H, V) → tonemapped composite into `output`.
    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        self.pass(
            encoder,
            &self.targets.bloom_a_view,
            &self.bright_pipeline,
            &self.targets.scene_bg,
            None,
            None,
            "Bloom Bright",
        );
        self.pass(
            encoder,
            &self.targets.bloom_b_view,
            &self.blur_h_pipeline,
            &self.targets.bloom_a_bg,
            None,
            None,
            "Bloom Blur H",
        );
        self.pass(
            encoder,
            &self.targets.bloom_a_view,
            &self.blur_v_pipeline,
            &self.targets.bloom_b_bg,
            None,
            None,
            "Bloom Blur V",
        );
        self.pass(
            encoder,
            output,
            &self.composite_pipeline,
            &self.targets.scene_bg,
            Some(&self.targets.bloom_a_tex_bg),
            Some(&self.post_bg),
            "Composite",
        );
    }

    fn pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        pipeline: &wgpu::RenderPipeline,
        bind0: &wgpu::BindGroup,
        bind1: Option<&wgpu::BindGroup>,
        post: Option<&wgpu::BindGroup>,
        label: &str,
    ) {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rp.set_pipeline(pipeline);
        rp.set_bind_group(0, bind0, &[]);
        if let Some(b1) = bind1 {
            rp.set_bind_group(1, b1, &[]);
        }
        if let Some(p) = post {
            rp.set_bind_group(3, p, &[]);
        }
        rp.draw(0..3, 0..1);
    }
}
