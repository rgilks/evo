# Cloudflare Deployment Guide

This guide explains how to deploy the Evolution Simulation to Cloudflare Pages.

## Prerequisites

1. **Cloudflare Account**: [Sign up](https://cloudflare.com)
2. **Wrangler CLI**: `npm install -g wrangler`
3. **Rust Toolchain**: `rustup default nightly-2024-08-02 && rustup target add wasm32-unknown-unknown`

## One-Command Deployment

The easiest way to deploy is using the npm script which handles building and deploying:

```bash
npm run deploy
```

This command runs `build:web` (compiling Rust to WASM) and then `wrangler pages deploy`.

## Manual Steps

If you prefer to run steps individually:

1. **Build Web Assembly**:
   ```bash
   npm run build:web
   ```
   *Output*: Generates `pkg/` and ensures `web/pkg/` contains the latest WASM.

2. **Deploy to Pages**:
   ```bash
   npx wrangler pages deploy web --project-name evo
   ```

## Local Testing

To simulate the exact Cloudflare environment locally:

```bash
npm run dev:worker
```
*Note: This uses `wrangler pages dev` which respects the `_headers` file.*

## Critical Configuration: SharedArrayBuffer

This project uses multi-threading (`rayon`), which requires `SharedArrayBuffer`. This feature is only available in secure contexts with specific headers:

- **Config File**: `web/_headers`
- **Required Headers**:
  ```
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp
  ```

**Troubleshooting:**
- **Error**: "SharedArrayBuffer is not defined"
- **Fix**: Ensure `web/_headers` exists and is deployed. Note that some local dev servers (like `python -m http.server`) do NOT send these headers. Use `npm run dev` (Node.js) or `npm run dev:worker` (Wrangler) for local testing.