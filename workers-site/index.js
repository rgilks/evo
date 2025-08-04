// Workers Sites - Static Asset Server
// Serves static files with proper CORS headers for WASM

import { getAssetFromKV } from '@cloudflare/kv-asset-handler'

/**
 * The DEBUG flag will do two things that help during development:
 * 1. we will skip caching on the edge, which makes it easier to
 *    debug.
 * 2. we will return an error message on exception in your Response rather
 *    than the default 404.html page.
 */
const DEBUG = false

addEventListener('fetch', event => {
  try {
    event.respondWith(handleEvent(event))
  } catch (e) {
    if (DEBUG) {
      return event.respondWith(
        new Response(e.message || e.toString(), {
          status: 500,
        }),
      )
    }
    event.respondWith(new Response('Internal Error', { status: 500 }))
  }
})

async function handleEvent(event) {
  const url = new URL(event.request.url)
  let options = {}

  /**
   * You can add custom logic to how we fetch your assets
   * by configuring the function `mapRequestToAsset`
   */
  // options.mapRequestToAsset = req => new Request(`${new URL(req.url).origin}/index.html`, req)

  try {
    if (DEBUG) {
      // customize caching
      options.cacheControl = {
        bypassCache: true,
      }
    }
    
    // Handle CORS preflight requests
    if (event.request.method === "OPTIONS") {
      return new Response(null, {
        status: 200,
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type",
          "Cross-Origin-Embedder-Policy": "require-corp",
          "Cross-Origin-Opener-Policy": "same-origin",
        },
      });
    }

    // Map root path to index.html
    if (url.pathname === "/") {
      url.pathname = "/index.html";
    }

    const page = await getAssetFromKV(event, options)
    
    // Add CORS headers for all responses
    const response = new Response(page.body, page)
    
    // Set CORS headers
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
    response.headers.set("Access-Control-Allow-Headers", "Content-Type")
    response.headers.set("Cross-Origin-Embedder-Policy", "require-corp")
    response.headers.set("Cross-Origin-Opener-Policy", "same-origin")
    
    // Add caching headers for WASM files
    if (url.pathname.endsWith('.wasm')) {
      response.headers.set("Cache-Control", "public, max-age=31536000") // 1 year
    } else if (url.pathname.endsWith('.js') || url.pathname.endsWith('.css')) {
      response.headers.set("Cache-Control", "public, max-age=3600") // 1 hour
    }
    
    return response

  } catch (e) {
    // if an error is thrown try to serve the asset at 404.html
    if (!DEBUG) {
      try {
        let notFoundResponse = await getAssetFromKV(event, {
          mapRequestToAsset: req => new Request(`${new URL(req.url).origin}/404.html`, req),
        })

        return new Response(notFoundResponse.body, { ...notFoundResponse, status: 404 })
      } catch (e) {}
    }

    return new Response(e.message || e.toString(), { status: 500 })
  }
}

/**
 * Here's one example of how to modify a request to
 * allow you to serve a single page app while still
 * having your static assets served correctly.
 */
function mapRequestToAsset(request) {
  const url = new URL(request.url)
  // if the request is for a static asset, serve it
  if (url.pathname.startsWith('/static/')) {
    return request
  }
  // otherwise, serve the index.html file
  return new Request(`${url.origin}/index.html`, request)
} 