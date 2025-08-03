#!/usr/bin/env python3
import http.server
import socketserver


class CORSHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        super().end_headers()


if __name__ == "__main__":
    PORT = 8000
    with socketserver.TCPServer(("", PORT), CORSHTTPRequestHandler) as httpd:
        print(f"Serving at http://localhost:{PORT}")
        print(
            "Make sure to build the WASM package first with: wasm-pack build --target web --out-dir ../pkg"
        )
        httpd.serve_forever()
