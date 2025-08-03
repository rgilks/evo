use crate::components::{Position, Size};
use hecs::Entity;

/// GPU-accelerated spatial system using compute shaders
pub struct GpuSpatialSystem {
    device: wgpu::Device,
    queue: wgpu::Queue,

    // Buffers for entity data
    entity_positions: wgpu::Buffer,
    entity_radii: wgpu::Buffer,
    entity_ids: wgpu::Buffer,

    // Buffers for spatial queries
    query_results: wgpu::Buffer,
    query_count: wgpu::Buffer,
    query_params: wgpu::Buffer,

    // Compute pipeline for spatial queries
    spatial_query_pipeline: wgpu::ComputePipeline,

    // Bind group for the compute pipeline
    bind_group: wgpu::BindGroup,

    entity_count: u32,
    world_size: f32,
}

impl GpuSpatialSystem {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        world_size: f32,
        max_entities: u32,
    ) -> Self {
        let entity_positions = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Positions"),
            size: (max_entities * 8) as u64, // 2 f32s per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entity_radii = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Radii"),
            size: (max_entities * 4) as u64, // 1 f32 per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entity_ids = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity IDs"),
            size: (max_entities * 4) as u64, // 1 u32 per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let query_results = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query Results"),
            size: (max_entities * 4) as u64, // 1 u32 per entity (max results)
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let query_count = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query Count"),
            size: 4, // 1 u32
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let query_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query Parameters"),
            size: 12, // 3 f32s
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create compute shader for spatial queries
        let spatial_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Spatial Query Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("spatial_query_shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Spatial Query Bind Group Layout"),
            entries: &[
                // Entity positions
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity radii
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity IDs
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Query results
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Query count
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Query parameters
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Spatial Query Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let spatial_query_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Spatial Query Pipeline"),
                layout: Some(&pipeline_layout),
                module: &spatial_shader,
                entry_point: "main",
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Spatial Query Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: entity_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: entity_radii.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: entity_ids.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: query_results.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: query_count.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: query_params.as_entire_binding(),
                },
            ],
        });

        Self {
            device,
            queue,
            entity_positions,
            entity_radii,
            entity_ids,
            query_results,
            query_count,
            query_params,
            spatial_query_pipeline,
            bind_group,
            entity_count: 0,
            world_size,
        }
    }

    /// Update entity data on GPU
    pub fn update_entities(&mut self, entities: &[(Entity, Position, Size)]) {
        self.entity_count = entities.len() as u32;

        // Prepare data for GPU
        let positions: Vec<f32> = entities
            .iter()
            .flat_map(|(_, pos, _)| vec![pos.x, pos.y])
            .collect();

        let radii: Vec<f32> = entities.iter().map(|(_, _, size)| size.radius).collect();

        let ids: Vec<u32> = entities
            .iter()
            .map(|(entity, _, _)| entity.to_bits().get() as u32)
            .collect();

        // Upload to GPU
        self.queue
            .write_buffer(&self.entity_positions, 0, bytemuck::cast_slice(&positions));
        self.queue
            .write_buffer(&self.entity_radii, 0, bytemuck::cast_slice(&radii));
        self.queue
            .write_buffer(&self.entity_ids, 0, bytemuck::cast_slice(&ids));
    }

    /// Query for entities within a radius of a point
    pub fn query_radius(&mut self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        // Reset query count
        let zero_count = [0u32];
        self.queue
            .write_buffer(&self.query_count, 0, bytemuck::cast_slice(&zero_count));

        // Update query parameters
        let query_params = [x, y, radius];
        self.queue
            .write_buffer(&self.query_params, 0, bytemuck::cast_slice(&query_params));

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Spatial Query Encoder"),
            });

        // Dispatch compute shader
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Spatial Query Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.spatial_query_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        // Dispatch with one thread per entity
        let workgroup_size = 256;
        let workgroup_count = (self.entity_count + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);

        drop(compute_pass);

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back results
        let staging_count = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Count"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let staging_results = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Results"),
            size: (self.entity_count * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Copy data to staging buffers
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Results Encoder"),
            });

        encoder.copy_buffer_to_buffer(&self.query_count, 0, &staging_count, 0, 4);
        encoder.copy_buffer_to_buffer(
            &self.query_results,
            0,
            &staging_results,
            0,
            (self.entity_count * 4) as u64,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back data
        staging_count
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| {});
        staging_results
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| {});

        self.device.poll(wgpu::Maintain::Wait);

        let count_data = staging_count.slice(..).get_mapped_range();
        let results_data = staging_results.slice(..).get_mapped_range();

        // Convert bytes back to u32 values
        let mut count = 0u32;
        if count_data.len() >= 4 {
            count =
                u32::from_le_bytes([count_data[0], count_data[1], count_data[2], count_data[3]]);
        }

        let mut results = Vec::new();
        for chunk in results_data.chunks(4) {
            if chunk.len() == 4 {
                let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                results.push(value);
            }
        }

        // Convert back to Entity IDs, limiting to the actual count
        results
            .into_iter()
            .take(count as usize)
            .filter_map(|id| Entity::from_bits(id as u64))
            .collect()
    }
}
