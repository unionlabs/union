# Store

The store package defines the interfaces, types and abstractions for Cosmos SDK
modules to read and write to Merkleized state within a Cosmos SDK application.
The store package provides many primitives for developers to use in order to
work with both state storage and state commitment. Below we describe the various
abstractions.

## Types

### `Store`

The bulk of the store interfaces are defined [here](https://github.com/cosmos/cosmos-sdk/blob/main/store/store.go),
where the base primitive interface, for which other interfaces build off of, is
the `Store` type. The `Store` interface defines the ability to tell the type of
the implementing store and the ability to cache wrap via the `CacheWrapper` interface.

### `CacheWrapper` & `CacheWrap`

One of the most important features a store has the ability to perform is the
ability to cache wrap. Cache wrapping is essentially the underlying store wrapping
itself within another store type that performs caching for both reads and writes
with the ability to flush writes via `Write()`.

### `KVStore` & `CacheKVStore`

One of the most important interfaces that both developers and modules interface
with, which also provides the basis of most state storage and commitment operations,
is the `KVStore`. The `KVStore` interface provides basic CRUD abilities and
prefix-based iteration, including reverse iteration.

Typically, each module has its own dedicated `KVStore` instance, which it can
get access to via the `sdk.Context` and the use of a pointer-based named key --
`KVStoreKey`. The `KVStoreKey` provides pseudo-OCAP. How a exactly a `KVStoreKey`
maps to a `KVStore` will be illustrated below through the `CommitMultiStore`.

Note, a `KVStore` cannot directly commit state. Instead, a `KVStore` can be wrapped
by a `CacheKVStore` which extends a `KVStore` and provides the ability for the
caller to execute `Write()` which commits state to the underlying state storage.
Note, this doesn't actually flush writes to disk as writes are held in memory
until `Commit()` is called on the `CommitMultiStore`.

### `CommitMultiStore`

The `CommitMultiStore` interface exposes the top-level interface that is used
to manage state commitment and storage by an SDK application and abstracts the
concept of multiple `KVStore`s which are used by multiple modules. Specifically,
it supports the following high-level primitives:

* Allows for a caller to retrieve a `KVStore` by providing a `KVStoreKey`.
* Exposes pruning mechanisms to remove state pinned against a specific height/version
  in the past.
* Allows for loading state storage at a particular height/version in the past to
  provide current head and historical queries.
* Provides the ability to rollback state to a previous height/version.
* Provides the ability to load state storage at a particular height/version
  while also performing store upgrades, which are used during live hard-fork
  application state migrations.
* Provides the ability to commit all current accumulated state to disk and performs
  Merkle commitment.

## Implementation Details

While there are many interfaces that the `store` package provides, there is
typically a core implementation for each main interface that modules and
developers interact with that are defined in the Cosmos SDK.

### `iavl.Store`

The `iavl.Store` provides the core implementation for state storage and commitment
by implementing the following interfaces:

* `KVStore`
* `CommitStore`
* `CommitKVStore`
* `Queryable`
* `StoreWithInitialVersion`

It allows for all CRUD operations to be performed along with allowing current
and historical state queries, prefix iteration, and state commitment along with
Merkle proof operations. The `iavl.Store` also provides the ability to remove
historical state from the state commitment layer.

An overview of the IAVL implementation can be found [here](https://github.com/cosmos/iavl/blob/master/docs/overview.md).
It is important to note that the IAVL store provides both state commitment and
logical storage operations, which comes with drawbacks as there are various
performance impacts, some of which are very drastic, when it comes to the
operations mentioned above.

When dealing with state management in modules and clients, the Cosmos SDK provides
various layers of abstractions or "store wrapping", where the `iavl.Store` is the
bottom most layer. When requesting a store to perform reads or writes in a module,
the typical abstraction layer in order is defined as follows:

```text
iavl.Store <- cachekv.Store <- gaskv.Store <- cachemulti.Store <- rootmulti.Store
```

### Concurrent use of IAVL store

The tree under `iavl.Store` is not safe for concurrent use. It is the
responsibility of the caller to ensure that concurrent access to the store is
not performed. 

The main issue with concurrent use is when data is written at the same time as
it's being iterated over. Doing so will cause a irrecoverable fatal error because
of concurrent reads and writes to an internal map.

Although it's not recommended, you can iterate through values while writing to
it by disabling "FastNode" **without guarantees that the values being written will
be returned during the iteration** (if you need this, you might want to reconsider
the design of your application). This is done by setting `iavl-disable-fastnode`
to `true` in the config TOML file.

### `cachekv.Store`

The `cachekv.Store` store wraps an underlying `KVStore`, typically a `iavl.Store`
and contains an in-memory cache for storing pending writes to underlying `KVStore`.
`Set` and `Delete` calls are executed on the in-memory cache, whereas `Has` calls
are proxied to the underlying `KVStore`. 

One of the most important calls to a `cachekv.Store` is `Write()`, which ensures
that key-value pairs are written to the underlying `KVStore` in a deterministic
and ordered manner by sorting the keys first. The store keeps track of "dirty"
keys and uses these to determine what keys to sort. In addition, it also keeps
track of deleted keys and ensures these are also removed from the underlying
`KVStore`.

The `cachekv.Store` also provides the ability to perform iteration and reverse
iteration. Iteration is performed through the `cacheMergeIterator` type and uses
both the dirty cache and underlying `KVStore` to iterate over key-value pairs.

Note, all calls to CRUD and iteration operations on a `cachekv.Store` are thread-safe.

### `gaskv.Store`

The `gaskv.Store` store provides a simple implementation of a `KVStore`.
Specifically, it just wraps an existing `KVStore`, such as a cache-wrapped
`iavl.Store`, and incurs configurable gas costs for CRUD operations via
`ConsumeGas()` calls defined on the `GasMeter` which exists in a `sdk.Context`
and then proxies the underlying CRUD call to the underlying store. Note, the
`GasMeter` is reset on each block.

### `cachemulti.Store` & `rootmulti.Store`

The `rootmulti.Store` acts as an abstraction around a series of stores. Namely,
it implements the `CommitMultiStore` an `Queryable` interfaces. Through the
`rootmulti.Store`, an SDK module can request access to a `KVStore` to perform
state CRUD operations and queries by holding access to a unique `KVStoreKey`.

The `rootmulti.Store` ensures these queries and state operations are performed
through cached-wrapped instances of `cachekv.Store` which is described above. The
`rootmulti.Store` implementation is also responsible for committing all accumulated
state from each `KVStore` to disk and returning an application state Merkle root.

Queries can be performed to return state data along with associated state
commitment proofs for both previous heights/versions and the current state root.
Queries are routed based on store name, i.e. a module, along with other parameters
which are defined in `abci.QueryRequest`.

The `rootmulti.Store` also provides primitives for pruning data at a given
height/version from state storage. When a height is committed, the `rootmulti.Store`
will determine if other previous heights should be considered for removal based
on the operator's pruning settings defined by `PruningOptions`, which defines
how many recent versions to keep on disk and the interval at which to remove
"staged" pruned heights from disk. During each interval, the staged heights are
removed from each `KVStore`. Note, it is up to the underlying `KVStore`
implementation to determine how pruning is actually performed. The `PruningOptions`
are defined as follows:

```go
type PruningOptions struct {
	// KeepRecent defines how many recent heights to keep on disk.
	KeepRecent uint64

	// Interval defines when the pruned heights are removed from disk.
	Interval uint64

	// Strategy defines the kind of pruning strategy. See below for more information on each.
	Strategy PruningStrategy
}
```

The Cosmos SDK defines a preset number of pruning "strategies": `default`, `everything`
`nothing`, and `custom`.

It is important to note that the `rootmulti.Store` considers each `KVStore` as a
separate logical store. In other words, they do not share a Merkle tree or
comparable data structure. This means that when state is committed via
`rootmulti.Store`, each store is committed in sequence and thus is not atomic.

In terms of store construction and wiring, each Cosmos SDK application contains
a `BaseApp` instance which internally has a reference to a `CommitMultiStore`
that is implemented by a `rootmulti.Store`. The application then registers one or
more `KVStoreKey` that pertain to a unique module and thus a `KVStore`. Through
the use of an `sdk.Context` and a `KVStoreKey`, each module can get direct access
to it's respective `KVStore` instance.

Example:

```go
func NewApp(...) Application {
  // ...
  
  bApp := baseapp.NewBaseApp(appName, logger, db, txConfig.TxDecoder(), baseAppOptions...)
  bApp.SetCommitMultiStoreTracer(traceStore)
  bApp.SetVersion(version.Version)
  bApp.SetInterfaceRegistry(interfaceRegistry)

	// ...

  keys := sdk.NewKVStoreKeys(...)
	transientKeys := sdk.NewTransientStoreKeys(...)
	memKeys := sdk.NewMemoryStoreKeys(...)

	// ...

	// initialize stores
	app.MountKVStores(keys)
	app.MountTransientStores(transientKeys)
	app.MountMemoryStores(memKeys)

	// ...
}
```

The `rootmulti.Store` itself can be cache-wrapped which returns an instance of a
`cachemulti.Store`. For each block, `BaseApp` ensures that the proper abstractions
are created on the `CommitMultiStore`, i.e. ensuring that the `rootmulti.Store`
is cached-wrapped and uses the resulting `cachemulti.Store` to be set on the
`sdk.Context` which is then used for block and transaction execution. As a result,
all state mutations due to block and transaction execution are actually held
ephemerally until `Commit()` is called by the ABCI client. This concept is further
expanded upon when the AnteHandler is executed per transaction to ensure state
is not committed for transactions that failed CheckTx.
