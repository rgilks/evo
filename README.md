# Evolution Simulation

A beautiful and performant evolution simulation written in Rust, featuring an Entity Component System (ECS) with parallel processing and GPU-accelerated graphics. Watch as complex behaviors emerge naturally from simple genetic rules!

## Features

- **Entity Component System**: Uses the `hecs` crate for efficient entity management
- **Parallel Processing**: Leverages `rayon` to maximize performance on multi-core systems
- **GPU Graphics**: Beautiful real-time visualization using `wgpu`
- **WebAssembly Support**: Run in web browsers with parallel processing via `wasm-bindgen-rayon`
- **Headless Mode**: Run simulations without UI for testing and analysis
- **Emergent Behaviors**: Complex predator-prey dynamics emerge from simple genetic rules
- **Population Balance**: Sophisticated population control mechanisms prevent explosions
- **Stable Physics**: Advanced boundary handling and drift correction
- **Modular Architecture**: Clean separation of concerns with focused modules

## Quick Start

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Rust nightly toolchain** (required for WebAssembly):
   ```bash
   rustup toolchain install nightly-2024-08-02
   ```

3. **Install wasm-pack** (for web builds):
   ```bash
   cargo install wasm-pack
   ```

### Desktop Application (Recommended)

The desktop version offers the best performance and visual experience with GPU-accelerated graphics.

#### Simple Commands (Recommended)

```bash
# First time setup
./setup.sh

# Run desktop application with UI
./run.sh desktop

# Run headless simulation
./run.sh headless --steps 1000

# Run tests
./run.sh test

# Build only (no run)
./run.sh build

# Show all available commands
./run.sh help
```

#### Manual Commands

```bash
# Build and run with UI (default)
cargo run --release

# Run headless mode (console only)
cargo run --release -- --headless --steps 1000

# Run with custom configuration
cargo run --release -- --config my_config.json

# Create a default configuration file
cargo run --release -- --create-config my_config.json
```

#### Advanced Options

```bash
# Run with specific world size
cargo run --release -- --world-size 800

# Run headless with custom parameters
cargo run --release -- --headless --steps 5000 --world-size 1000

# Build only (without running)
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
```

### Web Application

The web version runs in any modern browser with parallel processing support.

#### Simple Commands (Recommended)

```bash
# Build and serve web application
./run.sh web

# Or use the dedicated script:
./build-web.sh
```

#### Manual Commands

```bash
# Build and serve web application
wasm-pack build --target web --out-dir pkg
python3 web/server.py
```

#### Manual Web Build

```bash
# Build WASM package
wasm-pack build --target web --out-dir pkg

# Fix worker imports (if needed)
./fix-worker-imports.sh

# Serve with Python
python3 web/server.py

# Or serve with Node.js
npx serve web
```

Then open your browser to `http://localhost:8000`

### Platform-Specific Notes

#### macOS
- **Desktop**: Works out of the box with Metal GPU acceleration
- **Web**: Requires Python 3 for the development server

#### Linux
- **Desktop**: Uses Vulkan or OpenGL for GPU acceleration
- **Web**: Same as macOS

#### Windows
- **Desktop**: Uses DirectX 12 or Vulkan for GPU acceleration
- **Web**: Same as other platforms

### Troubleshooting

#### Common Issues

1. **Build errors with `-Zbuild-std requires --target`**:
   ```bash
   # Use native target explicitly
   cargo run --release --target x86_64-apple-darwin  # macOS Intel
   cargo run --release --target aarch64-apple-darwin # macOS Apple Silicon
   ```

2. **WebAssembly build fails**:
   ```bash
   # Ensure you have the correct toolchain
   rustup default nightly-2024-08-02
   rustup target add wasm32-unknown-unknown
   ```

3. **Missing dependencies**:
   ```bash
   # Update Rust and install missing tools
   rustup update
   cargo install wasm-pack
   ```

4. **Web server CORS issues**:
   ```bash
   # Use the provided Python server (recommended)
   python3 web/server.py
   ```

#### Performance Tips

- **Desktop**: Use `--release` flag for optimal performance
- **Web**: Ensure browser supports SharedArrayBuffer for parallel processing
- **Headless**: Use for batch processing or testing without graphics overhead

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
- **`ui.rs`**: GPU-accelerated rendering (desktop)
- **`web/`**: Web-specific modules for browser implementation

### Design Principles

- **Single Responsibility**: Each module has a focused purpose
- **Separation of Concerns**: Logic is cleanly separated into systems
- **Performance First**: Parallel processing and spatial optimization
- **Configuration Driven**: External JSON configuration for easy experimentation

## Evolution Mechanics

### Gene-Based Behaviors

Instead of predefined entity types, all behaviors emerge from genes organized into logical groups:

#### Movement Genes

- **Speed**: Movement velocity and hunting effectiveness (0.1-2.5)
- **Sense Radius**: Detection range for food and threats (5.0-150.0)

#### Energy Genes

- **Efficiency**: How efficiently energy is used and stored (0.3-3.0)
- **Loss Rate**: Base energy consumption per tick (0.05-2.0)
- **Gain Rate**: Efficiency of consuming other entities (0.2-4.5)
- **Size Factor**: How size relates to energy requirements (0.3-2.5)

#### Reproduction Genes

- **Rate**: Likelihood of successful reproduction (0.0005-0.15)
- **Mutation Rate**: How much genes change in offspring (0.005-0.15)

#### Appearance Genes

- **Hue**: Color hue (0.0-1.0)
- **Saturation**: Color saturation (0.2-1.0)

### Emergent Interactions

- **Predation**: Based on relative speed and size advantages
- **Energy Transfer**: Efficient energy gain with diminishing returns
- **Population Control**: Density-based reproduction and death rates
- **Size Constraints**: Natural limits prevent oversized entities

## Usage

### Desktop Application

#### With UI (Default)

```bash
cargo run --release
```

#### Headless Mode (Console Only)

```bash
cargo run --release -- --headless --steps 1000 --world-size 1000
```

#### Command Line Options

- `--headless`: Run without graphics (faster for testing)
- `--steps <number>`: Number of simulation steps in headless mode (default: 1000)
- `--world-size <number>`: Size of the simulation world (default: 1000)
- `--config <path>`: Load simulation configuration from JSON file
- `--create-config <path>`: Create a default configuration file at the specified path

### Web Application

#### Browser Requirements

- **WebAssembly Support**: Modern browsers (Chrome 57+, Firefox 52+, Safari 11+)
- **SharedArrayBuffer Support**: Required for parallel processing
- **Web Workers Support**: For multi-threading
- **Canvas 2D Context**: For rendering

#### Browser Compatibility

| Browser | Version | Status          |
| ------- | ------- | --------------- |
| Chrome  | 67+     | ✅ Full Support |
| Firefox | 79+     | ✅ Full Support |
| Safari  | 15.2+   | ✅ Full Support |
| Edge    | 79+     | ✅ Full Support |

#### Interactive Controls

- **Play/Pause**: Start or stop the simulation
- **Step**: Advance simulation by one step
- **Reset**: Restart simulation with fresh entities
- **Speed Slider**: Adjust simulation speed (1-60 FPS)
- **Live Statistics**: Real-time population and performance metrics

## Performance Optimizations

The simulation is heavily optimized for performance:

### Desktop Optimizations

- **Rayon Parallelization**:
  - Entity updates processed in parallel
  - Spatial grid data extraction parallelized
  - Metrics collection parallelized
  - Update preparation parallelized
- **Spatial Grid**: O(n²) → O(n) complexity for entity interactions
- **Efficient ECS**: Hecs for fast entity queries and updates
- **GPU Acceleration**: WGPU for smooth real-time graphics

### Web Optimizations

- **WebAssembly Performance**: Runs Rust code directly in browser
- **Parallel Processing**: Uses `wasm-bindgen-rayon` with Web Workers
- **Memory Management**: Optimized WASM memory usage
- **Rendering**: Canvas 2D rendering optimized for smooth animations
- **Scalability**: Performance scales with available CPU cores

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
- **initial_entities**: Number of entities at simulation start (default: 1000)
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

### Web Configuration

For the web version, modify the config object in `web/app.js`:

```javascript
const config = {
  initial_entities: 100, // Number of initial entities
  max_population: 500, // Maximum population limit
  entity_scale: 1.0, // Entity size scaling
  grid_cell_size: 50.0, // Spatial grid cell size
  spawn_radius_factor: 0.8, // Initial spawn area
  min_entity_radius: 2.0, // Minimum entity size
  max_entity_radius: 8.0, // Maximum entity size
  energy_decay_rate: 0.1, // Energy consumption rate
  reproduction_threshold: 80.0, // Energy needed for reproduction
  reproduction_cost: 40.0, // Energy cost of reproduction
  mutation_rate: 0.1, // Gene mutation probability
  mutation_strength: 0.2, // Mutation magnitude
  interaction_radius: 20.0, // Entity interaction range
  boundary_elasticity: 0.8, // Boundary bounce factor
  drift_correction: true, // Enable drift correction
};
```

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

## Web Implementation Details

### Project Structure

```
evo/
├── src/
│   ├── lib.rs                    # WASM library entry point
│   ├── web/                      # Web-specific modules
│   │   ├── mod.rs                # Web renderer and controls
│   │   ├── renderer.rs           # Canvas rendering utilities
│   │   ├── controls.rs           # UI control handlers
│   │   └── wasm_bridge.rs        # WASM-JS bridge utilities
│   └── [other simulation modules]
├── web/                          # Web assets
│   ├── index.html                # Main HTML page
│   ├── style.css                 # Styling
│   ├── app.js                    # Main JavaScript application
│   └── server.py                 # Development server
├── pkg/                          # Generated WASM package
├── Cargo.toml                    # Rust dependencies
├── package.json                  # Node.js dependencies
├── rust-toolchain.toml           # Rust toolchain configuration
└── .cargo/config.toml            # Cargo configuration
```

### Dependencies

#### Required Tools

1. **Rust Nightly Toolchain**: For WASM atomics support
2. **wasm-pack**: For building WebAssembly packages
3. **Python 3**: For serving the web application with proper CORS headers

#### Cargo.toml Dependencies

```toml
[dependencies]
# Core simulation
hecs = "0.9"
rayon = "1.8"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# WebAssembly support
wasm-bindgen = "0.2"
wasm-bindgen-rayon = "1.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "console", "Document", "Element", "HtmlCanvasElement",
    "CanvasRenderingContext2d", "Window", "request_animation_frame",
    "Performance", "Worker", "WorkerGlobalScope", "MessageEvent"
]}
```

### Build Process

```bash
# Build the WASM package for web
wasm-pack build --target web --out-dir pkg

# Serve with CORS headers (required for SharedArrayBuffer)
python3 web/server.py
```

### Deployment

The web application can be deployed to any static hosting service:

1. **GitHub Pages**: Push to a repository and enable Pages
2. **Netlify**: Drag and drop the `web/` directory
3. **Vercel**: Connect your repository

For production deployment, ensure your server sets the required headers:

```
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Opener-Policy: same-origin
```

## Troubleshooting

### Desktop Issues

1. **Build Errors**: Ensure you have the correct Rust toolchain
2. **Performance Issues**: Use `--release` flag for optimized builds
3. **Configuration Errors**: Check JSON syntax in config files

### Web Issues

1. **"SharedArrayBuffer is not defined"**
   - Ensure you're using the Python server or have proper CORS headers
   - Check browser compatibility

2. **"Failed to initialize simulation"**
   - Check browser console for detailed error messages
   - Verify WASM files are properly loaded

3. **Poor Performance**
   - Reduce entity count in configuration
   - Lower frame rate using the speed slider
   - Check if parallel processing is working

4. **Build Errors**
   - Ensure you have the correct Rust nightly toolchain
   - Check that all dependencies are installed

## Code Quality

The project demonstrates excellent software engineering practices:

- **Modular Design**: Clean separation of concerns with focused modules
- **Type Safety**: Comprehensive use of Rust's type system
- **Error Handling**: Proper error handling with `Result` types
- **Testing**: Comprehensive test coverage for critical components
- **Documentation**: Clear documentation and examples
- **Performance**: Optimized for both development and production use

## Recent Updates

### Enhanced Entity Diversity (Latest)

The simulation now features significantly increased entity diversity:

- **Doubled Population**: Initial entities increased from 500 to 1000
- **Expanded Gene Ranges**: All genetic traits now have wider ranges for greater variation
- **Enhanced Initial Conditions**: More diverse starting energy levels (15-75 vs 25-55)
- **Improved Size Variation**: Entities can now utilize the full size range from min to max radius

This creates a much more dynamic and interesting evolutionary environment with greater potential for diverse strategies to emerge and compete.

## Future Enhancements

- Environmental factors (terrain, resources)
- Social behaviors and group dynamics
- Advanced visualization options
- Data export and analysis tools
- More complex gene interactions and epigenetics
- Additional simulation systems (environment, weather, etc.)
- Mobile-optimized web interface
- Real-time multiplayer capabilities
