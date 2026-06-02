# Backlog

Forward-looking work, roughly ordered by what unblocks or de-risks the most. Present tense; this is intent, not history.

## P1 — Toolchain & dependency modernization

Pinned to `nightly-2024-08-02` because atomics + `-Z build-std` are nightly-only (still true in 2026 — that part is unavoidable, not debt). But the specific pin is ~21 months stale, which forces the `indexmap = "=2.2.6"` workaround, and the dep tree has drifted. Resuming serious work likely starts here.

- Bump to a recent nightly and drop the `indexmap` exact-pin.
- `wgpu` 24 → 29 (five majors; expect mechanical churn in `web/webgpu.rs` + the shader). Highest-effort item.
- `rand` 0.8 → 0.9 (collapses the `getrandom` 0.2/0.3 split), `hecs` 0.9 → 0.11, `dashmap` 5 → 6.
- Treat as one coordinated upgrade — these cascade.

## P1 — Performance: the neighbour read path

The dominant per-tick cost is the `world.get::<&T>()` storm — movement and interaction re-fetch each neighbour's components by random access (~12 scattered lookups per neighbour), and the deterministic nearest-N selection now also does a `Position` lookup per candidate. Fix the data layout, not the algorithm:

- During the grid rebuild (already a full pass), capture the hot neighbour fields (position, hue, energy, size) into a contiguous per-tick **SoA cache** indexed by a dense id; the grid stores indices. Every `world.get` in the hot loop becomes an array index, and nearest-N gets its distances for free.
- Replace the `DashMap` grid (cleared and rebuilt every tick; shard-lock contention in dense cells) with a **dense fixed grid** built by counting sort — lock-free, contiguous per-cell runs, no hashing. The world is bounded, so a flat grid fits.

These share one data-layout insight and together are the biggest non-rewrite performance win.

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
