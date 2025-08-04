// Cloudflare Worker for Evolution Simulation
// Simple test worker

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const path = url.pathname;

    console.log(`Request to: ${path}`);

    // Simple test response
    if (path === "/") {
      return new Response("Hello from Cloudflare Worker! Path: " + path, {
        status: 200,
        headers: {
          "Content-Type": "text/plain",
        },
      });
    }

    // Try to serve static assets
    try {
      console.log(`Trying to fetch: ${path}`);
      const response = await env.ASSETS.fetch(new URL(path, request.url));
      console.log(`Asset response status: ${response.status}`);
      
      if (response.status === 200) {
        return new Response(response.body, {
          status: 200,
          headers: {
            "Content-Type": "text/plain",
          },
        });
      }
    } catch (error) {
      console.error(`Error: ${error.message}`);
    }

    return new Response("Not Found: " + path, {
      status: 404,
      headers: {
        "Content-Type": "text/plain",
      },
    });
  },
};
