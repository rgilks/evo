# Evolution Simulation

A beautiful and performant evolution simulation written in Rust with WebGPU-accelerated graphics and WebAssembly, running entirely in the browser.

![Evolution Simulation Screenshot](screenshot.png)

## Quick Start

### Prerequisites

1. **Rust Nightly**: `rustup toolchain install nightly-2024-08-02`
2. **WASM Target**: `rustup target add wasm32-unknown-unknown`
3. **Node.js & npm**: [Install Node.js](https://nodejs.org/)
4. **WebGPU-capable browser**: Chrome 113+, Firefox 121+, or Safari 17.4+

### Installation

```bash
# Install dependencies and set up environment
npm run setup
```

### Running

```bash
npm run dev
# Then open http://localhost:8788
```

## Deployment

Deploy to Cloudflare Pages:

```bash
npm run deploy
```

For detailed instructions, see [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md).

## Development

### Project Structure
```
evo/
├── src/
│   ├── components.rs     # ECS components
│   ├── genes/            # Genetic algorithms
│   ├── systems/          # Movement, Interaction, Reproduction
│   ├── simulation/       # Main simulation logic
│   └── web/              # WebGPU renderer
├── web/                  # Frontend assets (HTML, CSS, JS)
└── scripts/              # Build scripts
```

### Key Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Start local dev server |
| `npm run build` | Compile to WASM |
| `npm run deploy` | Build and deploy to Cloudflare Pages |
| `cargo test` | Run Rust tests |
| `cargo clippy` | Run linter |

## Simulation Details

For a deep dive into the simulation mechanics, see [docs/SIMULATION_SYSTEM.md](docs/SIMULATION_SYSTEM.md).

## License

MIT License.
