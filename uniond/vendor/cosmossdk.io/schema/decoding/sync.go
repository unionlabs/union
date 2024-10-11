package decoding

import (
	"cosmossdk.io/schema"
	"cosmossdk.io/schema/appdata"
)

// SyncSource is an interface that allows indexers to start indexing modules with pre-existing state.
// It should generally be a wrapper around the key-value store.
type SyncSource interface {
	// IterateAllKVPairs iterates over all key-value pairs for a given module.
	IterateAllKVPairs(moduleName string, fn func(key, value []byte) error) error
}

// SyncOptions are the options for Sync.
type SyncOptions struct {
	ModuleFilter func(moduleName string) bool
}

// Sync synchronizes existing state from the sync source to the listener using the resolver to decode data.
func Sync(listener appdata.Listener, source SyncSource, resolver DecoderResolver, opts SyncOptions) error {
	initializeModuleData := listener.InitializeModuleData
	onObjectUpdate := listener.OnObjectUpdate

	// no-op if not listening to decoded data
	if initializeModuleData == nil && onObjectUpdate == nil {
		return nil
	}

	return resolver.AllDecoders(func(moduleName string, cdc schema.ModuleCodec) error {
		if opts.ModuleFilter != nil && !opts.ModuleFilter(moduleName) {
			// ignore this module
			return nil
		}

		if initializeModuleData != nil {
			err := initializeModuleData(appdata.ModuleInitializationData{
				ModuleName: moduleName,
				Schema:     cdc.Schema,
			})
			if err != nil {
				return err
			}
		}

		if onObjectUpdate == nil || cdc.KVDecoder == nil {
			return nil
		}

		return source.IterateAllKVPairs(moduleName, func(key, value []byte) error {
			updates, err := cdc.KVDecoder(schema.KVPairUpdate{Key: key, Value: value})
			if err != nil {
				return err
			}

			if len(updates) == 0 {
				return nil
			}

			return onObjectUpdate(appdata.ObjectUpdateData{ModuleName: moduleName, Updates: updates})
		})
	})
}
