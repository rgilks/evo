use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub entity_scale: f32,
    pub max_population: u32,
    pub initial_entities: usize,
    pub spawn_radius_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub max_velocity: f32,
    pub max_entity_radius: f32,
    pub min_entity_radius: f32,
    pub grid_cell_size: f32,
    pub boundary_margin: f32,
    pub interaction_radius_offset: f32,
    pub velocity_bounce_factor: f32,
    pub center_pressure_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConfig {
    pub size_energy_cost_factor: f32,
    pub movement_energy_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproductionConfig {
    pub reproduction_energy_threshold: f32,
    pub reproduction_energy_cost: f32,
    pub child_energy_factor: f32,
    pub child_spawn_radius: f32,
    pub population_density_factor: f32,
    pub min_reproduction_chance: f32,
    pub death_chance_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub population: PopulationConfig,
    pub physics: PhysicsConfig,
    pub energy: EnergyConfig,
    pub reproduction: ReproductionConfig,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            population: PopulationConfig {
                entity_scale: 0.5,
                max_population: 2000,
                initial_entities: 500,
                spawn_radius_factor: 0.2,
            },
            physics: PhysicsConfig {
                max_velocity: 2.0,
                max_entity_radius: 20.0,
                min_entity_radius: 1.0,
                grid_cell_size: 25.0,
                boundary_margin: 5.0,
                interaction_radius_offset: 15.0,
                velocity_bounce_factor: 0.8,
                center_pressure_strength: 0.3,
            },
            energy: EnergyConfig {
                size_energy_cost_factor: 0.15,
                movement_energy_cost: 0.1,
            },
            reproduction: ReproductionConfig {
                reproduction_energy_threshold: 0.8,
                reproduction_energy_cost: 0.7,
                child_energy_factor: 0.4,
                child_spawn_radius: 15.0,
                population_density_factor: 0.8,
                min_reproduction_chance: 0.05,
                death_chance_factor: 0.1,
            },
        }
    }
}

impl SimulationConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: SimulationConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn create_default_config_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let default_config = SimulationConfig::default();
        default_config.save_to_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = SimulationConfig::default();

        // Test default values
        assert_eq!(config.population.entity_scale, 0.5);
        assert_eq!(config.population.max_population, 2000);
        assert_eq!(config.population.initial_entities, 500);
        assert_eq!(config.population.spawn_radius_factor, 0.2);
        assert_eq!(config.physics.max_velocity, 2.0);
        assert_eq!(config.physics.max_entity_radius, 20.0);
        assert_eq!(config.physics.min_entity_radius, 1.0);
        assert_eq!(config.physics.grid_cell_size, 25.0);
        assert_eq!(config.physics.boundary_margin, 5.0);
        assert_eq!(config.physics.interaction_radius_offset, 15.0);
        assert_eq!(config.physics.velocity_bounce_factor, 0.8);
        assert_eq!(config.energy.size_energy_cost_factor, 0.15);
        assert_eq!(config.energy.movement_energy_cost, 0.1);
        assert_eq!(config.reproduction.reproduction_energy_threshold, 0.8);
        assert_eq!(config.reproduction.reproduction_energy_cost, 0.7);
        assert_eq!(config.reproduction.child_energy_factor, 0.4);
        assert_eq!(config.reproduction.child_spawn_radius, 15.0);
        assert_eq!(config.reproduction.population_density_factor, 0.8);
        assert_eq!(config.reproduction.min_reproduction_chance, 0.05);
        assert_eq!(config.reproduction.death_chance_factor, 0.1);
    }

    #[test]
    fn test_config_serialization() {
        let config = SimulationConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SimulationConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            config.population.entity_scale,
            deserialized.population.entity_scale
        );
        assert_eq!(
            config.population.max_population,
            deserialized.population.max_population
        );
        assert_eq!(
            config.population.initial_entities,
            deserialized.population.initial_entities
        );
        assert_eq!(
            config.population.spawn_radius_factor,
            deserialized.population.spawn_radius_factor
        );
        assert_eq!(
            config.physics.max_velocity,
            deserialized.physics.max_velocity
        );
        assert_eq!(
            config.physics.max_entity_radius,
            deserialized.physics.max_entity_radius
        );
        assert_eq!(
            config.physics.min_entity_radius,
            deserialized.physics.min_entity_radius
        );
        assert_eq!(
            config.physics.grid_cell_size,
            deserialized.physics.grid_cell_size
        );
        assert_eq!(
            config.physics.boundary_margin,
            deserialized.physics.boundary_margin
        );
        assert_eq!(
            config.physics.interaction_radius_offset,
            deserialized.physics.interaction_radius_offset
        );
        assert_eq!(
            config.physics.velocity_bounce_factor,
            deserialized.physics.velocity_bounce_factor
        );
        assert_eq!(
            config.energy.size_energy_cost_factor,
            deserialized.energy.size_energy_cost_factor
        );
        assert_eq!(
            config.energy.movement_energy_cost,
            deserialized.energy.movement_energy_cost
        );
        assert_eq!(
            config.reproduction.reproduction_energy_threshold,
            deserialized.reproduction.reproduction_energy_threshold
        );
        assert_eq!(
            config.reproduction.reproduction_energy_cost,
            deserialized.reproduction.reproduction_energy_cost
        );
        assert_eq!(
            config.reproduction.child_energy_factor,
            deserialized.reproduction.child_energy_factor
        );
        assert_eq!(
            config.reproduction.child_spawn_radius,
            deserialized.reproduction.child_spawn_radius
        );
        assert_eq!(
            config.reproduction.population_density_factor,
            deserialized.reproduction.population_density_factor
        );
        assert_eq!(
            config.reproduction.min_reproduction_chance,
            deserialized.reproduction.min_reproduction_chance
        );
        assert_eq!(
            config.reproduction.death_chance_factor,
            deserialized.reproduction.death_chance_factor
        );
    }

    #[test]
    fn test_config_clone() {
        let config = SimulationConfig::default();
        let cloned = config.clone();

        assert_eq!(
            config.population.entity_scale,
            cloned.population.entity_scale
        );
        assert_eq!(
            config.population.max_population,
            cloned.population.max_population
        );
        assert_eq!(
            config.population.initial_entities,
            cloned.population.initial_entities
        );
        assert_eq!(
            config.population.spawn_radius_factor,
            cloned.population.spawn_radius_factor
        );
        assert_eq!(config.physics.max_velocity, cloned.physics.max_velocity);
        assert_eq!(
            config.physics.max_entity_radius,
            cloned.physics.max_entity_radius
        );
        assert_eq!(
            config.physics.min_entity_radius,
            cloned.physics.min_entity_radius
        );
        assert_eq!(config.physics.grid_cell_size, cloned.physics.grid_cell_size);
        assert_eq!(
            config.physics.boundary_margin,
            cloned.physics.boundary_margin
        );
        assert_eq!(
            config.physics.interaction_radius_offset,
            cloned.physics.interaction_radius_offset
        );
        assert_eq!(
            config.physics.velocity_bounce_factor,
            cloned.physics.velocity_bounce_factor
        );
        assert_eq!(
            config.energy.size_energy_cost_factor,
            cloned.energy.size_energy_cost_factor
        );
        assert_eq!(
            config.energy.movement_energy_cost,
            cloned.energy.movement_energy_cost
        );
        assert_eq!(
            config.reproduction.reproduction_energy_threshold,
            cloned.reproduction.reproduction_energy_threshold
        );
        assert_eq!(
            config.reproduction.reproduction_energy_cost,
            cloned.reproduction.reproduction_energy_cost
        );
        assert_eq!(
            config.reproduction.child_energy_factor,
            cloned.reproduction.child_energy_factor
        );
        assert_eq!(
            config.reproduction.child_spawn_radius,
            cloned.reproduction.child_spawn_radius
        );
        assert_eq!(
            config.reproduction.population_density_factor,
            cloned.reproduction.population_density_factor
        );
        assert_eq!(
            config.reproduction.min_reproduction_chance,
            cloned.reproduction.min_reproduction_chance
        );
        assert_eq!(
            config.reproduction.death_chance_factor,
            cloned.reproduction.death_chance_factor
        );
    }

    #[test]
    fn test_save_and_load_config() {
        let config = SimulationConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Test save
        let save_result = config.save_to_file(path);
        assert!(save_result.is_ok());

        // Test load
        let load_result = SimulationConfig::load_from_file(path);
        assert!(load_result.is_ok());

        let loaded_config = load_result.unwrap();
        assert_eq!(
            config.population.entity_scale,
            loaded_config.population.entity_scale
        );
        assert_eq!(
            config.population.max_population,
            loaded_config.population.max_population
        );
        assert_eq!(
            config.population.initial_entities,
            loaded_config.population.initial_entities
        );
        assert_eq!(
            config.population.spawn_radius_factor,
            loaded_config.population.spawn_radius_factor
        );
        assert_eq!(
            config.physics.max_velocity,
            loaded_config.physics.max_velocity
        );
        assert_eq!(
            config.physics.max_entity_radius,
            loaded_config.physics.max_entity_radius
        );
        assert_eq!(
            config.physics.min_entity_radius,
            loaded_config.physics.min_entity_radius
        );
        assert_eq!(
            config.physics.grid_cell_size,
            loaded_config.physics.grid_cell_size
        );
        assert_eq!(
            config.physics.boundary_margin,
            loaded_config.physics.boundary_margin
        );
        assert_eq!(
            config.physics.interaction_radius_offset,
            loaded_config.physics.interaction_radius_offset
        );
        assert_eq!(
            config.physics.velocity_bounce_factor,
            loaded_config.physics.velocity_bounce_factor
        );
        assert_eq!(
            config.energy.size_energy_cost_factor,
            loaded_config.energy.size_energy_cost_factor
        );
        assert_eq!(
            config.energy.movement_energy_cost,
            loaded_config.energy.movement_energy_cost
        );
        assert_eq!(
            config.reproduction.reproduction_energy_threshold,
            loaded_config.reproduction.reproduction_energy_threshold
        );
        assert_eq!(
            config.reproduction.reproduction_energy_cost,
            loaded_config.reproduction.reproduction_energy_cost
        );
        assert_eq!(
            config.reproduction.child_energy_factor,
            loaded_config.reproduction.child_energy_factor
        );
        assert_eq!(
            config.reproduction.child_spawn_radius,
            loaded_config.reproduction.child_spawn_radius
        );
        assert_eq!(
            config.reproduction.population_density_factor,
            loaded_config.reproduction.population_density_factor
        );
        assert_eq!(
            config.reproduction.min_reproduction_chance,
            loaded_config.reproduction.min_reproduction_chance
        );
        assert_eq!(
            config.reproduction.death_chance_factor,
            loaded_config.reproduction.death_chance_factor
        );
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = SimulationConfig::load_from_file("nonexistent_file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write invalid JSON
        fs::write(path, "invalid json content").unwrap();

        let result = SimulationConfig::load_from_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_default_config_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let result = SimulationConfig::create_default_config_file(path);
        assert!(result.is_ok());

        // Verify the file was created and contains valid JSON
        let load_result = SimulationConfig::load_from_file(path);
        assert!(load_result.is_ok());
    }

    #[test]
    fn test_custom_config_values() {
        let mut config = SimulationConfig::default();

        // Modify some values
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
        config.energy.size_energy_cost_factor = 0.2;
        config.energy.movement_energy_cost = 0.15;
        config.reproduction.reproduction_energy_threshold = 0.9;
        config.reproduction.reproduction_energy_cost = 0.8;
        config.reproduction.child_energy_factor = 0.5;
        config.reproduction.child_spawn_radius = 20.0;
        config.reproduction.population_density_factor = 0.9;
        config.reproduction.min_reproduction_chance = 0.1;
        config.reproduction.death_chance_factor = 0.2;

        // Test that values were set correctly
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
        assert_eq!(config.energy.size_energy_cost_factor, 0.2);
        assert_eq!(config.energy.movement_energy_cost, 0.15);
        assert_eq!(config.reproduction.reproduction_energy_threshold, 0.9);
        assert_eq!(config.reproduction.reproduction_energy_cost, 0.8);
        assert_eq!(config.reproduction.child_energy_factor, 0.5);
        assert_eq!(config.reproduction.child_spawn_radius, 20.0);
        assert_eq!(config.reproduction.population_density_factor, 0.9);
        assert_eq!(config.reproduction.min_reproduction_chance, 0.1);
        assert_eq!(config.reproduction.death_chance_factor, 0.2);
    }

    #[test]
    fn test_config_debug_format() {
        let config = SimulationConfig::default();
        let debug_str = format!("{:?}", config);

        // Should contain some key fields
        assert!(debug_str.contains("population"));
        assert!(debug_str.contains("physics"));
        assert!(debug_str.contains("energy"));
        assert!(debug_str.contains("reproduction"));
    }
}
