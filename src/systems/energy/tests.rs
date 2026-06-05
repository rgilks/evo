use super::*;
use crate::components::Size;
use crate::config::SimulationConfig;
use crate::genes::Genes;
use rand::thread_rng;

#[test]
fn test_energy_system_update_energy() {
    let system = EnergySystem;
    let mut new_energy = 50.0;
    let size = Size { radius: 10.0 };
    let mut rng = thread_rng();
    let genes = Genes::new_random(&mut rng);
    let config = SimulationConfig::default();

    system.update_energy(&mut new_energy, &size, &genes, &config, 0.5);

    // Energy should have changed due to loss and gain
    assert_ne!(new_energy, 50.0);
}

#[test]
fn test_energy_system_calculate_new_size() {
    let system = EnergySystem;
    let energy = 80.0;
    let mut rng = thread_rng();
    let genes = Genes::new_random(&mut rng);
    let config = SimulationConfig::default();

    let new_size = system.calculate_new_size(energy, &genes, &config);

    // Size should be positive and reasonable
    assert!(new_size > 0.0);
    assert!(new_size <= config.physics.max_entity_radius);
}

#[test]
fn test_energy_system_energy_bounds() {
    let system = EnergySystem;
    let mut new_energy = 0.0; // Start with no energy
    let size = Size { radius: 10.0 };
    let mut rng = thread_rng();
    let genes = Genes::new_random(&mut rng);
    let config = SimulationConfig::default();

    system.update_energy(&mut new_energy, &size, &genes, &config, 0.5);

    // Energy can go below 0 due to energy loss, but should be finite
    assert!(new_energy.is_finite());
}
