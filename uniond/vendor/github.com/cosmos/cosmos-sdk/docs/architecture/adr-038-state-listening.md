# ADR 038: KVStore state listening

## Changelog

* 11/23/2020: Initial draft
* 10/14/2022:
  * Add `ListenCommit`, flatten the state writes in a block to a single batch.
  * Remove listeners from cache stores, should only listen to `rootmulti.Store`.
  * Remove `HaltAppOnDeliveryError()`, the errors are propogated by default, the implementations should return nil if don't want to propogate errors.


## Status

Proposed

## Abstract

This ADR defines a set of changes to enable listening to state changes of individual KVStores and exposing these data to consumers.

## Context

Currently, KVStore data can be remotely accessed through [Queries](https://github.com/cosmos/cosmos-sdk/blob/master/docs/building-modules/messages-and-queries.md#queries)
which proceed either through Tendermint and the ABCI, or through the gRPC server.
In addition to these request/response queries, it would be beneficial to have a means of listening to state changes as they occur in real time.

## Decision

We will modify the `CommitMultiStore` interface and its concrete (`rootmulti`) implementations and introduce a new `listenkv.Store` to allow listening to state changes in underlying KVStores. We don't need to listen to cache stores, because we can't be sure that the writes will be committed eventually, and the writes are duplicated in `rootmulti.Store` eventually, so we should only listen to `rootmulti.Store`.
We will introduce a plugin system for configuring and running streaming services that write these state changes and their surrounding ABCI message context to different destinations.

### Listening interface

In a new file, `store/types/listening.go`, we will create a `WriteListener` interface for streaming out state changes from a KVStore.

```go
// WriteListener interface for streaming data out from a listenkv.Store
type WriteListener interface {
	// if value is nil then it was deleted
	// storeKey indicates the source KVStore, to facilitate using the same WriteListener across separate KVStores
	// delete bool indicates if it was a delete; true: delete, false: set
	OnWrite(storeKey StoreKey, key []byte, value []byte, delete bool) error
}
```

### Listener type

We will create two concrete implementations of the `WriteListener` interface in `store/types/listening.go`, that writes out protobuf
encoded KV pairs to an underlying `io.Writer`, and simply accumulate them in memory.

This will include defining a simple protobuf type for the KV pairs. In addition to the key and value fields this message
will include the StoreKey for the originating KVStore so that we can write out from separate KVStores to the same stream/file
and determine the source of each KV pair.

```protobuf
message StoreKVPair {
  optional string store_key = 1; // the store key for the KVStore this pair originates from
  required bool set = 2; // true indicates a set operation, false indicates a delete operation
  required bytes key = 3;
  required bytes value = 4;
}
```

```go
// StoreKVPairWriteListener is used to configure listening to a KVStore by writing out length-prefixed
// protobuf encoded StoreKVPairs to an underlying io.Writer
type StoreKVPairWriteListener struct {
	writer io.Writer
	marshaller codec.BinaryCodec
}

// NewStoreKVPairWriteListener wraps creates a StoreKVPairWriteListener with a provdied io.Writer and codec.BinaryCodec
func NewStoreKVPairWriteListener(w io.Writer, m codec.BinaryCodec) *StoreKVPairWriteListener {
	return &StoreKVPairWriteListener{
		writer: w,
		marshaller: m,
	}
}

// OnWrite satisfies the WriteListener interface by writing length-prefixed protobuf encoded StoreKVPairs
func (wl *StoreKVPairWriteListener) OnWrite(storeKey types.StoreKey, key []byte, value []byte, delete bool) error error {
    kvPair := new(types.StoreKVPair)
    kvPair.StoreKey = storeKey.Name()
    kvPair.Delete = Delete
    kvPair.Key = key
    kvPair.Value = value
    by, err := wl.marshaller.MarshalBinaryLengthPrefixed(kvPair)
    if err != nil {
        return err
    }
    if _, err := wl.writer.Write(by); err != nil {
        return err
    }
    return nil
}
```

```golang
// MemoryListener listens to the state writes and accumulate the records in memory.
type MemoryListener struct {
	key        StoreKey
	stateCache []StoreKVPair
}

// NewMemoryListener creates a listener that accumulate the state writes in memory.
func NewMemoryListener(key StoreKey) *MemoryListener {
	return &MemoryListener{key: key}
}

// OnWrite implements WriteListener interface
func (fl *MemoryListener) OnWrite(storeKey StoreKey, key []byte, value []byte, delete bool) error {
	fl.stateCache = append(fl.stateCache, StoreKVPair{
		StoreKey: storeKey.Name(),
		Delete:   delete,
		Key:      key,
		Value:    value,
	})
	return nil
}

// PopStateCache returns the current state caches and set to nil
func (fl *MemoryListener) PopStateCache() []StoreKVPair {
	res := fl.stateCache
	fl.stateCache = nil
	return res
}

// StoreKey returns the storeKey it listens to
func (fl *MemoryListener) StoreKey() StoreKey {
	return fl.key
}
```

### ListenKVStore

We will create a new `Store` type `listenkv.Store` that the `MultiStore` wraps around a `KVStore` to enable state listening.
We can configure the `Store` with a set of `WriteListener`s which stream the output to specific destinations.

```go
// Store implements the KVStore interface with listening enabled.
// Operations are traced on each core KVStore call and written to any of the
// underlying listeners with the proper key and operation permissions
type Store struct {
    parent    types.KVStore
    listeners []types.WriteListener
    parentStoreKey types.StoreKey
}

// NewStore returns a reference to a new traceKVStore given a parent
// KVStore implementation and a buffered writer.
func NewStore(parent types.KVStore, psk types.StoreKey, listeners []types.WriteListener) *Store {
    return &Store{parent: parent, listeners: listeners, parentStoreKey: psk}
}

// Set implements the KVStore interface. It traces a write operation and
// delegates the Set call to the parent KVStore.
func (s *Store) Set(key []byte, value []byte) {
    types.AssertValidKey(key)
    s.parent.Set(key, value)
    s.onWrite(false, key, value)
}

// Delete implements the KVStore interface. It traces a write operation and
// delegates the Delete call to the parent KVStore.
func (s *Store) Delete(key []byte) {
    s.parent.Delete(key)
    s.onWrite(true, key, nil)
}

// onWrite writes a KVStore operation to all the WriteListeners
func (s *Store) onWrite(delete bool, key, value []byte) {
    for _, l := range s.listeners {
        if err := l.OnWrite(s.parentStoreKey, key, value, delete); err != nil {
            // log error
        }
    }
}
```

### MultiStore interface updates

We will update the `CommitMultiStore` interface to allow us to wrap a set of listeners around a specific `KVStore`.

```go
type CommitMultiStore interface {
    ...

    // ListeningEnabled returns if listening is enabled for the KVStore belonging the provided StoreKey
    ListeningEnabled(key StoreKey) bool

    // AddListeners adds WriteListeners for the KVStore belonging to the provided StoreKey
    // It appends the listeners to a current set, if one already exists
    AddListeners(key StoreKey, listeners []WriteListener)
}
```

### MultiStore implementation updates

We will modify all of the `CommitMultiStore` implementations to satisfy these new interfaces, and adjust the `rootmulti` `GetKVStore` method
to wrap the returned `KVStore` with a `listenkv.Store` if listening is turned on for that `Store`.

```go
func (rs *Store) GetKVStore(key types.StoreKey) types.KVStore {
    store := rs.stores[key].(types.KVStore)

    if rs.TracingEnabled() {
        store = tracekv.NewStore(store, rs.traceWriter, rs.traceContext)
    }
    if rs.ListeningEnabled(key) {
        store = listenkv.NewStore(key, store, rs.listeners[key])
    }

    return store
}
```

We will also adjust the `rootmulti` `CacheMultiStore` method to wrap the stores with `listenkv.Store` to enable listening when the cache layer writes.

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

### Exposing the data

#### Streaming service

We will introduce a new `StreamingService` interface for exposing `WriteListener` data streams to external consumers.
In addition to streaming state changes as `StoreKVPair`s, the interface satisfies an `ABCIListener` interface that plugs
into the BaseApp and relays ABCI requests and responses so that the service can observe those block metadatas as well.

The `WriteListener`s of `StreamingService` listens to the `rootmulti.Store`, which is only written into at commit event by the cache store of `deliverState`.

```go
// ABCIListener interface used to hook into the ABCI message processing of the BaseApp
type ABCIListener interface {
    // ListenBeginBlock updates the streaming service with the latest BeginBlock messages
    ListenBeginBlock(ctx types.Context, req abci.RequestBeginBlock, res abci.ResponseBeginBlock) error
    // ListenEndBlock updates the steaming service with the latest EndBlock messages
    ListenEndBlock(ctx types.Context, req abci.RequestEndBlock, res abci.ResponseEndBlock) error
    // ListenDeliverTx updates the steaming service with the latest DeliverTx messages
    ListenDeliverTx(ctx types.Context, req abci.RequestDeliverTx, res abci.ResponseDeliverTx) error
    // ListenCommit updates the steaming service with the latest Commit message,
    // All the state writes of current block should have notified before this message.
    ListenCommit(ctx types.Context, res abci.ResponseCommit) error
}

// StreamingService interface for registering WriteListeners with the BaseApp and updating the service with the ABCI messages using the hooks
type StreamingService interface {
    // Stream is the streaming service loop, awaits kv pairs and writes them to a destination stream or file
    Stream(wg *sync.WaitGroup) error
    // Listeners returns the streaming service's listeners for the BaseApp to register
    Listeners() map[types.StoreKey][]store.WriteListener
    // ABCIListener interface for hooking into the ABCI messages from inside the BaseApp
    ABCIListener
    // Closer interface
    io.Closer
}
```

#### BaseApp registration

We will add a new method to the `BaseApp` to enable the registration of `StreamingService`s:

```go
// SetStreamingService is used to set a streaming service into the BaseApp hooks and load the listeners into the multistore
func (app *BaseApp) SetStreamingService(s StreamingService) {
	// add the listeners for each StoreKey
	for key, lis := range s.Listeners() {
		app.cms.AddListeners(key, lis)
	}
	// register the StreamingService within the BaseApp
	// BaseApp will pass BeginBlock, DeliverTx, and EndBlock requests and responses to the streaming services to update their ABCI context
	app.abciListeners = append(app.abciListeners, s)
}
```

We will also modify the `BeginBlock`, `EndBlock`, and `DeliverTx` methods to pass ABCI requests and responses to any streaming service hooks registered
with the `BaseApp`.

```go
func (app *BaseApp) BeginBlock(req abci.RequestBeginBlock) (res abci.ResponseBeginBlock) {

	...

	defer func() {
		// call the hooks with the BeginBlock messages
		for _, streamingListener := range app.abciListeners {
			if err := streamingListener.ListenBeginBlock(app.deliverState.ctx, req, res); err != nil {
				panic(sdkerrors.Wrapf(err, "BeginBlock listening hook failed, height: %d", req.Header.Height))
			}
		}
	}()

	return res
}
```

```go
func (app *BaseApp) EndBlock(req abci.RequestEndBlock) (res abci.ResponseEndBlock) {

	...

  defer func() {
		// Call the streaming service hooks with the EndBlock messages
		for _, streamingListener := range app.abciListeners {
			if err := streamingListener.ListenEndBlock(app.deliverState.ctx, req, res); err != nil {
				panic(sdkerrors.Wrapf(err, "EndBlock listening hook failed, height: %d", req.Height))
			}
		}
  }()

	return res
}
```

```go
func (app *BaseApp) DeliverTx(req abci.RequestDeliverTx) (res abci.ResponseDeliverTx) {

	defer func() {
		// call the hooks with the DeliverTx messages
		for _, streamingListener := range app.abciListeners {
			if err := streamingListener.ListenDeliverTx(app.deliverState.ctx, req, res); err != nil {
				panic(sdkerrors.Wrap(err, "DeliverTx listening hook failed"))
			}
		}
	}()

	...

	return res
}
```

```golang
func (app *BaseApp) Commit() abci.ResponseCommit {
	header := app.deliverState.ctx.BlockHeader()
	retainHeight := app.GetBlockRetentionHeight(header.Height)

	// Write the DeliverTx state into branched storage and commit the MultiStore.
	// The write to the DeliverTx state writes all state transitions to the root
	// MultiStore (app.cms) so when Commit() is called is persists those values.
	app.deliverState.ms.Write()
	commitID := app.cms.Commit()

	res := abci.ResponseCommit{
		Data:         commitID.Hash,
		RetainHeight: retainHeight,
	}

	// call the hooks with the Commit message
	for _, streamingListener := range app.abciListeners {
		if err := streamingListener.ListenCommit(app.deliverState.ctx, res); err != nil {
			panic(sdkerrors.Wrapf(err, "Commit listening hook failed, height: %d", header.Height))
		}
	}

	app.logger.Info("commit synced", "commit", fmt.Sprintf("%X", commitID))
  ...
}
```

#### Error Handling And Async Consumers

`ABCIListener`s are called synchronously inside the consensus state machine, the returned error causes panic which in turn halt the consensus state machine. The implementer should be careful not to break consensus unexpectedly or slow down it too much.

For some async use cases, one can spawn a go-routine internanlly to avoid slow down consensus state machine, and handle the errors internally and always returns `nil` to avoid halting consensus state machine on error.

Furthermore, for most of the cases, we only need to use the builtin file streamer to listen to state changes directly inside cosmos-sdk, the other consumers should subscribe to the file streamer output externally.

#### File Streamer

We provide a minimal filesystem based implementation inside cosmos-sdk, and provides options to write output files reliably, the output files can be further consumed by external consumers, so most of the state listeners actually don't need to live inside the sdk and node, which improves the node robustness and simplify sdk internals.

The file streamer can be wired in app like this:
```golang
exposeStoreKeys := ... // decide the key list to listen
service, err := file.NewStreamingService(streamingDir, "", exposeStoreKeys, appCodec, logger)
bApp.SetStreamingService(service)
```

#### Plugin system

We propose a plugin architecture to load and run `StreamingService` implementations. We will introduce a plugin
loading/preloading system that is used to load, initialize, inject, run, and stop Cosmos-SDK plugins. Each plugin
must implement the following interface:

```go
// Plugin is the base interface for all kinds of cosmos-sdk plugins
// It will be included in interfaces of different Plugins
type Plugin interface {
	// Name should return unique name of the plugin
	Name() string

	// Version returns current version of the plugin
	Version() string

	// Init is called once when the Plugin is being loaded
	// The plugin is passed the AppOptions for configuration
	// A plugin will not necessarily have a functional Init
	Init(env serverTypes.AppOptions) error

	// Closer interface for shutting down the plugin process
	io.Closer
}
```

The `Name` method returns a plugin's name.
The `Version` method returns a plugin's version.
The `Init` method initializes a plugin with the provided `AppOptions`.
The io.Closer is used to shut down the plugin service.

For the purposes of this ADR we introduce a single kind of plugin- a state streaming plugin.
We will define a `StateStreamingPlugin` interface which extends the above `Plugin` interface to support a state streaming service.

```go
// StateStreamingPlugin interface for plugins that load a baseapp.StreamingService onto a baseapp.BaseApp
type StateStreamingPlugin interface {
	// Register configures and registers the plugin streaming service with the BaseApp
	Register(bApp *baseapp.BaseApp, marshaller codec.BinaryCodec, keys map[string]*types.KVStoreKey) error

	// Start starts the background streaming process of the plugin streaming service
	Start(wg *sync.WaitGroup) error

	// Plugin is the base Plugin interface
	Plugin
}
```

The `Register` method is used during App construction to register the plugin's streaming service with an App's BaseApp using the BaseApp's `SetStreamingService` method.
The `Start` method is used during App construction to start the registered plugin streaming services and maintain synchronization with them.

e.g. in `NewSimApp`:

```go
func NewSimApp(
	logger log.Logger,
	db dbm.DB,
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

	pluginsOnKey := fmt.Sprintf("%s.%s", plugin.PLUGINS_TOML_KEY, plugin.PLUGINS_ON_TOML_KEY)
	if cast.ToBool(appOpts.Get(pluginsOnKey)) {
		// this loads the preloaded and any plugins found in `plugins.dir`
		pluginLoader, err := loader.NewPluginLoader(appOpts, logger)
		if err != nil {
			// handle error
		}

		// initialize the loaded plugins
		if err := pluginLoader.Initialize(); err != nil {
			// handle error
		}

		// register the plugin(s) with the BaseApp
		if err := pluginLoader.Inject(bApp, appCodec, keys); err != nil {
			// handle error
		}

		// start the plugin services, optionally use wg to synchronize shutdown using io.Closer
		wg := new(sync.WaitGroup)
		if err := pluginLoader.Start(wg); err != nil {
			// handler error
		}
	}

	...

	return app
}
```


#### Configuration

The plugin system will be configured within an app's app.toml file.

```toml
[plugins]
    on = false # turn the plugin system, as a whole, on or off
    enabled = ["list", "of", "plugin", "names", "to", "enable"]
    dir = "the directory to load non-preloaded plugins from; defaults to cosmos-sdk/plugin/plugins"
```

There will be three parameters for configuring the plugin system: `plugins.on`, `plugins.enabled` and `plugins.dir`.
`plugins.on` is a bool that turns on or off the plugin system at large, `plugins.dir` directs the system to a directory
to load plugins from, and `plugins.enabled` provides `opt-in` semantics to plugin names to enable (including preloaded plugins).

Configuration of a given plugin is ultimately specific to the plugin, but we will introduce some standards here:

Plugin TOML configuration should be split into separate sub-tables for each kind of plugin (e.g. `plugins.streaming`).

Within these sub-tables, the parameters for a specific plugin of that kind are included in another sub-table (e.g. `plugins.streaming.file`).
It is generally expected, but not required, that a streaming service plugin can be configured with a set of store keys
(e.g. `plugins.streaming.file.keys`) for the stores it listens to and a flag (e.g. `plugins.streaming.file.halt_app_on_delivery_error`)
that signifies whether the service operates in a fire-and-forget capacity, or stop the BaseApp when an error occurs in
any of `ListenBeginBlock`, `ListenEndBlock` and `ListenDeliverTx`.

e.g.

```toml
[plugins]
    on = false # turn the plugin system, as a whole, on or off
    enabled = ["list", "of", "plugin", "names", "to", "enable"]
    dir = "the directory to load non-preloaded plugins from; defaults to "
    [plugins.streaming] # a mapping of plugin-specific streaming service parameters, mapped to their plugin name
        [plugins.streaming.file] # the specific parameters for the file streaming service plugin
            keys = ["list", "of", "store", "keys", "we", "want", "to", "expose", "for", "this", "streaming", "service"]
            write_dir = "path to the write directory"
            prefix = "optional prefix to prepend to the generated file names"
            halt_app_on_delivery_error = "false" # false == fire-and-forget; true == stop the application
        [plugins.streaming.kafka]
            keys = []
            topic_prefix = "block" # Optional prefix for topic names where data will be stored.
            flush_timeout_ms = 5000 # Flush and wait for outstanding messages and requests to complete delivery when calling `StreamingService.Close(). (milliseconds)
            halt_app_on_delivery_error = true # Whether or not to halt the application when plugin fails to deliver message(s).
        ...
```

#### Encoding and decoding streams

ADR-038 introduces the interfaces and types for streaming state changes out from KVStores, associating this
data with their related ABCI requests and responses, and registering a service for consuming this data and streaming it to some destination in a final format.
Instead of prescribing a final data format in this ADR, it is left to a specific plugin implementation to define and document this format.
We take this approach because flexibility in the final format is necessary to support a wide range of streaming service plugins. For example,
the data format for a streaming service that writes the data out to a set of files will differ from the data format that is written to a Kafka topic.

## Consequences

These changes will provide a means of subscribing to KVStore state changes in real time.

### Backwards Compatibility

* This ADR changes the `CommitMultiStore` interface, implementations supporting the previous version of these interfaces will not support the new ones

### Positive

* Ability to listen to KVStore state changes in real time and expose these events to external consumers

### Negative

* Changes `CommitMultiStore`interface

### Neutral

* Introduces additional- but optional- complexity to configuring and running a cosmos application
* If an application developer opts to use these features to expose data, they need to be aware of the ramifications/risks of that data exposure as it pertains to the specifics of their application
