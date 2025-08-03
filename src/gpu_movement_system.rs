use crate::components::{Position, Velocity, Energy, Size};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use hecs::Entity;

/// GPU-accelerated movement system
pub struct GpuMovementSystem {
    device: wgpu::Device,
    queue: wgpu::Queue,
    
    // Buffers for entity data
    entity_positions: wgpu::Buffer,
    entity_velocities: wgpu::Buffer,
    entity_energies: wgpu::Buffer,
    entity_sizes: wgpu::Buffer,
    entity_genes: wgpu::Buffer,
    
    // Buffers for movement targets and nearby entities
    movement_targets: wgpu::Buffer,
    nearby_entities: wgpu::Buffer,
    
    // Compute pipeline for movement updates
    movement_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    
    entity_count: u32,
    world_size: f32,
}

impl GpuMovementSystem {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, world_size: f32, max_entities: u32) -> Self {
        let entity_positions = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Positions"),
            size: (max_entities * 8) as u64, // 2 f32s per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let entity_velocities = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Velocities"),
            size: (max_entities * 8) as u64, // 2 f32s per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let entity_energies = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Energies"),
            size: (max_entities * 4) as u64, // 1 f32 per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let entity_sizes = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Sizes"),
            size: (max_entities * 4) as u64, // 1 f32 per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entity_genes = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Genes"),
            size: (max_entities * 16) as u64, // 4 f32s per entity (genes structure)
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let movement_targets = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Movement Targets"),
            size: (max_entities * 8) as u64, // 2 f32s per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let nearby_entities = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Nearby Entities"),
            size: (max_entities * 100 * 4) as u64, // Up to 100 nearby entities per entity
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create compute shader for movement updates
        let movement_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Movement Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("movement_shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Movement Bind Group Layout"),
            entries: &[
                // Entity positions
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity velocities
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity energies
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity sizes
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Entity genes
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Movement targets
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Nearby entities
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Movement Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let movement_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Movement Pipeline"),
            layout: Some(&pipeline_layout),
            module: &movement_shader,
            entry_point: "main",
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Movement Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: entity_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: entity_velocities.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: entity_energies.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: entity_sizes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: entity_genes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: movement_targets.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: nearby_entities.as_entire_binding(),
                },
            ],
        });

        Self {
            device,
            queue,
            entity_positions,
            entity_velocities,
            entity_energies,
            entity_sizes,
            entity_genes,
            movement_targets,
            nearby_entities,
            movement_pipeline,
            bind_group,
            entity_count: 0,
            world_size,
        }
    }

    /// Update entity data on GPU
    pub fn update_entities(
        &mut self,
        entities: &[(Entity, Position, Velocity, Energy, Size, Genes)],
    ) {
        self.entity_count = entities.len() as u32;
        
        // Prepare data for GPU
        let positions: Vec<f32> = entities
            .iter()
            .flat_map(|(_, pos, _, _, _, _)| vec![pos.x, pos.y])
            .collect();
        
        let velocities: Vec<f32> = entities
            .iter()
            .flat_map(|(_, _, vel, _, _, _)| vec![vel.x, vel.y])
            .collect();
        
        let energies: Vec<f32> = entities
            .iter()
            .map(|(_, _, _, energy, _, _)| energy.current)
            .collect();
        
        let sizes: Vec<f32> = entities
            .iter()
            .map(|(_, _, _, _, size, _)| size.radius)
            .collect();
        
        let genes: Vec<f32> = entities
            .iter()
            .flat_map(|(_, _, _, _, _, genes)| {
                // Convert genes to a flat array of floats
                vec![
                    genes.speed(),
                    genes.energy_efficiency(),
                    genes.size_factor(),
                    genes.sense_radius(),
                ]
            })
            .collect();

        // Upload to GPU
        self.queue.write_buffer(&self.entity_positions, 0, bytemuck::cast_slice(&positions));
        self.queue.write_buffer(&self.entity_velocities, 0, bytemuck::cast_slice(&velocities));
        self.queue.write_buffer(&self.entity_energies, 0, bytemuck::cast_slice(&energies));
        self.queue.write_buffer(&self.entity_sizes, 0, bytemuck::cast_slice(&sizes));
        self.queue.write_buffer(&self.entity_genes, 0, bytemuck::cast_slice(&genes));
    }

    /// Update movement targets and nearby entities
    pub fn update_spatial_data(
        &mut self,
        targets: &[(f32, f32)], // (x, y) for each entity
        nearby: &[Vec<u32>], // List of nearby entity IDs for each entity
    ) {
        let targets_flat: Vec<f32> = targets
            .iter()
            .flat_map(|(x, y)| vec![*x, *y])
            .collect();
        
        let nearby_flat: Vec<u32> = nearby
            .iter()
            .flat_map(|ids| {
                let mut padded = ids.clone();
                padded.resize(100, 0); // Pad to 100 entities
                padded
            })
            .collect();

        self.queue.write_buffer(&self.movement_targets, 0, bytemuck::cast_slice(&targets_flat));
        self.queue.write_buffer(&self.nearby_entities, 0, bytemuck::cast_slice(&nearby_flat));
    }

    /// Process movement updates on GPU
    pub fn update_movement(&mut self, config: &SimulationConfig) {
        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Movement Update Encoder"),
        });

        // Dispatch compute shader
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Movement Update Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.movement_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        
        // Dispatch with one thread per entity
        let workgroup_size = 256;
        let workgroup_count = (self.entity_count + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);

        drop(compute_pass);

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Read back updated entity data
    pub fn read_entity_data(&mut self) -> (Vec<Position>, Vec<Velocity>, Vec<Energy>) {
        // Create staging buffers
        let staging_positions = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Positions"),
            size: (self.entity_count * 8) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let staging_velocities = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Velocities"),
            size: (self.entity_count * 8) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let staging_energies = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Energies"),
            size: (self.entity_count * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Copy data to staging buffers
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Data Encoder"),
        });

        encoder.copy_buffer_to_buffer(&self.entity_positions, 0, &staging_positions, 0, (self.entity_count * 8) as u64);
        encoder.copy_buffer_to_buffer(&self.entity_velocities, 0, &staging_velocities, 0, (self.entity_count * 8) as u64);
        encoder.copy_buffer_to_buffer(&self.entity_energies, 0, &staging_energies, 0, (self.entity_count * 4) as u64);

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back data
        staging_positions.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        staging_velocities.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        staging_energies.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        
        self.device.poll(wgpu::Maintain::Wait);

        let positions_data = staging_positions.slice(..).get_mapped_range();
        let velocities_data = staging_velocities.slice(..).get_mapped_range();
        let energies_data = staging_energies.slice(..).get_mapped_range();

        // Convert back to component types
        let positions: Vec<Position> = positions_data
            .chunks(8)
            .take(self.entity_count as usize)
            .map(|chunk| {
                let x_bytes = bytemuck::from_bytes::<f32>(&chunk[0..4]);
                let y_bytes = bytemuck::from_bytes::<f32>(&chunk[4..8]);
                Position { x: x_bytes[0], y: y_bytes[0] }
            })
            .collect();

        let velocities: Vec<Velocity> = velocities_data
            .chunks(8)
            .take(self.entity_count as usize)
            .map(|chunk| {
                let x_bytes = bytemuck::from_bytes::<f32>(&chunk[0..4]);
                let y_bytes = bytemuck::from_bytes::<f32>(&chunk[4..8]);
                Velocity { x: x_bytes[0], y: y_bytes[0] }
            })
            .collect();

        let energies: Vec<Energy> = energies_data
            .chunks(4)
            .take(self.entity_count as usize)
            .map(|chunk| {
                let current_bytes = bytemuck::from_bytes::<f32>(chunk);
                Energy { current: current_bytes[0], max: current_bytes[0] * 1.3 } // Approximate max
            })
            .collect();

        (positions, velocities, energies)
    }
} 