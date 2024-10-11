# ADR 038: KVStore state listening

## Changelog

* 11/23/2020: Initial draft
* 10/06/2022: Introduce plugin system based on hashicorp/go-plugin
* 10/14/2022:
    * Add `ListenCommit`, flatten the state writes in a block to a single batch.
    * Remove listeners from cache stores, should only listen to `rootmulti.Store`.
    * Remove `HaltAppOnDeliveryError()`, the errors are propagated by default, the implementations should return nil if don't want to propagate errors.
* 26/05/2023: Update with ABCI 2.0

## Status

Proposed

## Abstract

This ADR defines a set of changes to enable listening to state changes of individual KVStores and exposing these data to consumers.

## Context

Currently, KVStore data can be remotely accessed through [Queries](https://github.com/cosmos/cosmos-sdk/blob/main/docs/build/building-modules/02-messages-and-queries.md#queries)
which proceed either through Tendermint and the ABCI, or through the gRPC server.
In addition to these request/response queries, it would be beneficial to have a means of listening to state changes as they occur in real time.

## Decision

We will modify the `CommitMultiStore` interface and its concrete (`rootmulti`) implementations and introduce a new `listenkv.Store` to allow listening to state changes in underlying KVStores. We don't need to listen to cache stores, because we can't be sure that the writes will be committed eventually, and the writes are duplicated in `rootmulti.Store` eventually, so we should only listen to `rootmulti.Store`.
We will introduce a plugin system for configuring and running streaming services that write these state changes and their surrounding ABCI message context to different destinations.

### Listening

In a new file, `store/types/listening.go`, we will create a `MemoryListener` struct for streaming out protobuf encoded KV pairs state changes from a KVStore.
The `MemoryListener` will be used internally by the concrete `rootmulti` implementation to collect state changes from KVStores.

```go
// MemoryListener listens to the state writes and accumulate the records in memory.
type MemoryListener struct {
	stateCache []StoreKVPair
}

// NewMemoryListener creates a listener that accumulate the state writes in memory.
func NewMemoryListener() *MemoryListener {
	return &MemoryListener{}
}

// OnWrite writes state change events to the internal cache
func (fl *MemoryListener) OnWrite(storeKey StoreKey, key []byte, value []byte, delete bool) {
	fl.stateCache = append(fl.stateCache, StoreKVPair{
		StoreKey: storeKey.Name(),
		Delete:   delete,
		Key:      key,
		Value:    value,
	})
}

// PopStateCache returns the current state caches and set to nil
func (fl *MemoryListener) PopStateCache() []StoreKVPair {
	res := fl.stateCache
	fl.stateCache = nil
	return res
}
```

We will also define a protobuf type for the KV pairs. In addition to the key and value fields this message
will include the StoreKey for the originating KVStore so that we can collect information from separate KVStores and determine the source of each KV pair.

```protobuf
message StoreKVPair {
  optional string store_key = 1; // the store key for the KVStore this pair originates from
  required bool set = 2; // true indicates a set operation, false indicates a delete operation
  required bytes key = 3;
  required bytes value = 4;
}
```

### ListenKVStore

We will create a new `Store` type `listenkv.Store` that the `rootmulti` store will use to wrap a `KVStore` to enable state listening.
We will configure the `Store` with a `MemoryListener` which will collect state changes for output to specific destinations.

```go
// Store implements the KVStore interface with listening enabled.
// Operations are traced on each core KVStore call and written to any of the
// underlying listeners with the proper key and operation permissions
type Store struct {
    parent    types.KVStore
    listener  *types.MemoryListener
    parentStoreKey types.StoreKey
}

// NewStore returns a reference to a new traceKVStore given a parent
// KVStore implementation and a buffered writer.
func NewStore(parent types.KVStore, psk types.StoreKey, listener *types.MemoryListener) *Store {
    return &Store{parent: parent, listener: listener, parentStoreKey: psk}
}

// Set implements the KVStore interface. It traces a write operation and
// delegates the Set call to the parent KVStore.
func (s *Store) Set(key []byte, value []byte) {
    types.AssertValidKey(key)
    s.parent.Set(key, value)
    s.listener.OnWrite(s.parentStoreKey, key, value, false)
}

// Delete implements the KVStore interface. It traces a write operation and
// delegates the Delete call to the parent KVStore.
func (s *Store) Delete(key []byte) {
    s.parent.Delete(key)
    s.listener.OnWrite(s.parentStoreKey, key, nil, true)
}
```

### MultiStore interface updates

We will update the `CommitMultiStore` interface to allow us to wrap a `Memorylistener` to a specific `KVStore`.
Note that the `MemoryListener` will be attached internally by the concrete `rootmulti` implementation.

```go
type CommitMultiStore interface {
    ...

    // AddListeners adds a listener for the KVStore belonging to the provided StoreKey
    AddListeners(keys []StoreKey)

    // PopStateCache returns the accumulated state change messages from MemoryListener
    PopStateCache() []StoreKVPair
}
```


### MultiStore implementation updates

We will adjust the `rootmulti` `GetKVStore` method to wrap the returned `KVStore` with a `listenkv.Store` if listening is turned on for that `Store`.

```go
func (rs *Store) GetKVStore(key types.StoreKey) types.KVStore {
    store := rs.stores[key].(types.KVStore)

    if rs.TracingEnabled() {
        store = tracekv.NewStore(store, rs.traceWriter, rs.traceContext)
    }
    if rs.ListeningEnabled(key) {
        store = listenkv.NewStore(store, key, rs.listeners[key])
    }

    return store
}
```

We will implement `AddListeners` to manage KVStore listeners internally and implement `PopStateCache`
for a means of retrieving the current state.

```go
// AddListeners adds state change listener for a specific KVStore
func (rs *Store) AddListeners(keys []types.StoreKey) {
	listener := types.NewMemoryListener()
	for i := range keys {
		rs.listeners[keys[i]] = listener
	}
}
```

```go
func (rs *Store) PopStateCache() []types.StoreKVPair {
	var cache []types.StoreKVPair
	for _, ls := range rs.listeners {
		cache = append(cache, ls.PopStateCache()...)
	}
	sort.SliceStable(cache, func(i, j int) bool {
		return cache[i].StoreKey < cache[j].StoreKey
	})
	return cache
}
```

We will also adjust the `rootmulti` `CacheMultiStore` and `CacheMultiStoreWithVersion` methods to enable listening in
the cache layer.

```go
func (rs *Store) CacheMultiStore() types.CacheMultiStore {
    stores := make(map[types.StoreKey]types.CacheWrapper)
    for k, v := range rs.stores {
        store := v.(types.KVStore)
        // Wire the listenkv.Store to allow listeners to observe the writes from the cache store,
        // set same listeners on cache store will observe duplicated writes.
        if rs.ListeningEnabled(k) {
            store = listenkv.NewStore(store, k, rs.listeners[k])
        }
        stores[k] = store
    }
    return cachemulti.NewStore(rs.db, stores, rs.keysByName, rs.traceWriter, rs.getTracingContext())
}
```

```go
func (rs *Store) CacheMultiStoreWithVersion(version int64) (types.CacheMultiStore, error) {
 // ...

        // Wire the listenkv.Store to allow listeners to observe the writes from the cache store,
        // set same listeners on cache store will observe duplicated writes.
        if rs.ListeningEnabled(key) {
            cacheStore = listenkv.NewStore(cacheStore, key, rs.listeners[key])
        }

        cachedStores[key] = cacheStore
    }

    return cachemulti.NewStore(rs.db, cachedStores, rs.keysByName, rs.traceWriter, rs.getTracingContext()), nil
}
```

### Exposing the data

#### Streaming Service

We will introduce a new `ABCIListener` interface that plugs into the BaseApp and relays ABCI requests and responses
so that the service can group the state changes with the ABCI requests.

```go
// baseapp/streaming.go

// ABCIListener is the interface that we're exposing as a streaming service.
type ABCIListener interface {
	// ListenFinalizeBlock updates the streaming service with the latest FinalizeBlock messages
	ListenFinalizeBlock(ctx context.Context, req abci.RequestFinalizeBlock, res abci.ResponseFinalizeBlock) error
	// ListenCommit updates the steaming service with the latest Commit messages and state changes
	ListenCommit(ctx context.Context, res abci.ResponseCommit, changeSet []*StoreKVPair) error
}
```

#### BaseApp Registration

We will add a new method to the `BaseApp` to enable the registration of `StreamingService`s:

 ```go
 // SetStreamingService is used to set a streaming service into the BaseApp hooks and load the listeners into the multistore
func (app *BaseApp) SetStreamingService(s ABCIListener) {
    // register the StreamingService within the BaseApp
    // BaseApp will pass BeginBlock, DeliverTx, and EndBlock requests and responses to the streaming services to update their ABCI context
    app.abciListeners = append(app.abciListeners, s)
}
```

We will add two new fields to the `BaseApp` struct:

```go
type BaseApp struct {

    ...

    // abciListenersAsync for determining if abciListeners will run asynchronously.
    // When abciListenersAsync=false and stopNodeOnABCIListenerErr=false listeners will run synchronized but will not stop the node.
    // When abciListenersAsync=true stopNodeOnABCIListenerErr will be ignored.
    abciListenersAsync bool

    // stopNodeOnABCIListenerErr halts the node when ABCI streaming service listening results in an error.
    // stopNodeOnABCIListenerErr=true must be paired with abciListenersAsync=false.
    stopNodeOnABCIListenerErr bool
}
```

#### ABCI Event Hooks

We will modify the `FinalizeBlock` and `Commit` methods to pass ABCI requests and responses
to any streaming service hooks registered with the `BaseApp`.

```go
func (app *BaseApp) FinalizeBlock(req abci.RequestFinalizeBlock) abci.ResponseFinalizeBlock {

    var abciRes abci.ResponseFinalizeBlock
    defer func() {
        // call the streaming service hook with the FinalizeBlock messages
        for _, abciListener := range app.abciListeners {
            ctx := app.finalizeState.ctx
            blockHeight := ctx.BlockHeight()
            if app.abciListenersAsync {
                go func(req abci.RequestFinalizeBlock, res abci.ResponseFinalizeBlock) {
                    if err := app.abciListener.FinalizeBlock(blockHeight, req, res); err != nil {
                        app.logger.Error("FinalizeBlock listening hook failed", "height", blockHeight, "err", err)
                    }
                }(req, abciRes)
            } else {
                if err := app.abciListener.ListenFinalizeBlock(blockHeight, req, res); err != nil {
                    app.logger.Error("FinalizeBlock listening hook failed", "height", blockHeight, "err", err)
                    if app.stopNodeOnABCIListenerErr {
                        os.Exit(1)
                    }
                }
            }
        }
    }()

    ...

    return abciRes
}
```

```go
func (app *BaseApp) Commit() abci.ResponseCommit {

    ...

    res := abci.ResponseCommit{
        Data:         commitID.Hash,
        RetainHeight: retainHeight,
    }

    // call the streaming service hook with the Commit messages
    for _, abciListener := range app.abciListeners {
        ctx := app.deliverState.ctx
        blockHeight := ctx.BlockHeight()
        changeSet := app.cms.PopStateCache()
        if app.abciListenersAsync {
            go func(res abci.ResponseCommit, changeSet []store.StoreKVPair) {
                if err := app.abciListener.ListenCommit(ctx, res, changeSet); err != nil {
                    app.logger.Error("ListenCommit listening hook failed", "height", blockHeight, "err", err)
                }
            }(res, changeSet)
        } else {
            if err := app.abciListener.ListenCommit(ctx, res, changeSet); err != nil {
                app.logger.Error("ListenCommit listening hook failed", "height", blockHeight, "err", err)
                if app.stopNodeOnABCIListenerErr {
                    os.Exit(1)
                }
            }
        }
    }

    ...

    return res
}
```

#### Go Plugin System

We propose a plugin architecture to load and run `Streaming` plugins and other types of implementations. We will introduce a plugin
system over gRPC that is used to load and run Cosmos-SDK plugins. The plugin system uses [hashicorp/go-plugin](https://github.com/hashicorp/go-plugin).
Each plugin must have a struct that implements the `plugin.Plugin` interface and an `Impl` interface for processing messages over gRPC.
Each plugin must also have a message protocol defined for the gRPC service:

```go
// streaming/plugins/abci/{plugin_version}/interface.go

// Handshake is a common handshake that is shared by streaming and host.
// This prevents users from executing bad plugins or executing a plugin
// directory. It is a UX feature, not a security feature.
var Handshake = plugin.HandshakeConfig{
    ProtocolVersion:  1,
    MagicCookieKey:   "ABCI_LISTENER_PLUGIN",
    MagicCookieValue: "ef78114d-7bdf-411c-868f-347c99a78345",
}

// ListenerPlugin is the base struct for all kinds of go-plugin implementations
// It will be included in interfaces of different Plugins
type ABCIListenerPlugin struct {
    // GRPCPlugin must still implement the Plugin interface
    plugin.Plugin
    // Concrete implementation, written in Go. This is only used for plugins
    // that are written in Go.
    Impl baseapp.ABCIListener
}

func (p *ListenerGRPCPlugin) GRPCServer(_ *plugin.GRPCBroker, s *grpc.Server) error {
    RegisterABCIListenerServiceServer(s, &GRPCServer{Impl: p.Impl})
    return nil
}

func (p *ListenerGRPCPlugin) GRPCClient(
    _ context.Context,
    _ *plugin.GRPCBroker,
    c *grpc.ClientConn,
) (interface{}, error) {
    return &GRPCClient{client: NewABCIListenerServiceClient(c)}, nil
}
```

The `plugin.Plugin` interface has two methods `Client` and `Server`. For our GRPC service these are `GRPCClient` and `GRPCServer`
The `Impl` field holds the concrete implementation of our `baseapp.ABCIListener` interface written in Go.
Note: this is only used for plugin implementations written in Go.

The advantage of having such a plugin system is that within each plugin authors can define the message protocol in a way that fits their use case.
For example, when state change listening is desired, the `ABCIListener` message protocol can be defined as below (*for illustrative purposes only*).
When state change listening is not desired than `ListenCommit` can be omitted from the protocol.

```protobuf
syntax = "proto3";

...

message Empty {}

message ListenFinalizeBlockRequest {
  RequestFinalizeBlock  req = 1;
  ResponseFinalizeBlock res = 2;
}
message ListenCommitRequest {
  int64                block_height = 1;
  ResponseCommit       res          = 2;
  repeated StoreKVPair changeSet    = 3;
}

// plugin that listens to state changes
service ABCIListenerService {
  rpc ListenFinalizeBlock(ListenFinalizeBlockRequest) returns (Empty);
  rpc ListenCommit(ListenCommitRequest) returns (Empty);
}
```

```protobuf
...
// plugin that doesn't listen to state changes
service ABCIListenerService {
  rpc ListenFinalizeBlock(ListenFinalizeBlockRequest) returns (Empty);
  rpc ListenCommit(ListenCommitRequest) returns (Empty);
}
```

Implementing the service above:

```go
// streaming/plugins/abci/{plugin_version}/grpc.go

var (
    _ baseapp.ABCIListener = (*GRPCClient)(nil)
)

// GRPCClient is an implementation of the ABCIListener and ABCIListenerPlugin interfaces that talks over RPC.
type GRPCClient struct {
    client ABCIListenerServiceClient
}

func (m *GRPCClient) ListenFinalizeBlock(goCtx context.Context, req abci.RequestFinalizeBlock, res abci.ResponseFinalizeBlock) error {
    ctx := sdk.UnwrapSDKContext(goCtx)
    _, err := m.client.ListenDeliverTx(ctx, &ListenDeliverTxRequest{BlockHeight: ctx.BlockHeight(), Req: req, Res: res})
    return err
}

func (m *GRPCClient) ListenCommit(goCtx context.Context, res abci.ResponseCommit, changeSet []store.StoreKVPair) error {
    ctx := sdk.UnwrapSDKContext(goCtx)
    _, err := m.client.ListenCommit(ctx, &ListenCommitRequest{BlockHeight: ctx.BlockHeight(), Res: res, ChangeSet: changeSet})
    return err
}

// GRPCServer is the gRPC server that GRPCClient talks to.
type GRPCServer struct {
    // This is the real implementation
    Impl baseapp.ABCIListener
}

func (m *GRPCServer) ListenFinalizeBlock(ctx context.Context, req *ListenFinalizeBlockRequest) (*Empty, error) {
    return &Empty{}, m.Impl.ListenFinalizeBlock(ctx, req.Req, req.Res)
}

func (m *GRPCServer) ListenCommit(ctx context.Context, req *ListenCommitRequest) (*Empty, error) {
    return &Empty{}, m.Impl.ListenCommit(ctx, req.Res, req.ChangeSet)
}

```

And the pre-compiled Go plugin `Impl`(*this is only used for plugins that are written in Go*):

```go
// streaming/plugins/abci/{plugin_version}/impl/plugin.go

// Plugins are pre-compiled and loaded by the plugin system

// ABCIListener is the implementation of the baseapp.ABCIListener interface
type ABCIListener struct{}

func (m *ABCIListenerPlugin) ListenFinalizeBlock(ctx context.Context, req abci.RequestFinalizeBlock, res abci.ResponseFinalizeBlock) error {
    // send data to external system
}

func (m *ABCIListenerPlugin) ListenCommit(ctx context.Context, res abci.ResponseCommit, changeSet []store.StoreKVPair) error {
    // send data to external system
}

func main() {
    plugin.Serve(&plugin.ServeConfig{
        HandshakeConfig: grpc_abci_v1.Handshake,
        Plugins: map[string]plugin.Plugin{
           "grpc_plugin_v1": &grpc_abci_v1.ABCIListenerGRPCPlugin{Impl: &ABCIListenerPlugin{}},
        },

        // A non-nil value here enables gRPC serving for this streaming...
        GRPCServer: plugin.DefaultGRPCServer,
    })
}
```

We will introduce a plugin loading system that will return `(interface{}, error)`.
This provides the advantage of using versioned plugins where the plugin interface and gRPC protocol change over time.
In addition, it allows for building independent plugin that can expose different parts of the system over gRPC.

```go
func NewStreamingPlugin(name string, logLevel string) (interface{}, error) {
    logger := hclog.New(&hclog.LoggerOptions{
       Output: hclog.DefaultOutput,
       Level:  toHclogLevel(logLevel),
       Name:   fmt.Sprintf("plugin.%s", name),
    })

    // We're a host. Start by launching the streaming process.
    env := os.Getenv(GetPluginEnvKey(name))
    client := plugin.NewClient(&plugin.ClientConfig{
       HandshakeConfig: HandshakeMap[name],
       Plugins:         PluginMap,
       Cmd:             exec.Command("sh", "-c", env),
       Logger:          logger,
       AllowedProtocols: []plugin.Protocol{
           plugin.ProtocolNetRPC, plugin.ProtocolGRPC},
    })

    // Connect via RPC
    rpcClient, err := client.Client()
    if err != nil {
       return nil, err
    }

    // Request streaming plugin
    return rpcClient.Dispense(name)
}

```

We propose a `RegisterStreamingPlugin` function for the App to register `NewStreamingPlugin`s with the App's BaseApp.
Streaming plugins can be of `Any` type; therefore, the function takes in an interface vs a concrete type.
For example, we could have plugins of `ABCIListener`, `WasmListener` or `IBCListener`. Note that `RegisterStreamingPluing` function
is helper function and not a requirement. Plugin registration can easily be moved from the App to the BaseApp directly.

```go
// baseapp/streaming.go

// RegisterStreamingPlugin registers streaming plugins with the App.
// This method returns an error if a plugin is not supported.
func RegisterStreamingPlugin(
    bApp *BaseApp,
    appOpts servertypes.AppOptions,
    keys map[string]*types.KVStoreKey,
    streamingPlugin interface{},
) error {
    switch t := streamingPlugin.(type) {
    case ABCIListener:
        registerABCIListenerPlugin(bApp, appOpts, keys, t)
    default:
        return fmt.Errorf("unexpected plugin type %T", t)
    }
    return nil
}
```

```go
func registerABCIListenerPlugin(
    bApp *BaseApp,
    appOpts servertypes.AppOptions,
    keys map[string]*store.KVStoreKey,
    abciListener ABCIListener,
) {
    asyncKey := fmt.Sprintf("%s.%s.%s", StreamingTomlKey, StreamingABCITomlKey, StreamingABCIAsync)
    async := cast.ToBool(appOpts.Get(asyncKey))
    stopNodeOnErrKey := fmt.Sprintf("%s.%s.%s", StreamingTomlKey, StreamingABCITomlKey, StreamingABCIStopNodeOnErrTomlKey)
    stopNodeOnErr := cast.ToBool(appOpts.Get(stopNodeOnErrKey))
    keysKey := fmt.Sprintf("%s.%s.%s", StreamingTomlKey, StreamingABCITomlKey, StreamingABCIKeysTomlKey)
    exposeKeysStr := cast.ToStringSlice(appOpts.Get(keysKey))
    exposedKeys := exposeStoreKeysSorted(exposeKeysStr, keys)
    bApp.cms.AddListeners(exposedKeys)
    app.SetStreamingManager(
		storetypes.StreamingManager{
			ABCIListeners: []storetypes.ABCIListener{abciListener},
			StopNodeOnErr: stopNodeOnErr,
		},
	)
}
```

```go
func exposeAll(list []string) bool {
    for _, ele := range list {
        if ele == "*" {
            return true
        }
    }
    return false
}

func exposeStoreKeys(keysStr []string, keys map[string]*types.KVStoreKey) []types.StoreKey {
    var exposeStoreKeys []types.StoreKey
    if exposeAll(keysStr) {
        exposeStoreKeys = make([]types.StoreKey, 0, len(keys))
        for _, storeKey := range keys {
            exposeStoreKeys = append(exposeStoreKeys, storeKey)
        }
    } else {
        exposeStoreKeys = make([]types.StoreKey, 0, len(keysStr))
        for _, keyStr := range keysStr {
            if storeKey, ok := keys[keyStr]; ok {
                exposeStoreKeys = append(exposeStoreKeys, storeKey)
            }
        }
    }
    // sort storeKeys for deterministic output
    sort.SliceStable(exposeStoreKeys, func(i, j int) bool {
        return exposeStoreKeys[i].Name() < exposeStoreKeys[j].Name()
    })

    return exposeStoreKeys
}
```

The `NewStreamingPlugin` and `RegisterStreamingPlugin` functions are used to register a plugin with the App's BaseApp.

e.g. in `NewSimApp`:

```go
func NewSimApp(
    logger log.Logger,
    db corestore.KVStoreWithBatch,
    traceStore io.Writer,
    loadLatest bool,
    appOpts servertypes.AppOptions,
    baseAppOptions ...func(*baseapp.BaseApp),
) *SimApp {

    ...

    keys := sdk.NewKVStoreKeys(
       authtypes.StoreKey, banktypes.StoreKey, stakingtypes.StoreKey,
       minttypes.StoreKey, distrtypes.StoreKey, slashingtypes.StoreKey,
       govtypes.StoreKey, paramstypes.StoreKey, ibchost.StoreKey, upgradetypes.StoreKey,
       evidencetypes.StoreKey, ibctransfertypes.StoreKey, capabilitytypes.StoreKey,
    )

    ...

    // register streaming services
    streamingCfg := cast.ToStringMap(appOpts.Get(baseapp.StreamingTomlKey))
    for service := range streamingCfg {
        pluginKey := fmt.Sprintf("%s.%s.%s", baseapp.StreamingTomlKey, service, baseapp.StreamingPluginTomlKey)
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

    return app
```

#### Configuration

The plugin system will be configured within an App's TOML configuration files.

```toml
# gRPC streaming
[streaming]

# ABCI streaming service
[streaming.abci]

# The plugin version to use for ABCI listening
plugin = "abci_v1"

# List of kv store keys to listen to for state changes.
# Set to ["*"] to expose all keys.
keys = ["*"]

# Enable abciListeners to run asynchronously.
# When abciListenersAsync=false and stopNodeOnABCIListenerErr=false listeners will run synchronized but will not stop the node.
# When abciListenersAsync=true stopNodeOnABCIListenerErr will be ignored.
async = false

# Whether to stop the node on message deliver error.
stop-node-on-err = true
```

There will be four parameters for configuring `ABCIListener` plugin: `streaming.abci.plugin`, `streaming.abci.keys`, `streaming.abci.async` and `streaming.abci.stop-node-on-err`.
`streaming.abci.plugin` is the name of the plugin we want to use for streaming, `streaming.abci.keys` is a set of store keys for stores it listens to,
`streaming.abci.async` is bool enabling asynchronous listening and `streaming.abci.stop-node-on-err` is a bool that stops the node when true and when operating
on synchronized mode `streaming.abci.async=false`. Note that `streaming.abci.stop-node-on-err=true` will be ignored if `streaming.abci.async=true`.

The configuration above support additional streaming plugins by adding the plugin to the `[streaming]` configuration section
and registering the plugin with `RegisterStreamingPlugin` helper function.

Note the that each plugin must include `streaming.{service}.plugin` property as it is a requirement for doing the lookup and registration of the plugin
with the App. All other properties are unique to the individual services.

#### Encoding and decoding streams

ADR-038 introduces the interfaces and types for streaming state changes out from KVStores, associating this
data with their related ABCI requests and responses, and registering a service for consuming this data and streaming it to some destination in a final format.
Instead of prescribing a final data format in this ADR, it is left to a specific plugin implementation to define and document this format.
We take this approach because flexibility in the final format is necessary to support a wide range of streaming service plugins. For example,
the data format for a streaming service that writes the data out to a set of files will differ from the data format that is written to a Kafka topic.

## Consequences

These changes will provide a means of subscribing to KVStore state changes in real time.

### Backwards Compatibility

* This ADR changes the `CommitMultiStore` interface, implementations supporting the previous version of this interface will not support the new one

### Positive

* Ability to listen to KVStore state changes in real time and expose these events to external consumers

### Negative

* Changes `CommitMultiStore` interface and its implementations

### Neutral

* Introduces additional- but optional- complexity to configuring and running a cosmos application
* If an application developer opts to use these features to expose data, they need to be aware of the ramifications/risks of that data exposure as it pertains to the specifics of their application
