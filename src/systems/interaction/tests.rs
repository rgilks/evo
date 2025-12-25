use super::*;
use crate::components::{Position, Size};
use crate::genes::Genes;
use hecs::World;
use rand::prelude::*;

#[test]
fn test_interaction_system_handle_interactions() {
    let system = InteractionSystem;
    let mut new_energy = 50.0;
    let mut eaten_entity = None;
    let new_pos = Position { x: 0.0, y: 0.0 };
    let size = Size { radius: 10.0 };
    let mut rng = thread_rng();
    let genes = Genes::new_random(&mut rng);
    let nearby_entities = vec![];
    let world = World::new();
    let config = SimulationConfig::default();

    system.handle_interactions(InteractionParams {
        new_energy: &mut new_energy,
        eaten_entity: &mut eaten_entity,
        new_pos: &new_pos,
        size: &size,
        genes: &genes,
        nearby_entities: &nearby_entities,
        world: &world,
        config: &config,
    });

    // Energy should remain unchanged if no interactions
    assert_eq!(new_energy, 50.0);
    assert!(eaten_entity.is_none());
}

#[test]
fn test_interaction_system_drift() {
    use crate::config::SimulationConfig;
    use crate::simulation::Simulation;

    let mut config = SimulationConfig::default();
    // Disable reproduction to isolate interaction effects
    config.reproduction.reproduction_energy_threshold = 2.0; // Impossible threshold
    config.reproduction.min_reproduction_chance = 0.0;

    let world_size = 100.0;
    let mut simulation = Simulation::new_with_config(world_size, config);

    println!("Testing interaction system drift (reproduction disabled)...");

    let mut positions = Vec::new();

    // Run simulation for 100 steps, recording positions every 20 steps
    for step in 0..100 {
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

            println!(
                "Step {}: {} entities, Center ({:.1}, {:.1})",
                step,
                entities.len(),
                center_x,
                center_y
            );
        }
    }

    // Analyze drift direction
    if positions.len() >= 2 {
        let first = positions[0];
        let last = positions[positions.len() - 1];
        let drift_x = last.1 - first.1;
        let drift_y = last.2 - first.2;

        println!("\nInteraction System Drift Analysis:");
        println!("Start position: ({:.1}, {:.1})", first.1, first.2);
        println!("End position: ({:.1}, {:.1})", last.1, last.2);
        println!("Total drift: ({:.1}, {:.1})", drift_x, drift_y);

        if drift_x.abs() > 5.0 || drift_y.abs() > 5.0 {
            println!("INTERACTION SYSTEM IS CAUSING DRIFT!");
        } else {
            println!("Interaction system appears unbiased");
        }
    }
}

#[test]
fn test_interaction_processing_order() {
    use crate::config::SimulationConfig;
    use crate::simulation::Simulation;

    let mut config = SimulationConfig::default();
    // Make interactions more likely to see the effect
    config.physics.interaction_radius_offset = 25.0; // Larger interaction radius

    let world_size = 100.0;
    let mut simulation = Simulation::new_with_config(world_size, config);

    println!("Testing interaction processing order bias...");

    // Track which entities are being eaten and from which positions
    let mut eaten_positions: Vec<(f32, f32)> = Vec::new();
    let _predator_positions: Vec<(f32, f32)> = Vec::new();

    // Run simulation for 50 steps and track interactions
    for step in 0..50 {
        simulation.update();

        if step % 10 == 0 {
            let entities = simulation.get_entities();
            let mut total_x = 0.0;
            let mut total_y = 0.0;

            for (x, y, _, _, _, _) in &entities {
                total_x += x;
                total_y += y;
            }

            let center_x = total_x / entities.len() as f32;
            let center_y = total_y / entities.len() as f32;

            println!(
                "Step {}: {} entities, Center ({:.1}, {:.1})",
                step,
                entities.len(),
                center_x,
                center_y
            );

            // Store positions for analysis
            for (x, y, _, _, _, _) in &entities {
                eaten_positions.push((*x, *y));
            }
        }
    }

    // Analyze if there's a pattern in where entities are being eaten
    if !eaten_positions.is_empty() {
        let mut total_x = 0.0;
        let mut total_y = 0.0;

        for (x, y) in &eaten_positions {
            total_x += x;
            total_y += y;
        }

        let avg_x = total_x / eaten_positions.len() as f32;
        let avg_y = total_y / eaten_positions.len() as f32;

        println!("Average position of entities: ({:.1}, {:.1})", avg_x, avg_y);

        // Check if there's a bias towards certain quadrants
        let mut quadrant_counts = [0, 0, 0, 0]; // TL, TR, BL, BR

        for (x, y) in &eaten_positions {
            if *x < 0.0 && *y > 0.0 {
                quadrant_counts[0] += 1; // Top-left
            } else if *x > 0.0 && *y > 0.0 {
                quadrant_counts[1] += 1; // Top-right
            } else if *x < 0.0 && *y < 0.0 {
                quadrant_counts[2] += 1; // Bottom-left
            } else {
                quadrant_counts[3] += 1; // Bottom-right
            }
        }

        println!(
            "Entity distribution by quadrant: TL:{}, TR:{}, BL:{}, BR:{}",
            quadrant_counts[0], quadrant_counts[1], quadrant_counts[2], quadrant_counts[3]
        );

        // Check for significant bias
        let total = quadrant_counts.iter().sum::<i32>();
        let expected = total / 4;

        for (i, count) in quadrant_counts.iter().enumerate() {
            let bias = (*count as f32 - expected as f32) / total as f32;
            if bias.abs() > 0.1 {
                println!("SIGNIFICANT BIAS in quadrant {}: {:.1}%", i, bias * 100.0);
            }
        }
    }
}

#[test]
fn test_interaction_order_bias() {
    use crate::config::SimulationConfig;
    use crate::simulation::Simulation;

    let mut config = SimulationConfig::default();
    // Make interactions very likely
    config.physics.interaction_radius_offset = 30.0;

    let world_size = 100.0;
    let mut simulation = Simulation::new_with_config(world_size, config);

    println!("Testing interaction order bias...");

    // Track the positions of entities that survive vs those that don't
    let mut survivor_positions = Vec::new();
    let mut initial_positions = Vec::new();

    // Get initial positions
    let initial_entities = simulation.get_entities();
    for (x, y, _, _, _, _) in &initial_entities {
        initial_positions.push((*x, *y));
    }

    // Run simulation for a few steps
    for step in 0..20 {
        simulation.update();

        if step == 19 {
            // After 20 steps
            let final_entities = simulation.get_entities();
            for (x, y, _, _, _, _) in &final_entities {
                survivor_positions.push((*x, *y));
            }
        }
    }

    // Analyze the bias
    if !survivor_positions.is_empty() && !initial_positions.is_empty() {
        let mut initial_total_x = 0.0;
        let mut initial_total_y = 0.0;
        let mut survivor_total_x = 0.0;
        let mut survivor_total_y = 0.0;

        for (x, y) in &initial_positions {
            initial_total_x += x;
            initial_total_y += y;
        }

        for (x, y) in &survivor_positions {
            survivor_total_x += x;
            survivor_total_y += y;
        }

        let initial_center_x = initial_total_x / initial_positions.len() as f32;
        let initial_center_y = initial_total_y / initial_positions.len() as f32;
        let survivor_center_x = survivor_total_x / survivor_positions.len() as f32;
        let survivor_center_y = survivor_total_y / survivor_positions.len() as f32;

        let drift_x = survivor_center_x - initial_center_x;
        let drift_y = survivor_center_y - initial_center_y;

        println!(
            "Initial center: ({:.1}, {:.1})",
            initial_center_x, initial_center_y
        );
        println!(
            "Survivor center: ({:.1}, {:.1})",
            survivor_center_x, survivor_center_y
        );
        println!("Drift: ({:.1}, {:.1})", drift_x, drift_y);

        // Check if survivors are biased towards certain quadrants
        let mut survivor_quadrants = [0, 0, 0, 0]; // TL, TR, BL, BR

        for (x, y) in &survivor_positions {
            if *x < 0.0 && *y > 0.0 {
                survivor_quadrants[0] += 1; // Top-left
            } else if *x > 0.0 && *y > 0.0 {
                survivor_quadrants[1] += 1; // Top-right
            } else if *x < 0.0 && *y < 0.0 {
                survivor_quadrants[2] += 1; // Bottom-left
            } else {
                survivor_quadrants[3] += 1; // Bottom-right
            }
        }

        println!(
            "Survivor distribution: TL:{}, TR:{}, BL:{}, BR:{}",
            survivor_quadrants[0],
            survivor_quadrants[1],
            survivor_quadrants[2],
            survivor_quadrants[3]
        );

        // Check for significant bias
        let total_survivors = survivor_quadrants.iter().sum::<i32>();
        let expected = total_survivors / 4;

        for (i, count) in survivor_quadrants.iter().enumerate() {
            let bias = (*count as f32 - expected as f32) / total_survivors as f32;
            if bias.abs() > 0.1 {
                println!("SURVIVOR BIAS in quadrant {}: {:.1}%", i, bias * 100.0);
            }
        }

        if drift_x.abs() > 5.0 || drift_y.abs() > 5.0 {
            println!(
                "INTERACTION ORDER BIAS CONFIRMED: Drift of ({:.1}, {:.1})",
                drift_x, drift_y
            );
        }
    }
}
