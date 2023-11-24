package store

import "context"

// KVStoreService represents a unique, non-forgeable handle to a regular merkle-tree
// backed KVStore. It should be provided as a module-scoped dependency by the runtime
// module being used to build the app.
type KVStoreService interface {
	// OpenKVStore retrieves the KVStore from the context.
	OpenKVStore(context.Context) KVStore
}

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
type TransientStoreService interface {
	// OpenTransientStore retrieves the transient store from the context.
	OpenTransientStore(context.Context) KVStore
}
