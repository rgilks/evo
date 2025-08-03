# Evolution Simulation

A beautiful and performant evolution simulation written in Rust, featuring an Entity Component System (ECS) with parallel processing and GPU-accelerated graphics. Watch as complex behaviors emerge naturally from simple genetic rules!

## Features

- **Entity Component System**: Uses the `hecs` crate for efficient entity management
- **Parallel Processing**: Leverages `rayon` to maximize performance on multi-core systems
- **GPU Graphics**: Beautiful real-time visualization using `wgpu`
- **Headless Mode**: Run simulations without UI for testing and analysis
- **Emergent Behaviors**: Complex predator-prey dynamics emerge from simple genetic rules
- **Population Balance**: Sophisticated population control mechanisms prevent explosions
- **Stable Physics**: Advanced boundary handling and drift correction
- **Modular Architecture**: Clean separation of concerns with focused modules

## Architecture

The project follows a clean, modular architecture:

### Core Modules

- **`components.rs`**: ECS components (Position, Energy, Size, Velocity, Color)
- **`genes.rs`**: Genetic system with grouped traits (Movement, Energy, Reproduction, Appearance)
- **`systems.rs`**: Simulation systems (Movement, Interaction, Energy, Reproduction)
- **`spatial_grid.rs`**: Spatial optimization for efficient neighbor finding
- **`stats.rs`**: Analytics and statistics collection
- **`simulation.rs`**: Main simulation orchestration
- **`config.rs`**: Configuration management
- **`ui.rs`**: GPU-accelerated rendering

### Design Principles

- **Single Responsibility**: Each module has a focused purpose
- **Separation of Concerns**: Logic is cleanly separated into systems
- **Performance First**: Parallel processing and spatial optimization
- **Configuration Driven**: External JSON configuration for easy experimentation

## Evolution Mechanics

### Gene-Based Behaviors

Instead of predefined entity types, all behaviors emerge from genes organized into logical groups:

#### Movement Genes

- **Speed**: Movement velocity and hunting effectiveness
- **Sense Radius**: Detection range for food and threats

#### Energy Genes

- **Efficiency**: How efficiently energy is used and stored
- **Loss Rate**: Base energy consumption per tick
- **Gain Rate**: Efficiency of consuming other entities
- **Size Factor**: How size relates to energy requirements

#### Reproduction Genes

- **Rate**: Likelihood of successful reproduction
- **Mutation Rate**: How much genes change in offspring

#### Appearance Genes

- **Hue**: Color hue (0.0-1.0)
- **Saturation**: Color saturation (0.0-1.0)

### Emergent Interactions

- **Predation**: Based on relative speed and size advantages
- **Energy Transfer**: Efficient energy gain with diminishing returns
- **Population Control**: Density-based reproduction and death rates
- **Size Constraints**: Natural limits prevent oversized entities

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
- `--config <path>`: Load simulation configuration from JSON file
- `--create-config <path>`: Create a default configuration file at the specified path

## Performance Optimizations

The simulation is heavily optimized for performance:

- **Rayon Parallelization**:
  - Entity updates processed in parallel
  - Spatial grid data extraction parallelized
  - Metrics collection parallelized
  - Update preparation parallelized
- **Spatial Grid**: O(n²) → O(n) complexity for entity interactions
- **Efficient ECS**: Hecs for fast entity queries and updates
- **GPU Acceleration**: WGPU for smooth real-time graphics
- **Modular Systems**: Focused systems for better performance and maintainability

## Simulation Rules

### Core Mechanics

1. **Energy System**: All entities consume energy over time based on size and activity
2. **Movement**: Entities move toward targets within their sense radius
3. **Predation**: Larger/faster entities can consume smaller/slower ones
4. **Reproduction**: High-energy entities reproduce with genetic mutations
5. **Population Control**: Density-based reproduction suppression and death rates
6. **Size Constraints**: Entities are limited to reasonable size ranges

### Advanced Features

- **Boundary Handling**: Smart boundary detection with centering forces
- **Drift Prevention**: Position validation and correction mechanisms
- **Stable Spawning**: Uniform distribution prevents initial bias
- **Balanced Growth**: Multiple mechanisms prevent population explosions

## Technical Architecture

### ECS Components

- **Position**: 2D coordinates with boundary validation
- **Energy**: Current and maximum energy levels
- **Size**: Radius-based size with constraints
- **Velocity**: Movement vector with physics validation
- **Color**: RGB color derived from genetic traits
- **Genes**: Inheritable traits that define behavior

### Systems Architecture

- **MovementSystem**: Handles entity movement and boundary constraints
- **InteractionSystem**: Manages entity interactions and predation
- **EnergySystem**: Handles energy consumption and metabolism
- **ReproductionSystem**: Manages reproduction and population control

### Key Optimizations

- **Spatial Grid**: Efficient neighbor finding and collision detection
- **Parallel Processing**: Rayon-based parallel entity updates
- **Boundary Management**: Advanced boundary handling with drift correction
- **Population Control**: Multi-layered population balance mechanisms

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

## Configuration

The simulation supports configuration files for easy experimentation. Create a default configuration:

```bash
cargo run -- --create-config my_config.json
```

Modify the configuration file to adjust simulation parameters, then run with your custom settings:

```bash
cargo run -- --config my_config.json
```

### Configuration Parameters

- **entity_scale**: Global scaling factor for entity counts
- **max_population**: Maximum number of entities allowed
- **initial_entities**: Number of entities at simulation start
- **max_velocity**: Maximum movement speed
- **max_entity_radius**: Largest possible entity size
- **min_entity_radius**: Smallest possible entity size
- **spawn_radius_factor**: Initial spawn area size (relative to world size)
- **grid_cell_size**: Spatial grid cell size for optimization
- **boundary_margin**: Distance from world edge for boundary handling
- **interaction_radius_offset**: Extra radius for entity interactions
- **reproduction_energy_threshold**: Energy level required for reproduction
- **reproduction_energy_cost**: Energy cost of reproduction
- **child_energy_factor**: Initial energy of offspring
- **child_spawn_radius**: Distance from parent for child spawning
- **size_energy_cost_factor**: Energy cost multiplier for large entities
- **movement_energy_cost**: Energy cost of movement
- **population_density_factor**: Population pressure on reproduction
- **min_reproduction_chance**: Minimum reproduction probability
- **death_chance_factor**: Population density death rate multiplier
- **drift_compensation_x/y**: Compensation for systematic position drift
- **velocity_bounce_factor**: Velocity reduction on boundary collision

## Code Quality

The project demonstrates excellent software engineering practices:

- **Modular Design**: Clean separation of concerns with focused modules
- **Type Safety**: Comprehensive use of Rust's type system
- **Error Handling**: Proper error handling with `Result` types
- **Testing**: Comprehensive test coverage for critical components
- **Documentation**: Clear documentation and examples
- **Performance**: Optimized for both development and production use

## Future Enhancements

- Environmental factors (terrain, resources)
- Social behaviors and group dynamics
- Advanced visualization options
- Data export and analysis tools
- WebAssembly support for browser deployment
- More complex gene interactions and epigenetics
- Additional simulation systems (environment, weather, etc.)
