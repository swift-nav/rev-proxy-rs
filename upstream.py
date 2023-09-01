#!/usr/bin/env python3

import sys

from http.server import BaseHTTPRequestHandler, HTTPServer

HOSTNAME = "127.23.8.31"
SERVERPORT = 8080


class Server(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        if self.path == '/shutdown_123456':
            self.wfile.write(b"shutting down")
            sys.exit(0)
        else:
            self.wfile.write(b"Hi")


if __name__ == "__main__":

    webServer = HTTPServer((HOSTNAME, SERVERPORT), Server)
    print("Server started http://%s:%s" % (HOSTNAME, SERVERPORT))

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.")
