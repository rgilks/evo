# Web Browser Implementation Guide for Evolution Simulation

This document outlines how to implement a web browser UI for the evolution simulation using `wasm-bindgen-rayon` for parallel processing in WebAssembly.

## Overview

The goal is to create an alternative UI that runs in web browsers while maintaining the parallel processing capabilities of the desktop version. This will use:

- **WebAssembly (WASM)** for running the Rust simulation code
- **wasm-bindgen-rayon** for parallel processing with Web Workers
- **HTML5 Canvas** for rendering
- **JavaScript** for UI controls and WebAssembly integration

## Architecture

### 1. Project Structure

```
evo/
├── Cargo.toml                    # Main Rust project
├── src/
│   ├── lib.rs                    # WebAssembly library entry point
│   ├── simulation.rs             # Core simulation logic (shared)
│   ├── components.rs             # ECS components (shared)
│   ├── genes.rs                  # Genetic system (shared)
│   ├── systems.rs                # Simulation systems (shared)
│   ├── spatial_grid.rs           # Spatial optimization (shared)
│   ├── stats.rs                  # Statistics (shared)
│   ├── config.rs                 # Configuration (shared)
│   └── web/                      # Web-specific modules
│       ├── mod.rs                # Web module entry point
│       ├── renderer.rs           # Canvas rendering
│       ├── controls.rs           # UI controls
│       └── wasm_bridge.rs        # WASM-JS bridge
├── web/                          # Web assets
│   ├── index.html                # Main HTML page
│   ├── style.css                 # Styling
│   ├── app.js                    # Main JavaScript application
│   └── workers/                  # Web Worker scripts
└── pkg/                          # Generated WASM package
```

### 2. Dependencies

#### Cargo.toml Updates

```toml
[package]
name = "evo"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Core simulation (existing)
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
    "console",
    "Document",
    "Element",
    "HtmlCanvasElement",
    "WebGlRenderingContext",
    "WebGl2RenderingContext",
    "Window",
    "CanvasRenderingContext2d",
    "request_animation_frame",
    "Performance",
    "Blob",
    "Url",
    "Worker",
    "WorkerGlobalScope",
    "MessageEvent",
    "DedicatedWorkerGlobalScope"
]}

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-rayon = "1.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[profile.release]
opt-level = "s"  # Optimize for size in WASM
lto = true
codegen-units = 1
panic = "abort"
```

### 3. Rust Toolchain Configuration

#### rust-toolchain.toml
```toml
[toolchain]
channel = "nightly-2024-08-02"
components = ["rust-src"]
targets = ["wasm32-unknown-unknown"]
```

#### .cargo/config.toml
```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+atomics,+bulk-memory"]

[unstable]
build-std = ["panic_abort", "std"]
```

## Implementation Steps

### Step 1: Create WebAssembly Library Entry Point

#### src/lib.rs
```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_rayon::init_thread_pool;

mod components;
mod config;
mod genes;
mod simulation;
mod spatial_grid;
mod stats;
mod systems;

#[cfg(target_arch = "wasm32")]
mod web;

// Re-export the thread pool initialization
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub struct WebSimulation {
    simulation: simulation::Simulation,
    config: config::SimulationConfig,
}

#[wasm_bindgen]
impl WebSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(world_size: f32, config_json: &str) -> Result<WebSimulation, JsValue> {
        let config: config::SimulationConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config parse error: {}", e)))?;
        
        let simulation = simulation::Simulation::new_with_config(world_size, config.clone());
        
        Ok(WebSimulation {
            simulation,
            config,
        })
    }

    pub fn update(&mut self) {
        self.simulation.update();
    }

    pub fn get_entities(&self) -> JsValue {
        let entities = self.simulation.get_entities();
        serde_wasm_bindgen::to_value(&entities)
            .unwrap_or_else(|_| JsValue::NULL)
    }

    pub fn get_stats(&self) -> JsValue {
        let stats = stats::SimulationStats::from_world(
            self.simulation.world(),
            self.config.max_population as f32,
            self.config.entity_scale,
        );
        serde_wasm_bindgen::to_value(&stats)
            .unwrap_or_else(|_| JsValue::NULL)
    }

    pub fn get_world_size(&self) -> f32 {
        self.simulation.world_size()
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
```

### Step 2: Web-Specific Modules

#### src/web/mod.rs
```rust
pub mod renderer;
pub mod controls;
pub mod wasm_bridge;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl WebRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<WebRenderer, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;
        
        let ctx = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width();
        let height = canvas.height();

        Ok(WebRenderer {
            canvas,
            ctx,
            width,
            height,
        })
    }

    pub fn render(&self, entities: &JsValue) -> Result<(), JsValue> {
        // Parse entities from JS
        let entities: Vec<(f32, f32, f32, f32, f32, f32)> = 
            serde_wasm_bindgen::from_value(entities.clone())?;

        // Clear canvas
        self.ctx.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // Render each entity
        for (x, y, radius, r, g, b) in entities {
            self.ctx.begin_path();
            self.ctx.arc(
                x as f64, 
                y as f64, 
                radius as f64, 
                0.0, 
                2.0 * std::f64::consts::PI
            )?;
            
            self.ctx.set_fill_style(&JsValue::from_str(&format!(
                "rgba({}, {}, {}, 0.8)", 
                (r * 255.0) as u8, 
                (g * 255.0) as u8, 
                (b * 255.0) as u8
            )));
            self.ctx.fill()?;
        }

        Ok(())
    }
}
```

### Step 3: HTML Interface

#### web/index.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Evolution Simulation</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>Evolution Simulation</h1>
            <div class="controls">
                <button id="play-pause">Play</button>
                <button id="reset">Reset</button>
                <button id="step">Step</button>
                <input type="range" id="speed" min="1" max="60" value="30">
                <label for="speed">Speed</label>
            </div>
        </header>
        
        <main>
            <canvas id="simulation-canvas" width="800" height="600"></canvas>
            
            <div class="stats-panel">
                <h3>Statistics</h3>
                <div id="stats-display">
                    <div>Population: <span id="population">0</span></div>
                    <div>Step: <span id="step-count">0</span></div>
                    <div>FPS: <span id="fps">0</span></div>
                </div>
            </div>
        </main>
    </div>
    
    <script type="module" src="app.js"></script>
</body>
</html>
```

### Step 4: CSS Styling

#### web/style.css
```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
    color: white;
    min-height: 100vh;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

header {
    text-align: center;
    margin-bottom: 20px;
}

h1 {
    font-size: 2.5rem;
    margin-bottom: 20px;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
}

.controls {
    display: flex;
    gap: 15px;
    justify-content: center;
    align-items: center;
    flex-wrap: wrap;
}

button {
    padding: 10px 20px;
    border: none;
    border-radius: 5px;
    background: rgba(255,255,255,0.2);
    color: white;
    cursor: pointer;
    transition: all 0.3s ease;
    font-size: 1rem;
}

button:hover {
    background: rgba(255,255,255,0.3);
    transform: translateY(-2px);
}

button:active {
    transform: translateY(0);
}

input[type="range"] {
    width: 100px;
}

main {
    display: flex;
    gap: 20px;
    align-items: flex-start;
}

#simulation-canvas {
    border: 2px solid rgba(255,255,255,0.3);
    border-radius: 10px;
    background: rgba(0,0,0,0.2);
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
}

.stats-panel {
    background: rgba(255,255,255,0.1);
    padding: 20px;
    border-radius: 10px;
    min-width: 200px;
    backdrop-filter: blur(10px);
}

.stats-panel h3 {
    margin-bottom: 15px;
    text-align: center;
}

#stats-display div {
    margin-bottom: 10px;
    display: flex;
    justify-content: space-between;
}

#stats-display span {
    font-weight: bold;
    color: #4CAF50;
}

@media (max-width: 768px) {
    main {
        flex-direction: column;
    }
    
    #simulation-canvas {
        width: 100%;
        height: auto;
    }
}
```

### Step 5: JavaScript Application

#### web/app.js
```javascript
import init, { initThreadPool, WebSimulation, WebRenderer, init_panic_hook } from '../pkg/evo.js';

class EvolutionApp {
    constructor() {
        this.simulation = null;
        this.renderer = null;
        this.isRunning = false;
        this.animationId = null;
        this.lastTime = 0;
        this.frameCount = 0;
        this.fps = 0;
        
        this.init();
    }

    async init() {
        try {
            // Initialize WASM
            await init();
            init_panic_hook();
            
            // Initialize thread pool
            await initThreadPool(navigator.hardwareConcurrency);
            
            // Create simulation
            const config = {
                initial_entities: 100,
                max_population: 500,
                entity_scale: 1.0,
                grid_cell_size: 50.0,
                spawn_radius_factor: 0.8,
                min_entity_radius: 2.0,
                max_entity_radius: 8.0,
                energy_decay_rate: 0.1,
                reproduction_threshold: 80.0,
                reproduction_cost: 40.0,
                mutation_rate: 0.1,
                mutation_strength: 0.2,
                interaction_radius: 20.0,
                boundary_elasticity: 0.8,
                drift_correction: true
            };
            
            this.simulation = new WebSimulation(800, JSON.stringify(config));
            this.renderer = new WebRenderer('simulation-canvas');
            
            this.setupEventListeners();
            this.startRenderLoop();
            
        } catch (error) {
            console.error('Failed to initialize:', error);
            this.showError('Failed to initialize simulation');
        }
    }

    setupEventListeners() {
        const playPauseBtn = document.getElementById('play-pause');
        const resetBtn = document.getElementById('reset');
        const stepBtn = document.getElementById('step');
        const speedSlider = document.getElementById('speed');

        playPauseBtn.addEventListener('click', () => this.togglePlayPause());
        resetBtn.addEventListener('click', () => this.reset());
        stepBtn.addEventListener('click', () => this.step());
        speedSlider.addEventListener('input', (e) => {
            this.targetFPS = parseInt(e.target.value);
        });
    }

    togglePlayPause() {
        this.isRunning = !this.isRunning;
        const btn = document.getElementById('play-pause');
        btn.textContent = this.isRunning ? 'Pause' : 'Play';
        
        if (this.isRunning) {
            this.startRenderLoop();
        } else {
            this.stopRenderLoop();
        }
    }

    reset() {
        // Recreate simulation with same config
        const config = {
            initial_entities: 100,
            max_population: 500,
            entity_scale: 1.0,
            grid_cell_size: 50.0,
            spawn_radius_factor: 0.8,
            min_entity_radius: 2.0,
            max_entity_radius: 8.0,
            energy_decay_rate: 0.1,
            reproduction_threshold: 80.0,
            reproduction_cost: 40.0,
            mutation_rate: 0.1,
            mutation_strength: 0.2,
            interaction_radius: 20.0,
            boundary_elasticity: 0.8,
            drift_correction: true
        };
        
        this.simulation = new WebSimulation(800, JSON.stringify(config));
        this.updateStats();
    }

    step() {
        if (this.simulation) {
            this.simulation.update();
            this.updateStats();
        }
    }

    startRenderLoop() {
        if (!this.isRunning) return;
        
        const animate = (currentTime) => {
            if (!this.isRunning) return;
            
            // Calculate FPS
            this.frameCount++;
            if (currentTime - this.lastTime >= 1000) {
                this.fps = this.frameCount;
                this.frameCount = 0;
                this.lastTime = currentTime;
                this.updateStats();
            }
            
            // Update simulation at target FPS
            const targetInterval = 1000 / (this.targetFPS || 30);
            if (currentTime - this.lastUpdateTime >= targetInterval) {
                this.simulation.update();
                this.lastUpdateTime = currentTime;
            }
            
            // Render
            this.render();
            
            this.animationId = requestAnimationFrame(animate);
        };
        
        this.lastUpdateTime = performance.now();
        this.animationId = requestAnimationFrame(animate);
    }

    stopRenderLoop() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
    }

    render() {
        if (this.simulation && this.renderer) {
            const entities = this.simulation.get_entities();
            this.renderer.render(entities);
        }
    }

    updateStats() {
        if (this.simulation) {
            const stats = this.simulation.get_stats();
            if (stats) {
                document.getElementById('population').textContent = stats.population || 0;
                document.getElementById('step-count').textContent = stats.step || 0;
                document.getElementById('fps').textContent = this.fps;
            }
        }
    }

    showError(message) {
        const errorDiv = document.createElement('div');
        errorDiv.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #f44336;
            color: white;
            padding: 15px;
            border-radius: 5px;
            z-index: 1000;
        `;
        errorDiv.textContent = message;
        document.body.appendChild(errorDiv);
        
        setTimeout(() => {
            document.body.removeChild(errorDiv);
        }, 5000);
    }
}

// Start the application when the page loads
window.addEventListener('load', () => {
    new EvolutionApp();
});
```

## Build and Deployment

### Step 1: Build Configuration

#### package.json
```json
{
  "name": "evo-web",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "build": "wasm-pack build --target web --out-dir ../pkg",
    "dev": "python3 -m http.server 8000",
    "serve": "npx serve web"
  },
  "devDependencies": {
    "wasm-pack": "^0.12.0"
  }
}
```

### Step 2: Build Commands

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build the WASM package
wasm-pack build --target web --out-dir pkg

# Serve the web application
cd web && python3 -m http.server 8000
```

### Step 3: Cross-Origin Isolation Setup

For SharedArrayBuffer support (required for wasm-bindgen-rayon), you need to set up cross-origin isolation headers. Create a simple server:

#### web/server.py
```python
#!/usr/bin/env python3
import http.server
import socketserver

class CORSHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

if __name__ == "__main__":
    PORT = 8000
    with socketserver.TCPServer(("", PORT), CORSHTTPRequestHandler) as httpd:
        print(f"Serving at http://localhost:{PORT}")
        httpd.serve_forever()
```

## Performance Considerations

### 1. Memory Management
- Use `wasm-pack` with size optimizations
- Implement proper cleanup in JavaScript
- Monitor memory usage in browser dev tools

### 2. Rendering Optimization
- Use `requestAnimationFrame` for smooth rendering
- Implement frame rate limiting
- Consider using WebGL for better performance with large numbers of entities

### 3. Thread Pool Management
- Use `navigator.hardwareConcurrency` for optimal thread count
- Monitor thread pool performance
- Implement fallback for browsers without SharedArrayBuffer support

## Browser Compatibility

### Required Features
- WebAssembly support
- SharedArrayBuffer support
- Web Workers support
- Canvas 2D context

### Fallback Strategy
```javascript
// Feature detection
async function checkCompatibility() {
    const hasWasm = typeof WebAssembly === 'object';
    const hasSharedArrayBuffer = typeof SharedArrayBuffer !== 'undefined';
    const hasWorkers = typeof Worker !== 'undefined';
    
    if (!hasWasm) {
        throw new Error('WebAssembly not supported');
    }
    
    if (!hasSharedArrayBuffer) {
        console.warn('SharedArrayBuffer not supported - falling back to single-threaded mode');
        // Implement single-threaded fallback
    }
    
    return hasWasm && hasWorkers;
}
```

## Testing and Debugging

### 1. Development Tools
- Use browser dev tools for WASM debugging
- Monitor console for errors
- Use performance profiler for optimization

### 2. Testing Strategy
- Unit tests for Rust modules
- Integration tests for WASM bridge
- Browser compatibility testing

### 3. Debugging Tips
- Enable source maps in wasm-pack build
- Use `console.log` for JavaScript debugging
- Use `println!` macro for Rust debugging (shows in browser console)

## Deployment

### 1. Static Hosting
- Deploy to GitHub Pages, Netlify, or Vercel
- Ensure CORS headers are properly set
- Use HTTPS for SharedArrayBuffer support

### 2. CDN Considerations
- Serve WASM files with correct MIME types
- Implement proper caching strategies
- Consider using a CDN for better performance

This implementation provides a complete web-based UI for your evolution simulation while maintaining the parallel processing capabilities through wasm-bindgen-rayon. The modular architecture allows for easy extension and maintenance. 