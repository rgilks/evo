use hecs::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

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
        // Spawn initial entities with reduced counts to prevent performance issues
        let num_resources = 100; // Reduced from 150
        let num_herbivores = 50; // Reduced from 80
        let num_predators = 20; // Reduced from 30

        // Spawn resources (green) - spread them out more
        for _ in 0..num_resources {
            let x = rng.gen_range(-world_size / 2.0..world_size / 2.0);
            let y = rng.gen_range(-world_size / 2.0..world_size / 2.0);
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

        // Spawn herbivores (orange/brown) - spread them out
        for _ in 0..num_herbivores {
            let x = rng.gen_range(-world_size / 2.0..world_size / 2.0);
            let y = rng.gen_range(-world_size / 2.0..world_size / 2.0);
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

        // Spawn predators (red) - spread them out
        for _ in 0..num_predators {
            let x = rng.gen_range(-world_size / 2.0..world_size / 2.0);
            let y = rng.gen_range(-world_size / 2.0..world_size / 2.0);
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

        // Process entities sequentially to reduce CPU load and prevent flickering
        let updates: Vec<_> = entity_data
            .iter() // Use sequential processing to reduce CPU load
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
                let nearby_entities_to_check = if nearby_entities.len() > max_nearby_to_check {
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
                            if let Ok(nearby_energy) = self.world.get::<&Energy>(*nearby_entity) {
                                let distance = ((nearby_pos.x - clamped_x).powi(2)
                                    + (nearby_pos.y - clamped_y).powi(2))
                                .sqrt();

                                // Interaction distance based on entity size
                                if distance < (radius + 8.0) && nearby_energy.current > 0.0 {
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
                                        // Mark resource for removal and give energy to herbivore
                                        eaten_entity = Some(*nearby_entity);
                                        new_energy = (new_energy + 15.0).min(*max_energy);
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
                    // Resources grow faster and more sustainably
                    new_energy = (new_energy + 0.4).min(*max_energy); // Reduced from 0.8
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
                if new_energy > *max_energy * 0.6 && (is_predator || is_herbivore) {
                    if rng.gen::<f32>() < 0.12 {
                        // 12% chance to reproduce (increased from 8%)
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
            .collect();

        // Collect entities to remove
        let entities_to_remove: Vec<Entity> = updates
            .iter()
            .filter_map(|(_, _, _, _, _, _, _, _, eaten_entity)| *eaten_entity)
            .collect();

        // Remove eaten entities
        for entity in entities_to_remove {
            let _ = self.world.despawn(entity);
        }

        // Apply updates and handle reproduction
        for (entity, x, y, energy, max_energy, radius, color, should_reproduce, _) in updates {
            let _ = self.world.despawn(entity);
            if energy > 0.0 {
                self.world.spawn((
                    Position { x, y },
                    Energy {
                        current: energy,
                        max: max_energy,
                    },
                    Size { radius },
                    Color {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                    },
                ));

                // Handle reproduction only if population is not too high
                if should_reproduce && self.world.len() < 1500 { // Reduced from 2000 to prevent performance issues
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

        // Spawn new resources more frequently - use parallel processing for multiple spawns
        if self.step % 60 == 0 && self.rng.gen::<f32>() < 0.3 {
            // Spawn multiple resources in parallel
            let spawn_count = if self.rng.gen::<f32>() < 0.3 { 2 } else { 1 };
            let spawns: Vec<_> = (0..spawn_count)
                .map(|_| {
                    let x = self
                        .rng
                        .gen_range(-self.world_size / 2.0..self.world_size / 2.0);
                    let y = self
                        .rng
                        .gen_range(-self.world_size / 2.0..self.world_size / 2.0);
                    let energy = self.rng.gen_range(20.0..40.0);
                    (x, y, energy)
                })
                .collect();

            for (x, y, energy) in spawns {
                self.world.spawn((
                    Position { x, y },
                    Energy {
                        current: energy,
                        max: 50.0,
                    },
                    Size {
                        radius: (energy / 10.0).max(1.0),
                    },
                    Color {
                        r: 0.1,
                        g: 0.9,
                        b: 0.1,
                    },
                ));
            }
        }

        // Spawn new herbivores if population is low - use parallel processing for counting
        let herbivore_count = self
            .world
            .query::<(&Color,)>()
            .iter()
            .collect::<Vec<_>>()
            .par_iter()
            .filter(|(_, (color,))| {
                color.r > 0.7 && color.g > 0.4 && color.g < 0.6 && color.b > 0.05 && color.b < 0.15
            })
            .count();

        if herbivore_count < 50 && self.step % 100 == 0 {
            let x = self
                .rng
                .gen_range(-self.world_size / 2.0..self.world_size / 2.0);
            let y = self
                .rng
                .gen_range(-self.world_size / 2.0..self.world_size / 2.0);
            let energy = self.rng.gen_range(30.0..50.0);

            self.world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: 60.0,
                },
                Size {
                    radius: (energy / 15.0).max(1.5),
                },
                Color {
                    r: 0.9,
                    g: 0.5,
                    b: 0.1,
                },
            ));
        }

        // Spawn new predators if herbivore population is high - use parallel processing for counting
        let predator_count = self
            .world
            .query::<(&Color,)>()
            .iter()
            .collect::<Vec<_>>()
            .par_iter()
            .filter(|(_, (color,))| color.r > 0.7 && color.g < 0.3 && color.b < 0.3)
            .count();

        if herbivore_count > 80 && predator_count < 30 && self.step % 150 == 0 {
            let x = self
                .rng
                .gen_range(-self.world_size / 2.0..self.world_size / 2.0);
            let y = self
                .rng
                .gen_range(-self.world_size / 2.0..self.world_size / 2.0);
            let energy = self.rng.gen_range(40.0..60.0);

            self.world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: 70.0,
                },
                Size {
                    radius: (energy / 12.0).max(2.0),
                },
                Color {
                    r: 0.9,
                    g: 0.1,
                    b: 0.1,
                },
            ));
        }
    }

    pub fn entity_count(&self) -> usize {
        self.world.len() as usize
    }

    pub fn get_entities(&self) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        let mut entities = Vec::new();

        for (_, (pos, size, color)) in self.world.query::<(&Position, &Size, &Color)>().iter() {
            entities.push((pos.x, pos.y, size.radius, color.r, color.g, color.b));
        }

        entities
    }

    pub fn get_world_size(&self) -> f32 {
        self.world_size
    }
}
