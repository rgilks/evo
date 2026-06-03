# Evolution Simulation System Documentation

## Overview

The Evolution Simulation is a complex ecosystem simulation built in Rust using an Entity-Component-System (ECS) architecture. It simulates the evolution of entities through natural selection, genetic inheritance, and environmental pressures.

## Entity Lifecycle

Every creature follows the same lifecycle — born from the initial population or as a mutated offspring, updated each tick, and removed when it starves or is eaten:

```mermaid
stateDiagram-v2
    [*] --> Alive: spawn (initial or offspring)
    Alive --> Alive: each tick — move, eat, metabolize
    Alive --> Alive: reproduce → spawns a mutated offspring
    Alive --> Dead: energy ≤ 0 (starved) or eaten by a predator
    Dead --> [*]: despawn
```

For the per-tick machinery behind these transitions, see the [simulation tick diagram](diagrams/simulation-tick.png) and [ARCHITECTURE.md](ARCHITECTURE.md).

## Core Architecture

### Technology Stack

- **Language**: Rust
- **ECS Framework**: Hecs
- **Parallelism**: Rayon
- **Rendering**: WGPU (Desktop) / WebGL/WGPU (Web via wgpu)
- **Serialization**: Serde

### Design Principles

- **Modularity**: Systems (Movement, Interaction, Energy, Reproduction) are independent.
- **Parallel processing**: Heavy computations use `rayon` for multi-core scaling.
- **Configurability**: Simulation parameters are hot-swappable via JSON.

## System Components

### 1. Entity Components

Core data structures managed by the ECS:
- **Position & Velocity**: 2D Physics vectors.
- **Energy**: Life force; entities die at 0 energy.
- **Size**: Radius affecting energy cost and interaction range.
- **Color**: Visual phenotype derived from genes.
- **Genes**: The genetic blueprint (see below).

### 2. Genetic System

Genes determine all behavior and attributes. They are mutable and heritable.

| Category | Traits |
|----------|--------|
| **Movement** | `speed`, `sense_radius` |
| **Energy** | `efficiency`, `loss_rate`, `gain_rate`, `size_factor` |
| **Reproduction** | `rate`, `mutation_rate` |
| **Shape/Color** | `hue`, `saturation` |
| **Behavior** | `movement_style`, `social_tendency`, `gene_preference` |
| **Particle Life** | `interactions` — per-hue-sector attraction/repulsion weights |

### 3. Movement System

Entities exhibit one of five genetically determined movement styles:
1. **Random**: Baseline brownian-like motion.
2. **Flocking**: Cohesion, alignment, and separation (Boids algorithm) with genetically similar neighbors.
3. **Solitary**: Active avoidance of other entities.
4. **Predatory**: Active pursuit of prey based on genetic preference and size advantage.
5. **Grazing**: Slow, steady movement with minimal energy expenditure.

On top of these styles, every creature carries **particle-life interaction weights** — a per-hue-sector attraction/repulsion table applied as a force during the same single neighbour pass. This adds emergent clustering and pattern formation over the style-based behaviour, and is tunable live via the `particle_force_scale` and `particle_friction` parameters.

All of these forces — style behaviour, flocking, particle-life, and center pressure — accumulate into a single velocity that is then **capped at `max_velocity`** (by magnitude) each tick, so no combination of forces can push a creature past the speed limit.

### 4. Interaction System

- **Predation**: Larger entities eat smaller specific prey.
- **Gene Preference**: Predators prefer genetically distinct prey (promoting diversity).
- **Energy economy**: Movement and existence consume energy; predation transfers it between creatures. The only *input* is **primary production** — every creature draws a little energy from an ambient food field each tick (`energy.ambient_energy_gain`), scaled by `(1 - population_density)` so the field is finite. That finite input gives the ecosystem a **carrying capacity**: the population settles around the density where production balances metabolism, instead of decaying to a few survivors the way a closed, input-free system does.

### 5. Spatial System

- **Spatial Grid**: The world is partitioned into cells to optimize neighbor lookups (O(1) instead of O(N²)).
- **Boundaries**: Soft boundaries with increasing "center pressure" to keep populations active.

## Statistics

Real-time metrics tracking:
- Population counts by species/behavior.
- Average genetic drift (evolution speed).
- System performance (FPS, step time).

## Roadmap & Future Ideas

- **Environmental Complexity**: Terrain, obstacles, and localized resource patches.
- **Advanced Biology**: Aging, disease/parasites, and sexual dimorphism.
- **Complex Sociality**: Mating rituals, territorial defense, and memory/learning.
- **Multi-Species**: Symbiotic relationships and food webs.
