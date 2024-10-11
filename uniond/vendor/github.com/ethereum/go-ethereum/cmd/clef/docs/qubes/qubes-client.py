"""
This implements a dispatcher which listens to localhost:8550, and proxies
requests via qrexec to the service qubes.EthSign on a target domain
"""

import http.server
import socketserver,subprocess

PORT=8550
TARGET_DOMAIN= 'debian-work'

class Dispatcher(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        post_data = self.rfile.read(int(self.headers['Content-Length']))
        p = subprocess.Popen(['/usr/bin/qrexec-client-vm',TARGET_DOMAIN,'qubes.Clefsign'],stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        output = p.communicate(post_data)[0]
        self.wfile.write(output)


with socketserver.TCPServer(("",PORT), Dispatcher) as httpd:
    print("Serving at port", PORT)
    httpd.serve_forever()

