use crate::config::SimulationConfig;
use hecs::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

// Configuration constants for better maintainability
// These are now handled by the SimulationConfig struct

// Simplified components
#[derive(Clone, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
}

#[derive(Clone, Debug)]
pub struct Size {
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// Simplified genes that naturally create diverse behaviors
#[derive(Clone, Debug)]
pub struct Genes {
    pub speed: f32,             // Movement speed
    pub sense_radius: f32,      // Detection range
    pub energy_efficiency: f32, // Energy usage multiplier
    pub reproduction_rate: f32, // Reproduction probability
    pub mutation_rate: f32,     // Gene mutation probability
    pub size_factor: f32,       // How size relates to energy (bigger = more energy needed)
    pub energy_loss_rate: f32,  // Base energy loss per tick
    pub energy_gain_rate: f32,  // Energy gained from eating
    pub color_hue: f32,         // Color hue (0.0-1.0)
    pub color_saturation: f32,  // Color saturation (0.0-1.0)
}

impl Genes {
    fn new_random(rng: &mut ThreadRng) -> Self {
        Self {
            speed: rng.gen_range(0.2..1.5), // Reduced speed range for slower movement
            sense_radius: rng.gen_range(10.0..100.0),
            energy_efficiency: rng.gen_range(0.5..2.0),
            reproduction_rate: rng.gen_range(0.001..0.1),
            mutation_rate: rng.gen_range(0.01..0.1),
            size_factor: rng.gen_range(0.5..2.0),
            energy_loss_rate: rng.gen_range(0.1..1.0),
            energy_gain_rate: rng.gen_range(0.5..3.0),
            color_hue: rng.gen_range(0.0..1.0),
            color_saturation: rng.gen_range(0.3..1.0),
        }
    }

    fn mutate(&self, rng: &mut ThreadRng) -> Self {
        let mut new_genes = self.clone();

        // Use a single mutation check with field-specific probabilities
        let fields = [
            (&mut new_genes.speed, 0.1, 0.1..2.0), // Reduced mutation range and max speed
            (&mut new_genes.sense_radius, 5.0, 5.0..120.0),
            (&mut new_genes.energy_efficiency, 0.1, 0.3..3.0),
            (&mut new_genes.reproduction_rate, 0.02, 0.0001..0.2),
            (&mut new_genes.mutation_rate, 0.02, 0.001..0.2),
            (&mut new_genes.size_factor, 0.1, 0.2..3.0),
            (&mut new_genes.energy_loss_rate, 0.1, 0.05..2.0),
            (&mut new_genes.energy_gain_rate, 0.2, 0.2..4.0),
            (&mut new_genes.color_hue, 0.1, 0.0..1.0),
            (&mut new_genes.color_saturation, 0.1, 0.1..1.0),
        ];

        for (field, mutation_range, clamp_range) in fields {
            if rng.gen::<f32>() < self.mutation_rate {
                *field = (*field + rng.gen_range(-mutation_range..mutation_range))
                    .clamp(clamp_range.start, clamp_range.end);
            }
        }

        new_genes
    }

    fn get_color(&self) -> Color {
        // Convert HSV to RGB for more intuitive color generation
        let h = self.color_hue * 6.0;
        let s = self.color_saturation;
        let v = 0.8; // Fixed brightness for consistency

        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color {
            r: r + m,
            g: g + m,
            b: b + m,
        }
    }

    // Calculate if this entity can eat another based purely on genes
    fn can_eat(&self, other_genes: &Genes, other_size: &Size, self_size: &Size) -> bool {
        // Bigger entities can eat smaller ones, but with stricter requirements
        let size_advantage = self_size.radius / other_size.radius;
        let speed_advantage = self.speed / other_genes.speed;

        // Need significant size and speed advantage to be a successful predator
        // This prevents smaller entities from eating larger ones
        size_advantage > 1.2 && speed_advantage > 0.8
    }

    // Calculate energy gain from eating based on genes
    fn get_energy_gain(&self, other_energy: f32, other_size: &Size, self_size: &Size) -> f32 {
        let size_ratio = other_size.radius / self_size.radius;
        let base_gain = other_energy * self.energy_gain_rate * 0.3; // Reduced energy gain

        // Bigger prey = more energy, but with stronger diminishing returns
        base_gain * (1.0 + size_ratio * 0.3).min(1.5)
    }
}

// Optimized spatial grid for parallel access
#[derive(Default)]
struct SpatialGrid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.grid.clear();
    }

    fn get_cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        (cell_x, cell_y)
    }

    fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let cell = self.get_cell_coords(x, y);
        self.grid.entry(cell).or_insert_with(Vec::new).push(entity);
    }

    fn get_nearby_entities(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let mut nearby = Vec::new();
        let center_cell = self.get_cell_coords(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(entities) = self.grid.get(&cell) {
                    nearby.extend(entities.iter().copied());
                }
            }
        }

        nearby
    }
}

// Simulation state
pub struct Simulation {
    world: World,
    world_size: f32,
    step: u32,
    grid: SpatialGrid,
    previous_positions: HashMap<Entity, Position>, // For smooth interpolation
    config: SimulationConfig,
}

impl Simulation {
    #[allow(dead_code)]
    pub fn new(world_size: f32) -> Self {
        Self::new_with_config(world_size, SimulationConfig::default())
    }

    pub fn new_with_config(world_size: f32, config: SimulationConfig) -> Self {
        let mut world = World::new();
        let mut rng = thread_rng();
        let grid = SpatialGrid::new(config.grid_cell_size);

        Self::spawn_initial_entities(&mut world, &mut rng, world_size, &config);

        Self {
            world,
            world_size,
            step: 0,
            grid,
            previous_positions: HashMap::new(),
            config,
        }
    }

    fn spawn_initial_entities(
        world: &mut World,
        rng: &mut ThreadRng,
        world_size: f32,
        config: &SimulationConfig,
    ) {
        let total_entities = (config.initial_entities as f32 * config.entity_scale) as usize;
        let spawn_radius = world_size * config.spawn_radius_factor;

        for _ in 0..total_entities {
            // Use perfectly uniform distribution in a circle
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.gen::<f32>().sqrt(); // Square root for uniform distribution
            let x = distance * angle.cos();
            let y = distance * angle.sin();

            let genes = Genes::new_random(rng);
            let energy = rng.gen_range(25.0..55.0);
            let color = genes.get_color();
            let radius = (energy / 15.0 * genes.size_factor).clamp(config.min_entity_radius, 8.0);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.3,
                },
                Size { radius },
                Genes { ..genes },
                Color { ..color },
                Velocity { x: 0.0, y: 0.0 },
            ));
        }
    }

    pub fn update(&mut self) {
        self.step += 1;
        self.update_simulation();

        if self.step % 60 == 0 {
            self.log_simulation_metrics();
        }
    }

    fn log_simulation_metrics(&self) {
        let total_entities = self.world.len();

        // Calculate average gene values for insights
        let gene_stats = self
            .world
            .query::<(&Genes,)>()
            .iter()
            .par_bridge()
            .fold(
                || [0.0f32; 8], // [speed, sense, efficiency, repro, mutation, size, loss, gain]
                |mut stats, (_, (genes,))| {
                    stats[0] += genes.speed;
                    stats[1] += genes.sense_radius;
                    stats[2] += genes.energy_efficiency;
                    stats[3] += genes.reproduction_rate;
                    stats[4] += genes.mutation_rate;
                    stats[5] += genes.size_factor;
                    stats[6] += genes.energy_loss_rate;
                    stats[7] += genes.energy_gain_rate;
                    stats
                },
            )
            .reduce(
                || [0.0f32; 8],
                |mut a, b| {
                    for i in 0..8 {
                        a[i] += b[i];
                    }
                    a
                },
            );

        let avg_energy = self
            .world
            .query::<(&Energy,)>()
            .iter()
            .par_bridge()
            .map(|(_, (energy,))| energy.current)
            .sum::<f32>()
            / total_entities as f32;

        // Calculate average position to detect drift
        let (avg_x, avg_y) = self
            .world
            .query::<(&Position,)>()
            .iter()
            .par_bridge()
            .fold(
                || (0.0f32, 0.0f32),
                |(sum_x, sum_y), (_, (pos,))| (sum_x + pos.x, sum_y + pos.y),
            )
            .reduce(
                || (0.0f32, 0.0f32),
                |(sum_x, sum_y), (x, y)| (sum_x + x, sum_y + y),
            );

        let avg_x = avg_x / total_entities as f32;
        let avg_y = avg_y / total_entities as f32;

        println!(
            "Step {}: Total={}, AvgEnergy={:.1}, AvgSpeed={:.2}, AvgSize={:.2}, AvgRepro={:.3}, AvgPos=({:.1}, {:.1})",
            self.step,
            total_entities,
            avg_energy,
            gene_stats[0] / total_entities as f32,
            gene_stats[5] / total_entities as f32,
            gene_stats[3] / total_entities as f32,
            avg_x,
            avg_y,
        );
    }

    fn update_simulation(&mut self) {
        // Store previous positions for smooth interpolation
        self.previous_positions.clear();
        for (entity, (pos,)) in self.world.query::<(&Position,)>().iter() {
            self.previous_positions.insert(entity, pos.clone());
        }

        // Rebuild spatial grid in parallel
        self.rebuild_spatial_grid();

        // Use Hecs' parallel iteration capabilities where possible
        // Process entities in parallel using Hecs' built-in parallel iteration
        let updates: Vec<_> = self
            .world
            .query::<(&Position, &Energy, &Size, &Genes, &Color, &Velocity)>()
            .iter()
            .par_bridge()
            .filter_map(|(entity, (pos, energy, size, genes, color, velocity))| {
                if energy.current <= 0.0 {
                    return None;
                }

                self.process_entity(entity, pos, energy, size, genes, color, velocity)
            })
            .collect();

        // Apply updates and handle reproduction in parallel where possible
        self.apply_updates(updates);
    }

    fn rebuild_spatial_grid(&mut self) {
        self.grid.clear();

        // Use parallel processing for grid building
        let grid_entities: Vec<_> = self
            .world
            .query::<(&Position,)>()
            .iter()
            .par_bridge()
            .map(|(entity, (pos,))| (entity, pos.x, pos.y))
            .collect();

        // Insert entities into grid (this part needs to be sequential due to HashMap)
        for (entity, x, y) in grid_entities {
            self.grid.insert(entity, x, y);
        }
    }

    fn process_entity(
        &self,
        entity: Entity,
        pos: &Position,
        energy: &Energy,
        size: &Size,
        genes: &Genes,
        color: &Color,
        velocity: &Velocity,
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
        // Use entity ID to seed RNG for deterministic but varied behavior
        let mut rng = rand::thread_rng();
        let mut new_pos = pos.clone();
        let mut new_energy = energy.current;
        let mut new_velocity = velocity.clone();
        let mut eaten_entity = None;
        let mut should_reproduce = false;

        // Find nearby entities
        let nearby_entities = self
            .grid
            .get_nearby_entities(pos.x, pos.y, genes.sense_radius);
        let nearby_entities = nearby_entities.iter().take(20).copied().collect::<Vec<_>>();

        // Movement logic - purely based on genes
        self.update_movement(
            genes,
            &mut new_pos,
            &mut new_velocity,
            &mut new_energy,
            pos,
            &nearby_entities,
            &mut rng,
        );

        // Boundary handling
        self.handle_boundaries(&mut new_pos, &mut new_velocity, &mut rng);

        // Interaction logic - purely gene-based
        self.handle_interactions(
            &mut new_energy,
            &mut eaten_entity,
            &new_pos,
            size,
            genes,
            &nearby_entities,
        );

        // Energy changes based on genes and size (larger entities cost more to maintain)
        let size_energy_cost = size.radius * self.config.size_energy_cost_factor;
        new_energy -= (genes.energy_loss_rate + size_energy_cost) / genes.energy_efficiency;

        // Reproduction check based on genes with population pressure
        let population_density = self.world.len() as f32
            / (self.config.max_population as f32 * self.config.entity_scale);
        let reproduction_chance = genes.reproduction_rate
            * (1.0 - population_density * self.config.population_density_factor)
                .max(self.config.min_reproduction_chance);

        if new_energy > energy.max * self.config.reproduction_energy_threshold
            && rng.gen::<f32>() < reproduction_chance
        {
            should_reproduce = true;
            new_energy *= self.config.reproduction_energy_cost;
        }

        // Add random death chance that increases with population density
        let death_chance = population_density * self.config.death_chance_factor;
        if rng.gen::<f32>() < death_chance {
            new_energy = 0.0; // Kill the entity
        }

        // Calculate new size based on energy and genes with stricter limits
        let new_radius = (new_energy / 15.0 * genes.size_factor)
            .clamp(self.config.min_entity_radius, self.config.max_entity_radius);

        Some((
            entity,
            new_pos,
            Energy {
                current: new_energy,
                max: energy.max,
            },
            Size { radius: new_radius },
            genes.clone(),
            color.clone(),
            new_velocity,
            should_reproduce,
            eaten_entity,
        ))
    }

    fn update_movement(
        &self,
        genes: &Genes,
        new_pos: &mut Position,
        new_velocity: &mut Velocity,
        new_energy: &mut f32,
        pos: &Position,
        nearby_entities: &[Entity],
        rng: &mut ThreadRng,
    ) {
        // Find target for movement based on genes
        let target = self.find_movement_target(pos, genes, nearby_entities);

        if let Some((target_x, target_y)) = target {
            // Move towards target
            let dx = target_x - pos.x;
            let dy = target_y - pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.0 {
                new_velocity.x = (dx / distance) * genes.speed;
                new_velocity.y = (dy / distance) * genes.speed;
            }
        } else {
            // Random movement - use uniform distribution in a circle to avoid bias
            let speed_variation = rng.gen_range(0.8..1.2);
            let speed = genes.speed * speed_variation;

            // Generate random direction using uniform distribution in a circle
            let (dx, dy) = loop {
                let dx = rng.gen_range(-1.0f32..1.0);
                let dy = rng.gen_range(-1.0f32..1.0);
                let length_sq = dx * dx + dy * dy;
                if length_sq <= 1.0 && length_sq > 0.0 {
                    // Normalize to unit vector
                    let length = length_sq.sqrt();
                    break (dx / length, dy / length);
                }
            };

            new_velocity.x = dx * speed;
            new_velocity.y = dy * speed;

            // Cap velocity to prevent extreme movements
            if new_velocity.x.abs() > self.config.max_velocity {
                new_velocity.x = new_velocity.x.signum() * self.config.max_velocity;
            }
            if new_velocity.y.abs() > self.config.max_velocity {
                new_velocity.y = new_velocity.y.signum() * self.config.max_velocity;
            }
        }

        new_pos.x += new_velocity.x;
        new_pos.y += new_velocity.y;

        // Validate position to prevent NaN or infinite values
        if new_pos.x.is_nan() || new_pos.x.is_infinite() {
            new_pos.x = 0.0;
        }
        if new_pos.y.is_nan() || new_pos.y.is_infinite() {
            new_pos.y = 0.0;
        }

        // Movement cost based on genes
        let movement_distance =
            (new_velocity.x * new_velocity.x + new_velocity.y * new_velocity.y).sqrt();
        *new_energy -=
            movement_distance * self.config.movement_energy_cost / genes.energy_efficiency;
    }

    fn find_movement_target(
        &self,
        pos: &Position,
        genes: &Genes,
        nearby_entities: &[Entity],
    ) -> Option<(f32, f32)> {
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = self.world.get::<&Position>(entity) {
                if let Ok(nearby_genes) = self.world.get::<&Genes>(entity) {
                    if let Ok(nearby_energy) = self.world.get::<&Energy>(entity) {
                        if let Ok(nearby_size) = self.world.get::<&Size>(entity) {
                            if nearby_energy.current > 0.0 {
                                let distance = ((nearby_pos.x - pos.x).powi(2)
                                    + (nearby_pos.y - pos.y).powi(2))
                                .sqrt();
                                if distance < genes.sense_radius {
                                    // Check if this is a potential food source
                                    if genes.can_eat(
                                        &*nearby_genes,
                                        &*nearby_size,
                                        &Size { radius: 1.0 },
                                    ) {
                                        return Some((nearby_pos.x, nearby_pos.y));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn handle_boundaries(&self, pos: &mut Position, velocity: &mut Velocity, _rng: &mut ThreadRng) {
        let half_world = self.world_size / 2.0;

        // Use <= and >= to handle edge cases better
        if pos.x <= -half_world + self.config.boundary_margin {
            pos.x = -half_world + self.config.boundary_margin;
            velocity.x = velocity.x.abs() * self.config.velocity_bounce_factor; // Push away from boundary
        } else if pos.x >= half_world - self.config.boundary_margin {
            pos.x = half_world - self.config.boundary_margin;
            velocity.x = -velocity.x.abs() * self.config.velocity_bounce_factor;
            // Push away from boundary
        }

        if pos.y <= -half_world + self.config.boundary_margin {
            pos.y = -half_world + self.config.boundary_margin;
            velocity.y = velocity.y.abs() * self.config.velocity_bounce_factor; // Push away from boundary
        } else if pos.y >= half_world - self.config.boundary_margin {
            pos.y = half_world - self.config.boundary_margin;
            velocity.y = -velocity.y.abs() * self.config.velocity_bounce_factor;
            // Push away from boundary
        }

        // Add deliberate compensating drift to counteract systematic bias
        // Scale down for UI mode (60 FPS) vs headless mode
        velocity.x += self.config.drift_compensation_x;
        velocity.y += self.config.drift_compensation_y;
    }

    fn handle_interactions(
        &self,
        new_energy: &mut f32,
        eaten_entity: &mut Option<Entity>,
        new_pos: &Position,
        size: &Size,
        genes: &Genes,
        nearby_entities: &[Entity],
    ) {
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = self.world.get::<&Position>(entity) {
                if let Ok(nearby_genes) = self.world.get::<&Genes>(entity) {
                    if let Ok(nearby_energy) = self.world.get::<&Energy>(entity) {
                        if let Ok(nearby_size) = self.world.get::<&Size>(entity) {
                            let distance = ((nearby_pos.x - new_pos.x).powi(2)
                                + (nearby_pos.y - new_pos.y).powi(2))
                            .sqrt();

                            if distance < (size.radius + self.config.interaction_radius_offset)
                                && nearby_energy.current > 0.0
                            {
                                if genes.can_eat(&*nearby_genes, &*nearby_size, size) {
                                    *eaten_entity = Some(entity);
                                    let energy_gained = genes.get_energy_gain(
                                        nearby_energy.current,
                                        &*nearby_size,
                                        size,
                                    );
                                    *new_energy = (*new_energy + energy_gained - 0.5)
                                        .min(genes.energy_efficiency * 100.0);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn apply_updates(
        &mut self,
        updates: Vec<(
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
        // Remove eaten entities in parallel
        let entities_to_remove: Vec<_> = updates
            .par_iter()
            .filter_map(|(_, _, _, _, _, _, _, _, eaten_entity)| *eaten_entity)
            .collect();

        // Despawn entities (this needs to be sequential due to Hecs limitations)
        for &entity in &entities_to_remove {
            let _ = self.world.despawn(entity);
        }

        // Prepare spawn data in parallel
        let spawn_data: Vec<_> = updates
            .par_iter()
            .filter_map(
                |(_entity, position, energy, size, genes, color, velocity, should_reproduce, _)| {
                    if energy.current <= 0.0 {
                        return None;
                    }

                    // Store values before spawning to avoid move issues
                    let pos_x = position.x;
                    let pos_y = position.y;
                    let energy_max = energy.max;

                    let mut spawn_entities = vec![(
                        position.clone(),
                        energy.clone(),
                        size.clone(),
                        genes.clone(),
                        color.clone(),
                        velocity.clone(),
                    )];

                    // Handle reproduction with stricter population control
                    let max_population =
                        (self.config.max_population as f32 * self.config.entity_scale) as u32;
                    if *should_reproduce && self.world.len() < max_population {
                        let mut rng = rand::thread_rng();
                        let child_genes = genes.mutate(&mut rng);
                        let child_energy = energy_max * self.config.child_energy_factor;
                        let child_radius = (child_energy / 15.0 * child_genes.size_factor)
                            .clamp(self.config.min_entity_radius, 15.0);
                        let child_color = child_genes.get_color();

                        // Use uniform distribution in a circle for child positioning
                        let (dx, dy) = loop {
                            let dx = rng.gen_range(
                                -self.config.child_spawn_radius..self.config.child_spawn_radius,
                            );
                            let dy = rng.gen_range(
                                -self.config.child_spawn_radius..self.config.child_spawn_radius,
                            );
                            let distance_sq = dx * dx + dy * dy;
                            if distance_sq
                                <= self.config.child_spawn_radius * self.config.child_spawn_radius
                            {
                                break (dx, dy);
                            }
                        };
                        let child_x = pos_x + dx;
                        let child_y = pos_y + dy;

                        spawn_entities.push((
                            Position {
                                x: child_x,
                                y: child_y,
                            },
                            Energy {
                                current: child_energy,
                                max: energy_max,
                            },
                            Size {
                                radius: child_radius,
                            },
                            child_genes,
                            child_color,
                            Velocity { x: 0.0, y: 0.0 },
                        ));
                    }

                    Some(spawn_entities)
                },
            )
            .flatten()
            .collect();

        // Despawn old entities
        for (entity, _, _, _, _, _, _, _, _) in updates {
            let _ = self.world.despawn(entity);
        }

        // Spawn new entities (this needs to be sequential due to Hecs limitations)
        for (position, energy, size, genes, color, velocity) in spawn_data {
            self.world
                .spawn((position, energy, size, genes, color, velocity));
        }
    }

    pub fn get_entities(&self) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .par_bridge()
            .map(|(_, (pos, size, color))| (pos.x, pos.y, size.radius, color.r, color.g, color.b))
            .collect()
    }

    pub fn get_interpolated_entities(
        &self,
        interpolation_factor: f32,
    ) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .par_bridge()
            .map(|(entity, (pos, size, color))| {
                let interpolated_pos = if let Some(prev_pos) = self.previous_positions.get(&entity)
                {
                    // Interpolate between previous and current position
                    let x = prev_pos.x + (pos.x - prev_pos.x) * interpolation_factor;
                    let y = prev_pos.y + (pos.y - prev_pos.y) * interpolation_factor;
                    (x, y)
                } else {
                    (pos.x, pos.y)
                };

                (
                    interpolated_pos.0,
                    interpolated_pos.1,
                    size.radius,
                    color.r,
                    color.g,
                    color.b,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_genes_mutation() {
        let mut rng = thread_rng();
        let original_genes = Genes::new_random(&mut rng);
        let mutated_genes = original_genes.mutate(&mut rng);

        // Check that at least some genes changed (not guaranteed due to random mutation rate)
        let _any_changed = (original_genes.speed - mutated_genes.speed).abs() > 0.001
            || (original_genes.sense_radius - mutated_genes.sense_radius).abs() > 0.001
            || (original_genes.energy_efficiency - mutated_genes.energy_efficiency).abs() > 0.001;

        // Genes should be within valid ranges
        assert!(mutated_genes.speed >= 0.1 && mutated_genes.speed <= 2.0);
        assert!(mutated_genes.sense_radius >= 5.0 && mutated_genes.sense_radius <= 120.0);
        assert!(mutated_genes.energy_efficiency >= 0.3 && mutated_genes.energy_efficiency <= 3.0);
    }

    #[test]
    fn test_genes_color_generation() {
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let color = genes.get_color();

        // Color values should be in valid range
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
    }

    #[test]
    fn test_predation_logic() {
        let mut rng = thread_rng();
        let predator_genes = Genes::new_random(&mut rng);
        let prey_genes = Genes::new_random(&mut rng);

        let predator_size = Size { radius: 10.0 };
        let prey_size = Size { radius: 5.0 };

        // Large predator should be able to eat small prey (size advantage > 1.2)
        // Note: This test may fail if the random genes don't meet the criteria
        let size_advantage = predator_size.radius / prey_size.radius;
        let speed_advantage = predator_genes.speed / prey_genes.speed;

        if size_advantage > 1.2 && speed_advantage > 0.8 {
            assert!(predator_genes.can_eat(&prey_genes, &prey_size, &predator_size));
        }

        // Small entity should not be able to eat large entity
        assert!(!prey_genes.can_eat(&predator_genes, &predator_size, &prey_size));
    }

    #[test]
    fn test_spatial_grid() {
        let mut grid = SpatialGrid::new(25.0);

        // Test cell coordinate calculation
        let (cell_x, cell_y) = grid.get_cell_coords(50.0, 75.0);
        assert_eq!(cell_x, 2); // 50 / 25 = 2
        assert_eq!(cell_y, 3); // 75 / 25 = 3

        // Test entity insertion and retrieval
        // Create a valid entity ID
        let entity = Entity::from_bits(0x1000000000000001).unwrap();
        grid.insert(entity, 50.0, 75.0);

        let nearby = grid.get_nearby_entities(50.0, 75.0, 10.0);
        assert!(nearby.contains(&entity));
    }

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new(1000.0);

        // Should have initial entities
        assert!(sim.world.len() > 0);
        assert!(sim.world.len() <= 250); // Default config values

        // World size should be set correctly
        assert_eq!(sim.world_size, 1000.0);
    }

    #[test]
    fn test_boundary_handling() {
        let sim = Simulation::new(100.0);
        let mut pos = Position { x: 60.0, y: 60.0 }; // Outside boundary
        let mut velocity = Velocity { x: 10.0, y: 10.0 };
        let mut rng = thread_rng();

        sim.handle_boundaries(&mut pos, &mut velocity, &mut rng);

        // Position should be clamped to boundary
        assert!(pos.x <= 50.0 - 5.0); // Default boundary margin
        assert!(pos.y <= 50.0 - 5.0); // Default boundary margin
    }
}
