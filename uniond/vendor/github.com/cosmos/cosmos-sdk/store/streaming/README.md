# State Streaming Service

This package contains the constructors for the `StreamingService`s used to write
state changes out from individual KVStores to a file or stream, as described in
[ADR-038](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-038-state-listening.md)
and defined in [types/streaming.go](https://github.com/cosmos/cosmos-sdk/blob/main/baseapp/streaming.go).
The child directories contain the implementations for specific output destinations.

Currently, a `StreamingService` implementation that writes state changes out to
files is supported, in the future support for additional output destinations can
be added.

The `StreamingService` is configured from within an App using the `AppOptions`
loaded from the `app.toml` file:

```toml
# ...

[store]
# streaming is enabled if one or more streamers are defined
streamers = [
    # name of the streaming service, used by constructor
    "file"
]

[streamers]
[streamers.file]
    keys = ["list", "of", "store", "keys", "we", "want", "to", "expose", "for", "this", "streaming", "service"]
    write_dir = "path to the write directory"
    prefix = "optional prefix to prepend to the generated file names"
```

The `store.streamers` field contains a list of the names of the `StreamingService`
implementations to employ which are used by `ServiceTypeFromString` to return
the `ServiceConstructor` for that particular implementation:

```go
listeners := cast.ToStringSlice(appOpts.Get("store.streamers"))
for _, listenerName := range listeners {
    constructor, err := ServiceTypeFromString(listenerName)
    if err != nil {
    	// handle error
    }
}
```

The `streamers` field contains a mapping of the specific `StreamingService`
implementation name to the configuration parameters for that specific service.

The `streamers.x.keys` field contains the list of `StoreKey` names for the
KVStores to expose using this service and is required by every type of
`StreamingService`. In order to expose *ALL* KVStores, we can include `*` in
this list. An empty list is equivalent to turning the service off.

Additional configuration parameters are optional and specific to the implementation.
In the case of the file streaming service, the `streamers.file.write_dir` field
contains the path to the directory to write the files to, and `streamers.file.prefix`
contains an optional prefix to prepend to the output files to prevent potential
collisions with other App `StreamingService` output files.

The `ServiceConstructor` accepts `AppOptions`, the store keys collected using
`streamers.x.keys`, a `BinaryMarshaller` and returns a `StreamingService
implementation.

The `AppOptions` are passed in to provide access to any implementation specific
configuration options, e.g. in the case of the file streaming service the
`streamers.file.write_dir` and `streamers.file.prefix`.

```go
streamingService, err := constructor(appOpts, exposeStoreKeys, appCodec)
if err != nil {
    // handler error
}
```

The returned `StreamingService` is loaded into the BaseApp using the BaseApp's
`SetStreamingService` method.

The `Stream` method is called on the service to begin the streaming process.
Depending on the implementation this process may be synchronous or asynchronous
with the message processing of the state machine.

```go
bApp.SetStreamingService(streamingService)
wg := new(sync.WaitGroup)
quitChan := make(chan struct{})
streamingService.Stream(wg, quitChan)
```
