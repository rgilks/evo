# Evolution Simulation - Web Implementation

This directory contains the web browser implementation of the evolution simulation using WebAssembly and `wasm-bindgen-rayon` for parallel processing.

## Features

- **WebAssembly Performance**: Runs the Rust simulation code directly in the browser
- **Parallel Processing**: Uses `wasm-bindgen-rayon` for multi-threaded computation via Web Workers
- **Real-time Visualization**: HTML5 Canvas rendering with smooth animations
- **Interactive Controls**: Play/pause, step, reset, and speed control
- **Live Statistics**: Real-time population and performance metrics
- **Responsive Design**: Works on desktop and mobile devices

## Prerequisites

### Required Tools

1. **Rust Nightly Toolchain**: The project uses nightly Rust for WASM atomics support
2. **wasm-pack**: For building WebAssembly packages
3. **Python 3**: For serving the web application with proper CORS headers

### Installation

```bash
# Install Rust nightly toolchain
rustup toolchain install nightly-2024-08-02

# Install wasm-pack
cargo install wasm-pack

# Install Node.js dependencies (optional, for alternative serving)
npm install
```

## Building and Running

### Step 1: Build the WebAssembly Package

```bash
# Build the WASM package for web
wasm-pack build --target web --out-dir pkg
```

This command will:

- Compile the Rust code to WebAssembly
- Generate JavaScript bindings
- Create the `pkg/` directory with all necessary files

### Step 2: Serve the Web Application

#### Option A: Using Python (Recommended)

```bash
# Start the development server with CORS headers
python3 web/server.py
```

Then open your browser to `http://localhost:8000`

#### Option B: Using Node.js

```bash
# Install serve globally if not already installed
npm install -g serve

# Serve the web directory
npx serve web
```

**Note**: The Python server is recommended because it automatically sets the required CORS headers for SharedArrayBuffer support.

### Step 3: Access the Application

Open your web browser and navigate to:

- **Python server**: `http://localhost:8000`
- **Node.js server**: `http://localhost:3000` (or the port shown in the terminal)

## Browser Requirements

### Required Features

- **WebAssembly Support**: Modern browsers (Chrome 57+, Firefox 52+, Safari 11+)
- **SharedArrayBuffer Support**: Required for parallel processing
- **Web Workers Support**: For multi-threading
- **Canvas 2D Context**: For rendering

### Browser Compatibility

| Browser | Version | Status          |
| ------- | ------- | --------------- |
| Chrome  | 67+     | ✅ Full Support |
| Firefox | 79+     | ✅ Full Support |
| Safari  | 15.2+   | ✅ Full Support |
| Edge    | 79+     | ✅ Full Support |

### Fallback Support

For browsers without SharedArrayBuffer support, the application will fall back to single-threaded mode with a warning message.

## Project Structure

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

## Configuration

### Simulation Parameters

The simulation can be configured by modifying the config object in `web/app.js`:

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

### Performance Tuning

1. **Thread Count**: Automatically uses `navigator.hardwareConcurrency`
2. **Frame Rate**: Adjustable via the speed slider (1-60 FPS)
3. **Entity Count**: Modify `initial_entities` and `max_population` in config

## Development

### Adding New Features

1. **Rust Backend**: Add new functionality to the simulation modules
2. **WASM Bridge**: Expose new functions in `src/lib.rs`
3. **JavaScript Frontend**: Add UI controls and rendering in `web/app.js`

### Debugging

#### Rust/WASM Debugging

```bash
# Enable debug symbols
wasm-pack build --target web --out-dir pkg --debug

# Use browser dev tools to debug WASM
# Console logs from Rust will appear in browser console
```

#### JavaScript Debugging

- Use browser dev tools
- Check console for error messages
- Monitor performance with browser profiler

### Testing

```bash
# Run Rust tests
cargo test

# Test WASM build
wasm-pack test --headless --firefox
```

## Deployment

### Static Hosting

The web application can be deployed to any static hosting service:

1. **GitHub Pages**: Push to a repository and enable Pages
2. **Netlify**: Drag and drop the `web/` directory
3. **Vercel**: Connect your repository

### Production Build

```bash
# Optimized build for production
wasm-pack build --target web --out-dir pkg --release

# The pkg/ directory contains all necessary files
```

### CORS Configuration

For production deployment, ensure your server sets the required headers:

```
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Opener-Policy: same-origin
```

## Troubleshooting

### Common Issues

1. **"SharedArrayBuffer is not defined"**

   - Ensure you're using the Python server or have proper CORS headers
   - Check browser compatibility

2. **"Failed to initialize simulation"**

   - Check browser console for detailed error messages
   - Verify WASM files are properly loaded

3. **Poor Performance**

   - Reduce entity count in configuration
   - Lower frame rate using the speed slider
   - Check if parallel processing is working (should see multiple threads in dev tools)

4. **Build Errors**
   - Ensure you have the correct Rust nightly toolchain
   - Check that all dependencies are installed

### Getting Help

1. Check the browser console for error messages
2. Verify all prerequisites are installed
3. Ensure you're using a compatible browser
4. Check the main project README for additional information

## Performance Notes

- **Parallel Processing**: The simulation uses Web Workers for multi-threading
- **Memory Management**: WASM memory is automatically managed
- **Rendering**: Canvas 2D rendering is optimized for smooth animations
- **Scalability**: Performance scales with available CPU cores

## License

This web implementation follows the same license as the main project.
