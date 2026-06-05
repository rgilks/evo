use super::*;

#[test]
fn test_default_config() {
    let config = SimulationConfig::default();

    assert_eq!(config.population.entity_scale, 0.5);
    assert_eq!(config.population.max_population, 10000);
    assert_eq!(config.population.initial_entities, 2500);
    assert_eq!(config.population.spawn_radius_factor, 0.2);
    assert_eq!(config.physics.max_velocity, 2.0);
    assert_eq!(config.physics.max_entity_radius, 20.0);
    assert_eq!(config.physics.min_entity_radius, 1.0);
    assert_eq!(config.physics.grid_cell_size, 25.0);
    assert_eq!(config.physics.boundary_margin, 5.0);
    assert_eq!(config.physics.interaction_radius_offset, 6.0);
    assert_eq!(config.physics.velocity_bounce_factor, 0.8);
    assert_eq!(config.physics.edge_repulsion_strength, 0.3);
    assert_eq!(config.physics.particle_force_scale, 0.15);
    assert_eq!(config.physics.particle_friction, 0.95);
    assert_eq!(config.energy.size_energy_cost_factor, 0.15);
    assert_eq!(config.energy.movement_energy_cost, 0.1);
    assert_eq!(config.energy.ambient_energy_gain, 1.3);
    assert_eq!(config.energy.predator_graze_fraction, 0.6);
    assert_eq!(config.energy.predator_upkeep, 0.0);
    assert_eq!(config.reproduction.reproduction_energy_threshold, 0.6);
    assert_eq!(config.reproduction.reproduction_energy_cost, 0.7);
    assert_eq!(config.reproduction.child_energy_factor, 0.4);
    assert_eq!(config.reproduction.child_spawn_radius, 15.0);
    assert_eq!(config.reproduction.population_density_factor, 0.8);
    assert_eq!(config.reproduction.min_reproduction_chance, 0.05);
    assert_eq!(config.reproduction.death_chance_factor, 0.04);
    assert_eq!(config.reproduction.crowding_pressure_rate, 0.006);
    assert_eq!(config.reproduction.death_floor_density, 0.03);
    assert_eq!(config.reproduction.hue_crowding_factor, 1.2);
}

#[test]
fn test_config_serialization_round_trip() {
    let config = SimulationConfig::default();
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: SimulationConfig = serde_json::from_str(&serialized).unwrap();
    // PartialEq makes this exhaustive: a new field cannot silently escape coverage.
    assert_eq!(config, deserialized);
}

#[test]
fn test_config_clone() {
    let config = SimulationConfig::default();
    assert_eq!(config, config.clone());
}

#[test]
fn test_custom_config_values() {
    let mut config = SimulationConfig::default();

    config.population.entity_scale = 1.0;
    config.population.max_population = 1000;
    config.population.initial_entities = 200;
    config.population.spawn_radius_factor = 0.3;
    config.physics.max_velocity = 3.0;
    config.physics.max_entity_radius = 25.0;
    config.physics.min_entity_radius = 2.0;
    config.physics.grid_cell_size = 30.0;
    config.physics.boundary_margin = 10.0;
    config.physics.interaction_radius_offset = 20.0;
    config.physics.velocity_bounce_factor = 0.9;
    config.physics.edge_repulsion_strength = 1.0;
    config.physics.particle_force_scale = 0.5;
    config.physics.particle_friction = 0.9;
    config.energy.size_energy_cost_factor = 0.2;
    config.energy.movement_energy_cost = 0.15;
    config.energy.ambient_energy_gain = 1.5;
    config.energy.predator_graze_fraction = 0.5;
    config.energy.predator_upkeep = 0.4;
    config.reproduction.hue_crowding_factor = 2.0;
    config.reproduction.crowding_pressure_rate = 0.1;
    config.reproduction.death_floor_density = 0.05;
    config.reproduction.reproduction_energy_threshold = 0.9;
    config.reproduction.reproduction_energy_cost = 0.8;
    config.reproduction.child_energy_factor = 0.5;
    config.reproduction.child_spawn_radius = 20.0;
    config.reproduction.population_density_factor = 0.9;
    config.reproduction.min_reproduction_chance = 0.1;
    config.reproduction.death_chance_factor = 0.2;

    assert_eq!(config.population.entity_scale, 1.0);
    assert_eq!(config.population.max_population, 1000);
    assert_eq!(config.population.initial_entities, 200);
    assert_eq!(config.population.spawn_radius_factor, 0.3);
    assert_eq!(config.physics.max_velocity, 3.0);
    assert_eq!(config.physics.max_entity_radius, 25.0);
    assert_eq!(config.physics.min_entity_radius, 2.0);
    assert_eq!(config.physics.grid_cell_size, 30.0);
    assert_eq!(config.physics.boundary_margin, 10.0);
    assert_eq!(config.physics.interaction_radius_offset, 20.0);
    assert_eq!(config.physics.velocity_bounce_factor, 0.9);
    assert_eq!(config.physics.edge_repulsion_strength, 1.0);
    assert_eq!(config.physics.particle_force_scale, 0.5);
    assert_eq!(config.physics.particle_friction, 0.9);
    assert_eq!(config.energy.size_energy_cost_factor, 0.2);
    assert_eq!(config.energy.movement_energy_cost, 0.15);
    assert_eq!(config.energy.ambient_energy_gain, 1.5);
    assert_eq!(config.energy.predator_graze_fraction, 0.5);
    assert_eq!(config.energy.predator_upkeep, 0.4);
    assert_eq!(config.reproduction.hue_crowding_factor, 2.0);
    assert_eq!(config.reproduction.reproduction_energy_threshold, 0.9);
    assert_eq!(config.reproduction.reproduction_energy_cost, 0.8);
    assert_eq!(config.reproduction.child_energy_factor, 0.5);
    assert_eq!(config.reproduction.child_spawn_radius, 20.0);
    assert_eq!(config.reproduction.population_density_factor, 0.9);
    assert_eq!(config.reproduction.min_reproduction_chance, 0.1);
    assert_eq!(config.reproduction.death_chance_factor, 0.2);
    assert_eq!(config.reproduction.crowding_pressure_rate, 0.1);
    assert_eq!(config.reproduction.death_floor_density, 0.05);
}

#[test]
fn test_config_debug_format() {
    let config = SimulationConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("population"));
    assert!(debug_str.contains("physics"));
    assert!(debug_str.contains("energy"));
    assert!(debug_str.contains("reproduction"));
}
