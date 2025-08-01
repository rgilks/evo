# Evolution Simulation

A beautiful and performant evolution simulation written in Rust, featuring an Entity Component System (ECS) with parallel processing and GPU-accelerated graphics.

## Features

- **Entity Component System**: Uses the `hecs` crate for efficient entity management
- **Parallel Processing**: Leverages `rayon` to maximize performance on multi-core systems (especially M1 Macs)
- **GPU Graphics**: Beautiful real-time visualization using `wgpu`
- **Headless Mode**: Run simulations without UI for testing and analysis
- **Evolution Mechanics**:
  - Entities have genes controlling behavior
  - Energy-based survival system
  - Size changes based on energy levels
  - Reproduction with mutations
  - Predator-prey relationships
  - Resource growth over time

## Entity Types

1. **Resources (Green)**: Plants that grow slowly over time, don't move, and provide energy to herbivores
2. **Herbivores (Orange)**: Eat resources to gain energy, reproduce when energy is high
3. **Predators (Red)**: Hunt herbivores for energy, faster and more aggressive

## Genes

Each living entity has genes that control:

- **Speed**: How fast the entity moves
- **Sense Radius**: How far it can detect food/prey
- **Energy Efficiency**: How efficiently it uses energy
- **Reproduction Threshold**: Energy level required to reproduce
- **Mutation Rate**: How likely genes are to mutate in offspring

## Usage

### With UI (Default)

```bash
cargo run
```

### Headless Mode (Console Only)

```bash
cargo run -- --headless --steps 1000 --world-size 1000
```

### Command Line Options

- `--headless`: Run without graphics (faster for testing)
- `--steps <number>`: Number of simulation steps in headless mode (default: 1000)
- `--world-size <number>`: Size of the simulation world (default: 1000)

## Performance

The simulation is optimized for:

- **M1 Mac Performance Cores**: Uses rayon for parallel processing
- **Spatial Grid Optimization**: O(n²) → O(n) complexity for entity interactions
- **GPU Acceleration**: WGPU for smooth real-time graphics
- **Efficient ECS**: Hecs for fast entity queries and updates

## Building

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repository>
cd evo
cargo build --release

# Run
cargo run --release
```

## Simulation Rules

1. **Energy System**: All entities have energy that decreases over time
2. **Size**: Entity size is proportional to current energy
3. **Movement**: Entities move toward nearest food/prey within their sense radius
4. **Eating**: Collision detection triggers energy transfer and entity removal
5. **Reproduction**: High energy entities split into two with mutated genes
6. **Death**: Entities die when energy reaches zero
7. **Resources**: Plants grow slowly and don't need to eat

## Technical Details

- **ECS Framework**: Hecs for component-based architecture
- **Spatial Grid System**: Optimized O(n²) → O(n) collision detection and interactions
- **Parallel Processing**: Rayon for multi-threaded updates
- **Graphics**: WGPU for cross-platform GPU rendering
- **Window Management**: Winit for window and event handling
- **Random Generation**: Rand for stochastic evolution

## Future Enhancements

- More complex gene interactions
- Environmental factors (temperature, terrain)
- Social behaviors and group dynamics
- Advanced visualization options
- Data export and analysis tools
- WebAssembly support for browser deployment
