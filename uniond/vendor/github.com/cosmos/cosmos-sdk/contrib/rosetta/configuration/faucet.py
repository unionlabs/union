from http.server import HTTPServer, BaseHTTPRequestHandler
import subprocess

import os


class SimpleHTTPRequestHandler(BaseHTTPRequestHandler):

    def do_POST(self):
        try:
            content_len = int(self.headers.get('Content-Length'))
            addr = self.rfile.read(content_len).decode("utf-8")
            print("sending funds to " + addr)
            subprocess.call(['sh', './send_funds.sh', addr])
            self.send_response(200)
            self.end_headers()
        except Exception as e:
            print("failed " + str(e))
            os._exit(1)


if __name__ == "__main__":
    print("starting faucet server...")
    httpd = HTTPServer(('0.0.0.0', 8000), SimpleHTTPRequestHandler)
    httpd.serve_forever()
