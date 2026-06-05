# Backlog

Forward-looking work, roughly ordered by what unblocks or de-risks the most. Present tense; this is intent, not history.

## P1 — Toolchain & dependency modernization (blocked on wasm threads)

Pinned to `nightly-2024-08-02` because atomics + `-Z build-std` are nightly-only (still true in 2026 — unavoidable, not debt). The pin is ~21 months stale, which forces the `indexmap = "=2.2.6"` workaround, and the dep tree has drifted.

**Blocker — newer nightlies break wasm threads.** On `nightly-2026-05-01` (rustc 1.97) the wasm build stops emitting a *shared* memory, so `wasm-bindgen-rayon`'s `initThreadPool` fails at load (`DataCloneError: #<Memory> could not be cloned`; `Atomics.waitAsync` on a non-shared `Int32Array`) and the app never initializes. The same `wasm-bindgen` (0.2.100) and `wasm-bindgen-rayon` (1.3.0) work on the old nightly and break on the new one, so it is the compiler's shared-memory emission that changed — not the JS glue. Resolve this first (likely an explicit `-C link-arg=--shared-memory -C link-arg=--max-memory=…`, or an added target-feature) and confirm `initThreadPool` succeeds in the browser before bumping anything else.

Once threads work on a recent nightly, the rest follows and has been scouted:

- Drop the `indexmap = "=2.2.6"` exact-pin (newer nightlies accept current indexmap; it's only a transitive of wgpu/naga).
- `hecs` 0.9 → 0.11: query iterators now yield `Q::Item` instead of `(Entity, Q::Item)`, and `Entity` implements `Query` — add `Entity` to the queries that need the id and flatten the bindings. Also `rand` 0.8 → 0.9.
- `wgpu` 24 → 29: mechanical churn in `web/webgpu.rs` — by-value `InstanceDescriptor` (no `Default`), `request_adapter` returns `Result`, `request_device` takes one arg, `bind_group_layouts: &[Option<&_>]` + `immediate_size`, `multiview` → `multiview_mask`, `get_current_texture` returns the `CurrentSurfaceTexture` enum, and `RenderPassColorAttachment.depth_slice`. wgpu 29 forces `wasm-bindgen` 0.2.122, which needs the newer nightly — so it is gated behind the threads fix above.

Keep the CI workflow's pinned nightly in sync with `rust-toolchain.toml`: the workflow installs rustfmt/clippy/wasm components for its own pinned value, so a mismatch fails the build.

## P2 — Performance: dense spatial grid (SoA)

The `world.get::<&T>()` storm is gone: the grid rebuild also builds a per-tick `NeighborCache` (`systems/mod.rs`) of each entity's hot fields, so movement, interaction, and nearest-N selection read one cached snapshot per neighbour instead of a scatter of component fetches. The grid itself is a `HashMap` of cell → entities, rebuilt serially.

What remains is a smaller, more invasive refinement: replace the cell `HashMap` with a **dense fixed grid** built by counting sort (lock-free, contiguous per-cell runs, no hashing), keyed by a dense slot id so neighbour data is fully contiguous (true SoA). The world is bounded, so a flat grid fits. This is the step from "good locality" to "optimal locality" — measure it against the headless bench harness before taking on the churn.

## P2 — Headless run mode + benchmarks

The crate is `cdylib`-only, so there's no native way to profile or benchmark. Add `"rlib"` to `crate-type`, an `examples/headless.rs` that runs N ticks from a fixed seed (now possible — the sim is deterministic), and `criterion` benches over the per-tick hot loop. Prerequisite for measuring any of the performance work above.

## P2 — GPU compute for scale (100K–1M entities)

Reaching 100K–1M is a **separate GPU-compute engine**, not an optimization of the current one: ping-pong storage buffers, a counting-sort spatial grid + prefix sum on the GPU, force/movement compute shaders, and indirect draw. CPU + rayon cannot reach that scale. This sim's per-entity logic (predation, energy transfer, births/deaths = structural mutation needing GPU compaction) makes the port heavier than a plain particle-life sim. Revisit only after the toolchain bump, and only if the scale is genuinely wanted.

## P2 — Build robustness

- **Replace the `sed` cache-busting.** `scripts/build-web.sh` injects a git-SHA `?v=` query via a chain of `sed` rewrites — brittle string-surgery on generated output, and largely redundant with `web/_headers`. As a first step, deduplicate it: run every rewrite on `pkg/` first, then `cp -r pkg web/` last so the web copy inherits the patched files (removes the duplicated `web/pkg` rewrite blocks). Longer term, fix the one load-bearing worker-import path via `wasm-bindgen-rayon`'s `no-bundler` feature, or move to content-hashed filenames / an import map.
- **One wasm-pack source.** CI installs wasm-pack via `curl` *and* `package.json` lists it as a devDependency; `npm run build` would prefer the npm copy. Pick one (drop the devDependency, or switch the script to `npx wasm-pack` and drop the curl step) so the build uses a single known version.

## P2 — Deploy URL

The public URL is the custom domain `https://evo.tre.systems` (in the README). The underlying Cloudflare Pages project is `evo-dgc` (Cloudflare suffixed `evo` because the name was taken) and `evo-dgc.pages.dev` also serves the app. Optional cleanup: rename the Pages project to reclaim `evo.pages.dev`, or leave it — the custom domain is the canonical entry point.

## P3 — Rendering polish

- **Skip redundant re-uploads.** The render loop repacks/re-uploads the instance buffer every `requestAnimationFrame`, even on frames where the sim didn't tick (only the interpolation uniform changed). Track the last rendered step (`get_step` exists) and gate the repack+upload on a sim tick — free FPS on high-refresh displays.
- **WebGPU-unavailable UX.** Replace the 5-second error toast with a persistent "WebGPU required" message, and request `downlevel` device limits so low-end adapters degrade rather than fail. (The renderer is WebGPU-only — the `wgpu` `webgl` feature has been dropped — so a real WebGL2 fallback would be a deliberate re-addition, only worth it for broad reach.)

## P3 — Code cleanups (from the architecture review)

- **Drop `Genes` from `EntityUpdate`.** It's cloned (~96 B) for every entity every tick but only reproducers use it (<1%); fetch the parent's genes in the apply phase instead. `NeighborSnapshot` clones `Genes` per entity too.
- **Single config owner.** `WebSimulation` duplicates `SimulationConfig` alongside `Simulation`'s copy and clones the whole struct on every `update_param`; give `Simulation` a `set_param` and drop the duplicate.
- **Predation self-size bug.** Target-finding (`movement/mod.rs`) passes a dummy `Size { radius: 1.0 }` into `can_eat`, while `InteractionSystem` passes the real size — so "what I chase" and "what I can eat" use different size logic. Thread the real predator size through. Behaviour-affecting: smoke-test the live sim after.
- **Reproduction floor.** In `check_reproduction`, `min_reproduction_chance` is `.max`'d onto the *crowding factor*, not the final chance — decide which is intended and make it explicit (the current reading is ambiguous). Behaviour-affecting.
- **Rename `center_pressure_strength` → `edge_repulsion_strength`** (and `SimParam::CenterPressure` → `EdgeRepulsion`, the `#center-pressure` slider, the JS `DEFAULT_CONFIG` key). The field now scales edge repulsion, not a centre pull; the name is left over from the old behaviour. Cross-cuts the Rust serde field and the JS config, so do it in one change and smoke-test.
- **Split the two oversized files.** `simulation/mod.rs` (~460 lines) and `systems/movement/mod.rs` (~390) exceed the 200-line guideline. Extract `simulation/rng.rs` (`mix_seed`, the salts, `generate_particle_matrix`) and the render-extract getters; in movement, extract the neighbour-accumulation loop and the flocking/solitary force helpers. Pure refactors — preserve arithmetic order so seed-determinism is unchanged.
- **Co-locate the remaining tests.** `energy.rs` and `reproduction.rs` use inline `#[cfg(test)] mod tests { … }`; every other module uses a co-located `tests.rs`. Convert them to match.
- **Trim unused stats.** `SimulationStats::from_world` computes `entity_counts`, per-colour classification, several averages, and `world_center_drift` on every `get_stats()`, but the UI consumes only `total_entities`. Either surface them in the UI or trim the struct (crosses the WASM boundary, so update `lib.rs`/JS/tests together).
- **Faster hot-loop RNG.** The per-entity-per-tick RNG is `StdRng` (ChaCha, cryptographic-grade). A small fast PRNG (the `splitmix64` finaliser already in `mix_seed`, or wyrand/pcg) would cut hot-loop cost and shrink the `rand`/`getrandom` footprint, keeping determinism. Re-seeds every run, so re-curate the default seed after.

## P3 — Frontend

- **Data-driven slider registry.** `app.js` `setupEventListeners` wires ~10 sliders by hand (~140 lines). Replace with a `SLIDERS` table (`{id, valueId, param, decimals}`) iterated to attach listeners, and split the button/keyboard/mouse wiring into small focused methods. Adding a slider becomes a one-line change.
- **De-duplicate sim construction.** `init()` and `reset()` repeat the canvas-size → worldSize → seed → `WebSimulation` construction; extract one `createSimulation()` and hoist `const CONFIG_JSON = JSON.stringify(DEFAULT_CONFIG)` to module scope.
- **Remove the no-op mobile block.** The `@media (max-width: 768px)` rules in `web/css/style.css` don't do anything useful; delete them (real mobile support is a separate item).
- **Consider TypeScript + prettier** for `web/js/app.js` (bundle via a build-step migration, not standalone).

## P3 — Testing & CI

- **Convert the diagnostic drift/bias tests to assertions.** Several "tests" (`test_simulation_clustering`, `test_drift_direction_analysis`, the interaction/movement drift + bias harnesses) compute drift/centroid numbers and only `println!`; now that runs are deterministic, assert "drift < ε over N ticks" (and delete the pure no-op `test_simulation_entity_processing`). Extract shared `centroid`/`quadrant_counts` test helpers, and replace `thread_rng()` in the deterministic analysis tests with a seeded RNG so the asserted statistics are reproducible.
- **Property tests** (`proptest`) for gene mutation bounds, energy clamping, and HSV↔RGB round-trip.
- **A single Playwright smoke test** in CI (page loads, canvas present, `#seed-display` set, no console errors).
- **Cache cargo in the CI deploy job** (the `build-std` WASM build is uncached there today).

## Roadmap — simulation depth

Longer-horizon mechanics, in rough order of appeal: environmental terrain and localized resource patches; aging and disease/parasites; mating rituals and territorial behaviour; and multi-species symbiosis / food webs. Each extends the systems in `src/systems/` and the gene model in `src/genes/`.

## Definition of Done

A change is done when:

- It passes the verification gate in [AGENTS.md](AGENTS.md#verification) (the pre-commit hook enforces it).
- Docs describing affected behaviour are updated to match.
- For user-visible changes: pushed, CI green, and smoke-tested on the live site.
