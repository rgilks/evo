#!/usr/bin/env node

import { createServer } from "http";
import { readFile } from "fs/promises";
import { extname, join } from "path";
import { fileURLToPath } from "url";
import { dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
// When running from project root, we need to serve files from the web directory
const webDir = __dirname;

// MIME types for different file extensions
const mimeTypes = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".png": "image/png",
  ".jpg": "image/jpg",
  ".gif": "image/gif",
  ".svg": "image/svg+xml",
  ".wav": "audio/wav",
  ".mp4": "video/mp4",
  ".woff": "application/font-woff",
  ".ttf": "application/font-ttf",
  ".eot": "application/vnd.ms-fontobject",
  ".otf": "application/font-otf",
  ".wasm": "application/wasm",
};

// Try different ports if the default is in use
const ports = [8000, 8001, 8002, 8003, 8004];

function createServerOnPort(port) {
  const server = createServer(async (req, res) => {
    let filePath;
    try {
      // Handle CORS headers
      res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
      res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
      res.setHeader("Access-Control-Allow-Origin", "*");
      res.setHeader(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS"
      );
      res.setHeader("Access-Control-Allow-Headers", "Content-Type");

      // Handle preflight requests
      if (req.method === "OPTIONS") {
        res.writeHead(200);
        res.end();
        return;
      }

      filePath = req.url === "/" ? "/index.html" : req.url.split("?")[0];
      filePath = join(webDir, filePath);

      // Debug logging
      console.log(`Request: ${req.url} -> ${filePath}`);

      // Security: prevent directory traversal
      if (!filePath.startsWith(webDir)) {
        console.log(`Forbidden: ${filePath} not in ${webDir}`);
        res.writeHead(403);
        res.end("Forbidden");
        return;
      }

      const ext = extname(filePath);
      const contentType = mimeTypes[ext] || "application/octet-stream";

      const content = await readFile(filePath);
      console.log(`Serving: ${filePath} (${contentType})`);
      res.writeHead(200, { "Content-Type": contentType });
      res.end(content);
    } catch (error) {
      if (error.code === "ENOENT") {
        // File not found
        console.log(`File not found: ${filePath || req.url}`);
        res.writeHead(404);
        res.end("File not found");
      } else {
        console.error("Server error:", error);
        res.writeHead(500);
        res.end("Internal server error");
      }
    }
  });

  server.listen(port, () => {
    console.log(`🌐 Serving at http://localhost:${port}`);
    console.log(
      "Make sure to build the WASM package first with: just build-web"
    );
  });

  server.on("error", (error) => {
    if (error.code === "EADDRINUSE") {
      console.log(`Port ${port} in use, trying next port...`);
      const nextPortIndex = ports.indexOf(port) + 1;
      if (nextPortIndex < ports.length) {
        createServerOnPort(ports[nextPortIndex]);
      } else {
        console.error(`Error: Could not start server on any port ${ports}`);
        process.exit(1);
      }
    } else {
      console.error("Server error:", error);
      process.exit(1);
    }
  });

  return server;
}

// Start the server
createServerOnPort(ports[0]);
