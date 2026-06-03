use crate::components::{MovementType, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use hecs::Entity;
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
            cache: ctx.cache,
            particle_matrix: ctx.particle_matrix,
            config: ctx.config,
            world_size: ctx.world_size,
            rng: &mut ctx.rng,
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
    pub cache: &'a crate::systems::NeighborCache,
    pub particle_matrix: &'a [[f32; 6]; 6],
    pub config: &'a SimulationConfig,
    pub world_size: f32,
    pub rng: &'a mut StdRng,
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
            cache,
            particle_matrix,
            config,
            world_size,
            rng,
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
        // Self's hue sector indexes into the shared particle-life matrix (constant
        // across the neighbour loop).
        let self_sector = ((genes.appearance.hue * 6.0).floor() as usize).min(5);

        // Single pass over nearby entities, reading from the per-tick cache.
        for &entity in nearby_entities {
            let Some(n) = cache.get(&entity) else {
                continue;
            };
            let dx = n.pos.x - pos.x;
            let dy = n.pos.y - pos.y;
            let distance_sq = dx * dx + dy * dy;

            if distance_sq > 0.001 && distance_sq < interaction_radius * interaction_radius {
                let distance = distance_sq.sqrt();

                // 1. Particle-life physics: coherent attraction/repulsion via the
                // shared matrix, indexed by both creatures' hue sectors.
                let other_sector = ((n.genes.appearance.hue * 6.0).floor() as usize).min(5);
                let force = particle_matrix[self_sector][other_sector];
                let strength = (1.0 - distance / interaction_radius) * force;
                particle_force_x += (dx / distance) * strength;
                particle_force_y += (dy / distance) * strength;

                // 2. Movement Targets (Predatory/Grazing logic)
                if n.energy.current > 0.0 && genes.can_eat(&n.genes, &n.size, &Size { radius: 1.0 })
                {
                    // Simplified self size check for target finding
                    let preference = genes.get_predation_preference(&n.genes);
                    if preference > best_preference {
                        target_x = n.pos.x;
                        target_y = n.pos.y;
                        best_preference = preference;
                        found_target = true;
                    }
                }

                // 3. Movement Styles
                match genes.behavior.movement_style.style {
                    MovementType::Flocking => {
                        let gene_similarity = genes.calculate_gene_similarity(&n.genes);
                        if gene_similarity < 0.7 {
                            flock_center_x += n.pos.x;
                            flock_center_y += n.pos.y;
                            flock_velocity_x += n.velocity.x;
                            flock_velocity_y += n.velocity.y;

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
                    MovementType::Predatory => {}
                    _ => {}
                }
            }
        }

        // Apply Target Movement
        if found_target {
            self.move_towards_target(pos, target_x, target_y, genes, new_velocity);
        } else if matches!(genes.behavior.movement_style.style, MovementType::Grazing) {
            self.apply_grazing_behavior(genes, new_velocity, config, rng);
        } else {
            self.move_randomly(genes, new_velocity, config, rng);
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

        // Enforce the speed limit on the final accumulated velocity. Previously only
        // `move_randomly`'s base velocity was capped, so the additive forces (targets,
        // flocking, particle-life) could drive entities far past `max_velocity`.
        self.cap_velocity(new_velocity, config);

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
        rng: &mut StdRng,
    ) {
        // Grazers move slowly and steadily
        let grazing_speed = genes.speed() * 0.6;

        // Add some gentle random movement
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

    fn move_randomly(
        &self,
        genes: &Genes,
        new_velocity: &mut Velocity,
        config: &SimulationConfig,
        rng: &mut StdRng,
    ) {
        let speed_variation = rng.gen_range(0.8..1.2);
        let speed = genes.speed() * speed_variation;

        // Generate random direction using uniform distribution in a circle
        let (dx, dy) = self.generate_random_direction(rng);
        new_velocity.x = dx * speed;
        new_velocity.y = dy * speed;

        self.cap_velocity(new_velocity, config);
    }

    fn generate_random_direction(&self, rng: &mut StdRng) -> (f32, f32) {
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

    /// Clamp speed (the velocity *magnitude*) to `max_velocity`, preserving
    /// direction. Applied to the final accumulated velocity each tick.
    fn cap_velocity(&self, velocity: &mut Velocity, config: &SimulationConfig) {
        let max = config.physics.max_velocity;
        let speed_sq = velocity.x * velocity.x + velocity.y * velocity.y;
        if speed_sq > max * max {
            let scale = max / speed_sq.sqrt();
            velocity.x *= scale;
            velocity.y *= scale;
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
        // Containment, not a constant pull to the centre: push inward only within a
        // margin of the boundary (ramping up quadratically toward the edge), leaving
        // the interior free so clusters can form and roam instead of collapsing into
        // a single central blob.
        let half_world = world_size / 2.0;
        let margin = (half_world * 0.5).max(1.0);
        let distance_from_edge = (half_world - pos.x.abs()).min(half_world - pos.y.abs());

        if distance_from_edge < margin {
            let distance_from_center = (pos.x * pos.x + pos.y * pos.y).sqrt();
            if distance_from_center > 0.0 {
                // 0 at the margin, 1 at the edge, >1 past it.
                let edge_factor = (margin - distance_from_edge) / margin;
                let pressure =
                    config.physics.center_pressure_strength * edge_factor * edge_factor * 10.0;
                velocity.x += (-pos.x / distance_from_center) * pressure;
                velocity.y += (-pos.y / distance_from_center) * pressure;
            }
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
