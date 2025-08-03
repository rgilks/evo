#!/usr/bin/env python3
import http.server
import socketserver


class CORSHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        super().end_headers()


if __name__ == "__main__":
    import sys
    
    # Try different ports if 8000 is in use
    ports = [8000, 8001, 8002, 8003, 8004]
    
    # Change to web directory before starting server
    import os
    web_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(web_dir)
    
    for PORT in ports:
        try:
            with socketserver.TCPServer(("", PORT), CORSHTTPRequestHandler) as httpd:
                print(f"Serving at http://localhost:{PORT}")
                print(
                    "Make sure to build the WASM package first with: wasm-pack build --target web --out-dir ../pkg"
                )
                httpd.serve_forever()
                break
        except OSError as e:
            if PORT == ports[-1]:  # Last port
                print(f"Error: Could not start server on any port {ports}")
                print(f"Last error: {e}")
                sys.exit(1)
            print(f"Port {PORT} in use, trying next port...")
            continue
