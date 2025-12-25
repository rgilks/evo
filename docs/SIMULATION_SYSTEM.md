# Evolution Simulation System Documentation

## Overview

The Evolution Simulation is a complex ecosystem simulation built in Rust using an Entity-Component-System (ECS) architecture. It simulates the evolution of entities through natural selection, genetic inheritance, and environmental pressures.

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

### 3. Movement System

Entities exhibit one of five genetically determined movement styles:
1. **Random**: Baseline brownian-like motion.
2. **Flocking**: Cohesion, alignment, and separation (Boids algorithm) with genetically similar neighbors.
3. **Solitary**: Active avoidance of other entities.
4. **Predatory**: Active pursuit of prey based on genetic preference and size advantage.
5. **Grazing**: Slow, steady movement with minimal energy expenditure.

### 4. Interaction System

- **Predation**: Larger entities eat smaller specific prey.
- **Gene Preference**: Predators prefer genetically distinct prey (promoting diversity).
- **Energy Transfer**: Eating yields energy; movement and existence consume it.

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
