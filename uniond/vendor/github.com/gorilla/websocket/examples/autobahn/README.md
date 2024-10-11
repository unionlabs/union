# Test Server

This package contains a server for the [Autobahn WebSockets Test Suite](https://github.com/crossbario/autobahn-testsuite).

To test the server, run

    go run server.go

and start the client test driver

    mkdir -p reports
    docker run -it --rm \
        -v ${PWD}/config:/config \
        -v ${PWD}/reports:/reports \
        crossbario/autobahn-testsuite \
        wstest -m fuzzingclient -s /config/fuzzingclient.json

When the client completes, it writes a report to reports/index.html.
