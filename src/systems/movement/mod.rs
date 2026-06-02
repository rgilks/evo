use crate::components::{Energy, MovementType, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use hecs::{Entity, World};
use rand::prelude::*;

/// Movement system - handles entity movement and boundary constraints
pub struct MovementSystem;

impl crate::systems::System for MovementSystem {
    fn run(&self, ctx: &mut crate::systems::EntityContext) {
        self.update_movement(MovementUpdateParams {
            genes: ctx.genes,
            new_pos: &mut ctx.new_pos,
            new_velocity: &mut ctx.new_velocity,
            new_energy: &mut ctx.new_energy,
            pos: ctx.pos,
            nearby_entities: ctx.nearby_entities,
            world: ctx.world,
            config: ctx.config,
            world_size: ctx.world_size,
        });
        self.handle_boundaries(
            &mut ctx.new_pos,
            &mut ctx.new_velocity,
            ctx.world_size,
            ctx.config,
        );
    }
}

pub struct MovementUpdateParams<'a> {
    pub genes: &'a Genes,
    pub new_pos: &'a mut Position,
    pub new_velocity: &'a mut Velocity,
    pub new_energy: &'a mut f32,
    pub pos: &'a Position,
    pub nearby_entities: &'a [Entity],
    pub world: &'a World,
    pub config: &'a SimulationConfig,
    pub world_size: f32,
}

impl MovementSystem {
    pub fn update_movement(&self, params: MovementUpdateParams) {
        let MovementUpdateParams {
            genes,
            new_pos,
            new_velocity,
            new_energy,
            pos,
            nearby_entities,
            world,
            config,
            world_size,
        } = params;
        // Initialize accumulators
        let mut target_x = 0.0;
        let mut target_y = 0.0;
        let mut best_preference = 0.0;
        let mut found_target = false;

        let mut flock_center_x = 0.0;
        let mut flock_center_y = 0.0;
        let mut flock_velocity_x = 0.0;
        let mut flock_velocity_y = 0.0;
        let mut flock_count = 0;
        let mut separation_x = 0.0;
        let mut separation_y = 0.0;

        let mut avoidance_x = 0.0;
        let mut avoidance_y = 0.0;

        let mut particle_force_x = 0.0;
        let mut particle_force_y = 0.0;

        let interaction_radius = genes.sense_radius();
        let separation_dist = genes.behavior.movement_style.separation_distance;

        // Single pass over nearby entities
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = world.get::<&Position>(entity) {
                let dx = nearby_pos.x - pos.x;
                let dy = nearby_pos.y - pos.y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq > 0.001 && distance_sq < interaction_radius * interaction_radius {
                    let distance = distance_sq.sqrt();

                    if let Ok(nearby_genes) = world.get::<&Genes>(entity) {
                        // 1. Particle Life Physics
                        let sector = (nearby_genes.appearance.hue * 6.0).floor() as usize;
                        let sector = sector.min(5);
                        let force = genes.particle_life.interactions[sector];
                        let strength = (1.0 - distance / interaction_radius) * force;
                        particle_force_x += (dx / distance) * strength;
                        particle_force_y += (dy / distance) * strength;

                        // 2. Movement Targets (Predatory/Grazing logic)
                        if let (Ok(nearby_energy), Ok(nearby_size)) =
                            (world.get::<&Energy>(entity), world.get::<&Size>(entity))
                        {
                            if nearby_energy.current > 0.0
                                && genes.can_eat(&nearby_genes, &nearby_size, &Size { radius: 1.0 })
                            {
                                // Simplified self size check for target finding
                                let preference = genes.get_predation_preference(&nearby_genes);
                                if preference > best_preference {
                                    target_x = nearby_pos.x;
                                    target_y = nearby_pos.y;
                                    best_preference = preference;
                                    found_target = true;
                                }
                            }
                        }

                        // 3. Movement Styles
                        match genes.behavior.movement_style.style {
                            MovementType::Flocking => {
                                let gene_similarity =
                                    genes.calculate_gene_similarity(&nearby_genes);
                                if gene_similarity < 0.7 {
                                    flock_center_x += nearby_pos.x;
                                    flock_center_y += nearby_pos.y;

                                    if let Ok(nearby_vel) = world.get::<&Velocity>(entity) {
                                        flock_velocity_x += nearby_vel.x;
                                        flock_velocity_y += nearby_vel.y;
                                    }

                                    if distance < separation_dist {
                                        let sep_force = (separation_dist - distance) / distance;
                                        separation_x -= dx * sep_force;
                                        separation_y -= dy * sep_force;
                                    }
                                    flock_count += 1;
                                }
                            }
                            MovementType::Solitary => {
                                let avoid_force = interaction_radius / (distance + 1.0);
                                avoidance_x -= dx * avoid_force;
                                avoidance_y -= dy * avoid_force;
                            }
                            MovementType::Predatory => {
                                // Handled in target finding mostly, but we could add specific logic here if needed
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Apply Target Movement
        if found_target {
            self.move_towards_target(pos, target_x, target_y, genes, new_velocity);
        } else if matches!(genes.behavior.movement_style.style, MovementType::Grazing) {
            self.apply_grazing_behavior(genes, new_velocity, config);
        } else {
            self.move_randomly(genes, new_velocity, config);
        }

        // Apply Flocking Forces
        if matches!(genes.behavior.movement_style.style, MovementType::Flocking) && flock_count > 0
        {
            let flock_strength = genes.behavior.movement_style.flocking_strength;

            // Cohesion
            if genes.behavior.movement_style.cohesion_strength > 0.0 {
                flock_center_x /= flock_count as f32;
                flock_center_y /= flock_count as f32;
                let coh_x = (flock_center_x - pos.x)
                    * genes.behavior.movement_style.cohesion_strength
                    * flock_strength;
                let coh_y = (flock_center_y - pos.y)
                    * genes.behavior.movement_style.cohesion_strength
                    * flock_strength;
                new_velocity.x += coh_x * 0.1;
                new_velocity.y += coh_y * 0.1;
            }

            // Alignment
            if genes.behavior.movement_style.alignment_strength > 0.0 {
                flock_velocity_x /= flock_count as f32;
                flock_velocity_y /= flock_count as f32;
                let align_x = flock_velocity_x
                    * genes.behavior.movement_style.alignment_strength
                    * flock_strength;
                let align_y = flock_velocity_y
                    * genes.behavior.movement_style.alignment_strength
                    * flock_strength;
                new_velocity.x += align_x * 0.1;
                new_velocity.y += align_y * 0.1;
            }

            // Separation
            let sep_strength = flock_strength * 0.2;
            new_velocity.x += separation_x * sep_strength;
            new_velocity.y += separation_y * sep_strength;
        }

        // Apply Solitary Avoidance
        if matches!(genes.behavior.movement_style.style, MovementType::Solitary) {
            let avoid_strength = genes.behavior.social_tendency * 0.3;
            new_velocity.x += avoidance_x * avoid_strength;
            new_velocity.y += avoidance_y * avoid_strength;
        }

        // Apply Particle Physics Forces
        new_velocity.x += particle_force_x * config.physics.particle_force_scale;
        new_velocity.y += particle_force_y * config.physics.particle_force_scale;

        // Friction damps both axes equally (Y-only damping biased motion horizontally)
        new_velocity.x *= config.physics.particle_friction;
        new_velocity.y *= config.physics.particle_friction;

        self.update_position(new_pos, new_velocity);
        self.apply_center_pressure(new_pos, new_velocity, config, world_size);
        self.validate_position(new_pos);
        self.apply_movement_cost(new_velocity, new_energy, genes, config);
    }

    fn apply_grazing_behavior(
        &self,
        genes: &Genes,
        new_velocity: &mut Velocity,
        config: &SimulationConfig,
    ) {
        // Grazers move slowly and steadily
        let grazing_speed = genes.speed() * 0.6;

        // Add some gentle random movement
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed_variation = rng.gen_range(0.8..1.2);

        new_velocity.x = angle.cos() * grazing_speed * speed_variation;
        new_velocity.y = angle.sin() * grazing_speed * speed_variation;

        self.cap_velocity(new_velocity, config);
    }

    fn move_towards_target(
        &self,
        pos: &Position,
        target_x: f32,
        target_y: f32,
        genes: &Genes,
        new_velocity: &mut Velocity,
    ) {
        let dx = target_x - pos.x;
        let dy = target_y - pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance > 0.0 {
            new_velocity.x = (dx / distance) * genes.speed();
            new_velocity.y = (dy / distance) * genes.speed();
        }
    }

    fn move_randomly(&self, genes: &Genes, new_velocity: &mut Velocity, config: &SimulationConfig) {
        let mut rng = thread_rng();
        let speed_variation = rng.gen_range(0.8..1.2);
        let speed = genes.speed() * speed_variation;

        // Generate random direction using uniform distribution in a circle
        let (dx, dy) = self.generate_random_direction(&mut rng);
        new_velocity.x = dx * speed;
        new_velocity.y = dy * speed;

        self.cap_velocity(new_velocity, config);
    }

    fn generate_random_direction(&self, rng: &mut ThreadRng) -> (f32, f32) {
        loop {
            let dx = rng.gen_range(-1.0f32..1.0);
            let dy = rng.gen_range(-1.0f32..1.0);
            let length_sq = dx * dx + dy * dy;
            if length_sq <= 1.0 && length_sq > 0.0 {
                // Normalize to unit vector
                let length = length_sq.sqrt();
                return (dx / length, dy / length);
            }
        }
    }

    fn cap_velocity(&self, velocity: &mut Velocity, config: &SimulationConfig) {
        if velocity.x.abs() > config.physics.max_velocity {
            velocity.x = velocity.x.signum() * config.physics.max_velocity;
        }
        if velocity.y.abs() > config.physics.max_velocity {
            velocity.y = velocity.y.signum() * config.physics.max_velocity;
        }
    }

    fn update_position(&self, new_pos: &mut Position, new_velocity: &Velocity) {
        new_pos.x += new_velocity.x;
        new_pos.y += new_velocity.y;
    }

    fn validate_position(&self, new_pos: &mut Position) {
        if new_pos.x.is_nan() || new_pos.x.is_infinite() {
            new_pos.x = 0.0;
        }
        if new_pos.y.is_nan() || new_pos.y.is_infinite() {
            new_pos.y = 0.0;
        }
    }

    fn apply_center_pressure(
        &self,
        pos: &Position,
        velocity: &mut Velocity,
        config: &SimulationConfig,
        world_size: f32,
    ) {
        let half_world = world_size / 2.0;

        // Calculate distance from center
        let distance_from_center = (pos.x * pos.x + pos.y * pos.y).sqrt();

        // Calculate distance from edge (how close to boundary)
        let distance_from_edge_x = half_world - pos.x.abs();
        let distance_from_edge_y = half_world - pos.y.abs();
        let distance_from_edge = distance_from_edge_x.min(distance_from_edge_y);

        // Only apply pressure if entity is away from center
        if distance_from_center > 10.0 {
            // Calculate direction towards center
            let center_dx = -pos.x / distance_from_center;
            let center_dy = -pos.y / distance_from_center;

            // Base pressure strength
            let base_pressure = config.physics.center_pressure_strength;

            // Increase pressure strength when closer to edges
            // Pressure is strongest at edges (distance_from_edge = 0) and weakest in center
            let edge_multiplier = if distance_from_edge < 50.0 {
                // Exponential increase as we get closer to edges
                let edge_factor = (50.0 - distance_from_edge) / 50.0;
                1.0 + edge_factor * edge_factor * 8.0 // Up to 9x stronger at edges
            } else {
                1.0
            };

            let pressure_strength = base_pressure * edge_multiplier;
            velocity.x += center_dx * pressure_strength;
            velocity.y += center_dy * pressure_strength;
        }
    }

    fn apply_movement_cost(
        &self,
        new_velocity: &Velocity,
        new_energy: &mut f32,
        genes: &Genes,
        config: &SimulationConfig,
    ) {
        let movement_distance =
            (new_velocity.x * new_velocity.x + new_velocity.y * new_velocity.y).sqrt();
        *new_energy -=
            movement_distance * config.energy.movement_energy_cost / genes.energy_efficiency();
    }

    pub fn handle_boundaries(
        &self,
        pos: &mut Position,
        velocity: &mut Velocity,
        world_size: f32,
        config: &SimulationConfig,
    ) {
        let half_world = world_size / 2.0;

        // Use <= and >= to handle edge cases better
        if pos.x <= -half_world + config.physics.boundary_margin {
            pos.x = -half_world + config.physics.boundary_margin;
            velocity.x = velocity.x.abs() * config.physics.velocity_bounce_factor;
        } else if pos.x >= half_world - config.physics.boundary_margin {
            pos.x = half_world - config.physics.boundary_margin;
            velocity.x = -velocity.x.abs() * config.physics.velocity_bounce_factor;
        }

        if pos.y <= -half_world + config.physics.boundary_margin {
            pos.y = -half_world + config.physics.boundary_margin;
            velocity.y = velocity.y.abs() * config.physics.velocity_bounce_factor;
        } else if pos.y >= half_world - config.physics.boundary_margin {
            pos.y = half_world - config.physics.boundary_margin;
            velocity.y = -velocity.y.abs() * config.physics.velocity_bounce_factor;
        }
    }
}

#[cfg(test)]
mod tests;
