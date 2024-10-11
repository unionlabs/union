package store

import "context"

// KVStoreService represents a unique, non-forgeable handle to a regular merkle-tree
// backed KVStore. It should be provided as a module-scoped dependency by the runtime
// module being used to build the app.
type KVStoreService interface {
	// OpenKVStore retrieves the KVStore from the context.
	OpenKVStore(context.Context) KVStore
}

// KVStoreServiceFactory is a function that creates a new KVStoreService.
// It can be used to override the default KVStoreService bindings for cases
// where an application must supply a custom stateful backend.
type KVStoreServiceFactory func([]byte) KVStoreService

// MemoryStoreService represents a unique, non-forgeable handle to a memory-backed
// KVStore. It should be provided as a module-scoped dependency by the runtime
// module being used to build the app.
type MemoryStoreService interface {
	// OpenMemoryStore retrieves the memory store from the context.
	OpenMemoryStore(context.Context) KVStore
}

// TransientStoreService represents a unique, non-forgeable handle to a memory-backed
// KVStore which is reset at the start of every block. It should be provided as
// a module-scoped dependency by the runtime module being used to build the app.
// WARNING: This service is not available in server/v2 apps. Store/v2 does not support
// transient stores.
type TransientStoreService interface {
	// OpenTransientStore retrieves the transient store from the context.
	OpenTransientStore(context.Context) KVStore
}
