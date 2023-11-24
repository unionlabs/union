# Cosmos-SDK Plugins

This package contains an extensible plugin system for the Cosmos-SDK. The plugin system leverages the [hashicorp/go-plugin](https://github.com/hashicorp/go-plugin) system. This system is designed to work over RPC. 

Although the `go-plugin` is built to work over RPC, it is currently only designed to work over a local network.

## Pre requisites

For an overview of supported features by the `go-plugin` system, please see https://github.com/hashicorp/go-plugin. The `go-plugin` documentation is located [here](https://github.com/hashicorp/go-plugin/tree/master/docs). You can also directly visit any of the links below:

* [Writing plugins without Go](https://github.com/hashicorp/go-plugin/blob/master/docs/guide-plugin-write-non-go.md) 
* [Go Plugin Tutorial](https://github.com/hashicorp/go-plugin/blob/master/docs/extensive-go-plugin-tutorial.md)
* [Plugin Internals](https://github.com/hashicorp/go-plugin/blob/master/docs/internals.md)
* [Plugin Architecture](https://www.youtube.com/watch?v=SRvm3zQQc1Q) (start here)

## Exposing plugins

To expose plugins to the plugin system, you will need to:

1. Implement the gRPC message protocol service of the plugin
2. Build the plugin binary
3. Export it

Read the plugin documentation in the [Streaming Plugins](#streaming-plugins) section for examples on how to build a plugin.

## Streaming Plugins

List of support streaming plugins

* [ABCI State Streaming Plugin](abci/README.md)
