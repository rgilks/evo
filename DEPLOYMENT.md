# Cloudflare Deployment Guide

This guide explains how to deploy the Evolution Simulation to Cloudflare Workers.

## Prerequisites

1. **Cloudflare Account**: Sign up at [cloudflare.com](https://cloudflare.com)
2. **Wrangler CLI**: Install the Cloudflare Workers CLI
3. **Domain**: A domain name (optional, you can use the default workers.dev subdomain)

## Setup

### 1. Install Wrangler CLI

```bash
npm install -g wrangler
```

### 2. Login to Cloudflare

```bash
wrangler login
```

### 3. Configure the Project

Edit `wrangler.toml` and update the following:

- `name`: Your preferred worker name
- `route`: Your domain (if you have one)
- `kv_namespaces`: Remove or configure if you need KV storage

### 4. Build the Project

```bash
npm run build:web
```

This will:
- Compile the Rust code to WebAssembly
- Generate the necessary JavaScript bindings
- Place the files in the `pkg/` directory

## Deployment

### Quick Deploy

```bash
npm run deploy
```

This will:
1. Build the web assets
2. Deploy to Cloudflare Workers
3. Provide you with a URL

### Environment-Specific Deployments

```bash
# Deploy to staging
npm run deploy:staging

# Deploy to production
npm run deploy:production
```

### Local Development

Test the worker locally before deploying:

```bash
npm run dev:worker
```

This will start a local development server that mimics the Cloudflare Workers environment.

## Configuration

### wrangler.toml

The main configuration file contains:

- **Worker settings**: Name, compatibility date, etc.
- **Site configuration**: Points to the `./web` directory for static assets
- **Build command**: Automatically runs `npm run build:web` before deployment
- **Environment settings**: Separate configs for staging and production

### Environment Variables

You can add environment variables in `wrangler.toml`:

```toml
[vars]
API_KEY = "your-api-key"
ENVIRONMENT = "production"
```

### Custom Domains

To use a custom domain:

1. Add your domain to Cloudflare
2. Update the `route` in `wrangler.toml`
3. Deploy with `npm run deploy:production`

## File Structure

```
├── worker.js          # Cloudflare Worker code
├── wrangler.toml      # Worker configuration
├── web/               # Static assets
│   ├── index.html
│   ├── css/
│   ├── js/
│   └── pkg/           # WASM files (generated)
└── pkg/               # WASM package (generated)
```

## Troubleshooting

### Common Issues

1. **WASM not loading**: Ensure CORS headers are set correctly
2. **Build failures**: Check that all dependencies are installed
3. **Deployment errors**: Verify your Cloudflare account and permissions

### Debug Commands

```bash
# Check worker logs
wrangler tail

# Test locally
wrangler dev

# Check build output
ls -la pkg/
```

## Performance

The worker is optimized for:

- **Fast loading**: WASM files are cached for 1 year
- **CORS compliance**: Proper headers for cross-origin requests
- **Static asset serving**: Efficient delivery of HTML, CSS, and JS

## Monitoring

Monitor your deployment:

1. **Cloudflare Dashboard**: View analytics and logs
2. **Worker Logs**: Use `wrangler tail` for real-time logs
3. **Performance**: Check the Cloudflare Analytics dashboard

## Security

The worker includes:

- **CORS headers**: Proper cross-origin resource sharing
- **Security headers**: COEP and COOP for isolation
- **Input validation**: Prevents directory traversal attacks

## Cost

Cloudflare Workers pricing:

- **Free tier**: 100,000 requests/day
- **Paid tier**: $5/month for 10M requests
- **Additional**: $0.50 per million requests

Most small to medium projects will fit within the free tier. 