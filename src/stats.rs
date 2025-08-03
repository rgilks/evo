use crate::components::{Energy, Position};
use crate::genes::Genes;
use hecs::World;
use rayon::prelude::*;
use std::collections::HashMap;
use serde::Serialize;

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
mod tests {
    use super::*;
    use crate::components::{Color, Energy, Position, Size};
    use crate::genes::Genes;
    use hecs::World;
    use rand::thread_rng;

    fn create_test_world() -> World {
        let mut world = World::new();
        let mut rng = thread_rng();

        // Create some test entities with different colors
        for i in 0..10 {
            let genes = Genes::new_random(&mut rng);
            let _color = genes.get_color();

            // Modify genes to create different entity types
            let mut modified_genes = genes.clone();
            match i % 5 {
                0 => {
                    // Red dominant
                    modified_genes.appearance.hue = 0.0;
                    modified_genes.appearance.saturation = 1.0;
                }
                1 => {
                    // Green dominant
                    modified_genes.appearance.hue = 0.33;
                    modified_genes.appearance.saturation = 1.0;
                }
                2 => {
                    // Blue dominant
                    modified_genes.appearance.hue = 0.67;
                    modified_genes.appearance.saturation = 1.0;
                }
                3 => {
                    // Purple
                    modified_genes.appearance.hue = 0.8;
                    modified_genes.appearance.saturation = 0.8;
                }
                _ => {
                    // Mixed
                    modified_genes.appearance.hue = 0.5;
                    modified_genes.appearance.saturation = 0.5;
                }
            }

            world.spawn((
                Position {
                    x: i as f32 * 10.0,
                    y: i as f32 * 10.0,
                },
                Energy {
                    current: 50.0 + i as f32,
                    max: 100.0,
                },
                Size {
                    radius: 5.0 + i as f32 * 0.5,
                },
                modified_genes,
            ));
        }

        world
    }

    #[test]
    fn test_simulation_stats_creation() {
        let world = create_test_world();
        let stats = SimulationStats::from_world(&world, 1000.0, 1.0);

        assert_eq!(stats.total_entities, 10);
        assert!(stats.population_density > 0.0);
        assert!(stats.population_density <= 1.0);
    }

    #[test]
    fn test_entity_classification() {
        let world = create_test_world();
        let entity_counts = SimulationStats::classify_entities(&world);

        // Should have classified all entities
        let total_classified: usize = entity_counts.values().sum();
        assert_eq!(total_classified, 10);

        // Should have at least some different types
        assert!(entity_counts.len() > 1);
    }

    #[test]
    fn test_classify_by_color() {
        // Test red dominant
        let red_color = Color {
            r: 0.8,
            g: 0.2,
            b: 0.2,
        };
        assert_eq!(
            SimulationStats::classify_by_color(&red_color),
            EntityType::RedDominant
        );

        // Test green dominant
        let green_color = Color {
            r: 0.2,
            g: 0.8,
            b: 0.2,
        };
        assert_eq!(
            SimulationStats::classify_by_color(&green_color),
            EntityType::GreenDominant
        );

        // Test blue dominant
        let blue_color = Color {
            r: 0.2,
            g: 0.2,
            b: 0.8,
        };
        assert_eq!(
            SimulationStats::classify_by_color(&blue_color),
            EntityType::BlueDominant
        );

        // Test purple
        let purple_color = Color {
            r: 0.6,
            g: 0.3,
            b: 0.6,
        };
        assert_eq!(
            SimulationStats::classify_by_color(&purple_color),
            EntityType::Purple
        );

        // Test mixed
        let mixed_color = Color {
            r: 0.4,
            g: 0.4,
            b: 0.4,
        };
        assert_eq!(
            SimulationStats::classify_by_color(&mixed_color),
            EntityType::Mixed
        );
    }

    #[test]
    fn test_average_metrics_calculation() {
        let world = create_test_world();
        let metrics = SimulationStats::calculate_average_metrics(&world, 10);

        // All averages should be positive
        assert!(metrics.average_energy > 0.0);
        assert!(metrics.average_speed > 0.0);
        assert!(metrics.average_size > 0.0);
        assert!(metrics.average_reproduction_rate > 0.0);
        assert!(metrics.average_sense_radius > 0.0);
        assert!(metrics.average_energy_efficiency > 0.0);

        // Averages should be reasonable
        assert!(metrics.average_energy <= 100.0);
        assert!(metrics.average_speed <= 2.0);
        assert!(metrics.average_size <= 20.0);
    }

    #[test]
    fn test_world_center_drift_calculation() {
        let world = create_test_world();
        let drift = SimulationStats::calculate_world_center_drift(&world, 10);

        // Drift should be finite
        assert!(drift.0.is_finite());
        assert!(drift.1.is_finite());
    }

    #[test]
    fn test_format_summary() {
        let world = create_test_world();
        let stats = SimulationStats::from_world(&world, 1000.0, 1.0);
        let summary = stats.format_summary(42);

        // Should contain step number
        assert!(summary.contains("42"));

        // Should contain entity count
        assert!(summary.contains("10"));

        // Should contain entity type counts
        assert!(summary.contains("Red:"));
        assert!(summary.contains("Green:"));
        assert!(summary.contains("Blue:"));
    }

        #[test]
    fn test_format_detailed() {
        let world = create_test_world();
        let stats = SimulationStats::from_world(&world, 1000.0, 1.0);
        let detailed = stats.format_detailed(42);

        // Should contain step number
        assert!(detailed.contains("42"));
        
        // Should contain entity count
        assert!(detailed.contains("10"));
        
        // Should contain average metrics
        assert!(detailed.contains("AvgEnergy"));
        assert!(detailed.contains("AvgSpeed"));
        assert!(detailed.contains("AvgSense"));
    }

    #[test]
    fn test_entity_type_equality() {
        let red1 = EntityType::RedDominant;
        let red2 = EntityType::RedDominant;
        let green = EntityType::GreenDominant;

        assert_eq!(red1, red2);
        assert_ne!(red1, green);
    }

    #[test]
    fn test_entity_metrics_clone() {
        let metrics = EntityMetrics {
            average_energy: 50.0,
            average_speed: 1.0,
            average_size: 10.0,
            average_reproduction_rate: 0.05,
            average_sense_radius: 50.0,
            average_energy_efficiency: 1.5,
        };

        let cloned = metrics.clone();
        assert_eq!(metrics.average_energy, cloned.average_energy);
        assert_eq!(metrics.average_speed, cloned.average_speed);
        assert_eq!(metrics.average_size, cloned.average_size);
        assert_eq!(
            metrics.average_reproduction_rate,
            cloned.average_reproduction_rate
        );
        assert_eq!(metrics.average_sense_radius, cloned.average_sense_radius);
        assert_eq!(
            metrics.average_energy_efficiency,
            cloned.average_energy_efficiency
        );
    }

    #[test]
    fn test_simulation_stats_clone() {
        let world = create_test_world();
        let stats = SimulationStats::from_world(&world, 1000.0, 1.0);
        let cloned = stats.clone();

        assert_eq!(stats.total_entities, cloned.total_entities);
        assert_eq!(stats.population_density, cloned.population_density);
        assert_eq!(stats.world_center_drift, cloned.world_center_drift);
        assert_eq!(stats.entity_counts.len(), cloned.entity_counts.len());
    }

    #[test]
    fn test_empty_world_stats() {
        let world = World::new();
        let stats = SimulationStats::from_world(&world, 1000.0, 1.0);

        assert_eq!(stats.total_entities, 0);
        assert_eq!(stats.population_density, 0.0);
        assert_eq!(stats.entity_counts.len(), 0);
    }

    #[test]
    fn test_entity_type_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(EntityType::RedDominant, 1);
        map.insert(EntityType::GreenDominant, 2);
        map.insert(EntityType::BlueDominant, 3);

        assert_eq!(map.get(&EntityType::RedDominant), Some(&1));
        assert_eq!(map.get(&EntityType::GreenDominant), Some(&2));
        assert_eq!(map.get(&EntityType::BlueDominant), Some(&3));
        assert_eq!(map.get(&EntityType::Purple), None);
    }
}
