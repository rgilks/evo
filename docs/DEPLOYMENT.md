# Cloudflare Deployment Guide

This guide explains how to deploy the Evolution Simulation to Cloudflare Pages.

## Prerequisites

1. **Cloudflare Account**: Sign up at [cloudflare.com](https://cloudflare.com)
2. **Wrangler CLI**: Install the Cloudflare Workers/Pages CLI
3. **Rust Toolchain**: Nightly toolchain with WASM support (see README.md)

## Deployment Steps

### 1. Install Dependencies

```bash
npm install
```

### 2. Login to Cloudflare

```bash
npx wrangler login
```

### 3. Build the Project

```bash
npm run build:web
```

This commands compiles the Rust code to WebAssembly and places the output in the `pkg/` directory.

### 4. Deploy to Cloudflare Pages

```bash
# Deploy to production
npm run deploy

# Or manually:
npx wrangler pages deploy web --project-name evo
```

This uploads the `web/` directory (which includes the index.html, CSS, JS, and the generated WASM in `web/pkg/`) to Cloudflare Pages.

## Local Development

Test the Pages build locally:

```bash
npm run dev:worker
```

This builds the project and starts a local Pages server (using `wrangler pages dev`).

## Configuration

### COOP/COEP Headers regarding SharedArrayBuffer

The simulation uses `SharedArrayBuffer` for parallel processing (via `rayon`). This requires specific security headers to be served:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

These are configured in the `web/_headers` file, which Cloudflare Pages uses to apply headers.

## Troubleshooting

### "SharedArrayBuffer is not defined"
If you see this error in the browser console, it means the COOP/COEP headers are missing. Ensure `web/_headers` exists and is being deployed.