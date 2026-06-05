use super::*;
use crate::components::{Color, Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use rand::rng;

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
fn test_browser_like_long_run_no_panic() {
    // Mirror the browser path: ~846 world, default config (~1250 entities), many
    // ticks, plus the render read-back each tick. Sweep seeds (including the exact
    // browser time-seed) to catch seed-dependent panics the short determinism run
    // (40 ticks, DEFAULT_SEED) would miss.
    let seeds = [
        1_780_440_263_372u64,
        1,
        42,
        7,
        999_999,
        0xDEAD_BEEF,
        2_000_000_000_000,
    ];
    for &seed in &seeds {
        let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), seed);
        for _ in 0..600 {
            sim.update();
            let _ = sim.get_entities();
        }
        assert_eq!(sim.step(), 600, "seed {seed}");
    }
}

/// Density contrast at the current instant: (share of creatures inside patch
/// cores) / (share of the roamed area those cores cover). A value > 1 means
/// creatures are over-represented at the patches — i.e. they are gathering there.
fn patch_density_contrast(sim: &Simulation) -> f32 {
    let patches = sim.get_food_patches(); // (x, y, radius, intensity)
    let in_core = |x: f32, y: f32| {
        patches
            .iter()
            .any(|&(px, py, r, _)| (x - px).powi(2) + (y - py).powi(2) < (r * 0.5).powi(2))
    };
    let ents = sim.get_entities();
    if ents.is_empty() {
        return 0.0;
    }
    let frac_in = ents.iter().filter(|e| in_core(e.2, e.3)).count() as f32 / ents.len() as f32;
    let span = sim.world_size() / 2.0 * 0.85;
    let g = 50;
    let mut area_in = 0;
    for i in 0..g {
        for j in 0..g {
            let x = -span + (i as f32 + 0.5) / g as f32 * 2.0 * span;
            let y = -span + (j as f32 + 0.5) / g as f32 * 2.0 * span;
            if in_core(x, y) {
                area_in += 1;
            }
        }
    }
    let area_frac = (area_in as f32 / (g * g) as f32).max(1e-3);
    frac_in / area_frac
}

#[test]
fn test_food_field_keeps_population_stable_across_seeds() {
    // The spatial food field must not collapse the ecosystem. The base + patch
    // split is balanced (see `food_field_config`) so the population recovers from
    // the initial overshoot rather than starving out. The browser/gate seeds stay
    // lively; the inherently fragile seeds settle lower but never to a handful.
    // Guards against the food concentration silently killing the population.
    for &(seed, floor) in &[
        (21u64, 120),
        (12345, 250),
        (7, 250),
        (999, 20),
        (1, 20),
        (2024, 120),
    ] {
        let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), seed);
        for _ in 0..1000 {
            sim.update();
        }
        let pop = sim.world().len();
        assert!(
            pop as i32 > floor,
            "seed {seed} collapsed to {pop} (floor {floor}); the food field should sustain it"
        );
    }
}

#[test]
fn test_creatures_gather_at_food_patches() {
    // The behavioural payoff: creatures migrate toward and aggregate at the food
    // patches, so they are over-represented at the patch cores relative to the
    // area those cores cover. Averaged over a window of ticks (the patches drift,
    // so any single frame is noisy) the contrast stays clearly above 1.
    let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), 21);
    for _ in 0..300 {
        sim.update();
    }
    let mut total = 0.0;
    let samples = 40;
    for _ in 0..samples {
        sim.update();
        total += patch_density_contrast(&sim);
    }
    let avg = total / samples as f32;
    assert!(
        avg > 1.25,
        "creatures are not gathering at food: mean patch-core density contrast {avg:.2} (want > 1.25)"
    );
}

#[test]
fn test_population_sustains_via_primary_production() {
    // The ambient energy field (primary production) gives the ecosystem a carrying
    // capacity, so the population recovers from the initial overshoot instead of
    // decaying to a handful of survivors. Without it (ambient_energy_gain = 0) this
    // same run collapses to single digits — so this guards the fix.
    let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), 12345);
    for _ in 0..1000 {
        sim.update();
    }
    let pop = sim.world().len();
    assert!(
        pop > 250,
        "population collapsed to {pop}; primary production should sustain a healthy level"
    );
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
fn test_simulation_apply_updates() {
    let mut sim = Simulation::new(100.0);
    let _entity = sim.world.spawn((
        Position { x: 0.0, y: 0.0 },
        Energy {
            current: 50.0,
            max: 100.0,
        },
        Size { radius: 5.0 },
        Genes::new_random(&mut rng()),
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
        velocity: Velocity { x: 1.0, y: 1.0 },
        grazed: 0.0,
        should_reproduce: false,
        eaten_entity: None,
    }];

    sim.apply_entity_updates(updates);

    // Entity should be updated
    // Note: We can't easily test this due to borrowing rules
    // In a real scenario, you'd need to restructure the code
}

/// Mean (cur_x, cur_y) over the entity render tuples — shared by the drift and
/// clustering assertions below.
fn centroid(entities: &[(f32, f32, f32, f32, f32, f32, f32, f32)]) -> (f32, f32) {
    let n = entities.len().max(1) as f32;
    let (sx, sy) = entities
        .iter()
        .fold((0.0f32, 0.0f32), |(ax, ay), e| (ax + e.2, ay + e.3));
    (sx / n, sy / n)
}

#[test]
fn test_simulation_clustering() {
    // Deterministic DEFAULT_SEED run: the population should stay alive, remain
    // centred (no runaway drift to a corner), and stay spread across the world
    // rather than collapsing to a point.
    let mut simulation = Simulation::new_with_config(100.0, SimulationConfig::default());
    for _ in 0..100 {
        simulation.update();
    }

    let entities = simulation.get_entities();
    assert!(!entities.is_empty(), "population died out");

    let (center_x, center_y) = centroid(&entities);
    assert!(
        center_x.abs() < 20.0 && center_y.abs() < 20.0,
        "centroid drifted off-centre: ({center_x:.1}, {center_y:.1})"
    );

    let min_x = entities.iter().map(|e| e.2).fold(f32::INFINITY, f32::min);
    let max_x = entities
        .iter()
        .map(|e| e.2)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = entities.iter().map(|e| e.3).fold(f32::INFINITY, f32::min);
    let max_y = entities
        .iter()
        .map(|e| e.3)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_x - min_x > 20.0 && max_y - min_y > 20.0,
        "population collapsed: spread ({:.1}, {:.1})",
        max_x - min_x,
        max_y - min_y
    );
}

#[test]
fn test_drift_direction_analysis() {
    // Over 200 ticks the population centroid must not drift systematically toward
    // any edge — the simulation is centred and unbiased. The deterministic
    // DEFAULT_SEED run drifts ~(5, 1); 15 leaves margin while still catching a
    // real directional bias.
    let mut simulation = Simulation::new_with_config(100.0, SimulationConfig::default());

    let start = centroid(&simulation.get_entities());
    for _ in 0..200 {
        simulation.update();
    }
    let end_entities = simulation.get_entities();
    assert!(!end_entities.is_empty(), "population died out");
    let end = centroid(&end_entities);

    let drift_x = (end.0 - start.0).abs();
    let drift_y = (end.1 - start.1).abs();
    assert!(
        drift_x < 15.0 && drift_y < 15.0,
        "centroid drifted too far over 200 ticks: ({drift_x:.1}, {drift_y:.1})"
    );
}

#[test]
fn test_entity_data_format() {
    let sim = Simulation::new(100.0);
    let entities = sim.get_entities();

    // Each entity should have 8 components: prev_x, prev_y, cur_x, cur_y, radius, r, g, b
    for (_px, _py, cx, cy, radius, r, g, b) in &entities {
        // Position should be within world bounds
        assert!(*cx >= -50.0 && *cx <= 50.0, "cx={} out of bounds", cx);
        assert!(*cy >= -50.0 && *cy <= 50.0, "cy={} out of bounds", cy);

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
    let mut buffer: Vec<f32> = Vec::with_capacity(entities.len() * 8);
    for (px, py, cx, cy, radius, r, g, b) in entities.iter() {
        buffer.push(*px);
        buffer.push(*py);
        buffer.push(*cx);
        buffer.push(*cy);
        buffer.push(*radius);
        buffer.push(*r);
        buffer.push(*g);
        buffer.push(*b);
    }

    // Buffer length should be 8 * entity count
    assert_eq!(buffer.len(), entities.len() * 8);

    // Entity count calculation should match
    let entity_count = buffer.len() / 8;
    assert_eq!(entity_count, entities.len());

    // Verify data integrity by reading back
    for (i, (px, py, cx, cy, radius, r, g, b)) in entities.iter().enumerate() {
        let base = i * 8;
        assert_eq!(buffer[base], *px);
        assert_eq!(buffer[base + 1], *py);
        assert_eq!(buffer[base + 2], *cx);
        assert_eq!(buffer[base + 3], *cy);
        assert_eq!(buffer[base + 4], *radius);
        assert_eq!(buffer[base + 5], *r);
        assert_eq!(buffer[base + 6], *g);
        assert_eq!(buffer[base + 7], *b);
    }
}

#[test]
fn test_config_mut_updates_in_place() {
    let mut sim = Simulation::new(100.0);
    let original_velocity = sim.config.physics.max_velocity;

    sim.config_mut().physics.max_velocity = 5.0;

    assert_ne!(sim.config.physics.max_velocity, original_velocity);
    assert_eq!(sim.config.physics.max_velocity, 5.0);
}

#[test]
fn test_in_place_update_persists_and_moves_entities() {
    // Entities are updated in place rather than despawned and respawned each tick,
    // so an entity keeps its id across ticks. With the old respawn churn, none of
    // the starting ids would survive a single tick.
    let mut config = SimulationConfig::default();
    config.reproduction.death_chance_factor = 0.0; // no random death
    config.reproduction.reproduction_energy_threshold = 100.0; // unreachable -> no births
    let mut sim = Simulation::new_with_config(200.0, config);

    // Snapshot starting positions keyed by stable entity id.
    let start: std::collections::HashMap<_, _> = sim
        .world
        .query::<(Entity, &Position)>()
        .iter()
        .map(|(e, p)| (e, (p.x, p.y)))
        .collect();
    assert!(!start.is_empty());

    for _ in 0..5 {
        sim.update();
    }

    // Survivors keep their ids (no respawn churn) and at least one moved in place.
    let mut survivors = 0;
    let mut moved = 0;
    for (entity, pos) in sim.world.query::<(Entity, &Position)>().iter() {
        if let Some(&(sx, sy)) = start.get(&entity) {
            survivors += 1;
            assert!(pos.x.is_finite() && pos.y.is_finite());
            if (pos.x - sx).abs() > f32::EPSILON || (pos.y - sy).abs() > f32::EPSILON {
                moved += 1;
            }
        }
    }
    assert!(
        survivors > 0,
        "original entity ids should persist across ticks (stable ids)"
    );
    assert!(moved > 0, "in-place update should move surviving entities");
}

#[test]
fn test_predation_keeps_population_bounded() {
    // Eating removes prey, so the population oscillates — but over a long run it
    // must neither collapse to zero nor explode far past the cap.
    let config = SimulationConfig::default();
    let max_pop =
        (config.population.max_population as f32 * config.population.entity_scale) as usize;
    let mut sim = Simulation::new_with_config(200.0, config);

    for _ in 0..80 {
        sim.update();
    }

    let pop = sim.world().len() as usize;
    assert!(pop > 0, "population collapsed to zero under predation");
    assert!(
        pop < max_pop * 2,
        "population exploded past the cap (got {pop})"
    );
}

/// Run a seed for `ticks` and return the population each tick (the headless
/// dynamics measurement used by the boom/bust + safety tests below).
fn population_series(seed: u64, ticks: usize) -> Vec<usize> {
    let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), seed);
    let mut pops = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        sim.update();
        pops.push(sim.world().len() as usize);
    }
    pops
}

/// Bucket the current population's hues into 12 sectors; return (number of
/// sectors holding ≥4% of the population, dominant sector's share).
fn hue_modes(sim: &Simulation) -> (usize, f32) {
    let mut bins = [0usize; 12];
    let mut total = 0usize;
    for g in sim.world().query::<&Genes>().iter() {
        let h = g.appearance.hue.clamp(0.0, 0.9999);
        bins[(h * 12.0) as usize] += 1;
        total += 1;
    }
    if total == 0 {
        return (0, 0.0);
    }
    let modes = bins
        .iter()
        .filter(|&&c| c as f32 / total as f32 >= 0.04)
        .count();
    let dom = *bins.iter().max().unwrap() as f32 / total as f32;
    (modes, dom)
}

#[test]
fn test_population_stays_in_safe_band_long_run() {
    // SAFETY (load-bearing). Over a long run across several seeds — including the
    // browser default 21 and historically fragile ones — the population must stay
    // strictly bounded at *every* tick after warm-up: never near extinction
    // (the lagged-mortality boom/bust must never spiral down, guarded by the
    // death-floor gate) and never pinned at the cap (no runaway). Do NOT weaken
    // this to make a tuning change "pass".
    let cap = {
        let c = SimulationConfig::default();
        (c.population.max_population as f32 * c.population.entity_scale) as usize
    };
    const HARD_FLOOR: usize = 90;
    // The browser default (21) plus the historically weakest/lowest-floor seeds
    // (999, 88) — the stress cases most likely to violate the band. Kept to a
    // focused set so the (necessarily long ≥5000-tick) run stays affordable.
    for &seed in &[21u64, 999, 88] {
        let pops = population_series(seed, 5000);
        // Allow a warm-up window for the initial overshoot to settle.
        let warm = &pops[1200..];
        let lo = *warm.iter().min().unwrap();
        let hi = *warm.iter().max().unwrap();
        assert!(
            lo >= HARD_FLOOR,
            "seed {seed}: population dipped to {lo} (< hard floor {HARD_FLOOR}) — extinction risk"
        );
        assert!(
            hi < cap,
            "seed {seed}: population reached {hi} (>= cap {cap}) — runaway"
        );
    }
}

#[test]
fn test_population_oscillates() {
    // BOOM/BUST. On a representative seed the lagged-mortality + predator coupling
    // produces a real, sustained periodic swing — the population repeatedly rises
    // and falls rather than settling to a flat line. We assert a meaningful
    // peak/trough amplitude and several turning points over the run.
    let pops = population_series(12345, 6000);
    // Smooth hard to ignore tick noise, then measure over the warm tail.
    let w = 60usize;
    let warm = &pops[1000..];
    let sm: Vec<f32> = (0..warm.len() - w)
        .map(|i| warm[i..i + w].iter().sum::<usize>() as f32 / w as f32)
        .collect();
    let peak = sm.iter().cloned().fold(0.0f32, f32::max);
    let trough = sm.iter().cloned().fold(f32::INFINITY, f32::min);
    assert!(
        peak / trough > 1.3,
        "no boom/bust amplitude: smoothed peak {peak:.0} vs trough {trough:.0} (want ratio > 1.3)"
    );
    // Count smoothed turning points with a deadband, a proxy for cycle count.
    let mean = sm.iter().sum::<f32>() / sm.len() as f32;
    let deadband = mean * 0.06;
    let mut turns = 0;
    let mut rising = true;
    let mut anchor = sm[0];
    for &v in &sm {
        if rising && v < anchor - deadband {
            turns += 1;
            rising = false;
            anchor = v;
        } else if !rising && v > anchor + deadband {
            turns += 1;
            rising = true;
            anchor = v;
        }
        anchor = if rising { anchor.max(v) } else { anchor.min(v) };
    }
    assert!(
        turns >= 4,
        "population does not cycle: only {turns} smoothed turning points (want >= 4)"
    );
}

#[test]
fn test_speciation_multiple_persistent_hues() {
    // SPECIATION. Assortative hue inheritance + frequency-dependent reproduction
    // keep several distinct colour lineages coexisting: at multiple snapshots over
    // a long run there are >=3 well-populated hue sectors and no single hue
    // dominates the world. Guards against collapse to a uniform colour (one mode)
    // and confirms the diversity *persists* rather than being a transient.
    let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), 21);
    for _ in 0..1500 {
        sim.update();
    }
    let mut checks = 0;
    for _ in 0..6 {
        for _ in 0..400 {
            sim.update();
        }
        let (modes, dom) = hue_modes(&sim);
        assert!(
            modes >= 3,
            "speciation collapsed: only {modes} populated hue sectors (want >= 3)"
        );
        assert!(
            dom < 0.7,
            "one hue dominates ({:.0}% of the population) — not multiple lineages",
            dom * 100.0
        );
        checks += 1;
    }
    assert_eq!(checks, 6);
}

#[test]
fn test_same_seed_produces_identical_runs() {
    // Bit-exact reproducibility: the same seed must yield identical state after
    // N ticks, independent of thread scheduling. Different seeds should diverge.
    fn run(seed: u64) -> Vec<(u64, u32, u32)> {
        let mut sim = Simulation::new_with_config_seeded(200.0, SimulationConfig::default(), seed);
        for _ in 0..40 {
            sim.update();
        }
        let mut state: Vec<(u64, u32, u32)> = sim
            .world()
            .query::<(Entity, &Position)>()
            .iter()
            .map(|(e, p)| (e.to_bits().get(), p.x.to_bits(), p.y.to_bits()))
            .collect();
        state.sort();
        state
    }

    assert_eq!(run(0xABCD), run(0xABCD), "same seed must be bit-identical");
    assert_ne!(run(0xABCD), run(0x1234), "different seeds should diverge");
}
