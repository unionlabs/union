# Negotiated version KV Example

This example builds a simple key/value store CLI where the plugin version can
be negotiated between client and server.

```sh
# This builds the main CLI
$ go build -o kv

# This builds the plugin written in Go
$ go build -o kv-plugin ./plugin-go

# Write a value using proto version 3 and grpc
$ KV_PROTO=grpc ./kv put hello world

# Read it back using proto version 2 and netrpc
$ KV_PROTO=netrpc ./kv get hello
world

Written from plugin version 3
Read by plugin version 2
```

# Negotiated Protocol

The Client sends the list of available plugin versions to the server. When
presented with a list of plugin versions, the server iterates over them in
reverse, and uses the highest numbered match to choose the plugins to execute.
If a legacy client is used and no versions are sent to the server, the server
will default to the oldest version in its configuration.
