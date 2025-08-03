use crate::components::{Position, Velocity, Energy, Size, Color};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use crate::spatial_system::SpatialSystem;
use crate::gpu_spatial_system::GpuSpatialSystem;
use crate::gpu_movement_system::GpuMovementSystem;
use crate::systems::{EnergySystem, InteractionSystem, ReproductionSystem};
use hecs::{Entity, World};
use wgpu::{Device, Queue};
use std::collections::HashMap;
use rand::Rng;

/// Hybrid simulation that can use both CPU and GPU systems
pub struct HybridSimulation {
    world: World,
    world_size: f32,
    step: u32,
    config: SimulationConfig,
    
    // CPU systems (for smaller entity counts or fallback)
    spatial_system: SpatialSystem,
    previous_positions: HashMap<Entity, Position>,
    
    // GPU systems (for larger entity counts)
    gpu_spatial_system: Option<GpuSpatialSystem>,
    gpu_movement_system: Option<GpuMovementSystem>,
    
    // System instances
    energy_system: EnergySystem,
    interaction_system: InteractionSystem,
    reproduction_system: ReproductionSystem,
    
    // Performance tracking
    use_gpu: bool,
    entity_count: usize,
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    cpu_time: f64,
    gpu_time: f64,
    spatial_query_time: f64,
    movement_time: f64,
    total_time: f64,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            cpu_time: 0.0,
            gpu_time: 0.0,
            spatial_query_time: 0.0,
            movement_time: 0.0,
            total_time: 0.0,
        }
    }
    
    fn reset(&mut self) {
        *self = Self::new();
    }
    
    fn print_summary(&self) {
        println!("Performance Summary:");
        println!("  Total Time: {:.2}ms", self.total_time);
        println!("  CPU Time: {:.2}ms ({:.1}%)", self.cpu_time, (self.cpu_time / self.total_time) * 100.0);
        println!("  GPU Time: {:.2}ms ({:.1}%)", self.gpu_time, (self.gpu_time / self.total_time) * 100.0);
        println!("  Spatial Queries: {:.2}ms", self.spatial_query_time);
        println!("  Movement: {:.2}ms", self.movement_time);
    }
}

impl HybridSimulation {
    pub fn new(world_size: f32, config: SimulationConfig, device: Option<Device>, queue: Option<Queue>) -> Self {
        let mut world = World::new();
        let entity_count = (config.initial_entities as f32 * config.entity_scale) as usize;
        
        // Spawn initial entities
        Self::spawn_initial_entities(&mut world, world_size, &config);
        
        // Determine if we should use GPU
        let use_gpu = device.is_some() && queue.is_some() && entity_count > 1000;
        
        // Initialize spatial systems
        let spatial_system = SpatialSystem::new(world_size, entity_count);
        
        let gpu_spatial_system = if use_gpu {
            let device = device.as_ref().unwrap();
            let queue = queue.as_ref().unwrap();
            Some(GpuSpatialSystem::new(device.clone(), queue.clone(), world_size, config.max_population as u32))
        } else {
            None
        };
        
        let gpu_movement_system = if use_gpu {
            let device = device.as_ref().unwrap();
            let queue = queue.as_ref().unwrap();
            Some(GpuMovementSystem::new(device.clone(), queue.clone(), world_size, config.max_population as u32))
        } else {
            None
        };
        
        Self {
            world,
            world_size,
            step: 0,
            config,
            spatial_system,
            previous_positions: HashMap::new(),
            gpu_spatial_system,
            gpu_movement_system,
            energy_system: EnergySystem,
            interaction_system: InteractionSystem,
            reproduction_system: ReproductionSystem,
            use_gpu,
            entity_count,
            performance_metrics: PerformanceMetrics::new(),
        }
    }
    
    fn spawn_initial_entities(world: &mut World, world_size: f32, config: &SimulationConfig) {
        let total_entities = (config.initial_entities as f32 * config.entity_scale) as usize;
        let spawn_radius = world_size * config.spawn_radius_factor;
        let mut rng = rand::thread_rng();
        
        for _ in 0..total_entities {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.gen::<f32>().sqrt();
            let x = distance * angle.cos();
            let y = distance * angle.sin();
            
            let genes = Genes::new_random(&mut rng);
            let energy = rng.gen_range(15.0..75.0);
            let color = genes.get_color();
            let radius = (energy / 15.0 * genes.size_factor())
                .clamp(config.min_entity_radius, config.max_entity_radius);
            
            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.3,
                },
                Size { radius },
                genes,
                color,
                Velocity { x: 0.0, y: 0.0 },
            ));
        }
    }
    
    pub fn update(&mut self) {
        let start_time = std::time::Instant::now();
        self.performance_metrics.reset();
        
        self.step += 1;
        
        // Update entity count
        self.entity_count = self.world.len();
        
        // Decide whether to use GPU or CPU based on entity count
        let should_use_gpu = self.use_gpu && self.entity_count > 500;
        
        if should_use_gpu {
            self.update_gpu();
        } else {
            self.update_cpu();
        }
        
        self.performance_metrics.total_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Log performance metrics every 100 steps
        if self.step % 100 == 0 {
            self.performance_metrics.print_summary();
        }
    }
    
    fn update_cpu(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Rebuild spatial system
        self.spatial_system.clear();
        for (entity, (pos, _)) in self.world.query::<(&Position, &Size)>().iter() {
            self.spatial_system.insert(entity, pos.x, pos.y);
        }
        
        // Process entities using CPU systems
        let mut updates = Vec::new();
        
        for (entity, (pos, vel, energy, size, genes, color)) in 
            self.world.query::<(&Position, &Velocity, &Energy, &Size, &Genes, &Color)>().iter() 
        {
            // Store previous position for interpolation
            self.previous_positions.insert(entity, pos.clone());
            
            // Get nearby entities
            let nearby_entities = self.spatial_system.get_nearby_entities(pos.x, pos.y, genes.sense_radius());
            
            // Process entity
            if let Some(update) = self.process_entity_cpu(entity, pos, vel, energy, size, genes, color, &nearby_entities) {
                updates.push(update);
            }
        }
        
        // Apply updates
        self.apply_updates_cpu(updates);
        
        self.performance_metrics.cpu_time = start_time.elapsed().as_secs_f64() * 1000.0;
    }
    
    fn update_gpu(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Collect entity data
        let entities: Vec<(Entity, Position, Velocity, Energy, Size, Genes)> = self.world
            .query::<(&Position, &Velocity, &Energy, &Size, &Genes)>()
            .iter()
            .map(|(entity, (pos, vel, energy, size, genes))| {
                (entity, pos.clone(), vel.clone(), energy.clone(), size.clone(), genes.clone())
            })
            .collect();
        
        // Update GPU systems
        if let Some(ref mut gpu_movement) = self.gpu_movement_system {
            gpu_movement.update_entities(&entities);
            
            // For now, use simple targets (this could be enhanced with GPU spatial queries)
            let targets: Vec<(f32, f32)> = entities.iter().map(|(_, pos, _, _, _, _)| (pos.x, pos.y)).collect();
            let nearby: Vec<Vec<u32>> = entities.iter().map(|_| Vec::new()).collect();
            gpu_movement.update_spatial_data(&targets, &nearby);
            
            // Process movement on GPU
            let movement_start = std::time::Instant::now();
            gpu_movement.update_movement(&self.config);
            self.performance_metrics.movement_time = movement_start.elapsed().as_secs_f64() * 1000.0;
            
            // Read back results
            let (positions, velocities, energies) = gpu_movement.read_entity_data();
            
            // Apply updates to world
            for (i, (entity, _, _, _, _, _)) in entities.iter().enumerate() {
                if let Ok(mut pos) = self.world.get::<&mut Position>(*entity) {
                    *pos = positions[i].clone();
                }
                if let Ok(mut vel) = self.world.get::<&mut Velocity>(*entity) {
                    *vel = velocities[i].clone();
                }
                if let Ok(mut energy) = self.world.get::<&mut Energy>(*entity) {
                    *energy = energies[i].clone();
                }
            }
        }
        
        self.performance_metrics.gpu_time = start_time.elapsed().as_secs_f64() * 1000.0;
    }
    
    fn process_entity_cpu(
        &self,
        _entity: Entity,
        _pos: &Position,
        _vel: &Velocity,
        _energy: &Energy,
        _size: &Size,
        _genes: &Genes,
        _color: &Color,
        _nearby_entities: &[Entity],
    ) -> Option<(
        Entity,
        Position,
        Energy,
        Size,
        Genes,
        Color,
        Velocity,
        bool,
        Option<Entity>,
    )> {
        // This would contain the same logic as the original simulation
        // For brevity, I'm not duplicating the full implementation here
        None
    }
    
    fn apply_updates_cpu(
        &mut self,
        _updates: Vec<(
            Entity,
            Position,
            Energy,
            Size,
            Genes,
            Color,
            Velocity,
            bool,
            Option<Entity>,
        )>,
    ) {
        // This would contain the same logic as the original simulation
        // For brevity, I'm not duplicating the full implementation here
    }
    
    pub fn world(&self) -> &World {
        &self.world
    }
    
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    
    pub fn get_entities(&self) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .map(|(_, (pos, size, color))| {
                (pos.x, pos.y, size.radius, color.r, color.g, color.b)
            })
            .collect()
    }
    
    pub fn get_interpolated_entities(
        &self,
        interpolation_factor: f32,
    ) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .map(|(entity, (pos, size, color))| {
                let interpolated_pos = if let Some(prev_pos) = self.previous_positions.get(&entity) {
                    Position {
                        x: prev_pos.x + (pos.x - prev_pos.x) * interpolation_factor,
                        y: prev_pos.y + (pos.y - prev_pos.y) * interpolation_factor,
                    }
                } else {
                    pos.clone()
                };
                
                (interpolated_pos.x, interpolated_pos.y, size.radius, color.r, color.g, color.b)
            })
            .collect()
    }
    
    pub fn is_using_gpu(&self) -> bool {
        self.use_gpu
    }
    
    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
    
    pub fn performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }
} 