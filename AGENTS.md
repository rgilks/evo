# Agent Notes

Operational guidance for Claude Code and other repo agents.

## Project

evo is a browser-based artificial-life / evolution simulation. Thousands of gene-carrying creatures move, predate, reproduce, and mutate under natural selection. The simulation core is Rust compiled to WebAssembly, multithreaded in-browser via `rayon` + `SharedArrayBuffer`, rendered with WebGPU, and deployed as a static site to Cloudflare Pages.

Read these before substantial work:

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — how the code is organized and how a frame is produced.
- [docs/SIMULATION_SYSTEM.md](docs/SIMULATION_SYSTEM.md) — the simulation mechanics: genes, movement styles, predation, spatial grid.
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) — Cloudflare Pages deploy and the SharedArrayBuffer header requirement.
- [BACKLOG.md](BACKLOG.md) — ordered next work and known constraints.

## Workflow

- Work directly on `main`.
- Check `git status` before editing; preserve unrelated local changes.
- Stage explicit file paths, not `git add -A` / `git add .`.
- For user-visible code changes the standing flow is: commit, push, watch CI, then smoke-test the live site. Docs-only changes just need commit + push.

## Verification

Standard gate — mirrors CI (`.github/workflows/ci-cd.yml`) and the `.husky/pre-commit` hook exactly:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
CARGO_UNSTABLE_BUILD_STD=std,panic_abort cargo check --target wasm32-unknown-unknown
```

- Web build: `npm run build` (wasm-pack + cache-busting; see ARCHITECTURE).
- Local run: `npm run dev` (Wrangler on port 8788; honors `web/_headers` so threads work).
- Never bypass the hook with `--no-verify` unless explicitly asked.

## Architecture Rules

- Separate evolution logic from rendering and UI.
- Keep components small and focused; query only the components a system needs (hecs).
- Use `rayon` for CPU-intensive work over large populations; prefer `par_iter` over manual threads.
- The per-tick pattern is read-all → compute an update list in parallel → apply structural mutations serially. hecs spawn/despawn is single-threaded, so never mutate the world from inside a parallel query.
- Design for large populations. Keep simulation parameters adjustable and hot-swappable via `update_param`.

## Code Map

- WASM entry + JS bindings: `src/lib.rs` (`WebSimulation`: `update`, `update_entity_buffer`, `entity_count`, `get_stats`, `update_param`).
- Tick orchestrator: `src/simulation/mod.rs`.
- Components: `src/components.rs`.
- Genes (generation, mutation, similarity, predation preference, particle-life weights): `src/genes/mod.rs`.
- Systems: `src/systems/movement/`, `src/systems/interaction/` (predation), `src/systems/energy.rs`, `src/systems/reproduction.rs`.
- Neighbour lookups: `src/spatial_grid.rs` (DashMap spatial hash).
- Config / tunable parameters: `src/config/mod.rs`.
- Stats serialized to JS: `src/stats/mod.rs`.
- Renderer + shaders: `src/web/webgpu.rs`, `src/shader.wgsl`.
- Frontend: `web/index.html`, `web/js/app.js`.

## Tests

- Tests are co-located with the code they cover: `src/<module>/tests.rs` declared via `#[cfg(test)] mod tests;`.
- `cargo test` runs the full suite.
- Some legacy "tests" are non-asserting diagnostic harnesses (drift/clustering analysis); prefer real assertions for new tests. See BACKLOG.

## Commits

- Keep messages short and outcome-focused; reference `file.rs:line` where it helps a reader.
- Stage explicit paths.
- On a pre-commit hook failure, fix the issue and make a NEW commit — do not blind-`--amend`; the failed commit did not happen.

## Code Style

- Files preferably under 200 lines (more is fine when most of the file is tests). Functions preferably under 20 lines.
- `rustfmt` + `clippy` clean: fix all warnings. Write idiomatic, concise Rust.
- Organize code by domain, not by technical layer. Group related components and systems together.

## Docs

- Docs describe the current state in the present tense. Keep history in git, not in docs.
- Add a BACKLOG item for useful intent that should not be built immediately.
