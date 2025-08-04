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

## Quick Start

This project uses npm scripts for web deployment and standard Rust commands for desktop development.

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

4. **Node.js** (for web server and wasm-pack):

   ```bash
   # macOS (with Homebrew)
   brew install node
   
   # Ubuntu/Debian
   sudo apt install nodejs npm
   
   # Windows
   # Download from https://nodejs.org/
   ```

### First Time Setup

```bash
# Install dependencies and set up environment
npm run setup
```

### Running the Simulation

#### Desktop Application (Recommended)

```bash
# Run with beautiful GPU-accelerated graphics
just desktop

# Or manually:
cargo run --release
```

#### Web Application

```bash
# Run in your web browser
npm run dev

# Or manually:
npm run build:web
node web/server.js

Then open your browser to `http://localhost:8000`

#### Headless Mode (Console Only)

```bash
# Run without graphics (faster for testing)
cargo run --release -- --headless
```

### Development Commands

```bash
# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build only (no run)
npm run build:web    # Web build
cargo build --release # Desktop build

# Clean build artifacts
cargo clean
```

> **Note**: Some tests may fail due to the nightly toolchain configuration. This is normal and doesn't affect the build process.

### Web Deployment

The project is configured for easy deployment to Cloudflare Workers:

```bash
# Deploy to Cloudflare Workers
npm run deploy

# Test worker locally before deploying
npm run dev:worker
```

The application is live at: [https://evo-simulation.rob-gilks.workers.dev/](https://evo-simulation.rob-gilks.workers.dev/)

For detailed deployment instructions, see [DEPLOYMENT.md](DEPLOYMENT.md).

### Build Process

The web build process uses `npm run build:web` which:

1. **Cleans previous builds** to ensure a fresh start
2. **Builds the WASM package** using `wasm-pack`
3. **Fixes worker import paths** for `wasm-bindgen-rayon` (required for parallel processing in web browsers)
4. **Verifies the build** to ensure everything worked correctly

The worker import fix is necessary because `wasm-bindgen-rayon` generates worker helpers that expect to import the main WASM module using relative paths that don't work in our simple web setup without a bundler.

## Project Structure

```
evo/
├── src/                    # Rust source code
│   ├── components.rs       # ECS components
│   ├── genes.rs           # Genetic system
│   ├── systems.rs         # Simulation systems
│   ├── spatial_grid.rs    # Spatial optimization
│   ├── stats.rs           # Analytics and statistics
│   ├── simulation.rs      # Main simulation orchestration
│   ├── config.rs          # Configuration management
│   ├── ui.rs              # GPU-accelerated rendering (desktop)
│   ├── web/               # Web-specific modules
│   └── main.rs            # Application entry point
├── web/                    # Web application
│   ├── index.html         # Main HTML page
│   ├── css/style.css      # Stylesheets
│   ├── js/app.js          # JavaScript application
│   ├── assets/            # Static assets
│   └── server.py          # Development server
├── scripts/                # Build and utility scripts
├── pkg/                    # Generated WebAssembly files
├── config.json             # Default configuration
├── config.json            # Configuration file
└── README.md              # This documentation
```

## Evolution Mechanics

### Gene-Based Behaviors

All behaviors emerge from genes organized into logical groups:

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

## Configuration

Create a custom configuration:

```bash
cargo run -- --create-config my_config.json
```

Run with custom configuration:

```bash
cargo run -- --config my_config.json
```

### Key Configuration Parameters

- **initial_entities**: Number of entities at simulation start (default: 1000)
- **max_population**: Maximum number of entities allowed
- **entity_scale**: Global scaling factor for entity counts
- **world_size**: Size of the simulation world
- **grid_cell_size**: Spatial grid cell size for optimization

## Command Line Options

- `--headless`: Run without graphics (faster for testing)
- `--steps <number>`: Number of simulation steps in headless mode (default: 1000)
- `--world-size <number>`: Size of the simulation world (default: 1000)
- `--config <path>`: Load simulation configuration from JSON file
- `--create-config <path>`: Create a default configuration file at the specified path

## Performance Optimizations

### Desktop Optimizations

- **Rayon Parallelization**: Entity updates processed in parallel
- **Spatial Grid**: O(n²) → O(n) complexity for entity interactions
- **Efficient ECS**: Hecs for fast entity queries and updates
- **GPU Acceleration**: WGPU for smooth real-time graphics

### Web Optimizations

- **WebAssembly Performance**: Runs Rust code directly in browser
- **Parallel Processing**: Uses `wasm-bindgen-rayon` with Web Workers
- **Memory Management**: Optimized WASM memory usage
- **Rendering**: Canvas 2D rendering optimized for smooth animations

## Troubleshooting

### Common Issues

1. **Build errors with `-Zbuild-std requires --target`**:

   ```bash
   cargo run --release --target x86_64-apple-darwin  # macOS Intel
   cargo run --release --target aarch64-apple-darwin # macOS Apple Silicon
   ```

2. **WebAssembly build fails**:

   ```bash
   rustup default nightly-2024-08-02
   rustup target add wasm32-unknown-unknown
   ```

3. **Web server issues**:

   ```bash
   # Use the provided Node.js server (recommended)
   node web/server.js
   
   # Or use npm scripts
   npm run dev
   ```

4. **Performance issues**:
   - Use `--release` flag for optimal performance
   - Reduce entity count in configuration
   - Lower frame rate using the speed slider

### Browser Requirements

- **WebAssembly Support**: Modern browsers (Chrome 57+, Firefox 52+, Safari 11+)
- **SharedArrayBuffer Support**: Required for parallel processing
- **Web Workers Support**: For multi-threading

## Development

### Architecture

- **ECS Components**: Position, Energy, Size, Velocity, Color, Genes
- **Systems**: Movement, Interaction, Energy, Reproduction
- **Spatial Optimization**: Grid-based neighbor finding
- **Parallel Processing**: Rayon-based entity updates

### Key Modules

- **`components.rs`**: ECS components
- **`genes.rs`**: Genetic system with grouped traits
- **`systems.rs`**: Simulation systems
- **`spatial_grid.rs`**: Spatial optimization
- **`simulation.rs`**: Main simulation orchestration
- **`ui.rs`**: GPU-accelerated rendering (desktop)
- **`web/`**: Web-specific modules

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Recent Updates

### Enhanced Entity Diversity

- **Doubled Population**: Initial entities increased from 500 to 1000
- **Expanded Gene Ranges**: All genetic traits now have wider ranges for greater variation
- **Enhanced Initial Conditions**: More diverse starting energy levels (15-75 vs 25-55)
- **Improved Size Variation**: Entities can now utilize the full size range from min to max radius

## Future Enhancements

- Environmental factors (terrain, resources)
- Social behaviors and group dynamics
- Advanced visualization options
- Data export and analysis tools
- More complex gene interactions and epigenetics
- Mobile-optimized web interface
- Real-time multiplayer capabilities

---

**Happy evolving! 🧬**
