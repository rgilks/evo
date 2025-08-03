use hecs::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

// Global scale parameter to control entity counts
const ENTITY_SCALE: f32 = 0.5;

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

// Simplified genes with fewer, more meaningful parameters
#[derive(Clone, Debug)]
pub struct Genes {
    pub speed: f32,             // Movement speed
    pub sense_radius: f32,      // Detection range
    pub energy_efficiency: f32, // Energy usage multiplier
    pub reproduction_rate: f32, // Reproduction probability
    pub mutation_rate: f32,     // Gene mutation probability
    pub aggressiveness: f32,    // Behavior type (0.0 = passive, 1.0 = aggressive)
    pub color_hue: f32,         // Color hue (0.0-1.0)
    pub color_saturation: f32,  // Color saturation (0.0-1.0)
}

impl Genes {
    fn new_random(rng: &mut ThreadRng) -> Self {
        Self {
            speed: rng.gen_range(0.5..3.0),
            sense_radius: rng.gen_range(20.0..80.0),
            energy_efficiency: rng.gen_range(0.7..1.3),
            reproduction_rate: rng.gen_range(0.01..0.1),
            mutation_rate: rng.gen_range(0.01..0.1),
            aggressiveness: rng.gen_range(0.0..1.0),
            color_hue: rng.gen_range(0.0..1.0),
            color_saturation: rng.gen_range(0.5..1.0),
        }
    }

    fn mutate(&self, rng: &mut ThreadRng) -> Self {
        let mut new_genes = self.clone();

        // Use a single mutation check with field-specific probabilities
        let fields = [
            (&mut new_genes.speed, 0.2, 0.5..3.0),
            (&mut new_genes.sense_radius, 5.0, 10.0..100.0),
            (&mut new_genes.energy_efficiency, 0.1, 0.5..1.5),
            (&mut new_genes.reproduction_rate, 0.02, 0.001..0.2),
            (&mut new_genes.mutation_rate, 0.02, 0.001..0.2),
            (&mut new_genes.aggressiveness, 0.1, 0.0..1.0),
            (&mut new_genes.color_hue, 0.1, 0.0..1.0),
            (&mut new_genes.color_saturation, 0.1, 0.3..1.0),
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

    fn get_behavior_type(&self) -> BehaviorType {
        match self.aggressiveness {
            a if a < 0.3 => BehaviorType::Passive,
            a if a < 0.7 => BehaviorType::Neutral,
            _ => BehaviorType::Aggressive,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BehaviorType {
    Passive,    // Resources - slow, low energy loss
    Neutral,    // Balanced behavior
    Aggressive, // Predators - fast, high energy loss
}

impl BehaviorType {
    fn can_eat(&self, other: &BehaviorType) -> bool {
        matches!(
            (self, other),
            (
                BehaviorType::Aggressive,
                BehaviorType::Neutral | BehaviorType::Passive
            ) | (BehaviorType::Neutral, BehaviorType::Passive)
        )
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
}

impl Simulation {
    pub fn new(world_size: f32) -> Self {
        let mut world = World::new();
        let mut rng = thread_rng();
        let grid = SpatialGrid::new(25.0);

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

        for _ in 0..total_entities {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);
            let x = distance * angle.cos() + rng.gen_range(-10.0..10.0);
            let y = distance * angle.sin() + rng.gen_range(-10.0..10.0);

            let mut genes = Genes::new_random(rng);

            // Create diverse initial population
            let entity_type_roll = rng.gen::<f32>();
            match entity_type_roll {
                r if r < 0.2 => {
                    // Passive entities (resources)
                    genes.aggressiveness = rng.gen_range(0.0..0.3);
                    genes.speed = rng.gen_range(0.1..0.8);
                    genes.reproduction_rate = rng.gen_range(0.001..0.01);
                }
                r if r < 0.7 => {
                    // Neutral entities (herbivores)
                    genes.aggressiveness = rng.gen_range(0.3..0.7);
                    genes.speed = rng.gen_range(1.0..2.5);
                    genes.reproduction_rate = rng.gen_range(0.02..0.08);
                }
                _ => {
                    // Aggressive entities (predators)
                    genes.aggressiveness = rng.gen_range(0.7..1.0);
                    genes.speed = rng.gen_range(1.5..3.0);
                    genes.reproduction_rate = rng.gen_range(0.005..0.02);
                }
            }

            let energy = rng.gen_range(25.0..55.0);
            let behavior_type = genes.get_behavior_type();
            let color = genes.get_color();
            let radius = if energy / 10.0 > 2.0 {
                energy / 10.0
            } else {
                2.0
            };

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
        self.update_simulation();

        if self.step % 60 == 0 {
            self.log_simulation_metrics();
        }
    }

    fn log_simulation_metrics(&self) {
        // Use parallel processing for metrics collection
        let behavior_counts = self
            .world
            .query::<(&BehaviorType,)>()
            .iter()
            .par_bridge()
            .fold(
                || [0, 0, 0],
                |mut counts, (_, (behavior_type,))| {
                    match behavior_type {
                        BehaviorType::Passive => counts[0] += 1,
                        BehaviorType::Neutral => counts[1] += 1,
                        BehaviorType::Aggressive => counts[2] += 1,
                    }
                    counts
                },
            )
            .reduce(
                || [0, 0, 0],
                |mut a, b| {
                    a[0] += b[0];
                    a[1] += b[1];
                    a[2] += b[2];
                    a
                },
            );

        let total_entities = self.world.len();
        let avg_energy = self
            .world
            .query::<(&Energy,)>()
            .iter()
            .par_bridge()
            .map(|(_, (energy,))| energy.current)
            .sum::<f32>()
            / total_entities as f32;

        println!(
            "Step {}: Total={}, Passive={}, Neutral={}, Aggressive={}, AvgEnergy={:.1}",
            self.step,
            total_entities,
            behavior_counts[0],
            behavior_counts[1],
            behavior_counts[2],
            avg_energy
        );
    }

    fn update_simulation(&mut self) {
        // Rebuild spatial grid in parallel
        self.rebuild_spatial_grid();

        // Use Hecs' parallel iteration capabilities where possible
        // Process entities in parallel using Hecs' built-in parallel iteration
        let updates: Vec<_> = self
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
            .par_bridge()
            .filter_map(
                |(entity, (pos, energy, size, genes, behavior_type, color, velocity))| {
                    if energy.current <= 0.0 {
                        return None;
                    }

                    self.process_entity(
                        entity,
                        pos,
                        energy,
                        size,
                        genes,
                        behavior_type,
                        color,
                        velocity,
                    )
                },
            )
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
        behavior_type: &BehaviorType,
        color: &Color,
        velocity: &Velocity,
    ) -> Option<(
        Entity,
        Position,
        Energy,
        Size,
        Genes,
        BehaviorType,
        Color,
        Velocity,
        bool,
        Option<Entity>,
    )> {
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

        // Movement logic
        self.update_movement(
            behavior_type,
            genes,
            &mut new_pos,
            &mut new_velocity,
            &mut new_energy,
            pos,
            &nearby_entities,
            &mut rng,
        );

        // Boundary handling
        self.handle_boundaries(&mut new_pos, &mut new_velocity);

        // Interaction logic
        self.handle_interactions(
            &mut new_energy,
            &mut eaten_entity,
            &new_pos,
            size,
            behavior_type,
            genes,
            &nearby_entities,
        );

        // Energy changes
        new_energy -= 0.5 / genes.energy_efficiency;

        // Reproduction check
        if new_energy > energy.max * 0.7 && rng.gen::<f32>() < genes.reproduction_rate {
            should_reproduce = true;
            new_energy *= 0.6;
        }

        // Calculate new size
        let new_radius = (new_energy / 10.0).max(1.0f32);

        Some((
            entity,
            new_pos,
            Energy {
                current: new_energy,
                max: energy.max,
            },
            Size { radius: new_radius },
            genes.clone(),
            behavior_type.clone(),
            color.clone(),
            new_velocity,
            should_reproduce,
            eaten_entity,
        ))
    }

    fn update_movement(
        &self,
        behavior_type: &BehaviorType,
        genes: &Genes,
        new_pos: &mut Position,
        new_velocity: &mut Velocity,
        new_energy: &mut f32,
        pos: &Position,
        nearby_entities: &[Entity],
        rng: &mut ThreadRng,
    ) {
        match behavior_type {
            BehaviorType::Passive => {
                // Passive entities move very slowly
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = genes.speed * 0.1;
                new_pos.x += angle.cos() * speed;
                new_pos.y += angle.sin() * speed;
            }
            _ => {
                // Find target for movement
                let target = self.find_movement_target(pos, behavior_type, genes, nearby_entities);

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
                    // Random movement
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    let speed_variation = rng.gen_range(0.8..1.2);
                    new_velocity.x = angle.cos() * genes.speed * speed_variation;
                    new_velocity.y = angle.sin() * genes.speed * speed_variation;
                }

                new_pos.x += new_velocity.x;
                new_pos.y += new_velocity.y;

                // Movement cost
                let movement_distance =
                    (new_velocity.x * new_velocity.x + new_velocity.y * new_velocity.y).sqrt();
                *new_energy -= movement_distance * 0.1 / genes.energy_efficiency;
            }
        }
    }

    fn find_movement_target(
        &self,
        pos: &Position,
        behavior_type: &BehaviorType,
        genes: &Genes,
        nearby_entities: &[Entity],
    ) -> Option<(f32, f32)> {
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = self.world.get::<&Position>(entity) {
                if let Ok(nearby_behavior_type) = self.world.get::<&BehaviorType>(entity) {
                    if let Ok(nearby_energy) = self.world.get::<&Energy>(entity) {
                        if nearby_energy.current > 0.0
                            && behavior_type.can_eat(&*nearby_behavior_type)
                        {
                            let distance = ((nearby_pos.x - pos.x).powi(2)
                                + (nearby_pos.y - pos.y).powi(2))
                            .sqrt();
                            if distance < genes.sense_radius {
                                return Some((nearby_pos.x, nearby_pos.y));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn handle_boundaries(&self, pos: &mut Position, velocity: &mut Velocity) {
        let half_world = self.world_size / 2.0;

        if pos.x < -half_world {
            pos.x = -half_world;
            velocity.x = -velocity.x * 0.9;
        } else if pos.x > half_world {
            pos.x = half_world;
            velocity.x = -velocity.x * 0.9;
        }

        if pos.y < -half_world {
            pos.y = -half_world;
            velocity.y = -velocity.y * 0.9;
        } else if pos.y > half_world {
            pos.y = half_world;
            velocity.y = -velocity.y * 0.9;
        }
    }

    fn handle_interactions(
        &self,
        new_energy: &mut f32,
        eaten_entity: &mut Option<Entity>,
        new_pos: &Position,
        size: &Size,
        behavior_type: &BehaviorType,
        genes: &Genes,
        nearby_entities: &[Entity],
    ) {
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = self.world.get::<&Position>(entity) {
                if let Ok(nearby_behavior_type) = self.world.get::<&BehaviorType>(entity) {
                    if let Ok(nearby_energy) = self.world.get::<&Energy>(entity) {
                        let distance = ((nearby_pos.x - new_pos.x).powi(2)
                            + (nearby_pos.y - new_pos.y).powi(2))
                        .sqrt();

                        if distance < (size.radius + 15.0)
                            && nearby_energy.current > 0.0
                            && behavior_type.can_eat(&*nearby_behavior_type)
                        {
                            *eaten_entity = Some(entity);
                            let energy_gain_multiplier =
                                behavior_type.get_energy_gain_multiplier(&*nearby_behavior_type);
                            let energy_gained = nearby_energy.current * energy_gain_multiplier;
                            *new_energy = (*new_energy + energy_gained - 0.5)
                                .min(genes.energy_efficiency * 100.0);
                            break;
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
            BehaviorType,
            Color,
            Velocity,
            bool,
            Option<Entity>,
        )>,
    ) {
        // Remove eaten entities in parallel
        let entities_to_remove: Vec<_> = updates
            .par_iter()
            .filter_map(|(_, _, _, _, _, _, _, _, _, eaten_entity)| *eaten_entity)
            .collect();

        // Despawn entities (this needs to be sequential due to Hecs limitations)
        for &entity in &entities_to_remove {
            let _ = self.world.despawn(entity);
        }

        // Prepare spawn data in parallel
        let spawn_data: Vec<_> = updates
            .par_iter()
            .filter_map(
                |(
                    _entity,
                    position,
                    energy,
                    size,
                    genes,
                    behavior_type,
                    color,
                    velocity,
                    should_reproduce,
                    _,
                )| {
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
                        behavior_type.clone(),
                        color.clone(),
                        velocity.clone(),
                    )];

                    // Handle reproduction
                    if *should_reproduce && self.world.len() < (50000.0 * ENTITY_SCALE) as u32 {
                        let mut rng = rand::thread_rng();
                        let child_genes = genes.mutate(&mut rng);
                        let child_energy = energy_max * 0.4;
                        let child_radius = (child_energy / 10.0).max(1.0);
                        let child_behavior_type = child_genes.get_behavior_type();
                        let child_color = child_genes.get_color();

                        let child_x = pos_x + rng.gen_range(-15.0..15.0);
                        let child_y = pos_y + rng.gen_range(-15.0..15.0);

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
                            child_behavior_type,
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
        for (entity, _, _, _, _, _, _, _, _, _) in updates {
            let _ = self.world.despawn(entity);
        }

        // Spawn new entities (this needs to be sequential due to Hecs limitations)
        for (position, energy, size, genes, behavior_type, color, velocity) in spawn_data {
            self.world.spawn((
                position,
                energy,
                size,
                genes,
                behavior_type,
                color,
                velocity,
            ));
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
