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
    pub color_r: f32,                // Red component of color
    pub color_g: f32,                // Green component of color
    pub color_b: f32,                // Blue component of color
    pub size_factor: f32,            // How size relates to energy
    pub energy_loss_rate: f32,       // How quickly they lose energy
    pub energy_gain_rate: f32,       // How much energy they gain from eating
    pub movement_cost: f32,          // Energy cost per unit of movement
    pub eating_cost: f32,            // Energy cost for eating
}

impl Genes {
    fn new_random(rng: &mut ThreadRng) -> Self {
        Self {
            speed: rng.gen_range(0.1..2.0),
            sense_radius: rng.gen_range(20.0..80.0), // Larger sense radius
            energy_efficiency: rng.gen_range(0.5..1.5),
            reproduction_threshold: rng.gen_range(0.6..0.9),
            mutation_rate: rng.gen_range(0.01..0.1),
            color_r: rng.gen_range(0.0..1.0),
            color_g: rng.gen_range(0.0..1.0),
            color_b: rng.gen_range(0.0..1.0),
            size_factor: rng.gen_range(0.5..2.0),
            energy_loss_rate: rng.gen_range(0.1..1.0),
            energy_gain_rate: rng.gen_range(0.5..2.0),
            movement_cost: rng.gen_range(0.1..0.5),
            eating_cost: rng.gen_range(0.2..1.0),
        }
    }

    fn mutate(&self, rng: &mut ThreadRng) -> Self {
        let mut new_genes = self.clone();

        // Apply mutations based on mutation rate
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.speed = (self.speed + rng.gen_range(-0.2..0.2)).clamp(0.1, 3.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.sense_radius =
                (self.sense_radius + rng.gen_range(-5.0..5.0)).clamp(5.0, 100.0);
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
            new_genes.color_r = (self.color_r + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.color_g = (self.color_g + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.color_b = (self.color_b + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
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
                (self.energy_gain_rate + rng.gen_range(-0.2..0.2)).clamp(0.2, 3.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.movement_cost =
                (self.movement_cost + rng.gen_range(-0.1..0.1)).clamp(0.05, 1.0);
        }
        if rng.gen::<f32>() < self.mutation_rate {
            new_genes.eating_cost =
                (self.eating_cost + rng.gen_range(-0.2..0.2)).clamp(0.1, 2.0);
        }

        new_genes
    }
}

#[derive(Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct EntityType {
    pub is_resource: bool,
    pub is_herbivore: bool,
    pub is_predator: bool,
}

impl EntityType {
    fn new_from_genes(genes: &Genes) -> Self {
        // Determine entity type based on genes
        let is_resource = genes.color_g > 0.7 && genes.color_r < 0.3;
        let is_predator = genes.color_r > 0.7 && genes.color_g < 0.3;
        let is_herbivore = !is_resource && !is_predator;

        Self {
            is_resource,
            is_herbivore,
            is_predator,
        }
    }
}

#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    fn from_genes(genes: &Genes) -> Self {
        Self {
            r: genes.color_r,
            g: genes.color_g,
            b: genes.color_b,
        }
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
        let num_resources = (100.0 * ENTITY_SCALE) as usize; // Much fewer resources
        let num_herbivores = (800.0 * ENTITY_SCALE) as usize; // More herbivores to consume resources
        let num_predators = (200.0 * ENTITY_SCALE) as usize;

        let spawn_radius = world_size * 0.3;

        // Spawn resources (green entities that gain energy over time)
        for _ in 0..num_resources {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);
            let x = distance * angle.cos() + rng.gen_range(-10.0..10.0);
            let y = distance * angle.sin() + rng.gen_range(-10.0..10.0);

            let mut genes = Genes::new_random(rng);
            // Make them green and resource-like
            genes.color_r = rng.gen_range(0.0..0.2);
            genes.color_g = rng.gen_range(0.7..1.0);
            genes.color_b = rng.gen_range(0.0..0.3);
            genes.speed = rng.gen_range(0.0..0.5); // Resources move very slowly
            genes.energy_loss_rate = rng.gen_range(0.05..0.15); // Resources lose energy faster
            genes.energy_gain_rate = rng.gen_range(0.001..0.01); // Extremely slow growth
            genes.reproduction_threshold = rng.gen_range(0.7..0.9);

            let energy = rng.gen_range(20.0..40.0);
            let entity_type = EntityType::new_from_genes(&genes);
            let color = Color::from_genes(&genes);
            let radius = (energy / 10.0 * genes.size_factor).max(2.0);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.5,
                },
                Size { radius },
                Genes { ..genes },
                EntityType { ..entity_type },
                Color { ..color },
                Velocity { x: 0.0, y: 0.0 },
            ));
        }

        // Spawn herbivores (brown/orange entities that eat resources)
        for _ in 0..num_herbivores {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);
            let x = distance * angle.cos() + rng.gen_range(-10.0..10.0);
            let y = distance * angle.sin() + rng.gen_range(-10.0..10.0);

            let mut genes = Genes::new_random(rng);
            // Make them brown/orange
            genes.color_r = rng.gen_range(0.6..1.0);
            genes.color_g = rng.gen_range(0.3..0.7);
            genes.color_b = rng.gen_range(0.0..0.3);
            genes.speed = rng.gen_range(1.0..3.0); // Faster herbivores to find resources
            genes.energy_loss_rate = rng.gen_range(0.2..0.8);
            genes.energy_gain_rate = rng.gen_range(2.0..4.0); // Much higher energy gain rate for herbivores
            genes.reproduction_threshold = rng.gen_range(0.6..0.8);

            let energy = rng.gen_range(30.0..50.0);
            let entity_type = EntityType::new_from_genes(&genes);
            let color = Color::from_genes(&genes);
            let radius = (energy / 10.0 * genes.size_factor).max(3.0);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.2,
                },
                Size { radius },
                Genes { ..genes },
                EntityType { ..entity_type },
                Color { ..color },
                Velocity { x: 0.0, y: 0.0 },
            ));
        }

        // Spawn predators (red entities that eat herbivores)
        for _ in 0..num_predators {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);
            let x = distance * angle.cos() + rng.gen_range(-10.0..10.0);
            let y = distance * angle.sin() + rng.gen_range(-10.0..10.0);

            let mut genes = Genes::new_random(rng);
            // Make them red
            genes.color_r = rng.gen_range(0.7..1.0);
            genes.color_g = rng.gen_range(0.0..0.3);
            genes.color_b = rng.gen_range(0.0..0.3);
            genes.speed = rng.gen_range(1.0..3.0);
            genes.energy_loss_rate = rng.gen_range(1.5..3.0); // Much higher energy loss for predators
            genes.reproduction_threshold = rng.gen_range(0.8..0.95); // Higher threshold for reproduction

            let energy = rng.gen_range(40.0..60.0);
            let entity_type = EntityType::new_from_genes(&genes);
            let color = Color::from_genes(&genes);
            let radius = (energy / 10.0 * genes.size_factor).max(4.0);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.1,
                },
                Size { radius },
                Genes { ..genes },
                EntityType { ..entity_type },
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
    }

    fn log_simulation_metrics(&self) {
        let mut resource_count = 0;
        let mut herbivore_count = 0;
        let mut predator_count = 0;

        for (_, (entity_type,)) in self.world.query::<(&EntityType,)>().iter() {
            if entity_type.is_resource {
                resource_count += 1;
            } else if entity_type.is_herbivore {
                herbivore_count += 1;
            } else if entity_type.is_predator {
                predator_count += 1;
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
            "Step {}: Total={}, Resources={}, Herbivores={}, Predators={}, AvgEnergy={:.1}",
            self.step, total_entities, resource_count, herbivore_count, predator_count, avg_energy
        );
    }

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
                &EntityType,
                &Color,
                &Velocity,
            )>()
            .iter()
            .map(
                |(entity, (pos, energy, size, genes, entity_type, color, velocity))| {
                    (
                        entity,
                        pos.x,
                        pos.y,
                        energy.current,
                        energy.max,
                        size.radius,
                        genes.clone(),
                        entity_type.clone(),
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
                            entity_type,
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

                            // Movement based on genes and entity type
                            if entity_type.is_resource {
                                // Resources move very slowly or not at all
                                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                let speed = genes.speed * 0.1; // Very slow movement
                                new_x += angle.cos() * speed;
                                new_y += angle.sin() * speed;
                            } else {
                                // Herbivores and predators move towards food
                                let mut target_x = *x;
                                let mut target_y = *y;
                                let mut found_target = false;

                                for nearby_entity in &nearby_entities_to_check {
                                    if let Ok(nearby_pos) =
                                        self.world.get::<&Position>(*nearby_entity)
                                    {
                                        if let Ok(nearby_entity_type) =
                                            self.world.get::<&EntityType>(*nearby_entity)
                                        {
                                            if let Ok(nearby_energy) =
                                                self.world.get::<&Energy>(*nearby_entity)
                                            {
                                                if nearby_energy.current > 0.0 {
                                                    let distance = ((nearby_pos.x - *x).powi(2)
                                                        + (nearby_pos.y - *y).powi(2))
                                                    .sqrt();

                                                    // Check if this is a valid target
                                                    let is_valid_target = if entity_type.is_predator
                                                    {
                                                        nearby_entity_type.is_herbivore
                                                    } else if entity_type.is_herbivore {
                                                        nearby_entity_type.is_resource
                                                    } else {
                                                        false
                                                    };

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
                                    // Random movement with less persistence to prevent drift
                                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                    new_vel_x = new_vel_x * 0.3 + angle.cos() * genes.speed * 0.7;
                                    new_vel_y = new_vel_y * 0.3 + angle.sin() * genes.speed * 0.7;
                                }

                                new_x += new_vel_x;
                                new_y += new_vel_y;
                                
                                // Calculate movement cost based on distance moved and genes
                                let movement_distance = (new_vel_x * new_vel_x + new_vel_y * new_vel_y).sqrt();
                                let movement_energy_cost = movement_distance * genes.movement_cost / genes.energy_efficiency;
                                new_energy -= movement_energy_cost;
                            }

                            // Keep within bounds
                            let half_world = self.world_size / 2.0;
                            new_x = new_x.clamp(-half_world, half_world);
                            new_y = new_y.clamp(-half_world, half_world);

                            // Check for interactions (eating)
                            for nearby_entity in &nearby_entities_to_check {
                                if let Ok(nearby_pos) = self.world.get::<&Position>(*nearby_entity)
                                {
                                    if let Ok(nearby_entity_type) =
                                        self.world.get::<&EntityType>(*nearby_entity)
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
                                                let can_eat = if entity_type.is_predator {
                                                    nearby_entity_type.is_herbivore
                                                } else if entity_type.is_herbivore {
                                                    nearby_entity_type.is_resource
                                                } else {
                                                    false
                                                };

                                                if can_eat {
                                                    eaten_entity = Some(*nearby_entity);
                                                    let energy_gained = if entity_type.is_herbivore
                                                        && nearby_entity_type.is_resource
                                                    {
                                                        // Herbivores gain much more energy from eating resources
                                                        nearby_energy.current
                                                            * genes.energy_gain_rate
                                                            * 5.0
                                                    } else {
                                                        // Normal energy gain for other interactions
                                                        nearby_energy.current
                                                            * genes.energy_gain_rate
                                                    };
                                                    // Apply eating cost based on genes
                                                    let eating_energy_cost = genes.eating_cost / genes.energy_efficiency;
                                                    new_energy = (new_energy + energy_gained - eating_energy_cost)
                                                        .min(*max_energy);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Energy changes based on genes and entity type
                            if entity_type.is_resource {
                                // Resources lose energy slowly over time (no longer gain energy)
                                new_energy -= genes.energy_loss_rate;
                            } else {
                                // Other entities lose energy over time
                                new_energy -= genes.energy_loss_rate;
                            }

                            // Check for reproduction
                            if new_energy > *max_energy * genes.reproduction_threshold {
                                            let reproduction_chance = if entity_type.is_resource {
                0.0 // Resources never reproduce
            } else if entity_type.is_predator {
                0.01 // Predators reproduce rarely
            } else {
                0.05 // Herbivores reproduce normally
            };
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
                                entity_type.clone(),
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
            entity_type,
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
                    EntityType { ..entity_type },
                    Color { ..color },
                    Velocity { x: vel_x, y: vel_y },
                ));

                // Handle reproduction
                if should_reproduce && self.world.len() < (50000.0 * ENTITY_SCALE) as u32 {
                    let mut rng = rand::thread_rng();
                    let child_genes = genes.mutate(&mut rng);
                    let child_energy = max_energy * 0.4;
                    let child_radius = (child_energy / 10.0 * child_genes.size_factor).max(1.0);
                    let child_entity_type = EntityType::new_from_genes(&child_genes);
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
                        EntityType {
                            ..child_entity_type
                        },
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
