//! THROWAWAY analysis harness — remove before commit.

use super::*;
use crate::components::MovementType;
use crate::config::SimulationConfig;

fn predator_count(sim: &Simulation) -> usize {
    sim.world()
        .query::<&Genes>()
        .iter()
        .filter(|g| matches!(g.behavior.movement_style.style, MovementType::Predatory))
        .count()
}

/// (#hue bins >=4% of pop, normalized entropy 0..1, dominant-bin share).
fn hue_diversity(sim: &Simulation) -> (usize, f32, f32) {
    let mut bins = [0usize; 12];
    let mut total = 0usize;
    for g in sim.world().query::<&Genes>().iter() {
        let h = g.appearance.hue.clamp(0.0, 0.9999);
        bins[(h * 12.0) as usize] += 1;
        total += 1;
    }
    if total == 0 {
        return (0, 0.0, 0.0);
    }
    let mut modes = 0;
    let mut entropy = 0.0f32;
    let mut dominant = 0usize;
    for &c in &bins {
        if c as f32 / total as f32 >= 0.04 {
            modes += 1;
        }
        dominant = dominant.max(c);
        if c > 0 {
            let p = c as f32 / total as f32;
            entropy -= p * p.ln();
        }
    }
    (modes, entropy / (12f32).ln(), dominant as f32 / total as f32)
}

fn smooth(xs: &[usize], w: usize) -> Vec<f32> {
    if xs.len() < w {
        return xs.iter().map(|&x| x as f32).collect();
    }
    (0..xs.len() - w)
        .map(|i| xs[i..i + w].iter().sum::<usize>() as f32 / w as f32)
        .collect()
}

/// (peak, trough, ratio, turning-points) over the warm tail, heavily smoothed.
fn wave_stats(pops: &[usize]) -> (f32, f32, f32, usize) {
    let sm = smooth(&pops[pops.len() / 6..], 60);
    if sm.len() < 10 {
        return (0.0, 0.0, 1.0, 0);
    }
    let peak = sm.iter().cloned().fold(0.0f32, f32::max);
    let trough = sm.iter().cloned().fold(f32::INFINITY, f32::min);
    let mean = sm.iter().sum::<f32>() / sm.len() as f32;
    let deadband = mean * 0.08;
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
    (peak, trough, peak / trough.max(1.0), turns)
}

struct RunResult {
    min_warm: usize,
    peak: f32,
    trough: f32,
    ratio: f32,
    waves: usize,
    pred_max: usize,
    modes: usize,
    ent: f32,
    dom: f32,
}

fn run_cfg(seed: u64, ticks: usize, config: &SimulationConfig) -> RunResult {
    let mut sim = Simulation::new_with_config_seeded(846.0, config.clone(), seed);
    let mut pops = Vec::new();
    let mut preds = Vec::new();
    let mut min_warm = usize::MAX;
    for t in 0..ticks {
        sim.update();
        let pop = sim.world().len() as usize;
        pops.push(pop);
        preds.push(predator_count(&sim));
        if t >= 500 {
            min_warm = min_warm.min(pop);
        }
    }
    let (peak, trough, ratio, waves) = wave_stats(&pops);
    let (modes, ent, dom) = hue_diversity(&sim);
    RunResult {
        min_warm,
        peak,
        trough,
        ratio,
        waves,
        pred_max: *preds[500..].iter().max().unwrap_or(&0),
        modes,
        ent,
        dom,
    }
}

#[test]
#[ignore]
fn sweep_pressure() {
    // Death floor on (food 1.3, floor 0.03). Push the lag harder to wake the
    // plateau seeds — the floor gate should keep the trough safe.
    let seeds = [21u64, 12345, 7, 999, 2024, 314, 88, 1];
    let combos: &[(f32, f32)] = &[
        // (crowding_pressure_rate, death_chance_factor)
        (0.007, 0.04),
        (0.005, 0.05),
        (0.004, 0.06),
        (0.006, 0.05),
    ];
    for &(rate, dcf) in combos {
        let mut cfg = SimulationConfig::default();
        cfg.reproduction.crowding_pressure_rate = rate;
        cfg.reproduction.death_chance_factor = dcf;
        let food = cfg.energy.ambient_energy_gain;
        let mut worst_floor = usize::MAX;
        let mut sum_ratio = 0.0;
        let mut sum_waves = 0;
        let mut sum_ent = 0.0;
        for &s in &seeds {
            let r = run_cfg(s, 4000, &cfg);
            worst_floor = worst_floor.min(r.min_warm);
            sum_ratio += r.ratio;
            sum_waves += r.waves;
            sum_ent += r.ent;
            println!(
                "  rate={rate} dcf={dcf} seed={s:5}: floor={:4} peak={:.0} trough={:.0} ratio={:.1} waves={:2} predMax={:3} modes={} ent={:.2} dom={:.2}",
                r.min_warm, r.peak, r.trough, r.ratio, r.waves, r.pred_max, r.modes, r.ent, r.dom
            );
        }
        let _ = food;
        println!(
            "CFG rate={rate} dcf={dcf}: WORST_FLOOR={worst_floor} avgRatio={:.1} totWaves={sum_waves} avgEnt={:.2}\n",
            sum_ratio / seeds.len() as f32,
            sum_ent / seeds.len() as f32,
        );
    }
}

/// Verbose trace at the *current default* config: print a population sparkline so
/// the wave shape is visible by eye, plus the predator series.
fn trace(seed: u64, ticks: usize) {
    let mut sim = Simulation::new_with_config_seeded(846.0, SimulationConfig::default(), seed);
    let mut pops = Vec::new();
    let mut preds = Vec::new();
    let mut min_warm = usize::MAX;
    for t in 0..ticks {
        sim.update();
        pops.push(sim.world().len() as usize);
        preds.push(predator_count(&sim));
        if t >= 400 {
            min_warm = min_warm.min(pops[t]);
        }
    }
    let warm = &pops[400..];
    let lo = *warm.iter().min().unwrap() as f32;
    let hi = *warm.iter().max().unwrap() as f32;
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let step = (warm.len() / 90).max(1);
    let spark = |series: &[usize], lo: f32, hi: f32| -> String {
        series
            .iter()
            .step_by(step)
            .map(|&p| {
                let t = if hi > lo { (p as f32 - lo) / (hi - lo) } else { 0.0 };
                bars[((t * 7.0).round() as usize).min(7)]
            })
            .collect()
    };
    let pred_warm = &preds[400..];
    let phi = *pred_warm.iter().max().unwrap() as f32;
    let (peak, trough, ratio, waves) = wave_stats(&pops);
    let (modes, ent, dom) = hue_diversity(&sim);
    println!(
        "SEED {seed:6} minWarm={min_warm} lo={lo:.0} hi={hi:.0} peak={peak:.0} trough={trough:.0} ratio={ratio:.1} waves={waves} predMax={phi:.0} | modes={modes} ent={ent:.2} dom={dom:.2}\n  pop  {}\n  pred {}",
        spark(warm, lo, hi),
        spark(pred_warm, 0.0, phi.max(1.0)),
    );
}

#[test]
#[ignore]
fn probe_dynamics() {
    for &seed in &[21u64, 12345, 7, 1, 999, 2024, 314, 88] {
        trace(seed, 8000);
    }
}
