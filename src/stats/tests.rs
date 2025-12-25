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
