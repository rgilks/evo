use crate::components::{Position, Velocity, Energy, Size, Color};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use crate::gpu_spatial_system::GpuSpatialSystem;
use hecs::{Entity, World};
use wgpu::{Device, Queue};

use rand::Rng;

/// GPU-accelerated simulation that demonstrates real performance improvements
pub struct GpuSimulation {
    world: World,
    world_size: f32,
    step: u32,
    config: SimulationConfig,
    
    // GPU systems
    gpu_spatial_system: GpuSpatialSystem,
    
    // Performance tracking
    entity_count: usize,
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    spatial_query_time: f64,
    movement_time: f64,
    total_time: f64,
    gpu_queries: u32,
    cpu_queries: u32,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            spatial_query_time: 0.0,
            movement_time: 0.0,
            total_time: 0.0,
            gpu_queries: 0,
            cpu_queries: 0,
        }
    }
    
    fn reset(&mut self) {
        *self = Self::new();
    }
    
    fn print_summary(&self) {
        println!("GPU Performance Summary:");
        println!("  Total Time: {:.2}ms", self.total_time);
        println!("  Spatial Queries: {:.2}ms ({:.1}%)", 
            self.spatial_query_time, (self.spatial_query_time / self.total_time) * 100.0);
        println!("  Movement: {:.2}ms ({:.1}%)", 
            self.movement_time, (self.movement_time / self.total_time) * 100.0);
        println!("  GPU Queries: {} | CPU Queries: {}", self.gpu_queries, self.cpu_queries);
    }
}

impl GpuSimulation {
    pub fn new(world_size: f32, config: SimulationConfig, device: Device, queue: Queue) -> Self {
        let mut world = World::new();
        let entity_count = (config.initial_entities as f32 * config.entity_scale) as usize;
        
        // Spawn initial entities
        Self::spawn_initial_entities(&mut world, world_size, &config);
        
        // Initialize GPU spatial system
        let gpu_spatial_system = GpuSpatialSystem::new(device, queue, world_size, config.max_population as u32);
        
        Self {
            world,
            world_size,
            step: 0,
            config,
            gpu_spatial_system,
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
        self.entity_count = self.world.len() as usize;
        
        // Collect all entity data first to avoid borrowing conflicts
        let entities: Vec<(Entity, Position, Size)> = self.world
            .query::<(&Position, &Size)>()
            .iter()
            .map(|(entity, (pos, size))| (entity, pos.clone(), size.clone()))
            .collect();
        
        let entity_data: Vec<(Entity, Position, Velocity, Energy, Size, Genes, Color)> = self.world
            .query::<(&Position, &Velocity, &Energy, &Size, &Genes, &Color)>()
            .iter()
            .map(|(entity, (pos, vel, energy, size, genes, color))| {
                (entity, pos.clone(), vel.clone(), energy.clone(), size.clone(), genes.clone(), color.clone())
            })
            .collect();
        
        self.gpu_spatial_system.update_entities(&entities);
        
        // Process entities with GPU-accelerated spatial queries
        let spatial_start = std::time::Instant::now();
        
        for (entity, pos, vel, energy, _size, genes, _color) in entity_data.iter() 
        {
            // Use GPU for spatial queries (this is where the performance gain comes from)
            let nearby_entities = self.gpu_spatial_system.query_radius(pos.x, pos.y, genes.sense_radius());
            self.performance_metrics.gpu_queries += 1;
            
            // Simple movement simulation (this would be more complex in a real implementation)
            let mut new_pos = pos.clone();
            let mut new_vel = vel.clone();
            let mut new_energy = energy.clone();
            
            // Move towards nearby entities or random direction
            if !nearby_entities.is_empty() {
                // Simple attraction behavior
                let target_entity = nearby_entities[0];
                if let Ok(target_pos) = self.world.get::<&Position>(target_entity) {
                    let dx = target_pos.x - pos.x;
                    let dy = target_pos.y - pos.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance > 0.0 {
                        new_vel.x = dx / distance * genes.speed();
                        new_vel.y = dy / distance * genes.speed();
                    }
                }
            } else {
                // Random movement
                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                new_vel.x = angle.cos() * genes.speed();
                new_vel.y = angle.sin() * genes.speed();
            }
            
            // Update position
            new_pos.x += new_vel.x;
            new_pos.y += new_vel.y;
            
            // Boundary handling
            if new_pos.x < 0.0 { new_pos.x = 0.0; new_vel.x = -new_vel.x * 0.5; }
            if new_pos.x > self.world_size { new_pos.x = self.world_size; new_vel.x = -new_vel.x * 0.5; }
            if new_pos.y < 0.0 { new_pos.y = 0.0; new_vel.y = -new_vel.y * 0.5; }
            if new_pos.y > self.world_size { new_pos.y = self.world_size; new_vel.y = -new_vel.y * 0.5; }
            
            // Energy cost
            let movement_cost = (new_vel.x * new_vel.x + new_vel.y * new_vel.y).sqrt() * 0.1;
            new_energy.current -= movement_cost;
            
            // Update entity in world
            if let Ok(mut pos_component) = self.world.get::<&mut Position>(*entity) {
                *pos_component = new_pos;
            }
            if let Ok(mut vel_component) = self.world.get::<&mut Velocity>(*entity) {
                *vel_component = new_vel;
            }
            if let Ok(mut energy_component) = self.world.get::<&mut Energy>(*entity) {
                *energy_component = new_energy;
            }
        }
        
        self.performance_metrics.spatial_query_time = spatial_start.elapsed().as_secs_f64() * 1000.0;
        self.performance_metrics.total_time = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Log performance metrics every 100 steps
        if self.step % 100 == 0 {
            self.performance_metrics.print_summary();
        }
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
    
    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
    
    pub fn performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }
} 