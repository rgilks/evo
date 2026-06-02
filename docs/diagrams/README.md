# Diagrams

Graphviz / DOT sources plus rendered PNGs. The `.dot` files are the source of truth; the PNGs are committed for in-browser viewing on GitHub. Small, throwaway diagrams use Mermaid inline in Markdown instead (e.g. the entity lifecycle in [SIMULATION_SYSTEM.md](../SIMULATION_SYSTEM.md)).

## Files

| Diagram                                  | Source                 | Rendered               |
| ---------------------------------------- | ---------------------- | ---------------------- |
| System overview                          | `system-overview.dot`  | `system-overview.png`  |
| Simulation tick (read → compute → apply) | `simulation-tick.dot`  | `simulation-tick.png`  |

## Reading Order

1. **System overview** for the whole browser / WASM core / GPU / host shape and the data flow between them.
2. **Simulation tick** for what one `Simulation::update()` does: the read → compute → apply loop, the parallel/serial split, and how entities are born, updated, and removed.

## Conventions

Color coding by domain:

- Blue — the browser / client surface (render loop, UI controls).
- Green — the Rust WASM simulation core (orchestrator, systems, ECS world).
- Teal — host and the serial world-mutation (apply) layer (Cloudflare Pages; `apply_entity_updates`).
- Amber — parallel or time-driven work (rayon thread pool, grid rebuild, snapshot).
- Purple — the GPU rendering boundary (WebGPU renderer + shader).
- Gray — neutral lifecycle removals (despawns).
- Diamonds — decisions.
- Bold green outline — terminal success / output state.

Fonts: Avenir. Rendered at 220 DPI.

## Render

```
npm run diagrams          # render all .dot files to PNG next to the source
npm run check:diagrams    # verify each .dot renders cleanly and the PNG exists
```

Both scripts assume Graphviz is on PATH (`brew install graphviz`). CI installs Graphviz before running `check:diagrams` (see `.github/workflows/diagrams.yml`). On a machine without `dot`, `check:diagrams` skips with a clear message; refresh the PNGs with `npm run diagrams` before committing diagram changes.

To render one manually:

```
dot -Tpng:cairo docs/diagrams/<name>.dot -Gdpi=220 -o docs/diagrams/<name>.png
```
