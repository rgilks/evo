use super::*;

fn test_cfg() -> FoodFieldConfig {
    FoodFieldConfig {
        patch_count: 6,
        patch_radius: 120.0,
        drift_speed: 0.3,
        regen_rate: 0.02,
        graze_rate: 0.01,
        base: 0.3,
        patch_peak: 3.0,
    }
}

#[test]
fn field_has_requested_patch_count() {
    let field = FoodField::new(42, 846.0, test_cfg());
    assert_eq!(field.patches().len(), 6);
}

#[test]
fn patches_start_inside_the_world() {
    let field = FoodField::new(7, 846.0, test_cfg());
    let bound = 846.0 / 2.0;
    for p in field.patches() {
        assert!(p.x.abs() < bound && p.y.abs() < bound, "patch off-world");
        assert!(p.intensity > 0.0);
    }
}

#[test]
fn gain_is_highest_at_a_patch_centre() {
    let field = FoodField::new(123, 846.0, test_cfg());
    let p = &field.patches()[0];
    let at_centre = field.gain_at(p.x, p.y);
    let far = field.gain_at(p.x + p.radius * 2.0, p.y + p.radius * 2.0);
    assert!(at_centre > far, "centre should be richer than far away");
    assert!(at_centre > 0.0);
}

#[test]
fn gain_falls_to_the_base_outside_all_patches() {
    // A point well outside every patch radius gets only the uniform base — the
    // patch bonus is zero there, keeping between-patch creatures barely fed.
    let field = FoodField::new(999, 846.0, test_cfg());
    // World half is 423; patches sit within 0.7*423 ≈ 296 of the origin with
    // radius 120, so a point at (5000, 5000) is outside all of them.
    assert_eq!(field.gain_at(5000.0, 5000.0), test_cfg().base);
    assert_eq!(field.patch_gain_at(5000.0, 5000.0), 0.0);
}

#[test]
fn grazing_reduces_local_intensity() {
    let mut field = FoodField::new(55, 846.0, test_cfg());
    let (px, py) = (field.patches()[0].x, field.patches()[0].y);
    let before = field.gain_at(px, py);
    for _ in 0..50 {
        field.graze(px, py, 50.0);
    }
    let after = field.gain_at(px, py);
    assert!(after < before, "grazing should deplete the patch");
}

#[test]
fn update_is_deterministic_for_same_seed() {
    let snapshot = |seed: u64| {
        let mut field = FoodField::new(seed, 846.0, test_cfg());
        for step in 1..200 {
            field.update(seed, step);
            // Simulate some grazing so depletion/respawn paths are exercised.
            let (x, y) = (field.patches()[0].x, field.patches()[0].y);
            field.graze(x, y, 100.0);
        }
        field
            .patches()
            .iter()
            .map(|p| (p.x.to_bits(), p.y.to_bits(), p.intensity.to_bits()))
            .collect::<Vec<_>>()
    };
    assert_eq!(snapshot(0xABCD), snapshot(0xABCD), "same seed must match");
    assert_ne!(snapshot(0xABCD), snapshot(0x1234), "seeds should diverge");
}

#[test]
fn patches_stay_inside_world_while_drifting() {
    let seed = 314;
    let mut field = FoodField::new(seed, 846.0, test_cfg());
    let bound = 846.0 / 2.0;
    for step in 1..2000 {
        field.update(seed, step);
        for p in field.patches() {
            assert!(
                p.x.abs() <= bound && p.y.abs() <= bound,
                "patch drifted off-world at step {step}: ({}, {})",
                p.x,
                p.y
            );
        }
    }
}

#[test]
fn total_production_is_comparable_to_a_uniform_field() {
    // Sanity check on carrying-capacity preservation: the spatial *average* gain
    // sampled over the world should be in the same ballpark as the old flat
    // ambient gain (0.9), so concentrating food doesn't silently slash total
    // production. We accept a generous band — the point is "same order", not exact.
    let field = FoodField::new(2024, 846.0, test_cfg());
    let half = 846.0 / 2.0;
    let n = 60;
    let mut total = 0.0;
    let mut samples = 0;
    for i in 0..n {
        for j in 0..n {
            let x = -half + (i as f32 + 0.5) / n as f32 * 846.0;
            let y = -half + (j as f32 + 0.5) / n as f32 * 846.0;
            total += field.gain_at(x, y);
            samples += 1;
        }
    }
    let avg = total / samples as f32;
    assert!(
        (0.3..3.0).contains(&avg),
        "average production {avg} drifted far from the uniform baseline (~0.9)"
    );
}
