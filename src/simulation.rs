use hecs::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

// Global scale parameter to control entity counts
const ENTITY_SCALE: f32 = 0.5;

// Components
#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
}

#[derive(Clone)]
pub struct Size {
    pub radius: f32,
}

#[derive(Clone)]
pub struct Genes {
    pub speed: f32,                  // Movement speed
    pub sense_radius: f32,           // How far they can sense food/predators
    pub energy_efficiency: f32,      // How efficiently they use energy
    pub reproduction_threshold: f32, // Energy level required to reproduce
    pub mutation_rate: f32,          // How likely genes are to mutate
    pub aggressiveness: f32, // How aggressive the entity is (0.0 = passive, 1.0 = very aggressive)
    pub size_factor: f32,    // How size relates to energy
    pub energy_loss_rate: f32, // How quickly they lose energy
    pub energy_gain_rate: f32, // How much energy they gain from eating
    pub movement_cost: f32,  // Energy cost per unit of movement
    pub eating_cost: f32,    // Energy cost for eating
    pub reproduction_rate: f32, // How likely they are to reproduce
    pub color_r: f32,        // Red color component (0.0-1.0)
    pub color_g: f32,        // Green color component (0.0-1.0)
    pub color_b: f32,        // Blue color component (0.0-1.0)
}

impl Genes {
    fn new_random(rng: &mut ThreadRng) -> Self {
        Self {
            speed: rng.gen_range(0.1..3.0),
            sense_radius: rng.gen_range(20.0..100.0),
            energy_efficiency: rng.gen_range(0.5..1.5),
            reproduction_threshold: rng.gen_range(0.6..0.9),
            mutation_rate: rng.gen_range(0.01..0.1),
            aggressiveness: rng.gen_range(0.0..1.0),
            size_factor: rng.gen_range(0.5..2.0),
            energy_loss_rate: rng.gen_range(0.1..1.0),
            energy_gain_rate: rng.gen_range(0.5..3.0),
            movement_cost: rng.gen_range(0.1..0.5),
            eating_cost: rng.gen_range(0.2..1.0),
            reproduction_rate: rng.gen_range(0.01..0.1),
            color_r: rng.gen_range(0.0..1.0),
            color_g: rng.gen_range(0.0..1.0),
            color_b: rng.gen_range(0.0..1.0),
        }
    }

    fn mutate(&self, rng: &mut ThreadRng) -> Self {
        let mut new_genes = self.clone();

        // Apply mutations based on mutation rate
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.speed = (self.speed + rng.gen_range(-0.2..0.2)).clamp(0.1, 4.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.sense_radius =
                (self.sense_radius + rng.gen_range(-5.0..5.0)).clamp(5.0, 120.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.energy_efficiency =
                (self.energy_efficiency + rng.gen_range(-0.1..0.1)).clamp(0.1, 3.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.reproduction_threshold =
                (self.reproduction_threshold + rng.gen_range(-0.1..0.1)).clamp(0.3, 0.95);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.mutation_rate =
                (self.mutation_rate + rng.gen_range(-0.02..0.02)).clamp(0.001, 0.2);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.aggressiveness =
                (self.aggressiveness + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.size_factor = (self.size_factor + rng.gen_range(-0.2..0.2)).clamp(0.2, 3.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.energy_loss_rate =
                (self.energy_loss_rate + rng.gen_range(-0.1..0.1)).clamp(0.05, 2.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.energy_gain_rate =
                (self.energy_gain_rate + rng.gen_range(-0.2..0.2)).clamp(0.2, 4.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.movement_cost =
                (self.movement_cost + rng.gen_range(-0.1..0.1)).clamp(0.05, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.eating_cost = (self.eating_cost + rng.gen_range(-0.2..0.2)).clamp(0.1, 2.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.reproduction_rate =
                (self.reproduction_rate + rng.gen_range(-0.02..0.02)).clamp(0.001, 0.2);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.color_r = (self.color_r + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.color_g = (self.color_g + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.color_b = (self.color_b + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }

        new_genes
    }

    // Calculate color based on color genes with minimal influence from aggressiveness
    fn get_color(&self) -> (f32, f32, f32) {
        // Base color from color genes
        let mut red = self.color_r;
        let mut green = self.color_g;
        let mut blue = self.color_b;

        // Add very minimal influence from aggressiveness (redder = more aggressive)
        let aggressiveness_influence = 0.05; // Further reduced
        red = (red + self.aggressiveness * aggressiveness_influence).clamp(0.0, 1.0);
        green =
            (green + (1.0 - self.aggressiveness) * aggressiveness_influence * 0.2).clamp(0.0, 1.0);
        blue =
            (blue + (1.0 - self.aggressiveness) * aggressiveness_influence * 0.1).clamp(0.0, 1.0);

        (red, green, blue)
    }

    // Determine behavior type based on genes
    fn get_behavior_type(&self) -> BehaviorType {
        if self.aggressiveness < 0.3 {
            BehaviorType::Passive
        } else if self.aggressiveness < 0.7 {
            BehaviorType::Neutral
        } else {
            BehaviorType::Aggressive
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum BehaviorType {
    Passive,    // More like resources - slow, low energy loss, high reproduction
    Neutral,    // Balanced behavior
    Aggressive, // Fast, high energy loss, low reproduction rate
}

impl BehaviorType {
    fn can_eat(&self, other: &BehaviorType) -> bool {
        match (self, other) {
            (BehaviorType::Aggressive, BehaviorType::Neutral) => true,
            (BehaviorType::Aggressive, BehaviorType::Passive) => true,
            (BehaviorType::Neutral, BehaviorType::Passive) => true,
            _ => false,
        }
    }

    fn get_energy_gain_multiplier(&self, other: &BehaviorType) -> f32 {
        match (self, other) {
            (BehaviorType::Aggressive, BehaviorType::Neutral) => 2.0,
            (BehaviorType::Aggressive, BehaviorType::Passive) => 3.0,
            (BehaviorType::Neutral, BehaviorType::Passive) => 2.0,
            _ => 1.0,
        }
    }
}

#[derive(Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    fn from_genes(genes: &Genes) -> Self {
        let (r, g, b) = genes.get_color();
        Self { r, g, b }
    }
}

// Spatial Grid System for efficient neighbor finding
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
}

impl Simulation {
    pub fn new(world_size: f32) -> Self {
        let mut world = World::new();
        let mut rng = thread_rng();
        let grid = SpatialGrid::new(25.0);

        // Spawn initial entities
        Self::spawn_initial_entities(&mut world, &mut rng, world_size);

        Self {
            world,
            world_size,
            step: 0,
            grid,
        }
    }

    fn spawn_initial_entities(world: &mut World, rng: &mut ThreadRng, world_size: f32) {
        let total_entities = (1000.0 * ENTITY_SCALE) as usize;
        let spawn_radius = world_size * 0.3;

        // Spawn a diverse population with varied genes
        for _ in 0..total_entities {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);
            let x = distance * angle.cos() + rng.gen_range(-10.0..10.0);
            let y = distance * angle.sin() + rng.gen_range(-10.0..10.0);

            let mut genes = Genes::new_random(rng);

            // Create more diverse initial population
            // Some entities will be more passive (like resources), others more aggressive
            let entity_type_roll = rng.gen::<f32>();

            if entity_type_roll < 0.2 {
                // Passive entities (like resources)
                genes.aggressiveness = rng.gen_range(0.0..0.3);
                genes.speed = rng.gen_range(0.1..0.8);
                genes.energy_loss_rate = rng.gen_range(0.05..0.2);
                genes.energy_gain_rate = rng.gen_range(0.5..1.5);
                genes.reproduction_threshold = rng.gen_range(0.7..0.9);
                genes.reproduction_rate = rng.gen_range(0.001..0.01);
            } else if entity_type_roll < 0.7 {
                // Neutral entities (like herbivores)
                genes.aggressiveness = rng.gen_range(0.3..0.7);
                genes.speed = rng.gen_range(1.0..2.5);
                genes.energy_loss_rate = rng.gen_range(0.2..0.8);
                genes.energy_gain_rate = rng.gen_range(1.5..3.0);
                genes.reproduction_threshold = rng.gen_range(0.6..0.8);
                genes.reproduction_rate = rng.gen_range(0.02..0.08);
            } else {
                // Aggressive entities (like predators)
                genes.aggressiveness = rng.gen_range(0.7..1.0);
                genes.speed = rng.gen_range(1.5..3.0);
                genes.energy_loss_rate = rng.gen_range(0.8..2.0);
                genes.energy_gain_rate = rng.gen_range(2.0..4.0);
                genes.reproduction_threshold = rng.gen_range(0.8..0.95);
                genes.reproduction_rate = rng.gen_range(0.005..0.02);
            }

            // Add more color variation
            genes.color_r = rng.gen_range(0.0..1.0);
            genes.color_g = rng.gen_range(0.0..1.0);
            genes.color_b = rng.gen_range(0.0..1.0);

            let energy = rng.gen_range(25.0..55.0);
            let behavior_type = genes.get_behavior_type();
            let color = Color::from_genes(&genes);
            let radius = (energy / 10.0 * genes.size_factor).max(2.0);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.3,
                },
                Size { radius },
                Genes { ..genes },
                behavior_type,
                Color { ..color },
                Velocity { x: 0.0, y: 0.0 },
            ));
        }
    }

    pub fn update(&mut self) {
        self.step += 1;

        // Update simulation
        self.update_simulation();

        // Log simulation metrics every 60 frames
        if self.step % 60 == 0 {
            self.log_simulation_metrics();
        }

        // Redistribution removed to avoid ECS complexity
    }

    fn log_simulation_metrics(&self) {
        let mut passive_count = 0;
        let mut neutral_count = 0;
        let mut aggressive_count = 0;

        for (_, (behavior_type,)) in self.world.query::<(&BehaviorType,)>().iter() {
            if *behavior_type == BehaviorType::Passive {
                passive_count += 1;
            } else if *behavior_type == BehaviorType::Neutral {
                neutral_count += 1;
            } else if *behavior_type == BehaviorType::Aggressive {
                aggressive_count += 1;
            }
        }

        let total_entities = self.world.len();
        let avg_energy = self
            .world
            .query::<(&Energy,)>()
            .iter()
            .map(|(_, (energy,))| energy.current)
            .sum::<f32>()
            / total_entities as f32;

        println!(
            "Step {}: Total={}, Passive={}, Neutral={}, Aggressive={}, AvgEnergy={:.1}",
            self.step, total_entities, passive_count, neutral_count, aggressive_count, avg_energy
        );
    }

    // Redistribution method removed to avoid ECS complexity

    fn update_simulation(&mut self) {
        // Clear and rebuild spatial grid
        self.grid.clear();
        let grid_entities: Vec<_> = self
            .world
            .query::<(&Position,)>()
            .iter()
            .map(|(entity, (pos,))| (entity, pos.x, pos.y))
            .collect();

        for (entity, x, y) in grid_entities {
            self.grid.insert(entity, x, y);
        }

        // Collect all entity data for parallel processing
        let entity_data: Vec<_> = self
            .world
            .query::<(
                &Position,
                &Energy,
                &Size,
                &Genes,
                &BehaviorType,
                &Color,
                &Velocity,
            )>()
            .iter()
            .map(
                |(entity, (pos, energy, size, genes, behavior_type, color, velocity))| {
                    (
                        entity,
                        pos.x,
                        pos.y,
                        energy.current,
                        energy.max,
                        size.radius,
                        genes.clone(),
                        behavior_type.clone(),
                        color.clone(),
                        velocity.x,
                        velocity.y,
                    )
                },
            )
            .collect();

        // Process entities in parallel
        let chunk_size = (entity_data.len() / rayon::current_num_threads()).max(1);
        let updates: Vec<_> = entity_data
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                chunk
                    .iter()
                    .filter_map(
                        |(
                            entity,
                            x,
                            y,
                            energy,
                            max_energy,
                            radius,
                            genes,
                            behavior_type,
                            color,
                            vel_x,
                            vel_y,
                        )| {
                            if *energy <= 0.0 {
                                return None; // Entity is dead
                            }

                            let mut rng = rand::thread_rng();
                            let mut new_x = *x;
                            let mut new_y = *y;
                            let mut new_energy = *energy;
                            let mut new_vel_x = *vel_x;
                            let mut new_vel_y = *vel_y;
                            let mut eaten_entity = None;
                            let mut should_reproduce = false;

                            // Find nearby entities using spatial grid
                            let nearby_entities =
                                self.grid.get_nearby_entities(*x, *y, genes.sense_radius);
                            let max_nearby_to_check = 20;
                            let nearby_entities_to_check =
                                if nearby_entities.len() > max_nearby_to_check {
                                    nearby_entities
                                        .iter()
                                        .take(max_nearby_to_check)
                                        .copied()
                                        .collect::<Vec<_>>()
                                } else {
                                    nearby_entities
                                };

                            // Movement based on genes and behavior type
                            if *behavior_type == BehaviorType::Passive {
                                // Passive entities move very slowly
                                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                let speed = genes.speed * 0.1;
                                new_x += angle.cos() * speed;
                                new_y += angle.sin() * speed;
                            } else {
                                // Other entities move towards potential food
                                let mut target_x = *x;
                                let mut target_y = *y;
                                let mut found_target = false;

                                for nearby_entity in &nearby_entities_to_check {
                                    if let Ok(nearby_pos) =
                                        self.world.get::<&Position>(*nearby_entity)
                                    {
                                        if let Ok(nearby_behavior_type) =
                                            self.world.get::<&BehaviorType>(*nearby_entity)
                                        {
                                            if let Ok(nearby_energy) =
                                                self.world.get::<&Energy>(*nearby_entity)
                                            {
                                                if nearby_energy.current > 0.0 {
                                                    let distance = ((nearby_pos.x - *x).powi(2)
                                                        + (nearby_pos.y - *y).powi(2))
                                                    .sqrt();

                                                    // Check if this is a valid target based on behavior
                                                    let is_valid_target = behavior_type
                                                        .can_eat(&*nearby_behavior_type);

                                                    if is_valid_target
                                                        && distance < genes.sense_radius
                                                        && !found_target
                                                    {
                                                        target_x = nearby_pos.x;
                                                        target_y = nearby_pos.y;
                                                        found_target = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Move towards target or randomly
                                if found_target {
                                    let dx = target_x - *x;
                                    let dy = target_y - *y;
                                    let distance = (dx * dx + dy * dy).sqrt();
                                    if distance > 0.0 {
                                        new_vel_x = (dx / distance) * genes.speed;
                                        new_vel_y = (dy / distance) * genes.speed;
                                    }
                                } else {
                                    // Pure random movement - no velocity persistence to eliminate bias
                                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                    let speed_variation = rng.gen_range(0.8..1.2);
                                    new_vel_x = angle.cos() * genes.speed * speed_variation;
                                    new_vel_y = angle.sin() * genes.speed * speed_variation;
                                }

                                new_x += new_vel_x;
                                new_y += new_vel_y;

                                // Calculate movement cost based on distance moved and genes
                                let movement_distance =
                                    (new_vel_x * new_vel_x + new_vel_y * new_vel_y).sqrt();
                                let movement_energy_cost = movement_distance * genes.movement_cost
                                    / genes.energy_efficiency;
                                new_energy -= movement_energy_cost;
                            }

                            // Keep within bounds with uniform reflection
                            let half_world = self.world_size / 2.0;

                            // Check X boundaries
                            if new_x < -half_world {
                                new_x = -half_world;
                                new_vel_x = -new_vel_x * 0.9; // Reflect with minimal energy loss
                            } else if new_x > half_world {
                                new_x = half_world;
                                new_vel_x = -new_vel_x * 0.9; // Reflect with minimal energy loss
                            }

                            // Check Y boundaries
                            if new_y < -half_world {
                                new_y = -half_world;
                                new_vel_y = -new_vel_y * 0.9; // Reflect with minimal energy loss
                            } else if new_y > half_world {
                                new_y = half_world;
                                new_vel_y = -new_vel_y * 0.9; // Reflect with minimal energy loss
                            }

                            // Check for interactions (eating)
                            for nearby_entity in &nearby_entities_to_check {
                                if let Ok(nearby_pos) = self.world.get::<&Position>(*nearby_entity)
                                {
                                    if let Ok(nearby_behavior_type) =
                                        self.world.get::<&BehaviorType>(*nearby_entity)
                                    {
                                        if let Ok(nearby_energy) =
                                            self.world.get::<&Energy>(*nearby_entity)
                                        {
                                            let distance = ((nearby_pos.x - new_x).powi(2)
                                                + (nearby_pos.y - new_y).powi(2))
                                            .sqrt();

                                            if distance < (*radius + 15.0) // Larger interaction distance
                                                && nearby_energy.current > 0.0
                                            {
                                                let can_eat =
                                                    behavior_type.can_eat(&*nearby_behavior_type);

                                                if can_eat {
                                                    eaten_entity = Some(*nearby_entity);
                                                    let energy_gain_multiplier = behavior_type
                                                        .get_energy_gain_multiplier(
                                                            &*nearby_behavior_type,
                                                        );
                                                    let energy_gained = nearby_energy.current
                                                        * genes.energy_gain_rate
                                                        * energy_gain_multiplier;
                                                    // Apply eating cost based on genes
                                                    let eating_energy_cost =
                                                        genes.eating_cost / genes.energy_efficiency;
                                                    new_energy = (new_energy + energy_gained
                                                        - eating_energy_cost)
                                                        .min(*max_energy);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Energy changes based on genes and entity type
                            if *behavior_type == BehaviorType::Passive {
                                // Resources lose energy slowly over time (no longer gain energy)
                                new_energy -= genes.energy_loss_rate;
                            } else {
                                // Other entities lose energy over time
                                new_energy -= genes.energy_loss_rate;
                            }

                            // Check for reproduction
                            if new_energy > *max_energy * genes.reproduction_threshold {
                                let reproduction_chance = genes.reproduction_rate;
                                if rng.gen::<f32>() < reproduction_chance {
                                    should_reproduce = true;
                                    new_energy *= 0.6; // Parent loses energy
                                }
                            }

                            // Calculate new size based on energy and genes
                            let new_radius = (new_energy / 10.0 * genes.size_factor).max(1.0);

                            Some((
                                *entity,
                                new_x,
                                new_y,
                                new_energy,
                                *max_energy,
                                new_radius,
                                genes.clone(),
                                behavior_type.clone(),
                                color.clone(),
                                new_vel_x,
                                new_vel_y,
                                should_reproduce,
                                eaten_entity,
                            ))
                        },
                    )
                    .collect::<Vec<_>>()
            })
            .collect();

        // Remove eaten entities
        let mut entities_to_remove: Vec<Entity> = Vec::new();
        for (_, _, _, _, _, _, _, _, _, _, _, _, eaten_entity) in &updates {
            if let Some(eaten) = eaten_entity {
                entities_to_remove.push(*eaten);
            }
        }

        for entity in entities_to_remove {
            let _ = self.world.despawn(entity);
        }

        // Apply updates and handle reproduction
        for (
            entity,
            x,
            y,
            energy,
            max_energy,
            radius,
            genes,
            behavior_type,
            color,
            vel_x,
            vel_y,
            should_reproduce,
            _,
        ) in updates
        {
            let _ = self.world.despawn(entity);

            if energy > 0.0 {
                // Spawn updated entity
                self.world.spawn((
                    Position { x, y },
                    Energy {
                        current: energy,
                        max: max_energy,
                    },
                    Size { radius },
                    Genes { ..genes },
                    behavior_type,
                    Color { ..color },
                    Velocity { x: vel_x, y: vel_y },
                ));

                // Handle reproduction
                if should_reproduce && self.world.len() < (50000.0 * ENTITY_SCALE) as u32 {
                    let mut rng = rand::thread_rng();
                    let child_genes = genes.mutate(&mut rng);
                    let child_energy = max_energy * 0.4;
                    let child_radius = (child_energy / 10.0 * child_genes.size_factor).max(1.0);
                    let child_behavior_type = child_genes.get_behavior_type();
                    let child_color = Color::from_genes(&child_genes);

                    // Spawn child near parent
                    let child_x = x + rng.gen_range(-15.0..15.0);
                    let child_y = y + rng.gen_range(-15.0..15.0);

                    self.world.spawn((
                        Position {
                            x: child_x,
                            y: child_y,
                        },
                        Energy {
                            current: child_energy,
                            max: max_energy,
                        },
                        Size {
                            radius: child_radius,
                        },
                        Genes { ..child_genes },
                        child_behavior_type,
                        Color { ..child_color },
                        Velocity { x: 0.0, y: 0.0 },
                    ));
                }
            }
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
}
