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

The `world.get::<&T>()` storm is gone: the grid rebuild now also builds a per-tick `NeighborCache` (`systems/mod.rs`) of each entity's hot fields, so movement, interaction, and nearest-N selection read one cached snapshot per neighbour instead of a scatter of component fetches. The grid itself is a `HashMap` of cell → entities, rebuilt serially — no more `DashMap` shard-lock contention.

What remains is a smaller, more invasive refinement: replace the cell `HashMap` with a **dense fixed grid** built by counting sort (lock-free, contiguous per-cell runs, no hashing), and key the cache by a dense slot id so the grid stores indices and neighbour data is fully contiguous (true SoA). The world is bounded, so a flat grid fits. This is the step from "good locality" to "optimal locality" — measure it against the headless bench harness before taking on the churn.

## P2 — Headless run mode + benchmarks

The crate is `cdylib`-only, so there's no native way to profile or benchmark. Add `"rlib"` to `crate-type`, an `examples/headless.rs` that runs N ticks from a fixed seed (now possible — the sim is deterministic), and `criterion` benches over the per-tick hot loop. Prerequisite for measuring any of the performance work above.

## P2 — GPU compute for scale (100K–1M entities)

Reaching 100K–1M is a **separate GPU-compute engine**, not an optimization of the current one: ping-pong storage buffers, a counting-sort spatial grid + prefix sum on the GPU, force/movement compute shaders, and indirect draw. CPU + rayon cannot reach that scale. The deleted `million-scale-optimization` branch (tip `120fa37`) is an *investigation*, not a starting implementation — its shaders were brute-force O(N) scans, not a GPU spatial hash. This sim's per-entity logic (predation, energy transfer, births/deaths = structural mutation needing GPU compaction) makes the port heavier than a plain particle-life sim. Revisit only after the toolchain bump, and only if the scale is genuinely wanted.

## P2 — Build robustness: replace the `sed` cache-busting

`scripts/build-web.sh` injects a git-SHA `?v=` query via a chain of `sed` rewrites — brittle string-surgery on generated output, and largely redundant with `web/_headers` (`max-age=0, must-revalidate`). Drop most of it; fix the one load-bearing worker-import path via `wasm-bindgen-rayon`'s `no-bundler` feature, or move to content-hashed filenames / an import map.

## P2 — Deploy URL cleanup

The site is live at `https://evo-dgc.pages.dev` (Cloudflare suffixed the project because `evo` was taken). Either rename the Pages project to reclaim `evo.pages.dev`, or adopt `evo-dgc` and add the canonical URL to the README.

## P3 — Rendering polish

- **Skip redundant re-uploads.** The render loop repacks/re-uploads the instance buffer every `requestAnimationFrame`, even on frames where the sim didn't tick (only the interpolation uniform changed). Gate the upload on a sim tick — free FPS on high-refresh displays.
- **Additive blending for the glow.** Switch from alpha to additive blending so overlapping glows accumulate (order-independent, correct for particles on black) instead of draw-order-dependent occlusion.
- **Simplify the fragment glow** (six `smoothstep`s → a 2-term or gaussian falloff) once counts rise — overdraw dominates glow-heavy rendering.
- **WebGPU-unavailable UX.** Replace the 5-second error toast with a persistent "WebGPU required" message, and request `downlevel` device limits so low-end adapters degrade rather than fail. A real WebGL2 fallback (the `wgpu` `webgl` feature is already on) is larger and only worth it for broad reach.

## P3 — Code cleanups (from the architecture review)

- **Drop `Genes` from `EntityUpdate`.** It's cloned (~96 B) for every entity every tick but only reproducers use it (<1%); fetch the parent's genes in the apply phase instead.
- **Single config owner.** `WebSimulation` duplicates `SimulationConfig` alongside `Simulation`'s copy and clones the whole struct on every `update_param`; give `Simulation` a `set_param` and drop the duplicate.
- **Predation self-size bug.** Target-finding (`movement`) passes a dummy `Size { radius: 1.0 }` into `can_eat`, so "what I chase" and "what I can eat" use different sizes. Use the real size.

## P3 — Testing & CI

- **Convert the diagnostic drift/bias tests to assertions.** They already compute drift/centroid numbers and only `println!`; now that runs are deterministic, assert "drift < ε over N ticks." Turns the dead harnesses into real regression guards.
- Property tests (`proptest`) for gene mutation bounds, energy clamping, and HSV↔RGB round-trip.
- A single Playwright smoke test in CI (page loads, canvas present, no console errors).
- Cache cargo in the CI **deploy** job (the `build-std` WASM build is uncached there today).
- Consider TypeScript + prettier for `web/js/app.js` (bundle with a build-step migration, not standalone).

## Roadmap — simulation depth

Longer-horizon mechanics live in [docs/SIMULATION_SYSTEM.md](docs/SIMULATION_SYSTEM.md#roadmap--future-ideas): environmental terrain and resource patches, aging/disease, mating rituals, and multi-species food webs.

## Definition of Done

A change is done when:

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and the WASM `cargo check` all pass (the pre-commit hook enforces this).
- Docs describing affected behaviour are updated to match.
- For user-visible changes: pushed, CI green, and smoke-tested on the live site.
