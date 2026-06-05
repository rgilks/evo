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

- **WebGPU-unavailable UX.** Replace the 5-second error toast with a persistent "WebGPU required" message, and request `downlevel` device limits so low-end adapters degrade rather than fail. (The renderer is WebGPU-only — the `wgpu` `webgl` feature has been dropped — so a real WebGL2 fallback would be a deliberate re-addition, only worth it for broad reach.)

## P3 — Code cleanups (from the architecture review)

- **Split `systems/movement/mod.rs`** (~390 lines, over the 200-line guideline). Extract the neighbour-accumulation loop and the flocking/solitary force helpers from `update_movement`. Pure refactor — preserve arithmetic order so seed-determinism is unchanged. (`simulation/mod.rs` already had its seeding helpers pulled into `simulation/rng.rs`.)
- **Trim unused stats.** `SimulationStats::from_world` computes `entity_counts`, per-colour classification, several averages, and `world_center_drift` on every `get_stats()`, but the UI consumes only `total_entities`. Either surface them in the UI or trim the struct (crosses the WASM boundary, so update `lib.rs`/JS/tests together).
- **Faster hot-loop RNG.** The per-entity-per-tick RNG is `StdRng` (ChaCha, cryptographic-grade). A small fast PRNG (the `splitmix64` finaliser already in `mix_seed`, or wyrand/pcg) would cut hot-loop cost and shrink the `rand`/`getrandom` footprint, keeping determinism. Deferred deliberately: it changes every seed's output (the algorithm is unchanged, but each seed maps to a different run), so the curated default seed and any shared seeds need re-curating — a poor trade for a marginal speedup on a visual piece, and best done as its own visually-verified pass.

## P3 — Frontend

- **Split the remaining event wiring.** Sliders are now table-driven (the `SLIDERS` table → `setupSliders()`), but `setupEventListeners` still inlines the panel-drag, keyboard, and camera wiring; extract those into focused methods too.
- **Consider TypeScript + prettier** for `web/js/app.js` (bundle via a build-step migration, not standalone).

## P3 — Testing & CI

- **Convert the remaining drift/bias harnesses to assertions.** The simulation-level ones (`test_simulation_clustering`, `test_drift_direction_analysis`) now assert bounded drift via a shared `centroid` helper, and the no-op `test_simulation_entity_processing` is gone. The movement/interaction drift + bias harnesses still only `println!` — give them the same treatment (assert "drift < ε over N ticks"; seed any that use `thread_rng`).
- **Property tests** (`proptest`) for gene mutation bounds, energy clamping, and HSV↔RGB round-trip.
- **A single Playwright smoke test** in CI (page loads, canvas present, `#seed-display` set, no console errors).

## Roadmap — simulation depth

Longer-horizon mechanics, in rough order of appeal: environmental terrain and localized resource patches; aging and disease/parasites; mating rituals and territorial behaviour; and multi-species symbiosis / food webs. Each extends the systems in `src/systems/` and the gene model in `src/genes/`.

## Definition of Done

A change is done when:

- It passes the verification gate in [AGENTS.md](AGENTS.md#verification) (the pre-commit hook enforces it).
- Docs describing affected behaviour are updated to match.
- For user-visible changes: pushed, CI green, and smoke-tested on the live site.
