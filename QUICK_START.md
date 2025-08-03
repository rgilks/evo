# Quick Start Guide

## 🚀 First Time Setup

```bash
# Install dependencies and set up environment
./setup.sh
```

## 🎮 Run the Simulation

### Desktop Application (Recommended)
```bash
# Run with beautiful GPU-accelerated graphics
./run.sh desktop
```

### Web Application
```bash
# Run in your web browser
./run.sh web
```

### Headless Mode
```bash
# Run without graphics (faster for testing)
./run.sh headless --steps 1000
```

## 🛠️ Development Commands

```bash
# Run tests
./run.sh test

# Build only (no run)
./run.sh build

# Clean build artifacts
./run.sh clean

# Show all commands
./run.sh help
```

## 📁 Project Structure

```
evo/
├── src/                    # Rust source code
├── web/                    # Web assets (HTML, CSS, JS)
├── pkg/                    # Generated WebAssembly files
├── run.sh                  # Main run script
├── setup.sh                # First-time setup
├── build-desktop.sh        # Desktop build script
├── build-web.sh           # Web build script
└── README.md              # Detailed documentation
```

## 🎯 Common Use Cases

### Just Want to See It Run?
```bash
./setup.sh
./run.sh desktop
```

### Want to Run in Browser?
```bash
./setup.sh
./run.sh web
```

### Want to Test Performance?
```bash
./run.sh headless --steps 5000
```

### Want to Develop?
```bash
./run.sh test
./run.sh build
```

## 🔧 Troubleshooting

### Build Errors?
```bash
# Clean and rebuild
./run.sh clean
./run.sh build
```

### Web Not Working?
```bash
# Use the Python server (recommended)
./run.sh web
```

### Tests Failing?
```bash
# Run setup again
./setup.sh
./run.sh test
```

## 📖 More Information

- **Full Documentation**: See `README.md`
- **Configuration**: See `config.json` and `example_config.json`
- **Web Assets**: See `web/` directory
- **Source Code**: See `src/` directory

---

**Happy evolving! 🧬** 