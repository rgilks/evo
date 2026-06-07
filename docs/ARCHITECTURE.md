# Evolution Simulation — Architecture

> Scope: how the code is organized and how one rendered frame is produced. For the simulation *mechanics* — genes, movement styles, predation, statistics — see [SIMULATION_SYSTEM.md](SIMULATION_SYSTEM.md).

![System overview](diagrams/system-overview.png)

## Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Language | Rust (edition 2021) | Archetypal-ECS core; a few thousand lines across `src/` |
| ECS | `hecs` 0.11 | Lightweight archetypal ECS; systems are hand-orchestrated (no scheduler) |
| Parallelism | `rayon` | Parallel per-entity processing over ECS queries |
| Rendering | `wgpu` 29 (WebGPU) | Instanced quads → HDR scene + motion trails → bloom + ambient composite |
| In-browser threads | `wasm-bindgen-rayon` + `SharedArrayBuffer` | Requires cross-origin-isolation headers |
| Toolchain | nightly-2026-05-01 | Needed for `-Z build-std` (atomics-enabled `std` for WASM threads) |
| Host | Cloudflare Pages | Static `web/` directory served with `_headers` |

The toolchain is pinned because atomics + `build-std` are nightly-only. This pin is the project's main maintenance constraint — see [BACKLOG.md](../BACKLOG.md).

## Repo Layout

```
src/
├── lib.rs              # WASM entry: WebSimulation (#[wasm_bindgen]) wraps the sim for JS
├── config/             # SimulationConfig — all tunable parameters (serde, hot-swappable)
├── components.rs       # ECS components: Position, Velocity, Energy, Size, Color, MovementStyle
├── genes/              # Genetic model: generation, mutation, similarity, particle-life weights
├── simulation/         # Per-tick orchestrator (the read → compute → apply loop) + the drifting food field (food.rs)
├── spatial_grid.rs     # Spatial hash (cells → entities) for neighbour queries
├── systems/            # movement/, interaction/ (predation), energy, reproduction
├── stats/              # Population statistics, serialized to JS
├── web/                # WebGPU renderer + bloom post-process (wasm32-only)
├── shader.wgsl         # Particle shader: vertex (interpolation + camera) + fragment (additive glow)
└── post.wgsl           # Bloom: bright-pass → Gaussian blur → tonemapped composite
web/                    # Static frontend (index.html, js/app.js, _headers) — the deploy root
scripts/                # build-web.sh (wasm-pack + cache-busting), setup.sh
```

## Patterns

The simulation is built from a small set of recurring patterns. Naming them once is the fastest way to read any individual file — every system, component, and data path is an instance of one of these. The sections after this one show the load-bearing ones in detail.

**Entity-Component-System (ECS).** State lives in a `hecs::World` as entities composed of plain-data components (`Position`, `Velocity`, `Energy`, `Size`, `Color`, `Genes`, `MovementStyle` — in `components.rs` and `genes/`). Components carry no behaviour; behaviour lives in systems. There is no scheduler — the orchestrator calls systems by hand.

**Stateless system-as-unit-struct.** Each system is a zero-sized unit struct (`MovementSystem`, `InteractionSystem`, `EnergySystem`, `ReproductionSystem` in `systems/`), held as a field on `Simulation` and instantiated once. Systems hold no state; they are namespaces for behaviour that takes the world and components as arguments.

**Read–Compute–Apply (deferred mutation).** The cardinal tick pattern. Each step: (1) **read** the world through immutable queries, (2) **compute** a `Vec<EntityUpdate>` in parallel under rayon — pure functions of the read state, no world mutation, and (3) **apply** the results serially in the orchestrator — writing each survivor's components in place, despawning the dead and any eaten prey, and spawning offspring. The invariant: *no system mutates `World` structure from inside a parallel query* — the parallel compute reads only immutable state (the world plus a per-tick neighbour cache), and every structural change happens in the serial apply.

**Command object (`EntityUpdate`).** The carrier between compute and apply (`simulation/mod.rs`). Every per-entity result — new position/velocity/energy/size, whether it reproduced, and any prey it ate — is packaged into one `EntityUpdate`, and the apply phase is its sole interpreter. This is what lets the compute phase stay pure and parallel.

**Spatial hashing for neighbour queries.** `SpatialGrid` (`spatial_grid.rs`) buckets entities into fixed-size cells (a `HashMap` of cell → entities, cleared and rebuilt each tick). A neighbour lookup scans only the cells within the query radius — never the whole population — turning O(N²) all-pairs into near-O(N). The grid is queried once per entity per tick and the result is shared by movement and interaction. From the candidate cells each entity keeps its **nearest 20** neighbours (ranked by distance, ties broken by id) — deterministic, free of directional bias, and capped for a bounded per-entity cost.

**Per-tick neighbour cache.** The grid stores only entity handles, so reading a neighbour's data could mean a scatter of `world.get::<&T>()` calls per neighbour. Instead, the same pass that rebuilds the grid also captures each entity's hot fields — position, genes, energy, size, velocity — into a `NeighborCache` (`HashMap<Entity, NeighborSnapshot>`, `systems/mod.rs`), built once and then read immutably by every system. A neighbour read in the hot loop becomes one cache lookup rather than several component fetches, and because the cache is a fixed snapshot of the start-of-tick world it is safe to share across the parallel compute. Determinism is unaffected — the cached values are exactly what the world holds at rebuild.

**System pipeline over a shared context.** All four systems implement one trait — `System::run(&self, ctx: &mut EntityContext)` (`systems/`) — and the orchestrator runs them in a fixed order (movement → interaction → energy → reproduction) over a single `EntityContext` carrying the read-only inputs and the mutable `new_*` working state. The per-system `*Params` structs (`MovementUpdateParams`, `InteractionParams`) and the orchestrator's `ProcessEntityParams` are the parameter-object form used to pass many borrows without long argument lists.

**Centralized archetype.** The creature component bundle is built in one place, `systems::creature_bundle`, used by both the initial spawn and reproduction so the archetype cannot drift between the two.

**Grouped configuration.** `SimulationConfig` is a struct of domain sub-structs (`population`, `physics`, `energy`, `reproduction` — `config/mod.rs`), serde-(de)serializable, with a `Default`. It is threaded read-only as `&config` to every system, and individual fields are tunable live through `WebSimulation::update_param` via a typed `SimParam` enum. `Genes` mirrors this shape with one sub-struct per trait domain.

**Deterministic, seeded simulation.** All randomness derives from a single `u64` seed through a counter-based per-entity RNG (`mix_seed(seed, entity_id, tick)` → a fast SplitMix64 `FastRng`). Each entity's stream depends only on the seed, its id, and the tick — not on thread scheduling — and neighbour selection (nearest-N) and structural mutation (sorted by id) are order-independent, so the same seed reproduces a run bit-for-bit. The frontend runs a single fixed curated seed, so every visitor sees the same world. (The WASM API also exposes a wall-clock `new()` constructor and `get_seed`, currently unused by the UI.)

**Phenotype from genotype (derived data).** Visible and effective traits are computed from genes, never stored independently: `Color` via `Genes::get_color()` (HSV→RGB), edibility via `can_eat()`, prey choice via `get_predation_preference()`, plus flat accessor shims (`speed()`, `sense_radius()`, …) over the nested gene structs. Genes are the single source of truth; phenotype is a pure function of them.

**FFI boundary facade.** `WebSimulation` and `WebGpuRenderer` (`lib.rs`, `web/webgpu.rs`) are the only `#[wasm_bindgen]` types. They own all JS-facing marshalling and delegate to an FFI-free core (`Simulation` returns plain Rust tuples). The boundary is the one place `Result<_, JsValue>` and raw pointers appear.

**Zero-copy render buffer.** Instead of per-creature draw calls, entities are packed into one flat `[f32]` (12 floats each) exposed to JS by raw pointer; the renderer reads it as instance data for a single instanced draw, and the shader does interpolation and the camera transform on the GPU (see Rendering Pipeline).

**Snapshot for interpolation.** Each tick snapshots positions into `previous_positions` (keyed by entity) before moving entities, so the renderer can interpolate between the last two sim states — decoupling visual smoothness from tick rate. Because entities are updated in place, their ids are stable across ticks, so the snapshot matches the live entities and the interpolation is active.

**Graceful-skip error handling.** Component reads in the hot path use `if let Ok(...)` and silently skip entities that vanished mid-tick; despawns discard their `Result` (`let _ =`); the FFI boundary uses `Result`/`map_err`; other fallbacks use `unwrap_or(default)`. There are no `panic!`/`unwrap`/`expect` in non-test code.

Known consistency gaps and patterns under consideration are tracked in [BACKLOG.md](../BACKLOG.md).

## Simulation Tick

![Simulation tick: read, compute, apply](diagrams/simulation-tick.png)

`Simulation::update()` ([src/simulation/mod.rs](../src/simulation/mod.rs)) runs each step in three phases:

1. **Snapshot** — store previous positions (used for GPU-side interpolation between sim steps).
2. **Advance the food field** — drift and regrow the deterministic [food patches](SIMULATION_SYSTEM.md#food-field) (`src/simulation/food.rs`) serially, so the field is fixed for the whole tick and the parallel reads see a stable snapshot.
3. **Build spatial grid + neighbour cache** — one serial pass over the world rebuilds the `SpatialGrid` (cell → entities) and the `NeighborCache` (each entity's hot fields), so the compute phase reads contiguous cached data instead of scattered `world.get`s.
4. **Compute then apply** — process every entity *in parallel* into a list of `EntityUpdate`s, then apply that list *serially*: write each survivor's `Position`/`Velocity`/`Energy`/`Size` in place (via `query_mut`), deplete the food patches each fed creature grazed, despawn the starved/dead and any eaten prey, and spawn offspring.

This split exists because hecs mutation (in-place writes and spawn/despawn) is single-threaded. The compute phase is read-only against the world and fans out across cores under `rayon` (movement forces, predation targets, metabolism); each worker produces an `EntityUpdate` rather than touching shared state, and the serial apply step is the only writer. The cardinal rule: **never spawn/despawn or mutate the world from inside a parallel query.** (Neighbour queries and the spatial grid are covered under Patterns above.)

## Rendering Pipeline

The CPU never builds vertex geometry per entity. Instead:

1. `WebSimulation::update_entity_buffer()` ([src/lib.rs](../src/lib.rs)) flattens every entity into a packed `f32` buffer — **12 floats each**: `prev_x, prev_y, x, y, radius, r, g, b, health, style_id, speed_norm, sense_norm` — and returns a raw pointer into WASM linear memory (zero-copy). `health` (energy fraction), `style_id` (movement type), and the normalised `speed`/`sense` genes let the shader make the look reflect the creature's state *and* genotype.
2. `entity_count()` returns `buffer.len() / 12`.
3. The renderer ([src/web/webgpu.rs](../src/web/webgpu.rs)) reinterprets that slice directly as `&[Instance]` (`bytemuck::cast_slice` — the layouts are identical, so there's no per-frame copy), uploads it to a growable instance buffer, and issues a single instanced draw of a unit quad into an **HDR (`rgba16float`) scene target**. Creatures use **alpha blending** so dense colonies keep their colour instead of blowing out to white; food and effects (below) draw **additively** so they bloom.
4. Before the creatures, a second instanced draw renders the **food patches** (`update_food_buffer()` → 4 floats each: `x, y, radius, intensity`) into the HDR scene (`food_vs`/`food_fs` in `shader.wgsl`) as luminous nutrient fields — lush patches glow warm green-gold with a blooming core, grazed-out ones cool to a faint teal — so the food structure reads as the stage the swarm gathers on.
5. `shader.wgsl` does the heavy lifting on the GPU: it interpolates between `prev` and current position for smooth motion between sim steps, applies the world→screen + camera (zoom/pan) transform, and shapes each creature in the fragment stage by its state *and genes* — **size from real radius**, **luminance from `health`** (thriving cells bloom, the starving dim and fade, so births fade in and deaths fade out for free), plus an evolved **body plan**: each movement style gets its own silhouette (round grazers, sharp predator darts with a hot nucleus glint, spiky solitary stars, streamlined flockers), the **speed gene** narrows the body into a dart, and the **sense gene** widens its glowing aura — so speciation reads as visibly different creatures.
6. After the creatures, an **effects** pass (`update_effect_buffer()` → 6 floats each, `effect_vs`/`effect_fs`) draws transient expanding rings additively: hot-gold predation flashes, green bloom/seed bursts, and a red cull shockwave. The ring animation is interpolated between sim ticks for smoothness.
7. A bloom post-process ([src/web/postprocess.rs](../src/web/postprocess.rs), `post.wgsl`) turns the HDR scene into the final frame: a **bright-pass** isolates the glowing regions, a **separable Gaussian blur** at quarter resolution widens them, and a **tonemapped composite** adds the bloom back over an ambient nebula + vignette and maps it into the swapchain. A per-frame **trail fade** decays the scene before the creatures redraw, leaving glowing comet tails. The **Glow / Trails / Brightness / Size** sliders feed these stages live via `WebGpuRenderer::set_visual_params` (a group-3 post-params uniform plus the creature-size uniform). Each post pass draws one fullscreen triangle.

## Audio

Optional and off by default (browsers require a user gesture to start audio). When the **Sound** toggle is enabled, `web/js/app.js`'s `AudioEngine` builds a **fully-synthesised** Web Audio graph — no samples or external files: six detuned oscillator voices (one per hue sector) feeding a **procedurally-generated convolver reverb** and a feedback delay, into a brightness lowpass + compressor. Roughly seven times a second it reads `WebSimulation::audio_features()` — `[population, avg_health, hue-bin shares]`, one cheap pass over the world — and chases the parameters toward it with `setTargetAtTime`. So the chord is the on-screen palette, the filter brightness tracks health, and the drone's body tracks population: the soundscape *is* the ecosystem. Synthesis lives in JS (fast to tune by ear); the sim only exposes the feature vector.

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
- **No WebGL fallback.** The renderer targets WebGPU only (the `wgpu` `webgl` feature is not enabled); non-WebGPU browsers are unsupported.
- **A practical scaling ceiling.** The renderer's instance buffer starts at ~20,000 and grows on demand, but the CPU + rayon *simulation* tops out well below 100K–1M; that scale is exploratory and would need GPU compute (see BACKLOG).
