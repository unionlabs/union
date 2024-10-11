package decoding

import (
	"fmt"
	"sort"

	"cosmossdk.io/schema"
)

// DecoderResolver is an interface that allows indexers to discover and use module decoders.
type DecoderResolver interface {
	// DecodeModuleName decodes a module name from a byte slice passed as the actor in a KVPairUpdate.
	DecodeModuleName([]byte) (string, error)

	// EncodeModuleName encodes a module name into a byte slice that can be used as the actor in a KVPairUpdate.
	EncodeModuleName(string) ([]byte, error)

	// AllDecoders iterates over all available module decoders.
	AllDecoders(func(moduleName string, cdc schema.ModuleCodec) error) error

	// LookupDecoder looks up a specific module decoder.
	LookupDecoder(moduleName string) (decoder schema.ModuleCodec, found bool, err error)
}

// ModuleSetDecoderResolver returns DecoderResolver that will discover modules implementing
// DecodeableModule in the provided module set.
func ModuleSetDecoderResolver(moduleSet map[string]interface{}) DecoderResolver {
	return &moduleSetDecoderResolver{
		moduleSet: moduleSet,
	}
}

type moduleSetDecoderResolver struct {
	moduleSet map[string]interface{}
}

func (a moduleSetDecoderResolver) DecodeModuleName(bytes []byte) (string, error) {
	if _, ok := a.moduleSet[string(bytes)]; ok {
		return string(bytes), nil
	}
	return "", fmt.Errorf("module %s not found", bytes)
}

func (a moduleSetDecoderResolver) EncodeModuleName(s string) ([]byte, error) {
	if _, ok := a.moduleSet[s]; ok {
		return []byte(s), nil
	}
	return nil, fmt.Errorf("module %s not found", s)
}

func (a moduleSetDecoderResolver) AllDecoders(f func(string, schema.ModuleCodec) error) error {
	keys := make([]string, 0, len(a.moduleSet))
	for k := range a.moduleSet {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	for _, k := range keys {
		module := a.moduleSet[k]
		dm, ok := module.(schema.HasModuleCodec)
		if ok {
			decoder, err := dm.ModuleCodec()
			if err != nil {
				return err
			}
			err = f(k, decoder)
			if err != nil {
				return err
			}
		}
	}
	return nil
}

func (a moduleSetDecoderResolver) LookupDecoder(moduleName string) (schema.ModuleCodec, bool, error) {
	mod, ok := a.moduleSet[moduleName]
	if !ok {
		return schema.ModuleCodec{}, false, nil
	}

	dm, ok := mod.(schema.HasModuleCodec)
	if !ok {
		return schema.ModuleCodec{}, false, nil
	}

	decoder, err := dm.ModuleCodec()
	return decoder, true, err
}
