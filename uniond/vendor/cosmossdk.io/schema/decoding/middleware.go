package decoding

import (
	"cosmossdk.io/schema"
	"cosmossdk.io/schema/appdata"
)

type MiddlewareOptions struct {
	ModuleFilter func(moduleName string) bool
}

// Middleware decodes raw data passed to the listener as kv-updates into decoded object updates. Module initialization
// is done lazily as modules are encountered in the kv-update stream.
func Middleware(target appdata.Listener, resolver DecoderResolver, opts MiddlewareOptions) (appdata.Listener, error) {
	initializeModuleData := target.InitializeModuleData
	onObjectUpdate := target.OnObjectUpdate

	// no-op if not listening to decoded data
	if initializeModuleData == nil && onObjectUpdate == nil {
		return target, nil
	}

	onKVPair := target.OnKVPair

	moduleCodecs := map[string]*schema.ModuleCodec{}
	moduleNames := map[string]string{}

	target.OnKVPair = func(data appdata.KVPairData) error {
		// first forward kv pair updates
		if onKVPair != nil {
			err := onKVPair(data)
			if err != nil {
				return err
			}
		}

		for _, kvUpdate := range data.Updates {
			moduleName, ok := moduleNames[string(kvUpdate.Actor)]
			if !ok {
				var err error
				moduleName, err = resolver.DecodeModuleName(kvUpdate.Actor)
				if err != nil {
					return err
				}

				moduleNames[string(kvUpdate.Actor)] = moduleName
			}

			// look for an existing codec
			pcdc, ok := moduleCodecs[moduleName]
			if !ok {
				if opts.ModuleFilter != nil && !opts.ModuleFilter(moduleName) {
					// we don't care about this module so store nil and continue
					moduleCodecs[moduleName] = nil
					continue
				}

				// look for a new codec
				cdc, found, err := resolver.LookupDecoder(moduleName)
				if err != nil {
					return err
				}

				if !found {
					// store nil to indicate we've seen this module and don't have a codec
					// and keep processing the kv updates
					moduleCodecs[moduleName] = nil
					continue
				}

				pcdc = &cdc
				moduleCodecs[moduleName] = pcdc

				if initializeModuleData != nil {
					err = initializeModuleData(appdata.ModuleInitializationData{
						ModuleName: moduleName,
						Schema:     cdc.Schema,
					})
					if err != nil {
						return err
					}
				}
			}

			if pcdc == nil {
				// we've already seen this module and can't decode
				continue
			}

			if onObjectUpdate == nil || pcdc.KVDecoder == nil {
				// not listening to updates or can't decode so continue
				continue
			}

			for _, u := range kvUpdate.StateChanges {
				updates, err := pcdc.KVDecoder(u)
				if err != nil {
					return err
				}

				if len(updates) == 0 {
					// no updates
					continue
				}

				err = target.OnObjectUpdate(appdata.ObjectUpdateData{
					ModuleName: moduleName,
					Updates:    updates,
				})
				if err != nil {
					return err
				}
			}
		}

		return nil
	}

	return target, nil
}
