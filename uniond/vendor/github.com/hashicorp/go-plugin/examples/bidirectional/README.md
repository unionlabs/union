# Counter Example

This example builds a simple key/counter store CLI where the mechanism
for storing and retrieving keys is pluggable. However, in this example we don't
trust the plugin to do the summation work. We use bi-directional plugins to
call back into the main proccess to do the sum of two numbers. To build this example:

```sh
# This builds the main CLI
$ go build -o counter

# This builds the plugin written in Go
$ go build -o counter-go-grpc ./plugin-go-grpc

# This tells the Counter binary to use the "counter-go-grpc" binary
$ export COUNTER_PLUGIN="./counter-go-grpc"

# Read and write
$ ./counter put hello 1
$ ./counter put hello 1

$ ./counter get hello
2
```

### Plugin: plugin-go-grpc

This plugin uses gRPC to serve a plugin that is written in Go:

```
# This builds the plugin written in Go
$ go build -o counter-go-grpc ./plugin-go-grpc

# This tells the KV binary to use the "kv-go-grpc" binary
$ export COUNTER_PLUGIN="./counter-go-grpc"
```

## Updating the Protocol

If you update the protocol buffers file, you can regenerate the file
using the following command from this directory. You do not need to run
this if you're just trying the example.

```sh
$ buf generate
```
