use crate::components::{Energy, Position};
use crate::genes::Genes;
use hecs::World;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

/// Entity type classification based on dominant traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum EntityType {
    RedDominant,
    GreenDominant,
    BlueDominant,
    Purple,
    Mixed,
}

/// Comprehensive simulation statistics
#[derive(Debug, Clone, Serialize)]
pub struct SimulationStats {
    pub total_entities: usize,
    pub entity_counts: HashMap<EntityType, usize>,
    pub average_metrics: EntityMetrics,
    pub population_density: f32,
    pub world_center_drift: (f32, f32),
}

/// Average metrics across all entities
#[derive(Debug, Clone, Serialize)]
pub struct EntityMetrics {
    pub average_energy: f32,
    pub average_speed: f32,
    pub average_size: f32,
    pub average_reproduction_rate: f32,
    pub average_sense_radius: f32,
    pub average_energy_efficiency: f32,
}

impl SimulationStats {
    pub fn from_world(world: &World, max_population: f32, entity_scale: f32) -> Self {
        let total_entities = world.len();

        // Calculate entity type distribution
        let entity_counts = Self::classify_entities(world);

        // Calculate average metrics
        let average_metrics = Self::calculate_average_metrics(world, total_entities as usize);

        // Calculate population density
        let population_density = total_entities as f32 / (max_population * entity_scale);

        // Calculate world center drift
        let world_center_drift = Self::calculate_world_center_drift(world, total_entities as usize);

        Self {
            total_entities: total_entities as usize,
            entity_counts,
            average_metrics,
            population_density,
            world_center_drift,
        }
    }

    fn classify_entities(world: &World) -> HashMap<EntityType, usize> {
        let mut counts = HashMap::new();

        for (_, (genes,)) in world.query::<(&Genes,)>().iter() {
            let color = genes.get_color();
            let entity_type = Self::classify_by_color(&color);
            *counts.entry(entity_type).or_insert(0) += 1;
        }

        counts
    }

    fn classify_by_color(color: &crate::components::Color) -> EntityType {
        let r = color.r;
        let g = color.g;
        let b = color.b;

        // Red dominant
        if r > 0.6 && r > g && r > b {
            EntityType::RedDominant
        }
        // Green dominant
        else if g > 0.6 && g > r && g > b {
            EntityType::GreenDominant
        }
        // Blue dominant
        else if b > 0.6 && b > r && b > g {
            EntityType::BlueDominant
        }
        // Purple (high red and blue, low green)
        else if r > 0.5 && b > 0.5 && g < 0.4 {
            EntityType::Purple
        }
        // Mixed colors
        else {
            EntityType::Mixed
        }
    }

    fn calculate_average_metrics(world: &World, total_entities: usize) -> EntityMetrics {
        if total_entities == 0 {
            return EntityMetrics {
                average_energy: 0.0,
                average_speed: 0.0,
                average_size: 0.0,
                average_reproduction_rate: 0.0,
                average_sense_radius: 0.0,
                average_energy_efficiency: 0.0,
            };
        }

        let gene_stats = world
            .query::<(&Genes,)>()
            .iter()
            .par_bridge()
            .fold(
                || [0.0f32; 6], // [speed, sense, efficiency, repro, size, energy]
                |mut stats, (_, (genes,))| {
                    stats[0] += genes.speed();
                    stats[1] += genes.sense_radius();
                    stats[2] += genes.energy_efficiency();
                    stats[3] += genes.reproduction_rate();
                    stats[4] += genes.size_factor();
                    stats[5] += 0.0; // Will be calculated separately
                    stats
                },
            )
            .reduce(
                || [0.0f32; 6],
                |mut a, b| {
                    for i in 0..6 {
                        a[i] += b[i];
                    }
                    a
                },
            );

        let avg_energy = world
            .query::<(&Energy,)>()
            .iter()
            .par_bridge()
            .map(|(_, (energy,))| energy.current)
            .sum::<f32>()
            / total_entities as f32;

        EntityMetrics {
            average_energy: avg_energy,
            average_speed: gene_stats[0] / total_entities as f32,
            average_size: gene_stats[4] / total_entities as f32,
            average_reproduction_rate: gene_stats[3] / total_entities as f32,
            average_sense_radius: gene_stats[1] / total_entities as f32,
            average_energy_efficiency: gene_stats[2] / total_entities as f32,
        }
    }

    fn calculate_world_center_drift(world: &World, total_entities: usize) -> (f32, f32) {
        if total_entities == 0 {
            return (0.0, 0.0);
        }

        let (sum_x, sum_y) = world
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

        (sum_x / total_entities as f32, sum_y / total_entities as f32)
    }

    /// Format statistics for console output
    pub fn format_summary(&self, step: u32) -> String {
        let red_count = self
            .entity_counts
            .get(&EntityType::RedDominant)
            .unwrap_or(&0);
        let green_count = self
            .entity_counts
            .get(&EntityType::GreenDominant)
            .unwrap_or(&0);
        let blue_count = self
            .entity_counts
            .get(&EntityType::BlueDominant)
            .unwrap_or(&0);
        let purple_count = self.entity_counts.get(&EntityType::Purple).unwrap_or(&0);
        let mixed_count = self.entity_counts.get(&EntityType::Mixed).unwrap_or(&0);

        format!(
            "Step {}: {} entities (Red:{} Green:{} Blue:{} Purple:{} Mixed:{}) | AvgEnergy:{:.1} AvgSpeed:{:.2} AvgSize:{:.2} AvgRepro:{:.3} | Drift:({:.1}, {:.1})",
            step,
            self.total_entities,
            red_count,
            green_count,
            blue_count,
            purple_count,
            mixed_count,
            self.average_metrics.average_energy,
            self.average_metrics.average_speed,
            self.average_metrics.average_size,
            self.average_metrics.average_reproduction_rate,
            self.world_center_drift.0,
            self.world_center_drift.1,
        )
    }

    /// Format detailed metrics for analysis
    pub fn format_detailed(&self, step: u32) -> String {
        format!(
            "Step {}: Total={}, Density={:.3}, AvgEnergy={:.1}, AvgSpeed={:.2}, AvgSense={:.1}, AvgEfficiency={:.2}, AvgRepro={:.3}, AvgSize={:.2}, Drift=({:.1}, {:.1})",
            step,
            self.total_entities,
            self.population_density,
            self.average_metrics.average_energy,
            self.average_metrics.average_speed,
            self.average_metrics.average_sense_radius,
            self.average_metrics.average_energy_efficiency,
            self.average_metrics.average_reproduction_rate,
            self.average_metrics.average_size,
            self.world_center_drift.0,
            self.world_center_drift.1,
        )
    }
}

#[cfg(test)]
mod tests;
