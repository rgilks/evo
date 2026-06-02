# Backlog

Forward-looking work, roughly ordered by what unblocks or de-risks the most. Present tense; this is intent, not history.

## P1 — Toolchain & dependency modernization

The project is pinned to `nightly-2024-08-02` because atomics + `-Z build-std` are nightly-only. The ecosystem is starting to outgrow it: `indexmap` is pinned to `=2.2.6` to avoid an `E0658` on this nightly, and the lockfile carries both `getrandom` 0.2 and 0.3. Resuming serious work likely starts here.

- Bump the pinned nightly and re-evaluate the `indexmap` pin.
- `wgpu` 24 → current (high-churn; expect renderer changes).
- `rand` 0.8 → 0.9; collapse the `getrandom` 0.2/0.3 split.
- Treat this as one coordinated upgrade — these tend to cascade.

## P1 — Verify / restore the live deployment

`https://evo.pages.dev` (the project name CI deploys to) currently returns 404. The deploy config is sound, so the Pages project was likely deleted or never finished a deploy.

- Confirm the Cloudflare Pages project exists and the `CLOUDFLARE_API_TOKEN` / `CLOUDFLARE_ACCOUNT_ID` repo secrets are valid.
- Confirm the canonical public URL and add it to the README.

## P2 — GPU-accelerated spatial processing (100K–1M entities)

Rescued from the deleted `million-scale-optimization` branch (tip was `120fa37`; recoverable from history if revisited). The idea: move neighbour search / spatial hashing onto the GPU to push entity counts toward 100K–1M, with a benchmarking harness to measure it. The current CPU + DashMap grid and the ~20,000-instance render ceiling top out well below that. Revisit only after the toolchain bump, since it touches `wgpu` heavily.

## P2 — Robust cache-busting in the build

`scripts/build-web.sh` injects a git-SHA `?v=` query into generated files via a chain of `sed` rewrites. It works but is brittle string-surgery on generated output and was the source of repeated Cloudflare `LinkError` firefighting. Consider a cleaner mechanism (content hashing, an import map, or a small build step) so cache invalidation is not fragile.

## P3 — Test suite quality

`cargo test` reports ~99 tests, but several are non-asserting diagnostic harnesses left over from debugging visual drift/clustering (e.g. `test_drift_direction_analysis`, `test_simulation_clustering`) — they `println!` analysis and pass unconditionally. Convert these to real assertions, or move them to `examples/` / benches so the headline test count reflects the actual safety net.

## P3 — WebGPU-only rendering

The renderer targets WebGPU and does nothing useful on browsers without it. Either add a real WebGL fallback (the `wgpu` `webgl` feature is already enabled) or detect WebGPU support and surface a clear "WebGPU required" message in the UI.

## Roadmap — simulation depth

Longer-horizon mechanics live in [docs/SIMULATION_SYSTEM.md](docs/SIMULATION_SYSTEM.md#roadmap--future-ideas): environmental terrain and resource patches, aging/disease, mating rituals, and multi-species food webs.

## Definition of Done

A change is done when:

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and the WASM `cargo check` all pass (the pre-commit hook enforces this).
- Docs describing affected behaviour are updated to match.
- For user-visible changes: pushed, CI green, and smoke-tested on the live site.
