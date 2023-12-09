# ABCI and State Streaming Plugin (gRPC)

The `BaseApp` package contains the interface for a [ABCIListener](https://github.com/cosmos/cosmos-sdk/blob/main/baseapp/streaming.go)
service used to write state changes out from individual KVStores to external systems,
as described in [ADR-038](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-038-state-listening.md).

Specific `ABCIListener` service implementations are written and loaded as [hashicorp/go-plugin](https://github.com/hashicorp/go-plugin).

## Implementation

In this section we describe the implementation of the `ABCIListener` interface as a gRPC service.

### Service Protocol

The companion service protocol for the `ABCIListener` interface is described below.
See [proto/cosmos/store/streaming/abci/grpc.proto](https://github.com/cosmos/cosmos-sdk/blob/main/proto/cosmos/store/streaming/abci/grpc.proto) for full details.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/6cee22df52eb0cbb30e351fbb41f66d26c1f8300/proto/cosmos/store/streaming/abci/grpc.proto#L1-L36
```

### Generating the Code

To generate the stubs the local client implementation can call, run the following command:

```shell
make proto-gen
```

For other languages you'll need to [download](https://github.com/cosmos/cosmos-sdk/blob/main/third_party/proto/README.md)
the CosmosSDK protos into your project and compile. For language specific compilation instructions visit
[https://github.com/grpc](https://github.com/grpc) and look in the `examples` folder of your
language of choice `https://github.com/grpc/grpc-{language}/tree/master/examples` and [https://grpc.io](https://grpc.io)
for the documentation.

### gRPC Client and Server

Implementing the ABCIListener gRPC client and server is a simple and straight forward process.

To create the client and server we create a `ListenerGRPCPlugin` struct that implements the
`plugin.GRPCPlugin` interface and a `Impl` property that will contain a concrete implementation
of the `ABCIListener` plugin written in Go.

#### The Interface

The `BaseApp` `ABCIListener` interface will be what will define the plugins capabilities.

Boilerplate RPC implementation example of the `ABCIListener` interface. ([store/streaming/abci/grpc.go](https://github.com/cosmos/cosmos-sdk/blob/main/store/streaming/abci/grpc.go))

```go reference
https://github.com/cosmos/cosmos-sdk/blob/f851e188b3b9d46e7c63fa514ad137e6d558fdd9/store/streaming/abci/grpc.go#L13-L79
```

Our `ABCIlistener` service plugin. ([store/streaming/plugins/abci/v1/interface.go](interface.go))

```go reference
https://github.com/cosmos/cosmos-sdk/blob/f851e188b3b9d46e7c63fa514ad137e6d558fdd9/store/streaming/abci/interface.go#L13-L45
```

#### Plugin Implementation

Plugin implementations can be in a completely separate package but will need access
to the `ABCIListener` interface. One thing to note here is that plugin implementations
defined in the `ListenerGRPCPlugin.Impl` property are **only** required when building
plugins in Go. They are pre-compiled into Go modules. The `GRPCServer.Impl` calls methods
on this out-of-process plugin.

For Go plugins this is all that is required to process data that is sent over gRPC.
This provides the advantage of writing quick plugins that process data to different
external systems (i.e: DB, File, DB, Kafka, etc.) without the need for implementing
the gRPC server endpoints.

```go
// MyPlugin is the implementation of the ABCIListener interface
// For Go plugins this is all that is required to process data sent over gRPC.
type MyPlugin struct {
	...
}

func (a FilePlugin) ListenFinalizeBlock(ctx context.Context, req abci.RequestBeginBlock, res abci.ResponseBeginBlock) error {
	// process data
	return nil
}

func (a FilePlugin) ListenCommit(ctx context.Context, res abci.ResponseCommit, changeSet []*store.StoreKVPair) error {
    // process data
    return nil
}

func main() {
	plugin.Serve(&plugin.ServeConfig{
		HandshakeConfig: v1.Handshake,
		Plugins: map[string]plugin.Plugin{
			"abci": &ABCIListenerGRPCPlugin{Impl: &MyPlugin{}},
		},

		// A non-nil value here enables gRPC serving for this streaming...
		GRPCServer: plugin.DefaultGRPCServer,
	})
}
```

## Plugin Loading System

A general purpose plugin loading system has been provided by the SDK to be able to load not just
the `ABCIListener` service plugin but other protocol services as well. You can take a look
at how plugins are loaded by the SDK in [store/streaming/streaming.go](https://github.com/cosmos/cosmos-sdk/blob/main/store/streaming/streaming.go)

You'll need to add this in your `app.go`

```go
// app.go

func NewApp(...) *App {

    ...

    // register streaming services
    streamingCfg := cast.ToStringMap(appOpts.Get(baseapp.StreamingTomlKey))
    for service := range streamingCfg {
        pluginKey := fmt.Sprintf("%s.%s.%s", baseapp.StreamingTomlKey, service, baseapp.StreamingABCIPluginTomlKey)
        pluginName := strings.TrimSpace(cast.ToString(appOpts.Get(pluginKey)))
        if len(pluginName) > 0 {
            logLevel := cast.ToString(appOpts.Get(flags.FlagLogLevel))
            plugin, err := streaming.NewStreamingPlugin(pluginName, logLevel)
            if err != nil {
                tmos.Exit(err.Error())
            }
            if err := baseapp.RegisterStreamingPlugin(bApp, appOpts, keys, plugin); err != nil {
                tmos.Exit(err.Error())
            }
        }
    }

    ...
}
```

## Configuration

Update the streaming section in `app.toml`

```toml
# Streaming allows nodes to stream state to external systems
[streaming]

# streaming.abci specifies the configuration for the ABCI Listener streaming service
[streaming.abci]

# List of kv store keys to stream out via gRPC
# Set to ["*"] to expose all keys.
keys = ["*"]

# The plugin name used for streaming via gRPC
# Supported plugins: abci
plugin = "abci"

# stop-node-on-err specifies whether to stop the node when the 
stop-node-on-err = true
```

## Updating the protocol

If you update the protocol buffers file, you can regenerate the file and plugins using the
following commands from the project root directory. You do not need to run this if you're
just trying the examples, you can skip ahead to the [Testing](#testing) section.

```shell
make proto-gen 
```

* stdout plugin; from inside the `store/` dir, run:

```shell
go build -o streaming/abci/examples/stdout/stdout streaming/abci/examples/stdout/stdout.go
```

* file plugin (writes to `~/`); from inside the `store/` dir, run:

```shell
go build -o streaming/abci/examples/file/file streaming/abci/examples/file/file.go
```

### Testing

Export a plugin from one of the Go or Python examples.

* stdout plugin

```shell
export COSMOS_SDK_ABCI="{path to}/cosmos-sdk/store/streaming/abci/examples/stdout/stdout"
```

* file plugin (writes to ~/)

```shell
export COSMOS_SDK_ABCI="{path to}/cosmos-sdk/store/streaming/abci/examples/file/file"
```

where `{path to}` is the parent path to the `cosmos-sdk` repo on you system.

Test:

```shell
make test-sim-nondeterminism-streaming
```

The plugin system will look for the plugin binary in the `env` variable `COSMOS_SDK_{PLUGIN_NAME}` above
and if it does not find it, it will error out. The plugin UPPERCASE name is that of the
`streaming.abci.plugin` TOML configuration setting.
