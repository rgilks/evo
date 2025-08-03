use hecs::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

// Global scale parameter to control entity counts
const ENTITY_SCALE: f32 = 0.5; // 0.5 = half the current counts, 1.0 = current counts, 2.0 = double counts

// Components
pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Energy {
    pub current: f32,
    pub max: f32,
}

pub struct Size {
    pub radius: f32,
}

#[derive(Clone)]
pub struct Genes {
    pub speed: f32,
    pub sense_radius: f32,
    pub energy_efficiency: f32,
    pub reproduction_threshold: f32,
    pub mutation_rate: f32,
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub struct EntityType {
    pub is_predator: bool,
    pub is_resource: bool,
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

// Spatial Grid System
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
    rng: ThreadRng,
    step: u32,
    grid: SpatialGrid,
}

impl Simulation {
    pub fn new(world_size: f32) -> Self {
        let mut world = World::new();
        let mut rng = thread_rng();
        let grid = SpatialGrid::new(20.0); // Larger cell size to reduce grid operations

        // Spawn initial entities
        Self::spawn_initial_entities(&mut world, &mut rng, world_size);

        Self {
            world,
            world_size,
            rng,
            step: 0,
            grid,
        }
    }

    fn spawn_initial_entities(world: &mut World, rng: &mut ThreadRng, world_size: f32) {
        // Spawn initial entities with scalable counts
        let num_resources = (2500.0 * ENTITY_SCALE) as usize; // 1250 with 0.5 scale (halved again from 2500)
        let num_herbivores = (5000.0 * ENTITY_SCALE) as usize; // 2500 with 0.5 scale
        let num_predators = (2000.0 * ENTITY_SCALE) as usize; // 1000 with 0.5 scale

        // Spawn resources (green) - in a circle in the center
        let spawn_radius = world_size * 0.25; // 25% of world size for initial circle (increased from 15%)
        for _ in 0..num_resources {
            // Generate random angle and distance for circular distribution
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);

            let x = distance * angle.cos();
            let y = distance * angle.sin();
            let energy: f32 = rng.gen_range(15.0..35.0);
            let radius = (energy / 10.0).max(2.0_f32);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy,
                },
                Size { radius },
                Color {
                    r: 0.1,
                    g: 0.9 + rng.gen_range(-0.1..0.1),
                    b: 0.1,
                },
            ));
        }

        // Spawn herbivores (orange/brown) - in a circle in the center
        for _ in 0..num_herbivores {
            // Generate random angle and distance for circular distribution
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);

            let x = distance * angle.cos();
            let y = distance * angle.sin();
            let energy: f32 = rng.gen_range(25.0..45.0);
            let radius = (energy / 10.0).max(2.5_f32);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy,
                },
                Size { radius },
                Color {
                    r: 0.9 + rng.gen_range(-0.1..0.1),
                    g: 0.5 + rng.gen_range(-0.1..0.1),
                    b: 0.1 + rng.gen_range(-0.05..0.05),
                },
            ));
        }

        // Spawn predators (red) - in a circle in the center
        for _ in 0..num_predators {
            // Generate random angle and distance for circular distribution
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..spawn_radius);

            let x = distance * angle.cos();
            let y = distance * angle.sin();
            let energy: f32 = rng.gen_range(35.0..55.0);
            let radius = (energy / 10.0).max(3.0_f32);

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy,
                },
                Size { radius },
                Color {
                    r: 0.9 + rng.gen_range(-0.1..0.1),
                    g: 0.1 + rng.gen_range(-0.05..0.05),
                    b: 0.1 + rng.gen_range(-0.05..0.05),
                },
            ));
        }
    }

    pub fn update(&mut self) {
        self.step += 1;

        // Simple update with spatial grid optimization
        self.simple_update_with_grid();

        // Log simulation metrics every 60 frames (about once per second)
        if self.step % 60 == 0 {
            self.log_simulation_metrics();
        }
    }

    fn log_simulation_metrics(&self) {
        // Count different entity types
        let mut resource_count = 0;
        let mut herbivore_count = 0;
        let mut predator_count = 0;
        let mut other_count = 0;

        for (_, (color,)) in self.world.query::<(&Color,)>().iter() {
            if color.g > 0.7 && color.r < 0.3 {
                resource_count += 1;
            } else if color.r > 0.7
                && color.g > 0.4
                && color.g < 0.6
                && color.b > 0.05
                && color.b < 0.15
            {
                herbivore_count += 1;
            } else if color.r > 0.7 && color.g < 0.3 && color.b < 0.3 {
                predator_count += 1;
            } else {
                other_count += 1;
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
            "Step {}: Total={}, Resources={}, Herbivores={}, Predators={}, Other={}, AvgEnergy={:.1}",
            self.step, total_entities, resource_count, herbivore_count, predator_count, other_count, avg_energy
        );
    }

    fn simple_update_with_grid(&mut self) {
        // Clear and rebuild spatial grid
        self.grid.clear();
        let grid_entities: Vec<_> = self
            .world
            .query::<(&Position, &Energy, &Size, &Color)>()
            .iter()
            .map(|(entity, (pos, _, _, _))| (entity, pos.x, pos.y))
            .collect();

        // Insert entities into grid (sequential due to borrowing constraints)
        for (entity, x, y) in grid_entities {
            self.grid.insert(entity, x, y);
        }

        // Collect all entity data first
        let entity_data: Vec<_> = self
            .world
            .query::<(&Position, &Energy, &Size, &Color)>()
            .iter()
            .map(|(entity, (pos, energy, size, color))| {
                (
                    entity,
                    pos.x,
                    pos.y,
                    energy.current,
                    energy.max,
                    size.radius,
                    *color,
                )
            })
            .collect();

        // Process entities in parallel chunks for better CPU utilization
        let chunk_size = (entity_data.len() / rayon::current_num_threads()).max(1);
        let updates: Vec<_> = entity_data
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                chunk
                    .iter()
                    .filter_map(|(entity, x, y, energy, max_energy, radius, color)| {
                        if *energy <= 0.0 {
                            return None;
                        }

                        // Create a thread-local RNG for this entity
                        let mut rng = rand::thread_rng();

                        // Use spatial grid to find nearby entities for potential interactions
                        let nearby_entities = self.grid.get_nearby_entities(*x, *y, radius * 1.5);

                        // Limit the number of nearby entities we check to prevent performance issues
                        let max_nearby_to_check = 15; // Further reduced to prevent flickering
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

                        // Simple movement based on entity type (determined by color)
                        let mut new_x = *x;
                        let mut new_y = *y;

                        if color.r > 0.7 {
                            // Red entities (predators) - move randomly but faster
                            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let speed = 0.3; // Very slow movement for predators
                            new_x += angle.cos() * speed;
                            new_y += angle.sin() * speed;
                        } else if color.g > 0.7 {
                            // Green entities (resources) - move very slowly to spread out
                            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let speed = 0.1; // Extremely slow movement for resources
                            new_x += angle.cos() * speed;
                            new_y += angle.sin() * speed;
                        } else {
                            // Brown entities (herbivores) - move towards green areas
                            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let speed = 0.2; // Very slow movement for herbivores
                            new_x += angle.cos() * speed;
                            new_y += angle.sin() * speed;
                        }

                        // Limit maximum movement distance to prevent teleporting
                        let max_move_distance = 2.0;
                        let dx = new_x - *x;
                        let dy = new_y - *y;
                        let move_distance = (dx * dx + dy * dy).sqrt();

                        if move_distance > max_move_distance {
                            let scale = max_move_distance / move_distance;
                            new_x = *x + dx * scale;
                            new_y = *y + dy * scale;
                        }

                        // Keep within bounds with bounce effect
                        let mut clamped_x = new_x;
                        let mut clamped_y = new_y;

                        if new_x < -self.world_size / 2.0 {
                            clamped_x = -self.world_size / 2.0 + 10.0; // Bounce back
                        } else if new_x > self.world_size / 2.0 {
                            clamped_x = self.world_size / 2.0 - 10.0; // Bounce back
                        }

                        if new_y < -self.world_size / 2.0 {
                            clamped_y = -self.world_size / 2.0 + 10.0; // Bounce back
                        } else if new_y > self.world_size / 2.0 {
                            clamped_y = self.world_size / 2.0 - 10.0; // Bounce back
                        }

                        // Energy changes based on entity type
                        let mut new_energy = *energy;
                        let mut eaten_entity = None;

                        // Check for interactions with nearby entities using spatial grid
                        for nearby_entity in &nearby_entities_to_check {
                            if let Ok(nearby_pos) = self.world.get::<&Position>(*nearby_entity) {
                                if let Ok(nearby_color) = self.world.get::<&Color>(*nearby_entity) {
                                    if let Ok(nearby_energy) =
                                        self.world.get::<&Energy>(*nearby_entity)
                                    {
                                        let distance = ((nearby_pos.x - clamped_x).powi(2)
                                            + (nearby_pos.y - clamped_y).powi(2))
                                        .sqrt();

                                        // Interaction distance based on entity size
                                        if distance < (radius + 8.0) && nearby_energy.current > 0.0
                                        {
                                            // Check entity types
                                            let is_predator =
                                                color.r > 0.7 && color.g < 0.3 && color.b < 0.3;
                                            let is_herbivore = color.r > 0.7
                                                && color.g > 0.4
                                                && color.g < 0.6
                                                && color.b > 0.05
                                                && color.b < 0.15;
                                            let is_nearby_herbivore = nearby_color.r > 0.7
                                                && nearby_color.g > 0.4
                                                && nearby_color.g < 0.6
                                                && nearby_color.b > 0.05
                                                && nearby_color.b < 0.15;
                                            let is_nearby_resource =
                                                nearby_color.g > 0.6 && nearby_color.r < 0.4; // More lenient resource detection

                                            if is_predator && is_nearby_herbivore {
                                                // Predator (red) eats herbivore (brown)
                                                // Mark herbivore for removal and give energy to predator
                                                eaten_entity = Some(*nearby_entity);
                                                new_energy = (new_energy + 20.0).min(*max_energy);
                                                break;
                                            } else if is_herbivore && is_nearby_resource {
                                                // Herbivore (brown) eats resource (green)
                                                // Consume energy from resource instead of removing it
                                                let energy_consumed =
                                                    15.0_f32.min(nearby_energy.current);
                                                new_energy =
                                                    (new_energy + energy_consumed).min(*max_energy);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Determine entity type for energy changes
                        let is_resource = color.g > 0.7 && color.r < 0.3;
                        let is_predator = color.r > 0.7 && color.g < 0.3 && color.b < 0.3;
                        let is_herbivore = color.r > 0.7
                            && color.g > 0.4
                            && color.g < 0.6
                            && color.b > 0.05
                            && color.b < 0.15;

                        if is_resource {
                            // Resources grow slowly and can be consumed
                            new_energy = (new_energy + 0.1).min(*max_energy); // Much slower growth
                        } else if is_predator {
                            // Predators lose energy more slowly
                            new_energy -= 0.6;
                        } else if is_herbivore {
                            // Herbivores lose energy more slowly
                            new_energy -= 0.4; // Reduced from 0.5 to restore stability
                        } else {
                            // Default energy loss for unknown entities
                            new_energy -= 0.5;
                        }

                        // Cap energy at maximum
                        new_energy = new_energy.min(*max_energy);

                        // Reproduction logic
                        let mut should_reproduce = false;
                        if new_energy > *max_energy * 0.6 {
                            let reproduction_chance = if is_resource { 0.005 } else { 0.15 }; // Resources reproduce very slowly, others faster
                            if rng.gen::<f32>() < reproduction_chance {
                                should_reproduce = true;
                                new_energy *= 0.5; // Parent loses more energy
                            }
                        }

                        // Return the update for this entity
                        Some((
                            *entity,
                            clamped_x,
                            clamped_y,
                            new_energy,
                            *max_energy,
                            *radius,
                            *color,
                            should_reproduce,
                            eaten_entity,
                        ))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Collect entities to remove and track resource consumption
        let mut entities_to_remove: Vec<Entity> = Vec::new();
        let mut resource_consumption: Vec<(Entity, f32)> = Vec::new();

        for (entity, _, _, energy, _, _, color, _, eaten_entity) in &updates {
            if let Some(eaten) = eaten_entity {
                // Check if the eaten entity is a resource
                if let Ok(eaten_color) = self.world.get::<&Color>(*eaten) {
                    if eaten_color.g > 0.7 && eaten_color.r < 0.3 {
                        // It's a resource - consume energy instead of removing
                        if let Ok(eaten_energy) = self.world.get::<&Energy>(*eaten) {
                            let energy_consumed = 15.0_f32.min(eaten_energy.current);
                            resource_consumption
                                .push((*eaten, eaten_energy.current - energy_consumed));
                        }
                    } else {
                        // It's not a resource - remove it
                        entities_to_remove.push(*eaten);
                    }
                }
            }
        }

        // Remove eaten non-resource entities
        for entity in entities_to_remove {
            let _ = self.world.despawn(entity);
        }

        // Collect resource updates to avoid borrowing conflicts
        let mut resource_updates: Vec<(Entity, f32, f32, f32, f32, f32, f32, f32)> = Vec::new();
        for (entity, new_energy) in resource_consumption {
            if let Ok(energy) = self.world.get::<&Energy>(entity) {
                if let Ok(pos) = self.world.get::<&Position>(entity) {
                    if let Ok(color) = self.world.get::<&Color>(entity) {
                        resource_updates.push((
                            entity, pos.x, pos.y, new_energy, energy.max, color.r, color.g, color.b,
                        ));
                    }
                }
            }
        }

        // Update resource energy levels
        for (entity, x, y, new_energy, max_energy, r, g, b) in resource_updates {
            let _ = self.world.despawn(entity);
            let new_radius = (new_energy / 10.0).max(1.0);
            self.world.spawn((
                Position { x, y },
                Energy {
                    current: new_energy,
                    max: max_energy,
                },
                Size { radius: new_radius },
                Color { r, g, b },
            ));
        }

        // Apply updates and handle reproduction
        for (entity, x, y, energy, max_energy, radius, color, should_reproduce, _) in updates {
            let _ = self.world.despawn(entity);
            if energy > 0.0 {
                // Calculate size proportional to current energy
                let new_radius = (energy / 10.0).max(1.0);

                self.world.spawn((
                    Position { x, y },
                    Energy {
                        current: energy,
                        max: max_energy,
                    },
                    Size { radius: new_radius },
                    Color {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                    },
                ));

                // Handle reproduction only if population is not too high
                if should_reproduce && self.world.len() < (150000.0 * ENTITY_SCALE) as u32 {
                    // Scaled population limit
                    let mut rng = rand::thread_rng();
                    let child_energy = max_energy * 0.4;
                    let child_radius = (child_energy / 15.0).max(1.0);

                    // Slight color mutation for evolution
                    let mut child_color = color;
                    if rng.gen::<f32>() < 0.2 {
                        child_color.r = (child_color.r + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
                        child_color.g = (child_color.g + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
                        child_color.b = (child_color.b + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0);
                    }

                    // Spawn child near parent
                    let child_x = x + rng.gen_range(-20.0..20.0);
                    let child_y = y + rng.gen_range(-20.0..20.0);

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
                        Color {
                            r: child_color.r,
                            g: child_color.g,
                            b: child_color.b,
                        },
                    ));
                }
            }
        }

        // No automatic spawning after initial entities - all new entities come from reproduction
    }

    pub fn entity_count(&self) -> usize {
        self.world.len() as usize
    }

    pub fn get_entities(&self) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .par_bridge()
            .map(|(_, (pos, size, color))| (pos.x, pos.y, size.radius, color.r, color.g, color.b))
            .collect()
    }

    pub fn get_world_size(&self) -> f32 {
        self.world_size
    }
}
