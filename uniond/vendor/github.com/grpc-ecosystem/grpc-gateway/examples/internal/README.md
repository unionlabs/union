# One way to run the example

```bash
# Handle dependencies
$ dep init
```

Follow the guides from this [README.md](./browser/README.md) to run the server and gateway.
```bash
# Make sure you are in the correct directory: 
# $GOPATH/src/github.com/grpc-ecosystem/grpc-gateway/examples
$ cd examples/internal/browser
$ pwd

# Install gulp
$ npm install -g gulp-cli
$ npm install
$ gulp

# Run
$ gulp bower
$ gulp backends
```

Then you can use curl or a browser to test:

```bash
# List all apis
$ curl http://localhost:8080/swagger/echo_service.swagger.json

# Visit the apis
$ curl -XPOST http://localhost:8080/v1/example/echo/foo
{"id":"foo"}

$ curl  http://localhost:8080/v1/example/echo/foo/123
{"id":"foo","num":"123"}

```

So you have visited the apis by HTTP successfully. You can also try other apis.
