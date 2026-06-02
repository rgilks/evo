# Evolution Simulation — Architecture

> Scope: how the code is organized and how one rendered frame is produced. For the simulation *mechanics* — genes, movement styles, predation, statistics — see [SIMULATION_SYSTEM.md](SIMULATION_SYSTEM.md).

## Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Language | Rust (edition 2021) | ~5,500 lines across `src/` |
| ECS | `hecs` 0.9 | Lightweight archetypal ECS; systems are hand-orchestrated (no scheduler) |
| Parallelism | `rayon` | Parallel per-entity processing over ECS queries |
| Concurrency | `dashmap` | Backs the spatial grid for concurrent neighbour inserts |
| Rendering | `wgpu` 24 (WebGPU) | Instanced quads with a glow shader |
| In-browser threads | `wasm-bindgen-rayon` + `SharedArrayBuffer` | Requires cross-origin-isolation headers |
| Toolchain | nightly-2024-08-02 | Needed for `-Z build-std` (atomics-enabled `std` for WASM threads) |
| Host | Cloudflare Pages | Static `web/` directory served with `_headers` |

The toolchain is pinned because atomics + `build-std` are nightly-only. This pin is the project's main maintenance constraint — see [BACKLOG.md](../BACKLOG.md).

## Repo Layout

```
src/
├── lib.rs              # WASM entry: WebSimulation (#[wasm_bindgen]) wraps the sim for JS
├── config/             # SimulationConfig — all tunable parameters (serde, hot-swappable)
├── components.rs       # ECS components: Position, Velocity, Energy, Size, Color, MovementStyle
├── genes/              # Genetic model: generation, mutation, similarity, particle-life weights
├── simulation/         # Per-tick orchestrator (the read → compute → apply loop)
├── spatial_grid.rs     # DashMap-backed spatial hash for neighbour queries
├── systems/            # movement/, interaction/ (predation), energy, reproduction
├── stats/              # Population statistics, serialized to JS
├── web/                # WebGPU renderer (wasm32-only)
└── shader.wgsl         # Vertex (interpolation + camera) + fragment (glow) shaders
web/                    # Static frontend (index.html, js/app.js, _headers) — the deploy root
scripts/                # build-web.sh (wasm-pack + cache-busting), setup.sh
```

## Simulation Tick

`Simulation::update()` ([src/simulation/mod.rs](../src/simulation/mod.rs)) runs each step in three phases:

1. **Snapshot** — store previous positions (used for GPU-side interpolation between sim steps).
2. **Build spatial grid** — rebuild the `SpatialGrid` from current positions (concurrent inserts via DashMap).
3. **Compute then apply** — process every entity *in parallel* into a list of `EntityUpdate`s (new position/velocity/energy, eaten flags, offspring), then apply that list *serially*: despawn eaten and starved/old entities, spawn offspring.

This split exists because hecs structural mutation (spawn/despawn) is single-threaded. Reads and per-entity math fan out across cores; only the apply step touches world structure.

## Parallelism Model

Per-entity work (movement forces, predation targets, metabolism) is read-only against the world and runs under `rayon`. Each worker produces an `EntityUpdate` rather than mutating shared state; the serial apply step is the only writer. The cardinal rule: **never spawn/despawn or mutate the world from inside a parallel query.**

## Spatial Grid

`src/spatial_grid.rs` partitions the world into a grid of cells and stores entity handles per cell in a `DashMap`. A neighbour query returns candidates from the cell containing an entity plus its surrounding cells, turning the O(N²) all-pairs scan into a near-O(N) local scan. Concurrent inserts during the build phase are why the map is a `DashMap`.

## Rendering Pipeline

The CPU never builds vertex geometry per entity. Instead:

1. `WebSimulation::update_entity_buffer()` ([src/lib.rs](../src/lib.rs)) flattens every entity into a packed `f32` buffer — **8 floats each**: `prev_x, prev_y, x, y, radius, r, g, b` — and returns a raw pointer into WASM linear memory (zero-copy).
2. `entity_count()` returns `buffer.len() / 8`.
3. The renderer ([src/web/webgpu.rs](../src/web/webgpu.rs)) reads that slice, builds one 32-byte `Instance` per entity, and issues a single instanced draw of a unit quad.
4. `shader.wgsl` does the heavy lifting on the GPU: it interpolates between `prev` and current position for smooth motion between sim steps, applies the world→screen + camera (zoom/pan) transform, and draws a multi-layer glow in the fragment stage.

## Threading & WASM

In-browser multithreading needs `SharedArrayBuffer`, which browsers only expose in cross-origin-isolated contexts. That requires two response headers (set in `web/_headers`):

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

The thread pool is initialised from JS via `wasm-bindgen-rayon`. The atomics-enabled `std` is built from source (`-Z build-std`), which is why the toolchain is pinned to a specific nightly with `rust-src`.

## Build & Deploy

- `npm run build` → `scripts/build-web.sh`: `wasm-pack build --target web` with `CARGO_UNSTABLE_BUILD_STD=std,panic_abort`, then injects a git-SHA `?v=` cache-busting query into the generated JS/HTML and copies `pkg/` → `web/pkg/`.
- `npm run dev` → `wrangler pages dev web` (serves `web/`, honours `_headers`, port 8788).
- `npm run deploy` → build, then `wrangler pages deploy web --project-name evo`.
- CI (`.github/workflows/ci-cd.yml`) runs the verification gate on every push/PR and deploys to Cloudflare Pages on push to `main`.

`pkg/` and `web/pkg/` are generated and git-ignored. There is no `wrangler.toml`; the Pages project name lives only in the deploy command.

## What This Architecture Deliberately Does Not Include

- **No server or persistence.** Everything runs client-side; there is no backend, database, or save/load.
- **No GPU compute for the simulation.** Neighbour search and physics are CPU + rayon; only rendering uses the GPU. (GPU-side spatial processing is a backlog idea, not current behaviour.)
- **No practical WebGL fallback.** The `wgpu` `webgl` feature is enabled, but the renderer targets WebGPU; non-WebGPU browsers are unsupported.
- **A practical instance ceiling.** The renderer pre-allocates for ~20,000 instances; scaling to 100K–1M is exploratory (see BACKLOG).
- **No size-optimised WASM.** `wasm-opt` is disabled, favouring build simplicity over bundle size.
