# Project Structure Documentation

## Overview

The Evolution Simulation project is organized with a clean, modular structure that separates concerns and makes the codebase easy to navigate and maintain.

## Directory Structure

```
evo/
├── src/                    # Rust source code
├── web/                    # Web application
├── scripts/                # Build and utility scripts
├── pkg/                    # Generated WebAssembly files
├── target/                 # Rust build artifacts
├── config files            # Configuration and metadata
└── documentation           # Project documentation
```

## Detailed Breakdown

### 📁 `src/` - Rust Source Code

The core simulation logic written in Rust:

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library entry point (for WASM)
├── components.rs           # ECS components (Position, Energy, Size, etc.)
├── genes.rs               # Genetic system with trait groups
├── systems.rs             # Simulation systems (Movement, Interaction, etc.)
├── spatial_grid.rs        # Spatial optimization for neighbor finding
├── stats.rs               # Analytics and statistics collection
├── simulation.rs          # Main simulation orchestration
├── config.rs              # Configuration management
├── ui.rs                  # GPU-accelerated rendering (desktop)
└── web/                   # Web-specific modules
    ├── mod.rs             # Web module exports
    ├── renderer.rs        # Canvas rendering utilities
    ├── controls.rs        # UI control handlers
    └── wasm_bridge.rs     # WASM-JS bridge utilities
```

**Key Files:**
- **`main.rs`**: CLI argument parsing and application startup
- **`lib.rs`**: WASM library exports and JavaScript bindings
- **`components.rs`**: Entity Component System components
- **`genes.rs`**: Genetic traits that define entity behavior
- **`systems.rs`**: Simulation logic systems
- **`simulation.rs`**: Main simulation coordination
- **`ui.rs`**: WGPU-based desktop rendering

### 🌐 `web/` - Web Application

The web interface organized for clarity and maintainability:

```
web/
├── index.html             # Main HTML page with canvas and UI
├── css/
│   └── style.css          # Modern, responsive styling
├── js/
│   └── app.js             # Main JavaScript application
├── assets/                # Static assets (images, icons, etc.)
└── server.py              # Development server with CORS headers
```

**Key Files:**
- **`index.html`**: Clean, semantic HTML structure
- **`css/style.css`**: Modern CSS with Grid and Flexbox
- **`js/app.js`**: JavaScript application with WASM integration
- **`server.py`**: Python server with proper CORS headers

### 🔧 `scripts/` - Build and Utility Scripts

Comprehensive build and management scripts:

```
scripts/
├── run.sh                 # Main run script with all commands
├── setup.sh               # First-time environment setup
├── build-desktop.sh       # Desktop build with platform detection
├── build-web.sh          # Web build and serve
├── build-and-fix.sh      # WASM build with worker fixes
└── fix-worker-imports.sh # WASM worker import fixes
```

**Script Functions:**
- **`run.sh`**: Unified interface for all common tasks
- **`setup.sh`**: Automated environment setup
- **`build-desktop.sh`**: Platform-aware desktop builds
- **`build-web.sh`**: Complete web build workflow
- **`build-and-fix.sh`**: WASM build with fixes
- **`fix-worker-imports.sh`**: WASM worker import utilities

### 📦 `pkg/` - Generated WebAssembly Files

Output directory for WASM builds:

```
pkg/
├── evo_bg.wasm            # Compiled WebAssembly binary
├── evo.js                 # JavaScript bindings
├── snippets/              # Generated code snippets
└── package.json           # WASM package metadata
```

### ⚙️ Configuration Files

Project configuration and metadata:

```
├── Cargo.toml             # Rust dependencies and build config
├── package.json           # Node.js dependencies (web)
├── rust-toolchain.toml    # Rust toolchain specification
├── config.json            # Default simulation configuration
├── example_config.json    # Example configuration file
├── .cargo/config.toml     # Cargo build configuration
└── .gitignore             # Git ignore patterns
```

### 📚 Documentation

Project documentation:

```
├── README.md              # Main project documentation
├── QUICK_START.md         # Quick start guide
├── PROJECT_STRUCTURE.md   # This file
└── .cursor/rules/         # Development rules and guidelines
```

## Design Principles

### 1. **Separation of Concerns**
- Rust logic separated from web presentation
- Build scripts separated from source code
- Configuration separated from implementation

### 2. **Modular Organization**
- Related files grouped together
- Clear directory structure
- Logical file naming

### 3. **Platform Independence**
- Desktop and web versions share core logic
- Platform-specific code isolated
- Cross-platform build scripts

### 4. **Developer Experience**
- Simple commands for common tasks
- Comprehensive documentation
- Clear project structure

## File Naming Conventions

### Rust Files
- **snake_case.rs**: Source files
- **mod.rs**: Module definitions
- **lib.rs**: Library entry points
- **main.rs**: Binary entry points

### Web Files
- **kebab-case.html**: HTML files
- **camelCase.js**: JavaScript files
- **kebab-case.css**: Stylesheet files

### Scripts
- **kebab-case.sh**: Shell scripts
- **descriptive-names**: Clear purpose indication

### Configuration
- **kebab-case.json**: JSON configuration
- **kebab-case.toml**: TOML configuration

## Build Artifacts

### Generated Directories
- **`target/`**: Rust compilation output
- **`pkg/`**: WebAssembly build output
- **`node_modules/`**: Node.js dependencies (if used)

### Temporary Files
- **`*.rs.bk`**: Rust backup files
- **`*.log`**: Log files
- **`*.tmp`**: Temporary files

## Development Workflow

### 1. **Setup**
```bash
./setup.sh
```

### 2. **Development**
```bash
./run.sh test      # Run tests
./run.sh build     # Build application
./run.sh desktop   # Run desktop version
./run.sh web       # Run web version
```

### 3. **Deployment**
```bash
./scripts/build-web.sh    # Build for web deployment
./scripts/build-desktop.sh --no-run  # Build desktop binary
```

## Maintenance

### Adding New Files
1. Place in appropriate directory
2. Follow naming conventions
3. Update documentation if needed
4. Add to version control

### Moving Files
1. Update all references
2. Update build scripts
3. Update documentation
4. Test thoroughly

### Removing Files
1. Remove from version control
2. Update documentation
3. Clean build artifacts
4. Test build process

---

This structure provides a clean, maintainable, and scalable foundation for the Evolution Simulation project. 