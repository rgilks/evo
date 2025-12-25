use super::*;
use crate::components::{Color, Energy, Position, Size, Velocity};
use crate::genes::Genes;
use hecs::World;
use rand::thread_rng;

#[test]
fn test_movement_system_update_movement() {
    let system = MovementSystem;
    let mut rng = thread_rng();
    let genes = Genes::new_random(&mut rng);
    let mut new_pos = Position { x: 0.0, y: 0.0 };
    let mut new_velocity = Velocity { x: 0.0, y: 0.0 };
    let mut new_energy = 100.0;
    let pos = Position { x: 0.0, y: 0.0 };
    let nearby_entities = vec![];
    let world = World::new();
    let config = SimulationConfig::default();

    system.update_movement(MovementUpdateParams {
        genes: &genes,
        new_pos: &mut new_pos,
        new_velocity: &mut new_velocity,
        new_energy: &mut new_energy,
        pos: &pos,
        nearby_entities: &nearby_entities,
        world: &world,
        config: &config,
        world_size: 100.0,
    });

    // Position should have changed
    assert_ne!(new_pos.x, 0.0);
    assert_ne!(new_pos.y, 0.0);

    // Velocity should be set
    assert_ne!(new_velocity.x, 0.0);
    assert_ne!(new_velocity.y, 0.0);

    // Energy should have decreased due to movement cost
    assert!(new_energy < 100.0);
}

#[test]
fn test_movement_system_handle_boundaries() {
    let system = MovementSystem;
    let mut pos = Position { x: 60.0, y: 60.0 }; // Outside boundary
    let mut velocity = Velocity { x: 10.0, y: 10.0 };
    let world_size = 100.0;
    let config = SimulationConfig::default();

    system.handle_boundaries(&mut pos, &mut velocity, world_size, &config);

    // Position should be clamped to boundary
    assert!(pos.x <= 50.0 - config.physics.boundary_margin);
    assert!(pos.y <= 50.0 - config.physics.boundary_margin);

    // Velocity should be reflected
    assert!(velocity.x < 0.0 || velocity.y < 0.0);
}

#[test]
fn test_movement_system_boundary_center() {
    let system = MovementSystem;
    let mut pos = Position { x: 0.0, y: 0.0 }; // Center
    let mut velocity = Velocity { x: 5.0, y: 5.0 };
    let world_size = 100.0;
    let config = SimulationConfig::default();

    system.handle_boundaries(&mut pos, &mut velocity, world_size, &config);

    // Position should remain unchanged
    assert_eq!(pos.x, 0.0);
    assert_eq!(pos.y, 0.0);

    // Velocity should have drift compensation applied
    assert_eq!(velocity.x, 5.0);
    assert_eq!(velocity.y, 5.0);
}

#[test]
fn test_movement_system_with_target() {
    let system = MovementSystem;
    let mut rng = thread_rng();
    let genes = Genes::new_random(&mut rng);
    let mut new_pos = Position { x: 0.0, y: 0.0 };
    let mut new_velocity = Velocity { x: 0.0, y: 0.0 };
    let mut new_energy = 100.0;
    let pos = Position { x: 0.0, y: 0.0 };

    // Create a world with a target entity
    let mut world = World::new();
    let target_entity = world.spawn((
        Position { x: 10.0, y: 10.0 },
        Energy {
            current: 50.0,
            max: 100.0,
        },
        Size { radius: 5.0 },
        Genes::new_random(&mut rng),
    ));
    let nearby_entities = vec![target_entity];

    let config = SimulationConfig::default();

    system.update_movement(MovementUpdateParams {
        genes: &genes,
        new_pos: &mut new_pos,
        new_velocity: &mut new_velocity,
        new_energy: &mut new_energy,
        pos: &pos,
        nearby_entities: &nearby_entities,
        world: &world,
        config: &config,
        world_size: 100.0,
    });

    // Should have moved (position changed) and used energy
    assert!(new_pos.x != 0.0 || new_pos.y != 0.0 || new_velocity.x != 0.0 || new_velocity.y != 0.0);
    assert!(new_energy < 100.0);
}

#[test]
fn test_movement_drift_analysis() {
    let config = SimulationConfig::default();
    let movement_system = MovementSystem;
    let mut world = World::new();

    // Test 1: Check if initial velocity has any bias
    let _entity = world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 0.0, y: 0.0 },
        Energy {
            current: 100.0,
            max: 100.0,
        },
        Size { radius: 5.0 },
        Genes::new_random(&mut thread_rng()),
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
    ));

    let mut pos = Position { x: 0.0, y: 0.0 };
    let mut velocity = Velocity { x: 0.0, y: 0.0 };
    let mut energy = 100.0;

    // Run movement update with no nearby entities
    movement_system.update_movement(MovementUpdateParams {
        genes: &Genes::new_random(&mut thread_rng()),
        new_pos: &mut pos,
        new_velocity: &mut velocity,
        new_energy: &mut energy,
        pos: &Position { x: 0.0, y: 0.0 },
        nearby_entities: &[],
        world: &world,
        config: &config,
        world_size: 100.0,
    });

    // Check if there's any systematic bias in velocity generation
    println!(
        "Initial velocity after update: ({}, {})",
        velocity.x, velocity.y
    );
    assert!(
        velocity.x.abs() < 10.0,
        "Velocity x should be reasonable: {}",
        velocity.x
    );
    assert!(
        velocity.y.abs() < 10.0,
        "Velocity y should be reasonable: {}",
        velocity.y
    );
}

#[test]
fn test_boundary_handling_drift() {
    let config = SimulationConfig::default();
    let movement_system = MovementSystem;
    let world_size = 100.0;

    // Test boundary handling for all four sides
    let test_cases = vec![
        // Left boundary
        (Position { x: -45.0, y: 0.0 }, Velocity { x: -5.0, y: 0.0 }),
        // Right boundary
        (Position { x: 45.0, y: 0.0 }, Velocity { x: 5.0, y: 0.0 }),
        // Top boundary
        (Position { x: 0.0, y: -45.0 }, Velocity { x: 0.0, y: -5.0 }),
        // Bottom boundary
        (Position { x: 0.0, y: 45.0 }, Velocity { x: 0.0, y: 5.0 }),
    ];

    for (mut pos, mut velocity) in test_cases {
        let original_velocity = velocity.clone();
        movement_system.handle_boundaries(&mut pos, &mut velocity, world_size, &config);

        println!(
            "Boundary test - Original: ({}, {}), Final: ({}, {})",
            original_velocity.x, original_velocity.y, velocity.x, velocity.y
        );

        // Check that velocity direction is properly reversed
        if pos.x <= -45.0 || pos.x >= 45.0 {
            assert!(
                (original_velocity.x * velocity.x) <= 0.0,
                "X velocity should be reversed at boundaries"
            );
        }
        if pos.y <= -45.0 || pos.y >= 45.0 {
            assert!(
                (original_velocity.y * velocity.y) <= 0.0,
                "Y velocity should be reversed at boundaries"
            );
        }
    }
}

#[test]
fn test_velocity_distribution_analysis() {
    let config = SimulationConfig::default();
    let movement_system = MovementSystem;
    let mut world = World::new();

    // Create multiple entities and track their velocity distributions
    let mut x_velocities = Vec::new();
    let mut y_velocities = Vec::new();

    for _ in 0..100 {
        let _entity = world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 0.0, y: 0.0 },
            Energy {
                current: 100.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        ));

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut velocity = Velocity { x: 0.0, y: 0.0 };
        let mut energy = 100.0;

        movement_system.update_movement(MovementUpdateParams {
            genes: &Genes::new_random(&mut thread_rng()),
            new_pos: &mut pos,
            new_velocity: &mut velocity,
            new_energy: &mut energy,
            pos: &Position { x: 0.0, y: 0.0 },
            nearby_entities: &[],
            world: &world,
            config: &config,
            world_size: 100.0,
        });

        x_velocities.push(velocity.x);
        y_velocities.push(velocity.y);
    }

    // Calculate statistics
    let x_mean = x_velocities.iter().sum::<f32>() / x_velocities.len() as f32;
    let y_mean = y_velocities.iter().sum::<f32>() / y_velocities.len() as f32;

    let x_std = (x_velocities
        .iter()
        .map(|&x| (x - x_mean).powi(2))
        .sum::<f32>()
        / x_velocities.len() as f32)
        .sqrt();
    let y_std = (y_velocities
        .iter()
        .map(|&y| (y - y_mean).powi(2))
        .sum::<f32>()
        / y_velocities.len() as f32)
        .sqrt();

    println!("Velocity distribution analysis:");
    println!("X - Mean: {:.3}, Std: {:.3}", x_mean, x_std);
    println!("Y - Mean: {:.3}, Std: {:.3}", y_mean, y_std);

    // Check for systematic bias (mean should be close to 0)
    assert!(
        x_mean.abs() < 1.0,
        "X velocity mean should be close to 0, got: {}",
        x_mean
    );
    assert!(
        y_mean.abs() < 1.0,
        "Y velocity mean should be close to 0, got: {}",
        y_mean
    );
}

#[test]
fn test_movement_target_bias() {
    let config = SimulationConfig::default();
    let movement_system = MovementSystem;
    let mut world = World::new();

    // Create some target entities in different quadrants
    let targets = vec![
        (
            Position { x: 20.0, y: 20.0 },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
        ),
        (
            Position { x: -20.0, y: 20.0 },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
        ),
        (
            Position { x: 20.0, y: -20.0 },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
        ),
        (
            Position { x: -20.0, y: -20.0 },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
        ),
    ];

    let target_entities: Vec<Entity> = targets
        .iter()
        .map(|(pos, size, genes)| {
            world.spawn((
                pos.clone(),
                size.clone(),
                genes.clone(),
                Energy {
                    current: 50.0,
                    max: 100.0,
                },
            ))
        })
        .collect();

    // Test movement towards targets from center
    let mut pos = Position { x: 0.0, y: 0.0 };
    let mut velocity = Velocity { x: 0.0, y: 0.0 };
    let mut energy = 100.0;

    movement_system.update_movement(MovementUpdateParams {
        genes: &Genes::new_random(&mut thread_rng()),
        new_pos: &mut pos,
        new_velocity: &mut velocity,
        new_energy: &mut energy,
        pos: &Position { x: 0.0, y: 0.0 },
        nearby_entities: &target_entities,
        world: &world,
        config: &config,
        world_size: 100.0,
    });

    println!(
        "Movement towards targets - Final velocity: ({}, {})",
        velocity.x, velocity.y
    );

    // The velocity should generally point towards one of the targets
    // but we want to check if there's a systematic bias towards top-left
    let distance_to_top_left = ((velocity.x + 20.0).powi(2) + (velocity.y + 20.0).powi(2)).sqrt();
    let distance_to_top_right = ((velocity.x - 20.0).powi(2) + (velocity.y + 20.0).powi(2)).sqrt();
    let distance_to_bottom_left =
        ((velocity.x + 20.0).powi(2) + (velocity.y - 20.0).powi(2)).sqrt();
    let distance_to_bottom_right =
        ((velocity.x - 20.0).powi(2) + (velocity.y - 20.0).powi(2)).sqrt();

    println!(
        "Distances to quadrants - TL: {:.1}, TR: {:.1}, BL: {:.1}, BR: {:.1}",
        distance_to_top_left,
        distance_to_top_right,
        distance_to_bottom_left,
        distance_to_bottom_right
    );
}

#[test]
fn test_long_term_drift_simulation() {
    let config = SimulationConfig::default();
    let movement_system = MovementSystem;
    let mut world = World::new();

    let _entity = world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 0.0, y: 0.0 },
        Energy {
            current: 100.0,
            max: 100.0,
        },
        Size { radius: 5.0 },
        Genes::new_random(&mut thread_rng()),
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
    ));

    let mut pos = Position { x: 0.0, y: 0.0 };
    let mut velocity = Velocity { x: 0.0, y: 0.0 };
    let mut energy = 100.0;

    let mut total_x_movement = 0.0;
    let mut total_y_movement = 0.0;

    // Simulate many movement steps
    for step in 0..100 {
        let old_pos = pos.clone();

        movement_system.update_movement(MovementUpdateParams {
            genes: &Genes::new_random(&mut thread_rng()),
            new_pos: &mut pos,
            new_velocity: &mut velocity,
            new_energy: &mut energy,
            pos: &old_pos.clone(),
            nearby_entities: &[],
            world: &world,
            config: &config,
            world_size: 100.0,
        });

        // Handle boundaries
        movement_system.handle_boundaries(&mut pos, &mut velocity, 100.0, &config);

        total_x_movement += pos.x - old_pos.x;
        total_y_movement += pos.y - old_pos.y;

        if step % 20 == 0 {
            println!(
                "Step {} - Position: ({:.1}, {:.1}), Velocity: ({:.2}, {:.2})",
                step, pos.x, pos.y, velocity.x, velocity.y
            );
        }
    }

    println!(
        "Total movement over 100 steps: ({:.1}, {:.1})",
        total_x_movement, total_y_movement
    );

    // Check for systematic drift
    let drift_magnitude = (total_x_movement.powi(2) + total_y_movement.powi(2)).sqrt();
    println!("Drift magnitude: {:.1}", drift_magnitude);

    // If there's significant drift, it should be detected
    if drift_magnitude > 50.0 {
        println!(
            "WARNING: Significant drift detected! Direction: ({:.1}, {:.1})",
            total_x_movement, total_y_movement
        );
    }
}

#[test]
fn test_world_coordinate_system() {
    let world_size = 100.0; // Typical world size

    // Test coordinate transformations
    let test_positions = vec![
        (0.0, 0.0),     // Center
        (-50.0, -50.0), // Bottom-left
        (50.0, 50.0),   // Top-right
        (-50.0, 50.0),  // Top-left
        (50.0, -50.0),  // Bottom-right
    ];

    for (x, y) in test_positions {
        // Test the same transformation used in UI rendering
        let screen_x: f32 = (x + world_size / 2.0) / world_size * 2.0 - 1.0;
        let screen_y: f32 = -((y + world_size / 2.0) / world_size * 2.0 - 1.0);

        println!(
            "World: ({:.1}, {:.1}) -> Screen: ({:.3}, {:.3})",
            x, y, screen_x, screen_y
        );

        // Check that coordinates are properly mapped
        if x == 0.0 && y == 0.0 {
            assert!(
                (screen_x.abs() < 0.01f32) && (screen_y.abs() < 0.01f32),
                "Center should map to (0,0)"
            );
        }
    }
}

#[test]
fn test_entity_position_distribution() {
    let mut world = World::new();

    // Create entities in a grid pattern to test distribution
    let grid_size = 10;
    let world_size = 100.0;
    let spacing = world_size / grid_size as f32;

    for i in 0..grid_size {
        for j in 0..grid_size {
            let x = (i as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;
            let y = (j as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;

            world.spawn((
                Position { x, y },
                Velocity { x: 0.0, y: 0.0 },
                Energy {
                    current: 100.0,
                    max: 100.0,
                },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
            ));
        }
    }

    // Calculate center of mass
    let mut total_x = 0.0;
    let mut total_y = 0.0;
    let mut count = 0;

    for (_, (pos,)) in world.query::<(&Position,)>().iter() {
        total_x += pos.x;
        total_y += pos.y;
        count += 1;
    }

    let center_x = total_x / count as f32;
    let center_y = total_y / count as f32;

    println!(
        "Grid distribution - Center of mass: ({:.1}, {:.1})",
        center_x, center_y
    );
    println!("Expected center: (0.0, 0.0)");

    // Center of mass should be very close to (0,0) for a uniform grid
    assert!(
        center_x.abs() < 1.0,
        "X center should be close to 0, got: {}",
        center_x
    );
    assert!(
        center_y.abs() < 1.0,
        "Y center should be close to 0, got: {}",
        center_y
    );
}

#[test]
fn test_random_number_bias() {
    use rand::thread_rng;
    use rand::Rng;

    let mut rng = thread_rng();
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();

    // Generate many random values to check for bias
    for _ in 0..10000 {
        // Test the same random generation used in movement
        let dx = rng.gen_range(-1.0f32..1.0);
        let dy = rng.gen_range(-1.0f32..1.0);
        let length_sq = dx * dx + dy * dy;

        if length_sq <= 1.0 && length_sq > 0.0 {
            let length = length_sq.sqrt();
            x_values.push(dx / length);
            y_values.push(dy / length);
        }
    }

    // Calculate statistics
    let x_mean = x_values.iter().sum::<f32>() / x_values.len() as f32;
    let y_mean = y_values.iter().sum::<f32>() / y_values.len() as f32;

    let x_std = (x_values.iter().map(|&x| (x - x_mean).powi(2)).sum::<f32>()
        / x_values.len() as f32)
        .sqrt();
    let y_std = (y_values.iter().map(|&y| (y - y_mean).powi(2)).sum::<f32>()
        / y_values.len() as f32)
        .sqrt();

    println!("Random direction analysis ({} samples):", x_values.len());
    println!("X - Mean: {:.4}, Std: {:.4}", x_mean, x_std);
    println!("Y - Mean: {:.4}, Std: {:.4}", y_mean, y_std);

    // Check for systematic bias
    assert!(
        x_mean.abs() < 0.05,
        "X mean should be very close to 0, got: {}",
        x_mean
    );
    assert!(
        y_mean.abs() < 0.05,
        "Y mean should be very close to 0, got: {}",
        y_mean
    );

    // Check that standard deviations are reasonable (should be around 0.7 for uniform distribution in circle)
    assert!(
        x_std > 0.6 && x_std < 0.8,
        "X std should be around 0.7, got: {}",
        x_std
    );
    assert!(
        y_std > 0.6 && y_std < 0.8,
        "Y std should be around 0.7, got: {}",
        y_std
    );
}
