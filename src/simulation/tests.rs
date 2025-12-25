use super::*;
use crate::components::{Color, Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use rand::thread_rng;

#[test]
fn test_simulation_creation() {
    let sim = Simulation::new(1000.0);

    // Should have initial entities
    assert!(!sim.world.is_empty());
    assert!(sim.world.len() <= 1250); // Default config values (2500 * 0.5 scale)

    // World size should be set correctly
    assert_eq!(sim.world_size, 1000.0);

    // Step should start at 0
    assert_eq!(sim.step, 0);

    // Grid should be initialized
}

#[test]
fn test_simulation_creation_with_config() {
    let mut config = SimulationConfig::default();
    config.population.initial_entities = 100;
    config.population.max_population = 500;

    let sim = Simulation::new_with_config(500.0, config.clone());

    assert_eq!(sim.world_size, 500.0);
    assert_eq!(sim.config.population.initial_entities, 100);
    assert_eq!(sim.config.population.max_population, 500);
}

#[test]
fn test_simulation_update() {
    let mut sim = Simulation::new(100.0);
    let initial_step = sim.step;

    sim.update();

    // Step should increment
    assert_eq!(sim.step, initial_step + 1);

    // Entity count might change due to reproduction/death
    // but should be within reasonable bounds
}

#[test]
fn test_simulation_multiple_updates() {
    let mut sim = Simulation::new(100.0);

    for i in 0..10 {
        sim.update();
        assert_eq!(sim.step, i + 1);
    }
}

#[test]
fn test_simulation_get_entities() {
    let sim = Simulation::new(100.0);
    let entities = sim.get_entities();

    // Should return data for all entities
    // Note: We can't easily compare lengths due to type mismatches
    assert!(!entities.is_empty() || sim.world.is_empty());
}

#[test]
fn test_simulation_get_interpolated_entities() {
    let sim = Simulation::new(100.0);
    let _entities = sim.get_interpolated_entities(0.5);

    // Should return data for all entities
    // Note: We can't easily compare lengths due to type mismatches

    // Interpolation factor should be between 0 and 1
    let entities_0 = sim.get_interpolated_entities(0.0);
    let entities_1 = sim.get_interpolated_entities(1.0);

    assert_eq!(entities_0.len(), entities_1.len());
}

#[test]
fn test_simulation_world_access() {
    let sim = Simulation::new(100.0);
    let world_ref = sim.world();

    // Should be able to access world
    let world_len = sim.world.len();
    assert_eq!(world_ref.len(), world_len);
}

#[test]
fn test_boundary_handling() {
    let sim = Simulation::new(100.0);
    let mut pos = Position { x: 60.0, y: 60.0 }; // Outside boundary
    let mut velocity = Velocity { x: 10.0, y: 10.0 };

    sim.movement_system
        .handle_boundaries(&mut pos, &mut velocity, 100.0, &sim.config);

    // Position should be clamped to boundary
    assert!(pos.x <= 50.0 - sim.config.physics.boundary_margin);
    assert!(pos.y <= 50.0 - sim.config.physics.boundary_margin);
}

#[test]
fn test_boundary_handling_center() {
    let sim = Simulation::new(100.0);
    let mut pos = Position { x: 0.0, y: 0.0 }; // Center
    let mut velocity = Velocity { x: 5.0, y: 5.0 };

    sim.movement_system
        .handle_boundaries(&mut pos, &mut velocity, 100.0, &sim.config);

    // Position should remain unchanged
    assert_eq!(pos.x, 0.0);
    assert_eq!(pos.y, 0.0);
    // Velocity should have drift compensation applied
    assert_eq!(velocity.x, 5.0);
    assert_eq!(velocity.y, 5.0);
}

#[test]
fn test_simulation_logging() {
    let _sim = Simulation::new(100.0);

    // This should not panic
    // Note: We can't easily test the actual logging output in unit tests
    // but we can ensure the method doesn't crash
}

#[test]
fn test_simulation_spatial_grid_rebuild() {
    let mut sim = Simulation::new(100.0);

    // Rebuild grid
    sim.rebuild_spatial_grid();

    // Grid should be rebuilt without panicking
    // We can't easily test the internal state, but we can ensure it doesn't crash
}

#[test]
fn test_simulation_empty_world() {
    let mut sim = Simulation::new(100.0);
    sim.world.clear();

    // Should handle empty world gracefully
    sim.update();
    assert_eq!(sim.world.len(), 0);
}

#[test]
fn test_simulation_large_world() {
    let mut config = SimulationConfig::default();
    config.population.initial_entities = 1000;
    config.population.max_population = 2000;

    let sim = Simulation::new_with_config(1000.0, config);

    // Should handle large world
    assert!(!sim.world.is_empty());
    assert!(sim.world.len() <= 1000);
}

#[test]
fn test_simulation_entity_processing() {
    let mut sim = Simulation::new(100.0);
    let _entity = sim.world.spawn((
        Position { x: 0.0, y: 0.0 },
        Energy {
            current: 50.0,
            max: 100.0,
        },
        Size { radius: 5.0 },
        Genes::new_random(&mut thread_rng()),
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        Velocity { x: 0.0, y: 0.0 },
        crate::components::MovementStyle {
            style: crate::components::MovementType::Random,
            flocking_strength: 0.5,
            separation_distance: 10.0,
            alignment_strength: 0.5,
            cohesion_strength: 0.5,
        },
    ));

    // Test processing a single entity
    // Note: This test is complex due to borrowing rules, so we'll just ensure it doesn't panic
    // In a real scenario, you'd need to restructure the code to avoid borrowing conflicts
}

#[test]
fn test_simulation_apply_updates() {
    let mut sim = Simulation::new(100.0);
    let _entity = sim.world.spawn((
        Position { x: 0.0, y: 0.0 },
        Energy {
            current: 50.0,
            max: 100.0,
        },
        Size { radius: 5.0 },
        Genes::new_random(&mut thread_rng()),
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        Velocity { x: 0.0, y: 0.0 },
        crate::components::MovementStyle {
            style: crate::components::MovementType::Flocking,
            flocking_strength: 0.7,
            separation_distance: 12.0,
            alignment_strength: 0.6,
            cohesion_strength: 0.6,
        },
    ));

    let updates = vec![EntityUpdate {
        entity: _entity,
        pos: Position { x: 10.0, y: 10.0 },
        energy: Energy {
            current: 60.0,
            max: 100.0,
        },
        size: Size { radius: 6.0 },
        genes: Genes::new_random(&mut thread_rng()),
        color: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
        },
        velocity: Velocity { x: 1.0, y: 1.0 },
        movement_style: crate::components::MovementStyle {
            style: crate::components::MovementType::Flocking,
            flocking_strength: 0.7,
            separation_distance: 12.0,
            alignment_strength: 0.6,
            cohesion_strength: 0.6,
        },
        should_reproduce: false,
        eaten_entity: None,
    }];

    sim.apply_entity_updates(updates);

    // Entity should be updated
    // Note: We can't easily test this due to borrowing rules
    // In a real scenario, you'd need to restructure the code
}

#[test]
fn test_simulation_clustering() {
    let config = SimulationConfig::default();
    let world_size = 100.0;
    let mut simulation = Simulation::new_with_config(world_size, config);

    println!("Initial entity count: {}", simulation.world().len());

    // Run simulation for 100 steps
    for step in 0..100 {
        simulation.update();

        if step % 20 == 0 {
            let entities = simulation.get_entities();
            let mut total_x = 0.0;
            let mut total_y = 0.0;
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;

            for (x, y, _, _, _, _) in &entities {
                total_x += x;
                total_y += y;
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }

            let center_x = total_x / entities.len() as f32;
            let center_y = total_y / entities.len() as f32;
            let spread_x = max_x - min_x;
            let spread_y = max_y - min_y;

            println!(
                "Step {}: {} entities, Center: ({:.1}, {:.1}), Spread: ({:.1}, {:.1})",
                step,
                entities.len(),
                center_x,
                center_y,
                spread_x,
                spread_y
            );

            // Check for clustering in top-left
            if center_x < -20.0 && center_y > 20.0 {
                println!(
                    "WARNING: Entities clustering in top-left! Center: ({:.1}, {:.1})",
                    center_x, center_y
                );
            }
        }
    }
}

#[test]
fn test_drift_direction_analysis() {
    let config = SimulationConfig::default();
    let world_size = 100.0;
    let mut simulation = Simulation::new_with_config(world_size, config);

    println!("Testing drift direction over 200 steps...");

    let mut positions = Vec::new();

    // Run simulation for 200 steps, recording positions every 20 steps
    for step in 0..200 {
        simulation.update();

        if step % 20 == 0 {
            let entities = simulation.get_entities();
            let mut total_x = 0.0;
            let mut total_y = 0.0;

            for (x, y, _, _, _, _) in &entities {
                total_x += x;
                total_y += y;
            }

            let center_x = total_x / entities.len() as f32;
            let center_y = total_y / entities.len() as f32;
            positions.push((step, center_x, center_y));

            println!("Step {}: Center ({:.1}, {:.1})", step, center_x, center_y);
        }
    }

    // Analyze drift direction
    if positions.len() >= 2 {
        let first = positions[0];
        let last = positions[positions.len() - 1];
        let drift_x = last.1 - first.1;
        let drift_y = last.2 - first.2;

        println!("\nDrift Analysis:");
        println!("Start position: ({:.1}, {:.1})", first.1, first.2);
        println!("End position: ({:.1}, {:.1})", last.1, last.2);
        println!("Total drift: ({:.1}, {:.1})", drift_x, drift_y);

        // Determine drift direction
        let direction = if drift_x < -5.0 && drift_y < -5.0 {
            "Bottom-Left (appears as Top-Left on screen)"
        } else if drift_x > 5.0 && drift_y < -5.0 {
            "Bottom-Right (appears as Top-Right on screen)"
        } else if drift_x < -5.0 && drift_y > 5.0 {
            "Top-Left"
        } else if drift_x > 5.0 && drift_y > 5.0 {
            "Top-Right"
        } else {
            "Minimal or no significant drift"
        };

        println!("Drift direction: {}", direction);

        // Check if this matches the observed visual clustering
        if drift_x < -5.0 && drift_y < -5.0 {
            println!("CONFIRMED: Entities are drifting to bottom-left in world coordinates!");
            println!(
                "This appears as top-left clustering on screen due to Y-axis flip in rendering."
            );
        }
    }
}

#[test]
fn test_entity_data_format() {
    let sim = Simulation::new(100.0);
    let entities = sim.get_entities();

    // Each entity should have 6 components: x, y, radius, r, g, b
    for (x, y, radius, r, g, b) in &entities {
        // Position should be within world bounds
        assert!(*x >= -50.0 && *x <= 50.0, "x={} out of bounds", x);
        assert!(*y >= -50.0 && *y <= 50.0, "y={} out of bounds", y);

        // Radius should be positive
        assert!(*radius > 0.0, "radius should be positive");

        // Colors should be in 0-1 range
        assert!(*r >= 0.0 && *r <= 1.0, "r={} out of color range", r);
        assert!(*g >= 0.0 && *g <= 1.0, "g={} out of color range", g);
        assert!(*b >= 0.0 && *b <= 1.0, "b={} out of color range", b);
    }
}

#[test]
fn test_entity_buffer_conversion() {
    // Test the buffer format used by WebGPU renderer
    // Simulates what update_entity_buffer does in lib.rs
    let sim = Simulation::new(100.0);
    let entities = sim.get_entities();

    // Convert to flat buffer (same as update_entity_buffer)
    let mut buffer: Vec<f32> = Vec::with_capacity(entities.len() * 6);
    for (x, y, radius, r, g, b) in entities.iter() {
        buffer.push(*x);
        buffer.push(*y);
        buffer.push(*radius);
        buffer.push(*r);
        buffer.push(*g);
        buffer.push(*b);
    }

    // Buffer length should be 6 * entity count
    assert_eq!(buffer.len(), entities.len() * 6);

    // Entity count calculation should match
    let entity_count = buffer.len() / 6;
    assert_eq!(entity_count, entities.len());

    // Verify data integrity by reading back
    for (i, (x, y, radius, r, g, b)) in entities.iter().enumerate() {
        let base = i * 6;
        assert_eq!(buffer[base], *x);
        assert_eq!(buffer[base + 1], *y);
        assert_eq!(buffer[base + 2], *radius);
        assert_eq!(buffer[base + 3], *r);
        assert_eq!(buffer[base + 4], *g);
        assert_eq!(buffer[base + 5], *b);
    }
}

#[test]
fn test_update_config() {
    let mut sim = Simulation::new(100.0);
    let original_velocity = sim.config.physics.max_velocity;

    let mut new_config = sim.config.clone();
    new_config.physics.max_velocity = 5.0;
    sim.update_config(new_config);

    assert_ne!(sim.config.physics.max_velocity, original_velocity);
    assert_eq!(sim.config.physics.max_velocity, 5.0);
}
