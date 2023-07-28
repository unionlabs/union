package circuitdefer

import (
	"github.com/consensys/gnark/internal/kvstore"
)

type deferKey struct{}

func Put[T any](builder any, cb T) {
	// we use generics for type safety but to avoid import cycles.
	// TODO: compare with using any and type asserting at caller
	kv, ok := builder.(kvstore.Store)
	if !ok {
		panic("builder does not implement kvstore.Store")
	}
	val := kv.GetKeyValue(deferKey{})
	var deferred []T
	if val != nil {
		var ok bool
		deferred, ok = val.([]T)
		if !ok {
			panic("stored deferred functions not []func(frontend.API) error")
		}
	}
	deferred = append(deferred, cb)
	kv.SetKeyValue(deferKey{}, deferred)
}

func GetAll[T any](builder any) []T {
	kv, ok := builder.(kvstore.Store)
	if !ok {
		panic("builder does not implement kvstore.Store")
	}
	val := kv.GetKeyValue(deferKey{})
	if val == nil {
		return nil
	}
	deferred, ok := val.([]T)
	if !ok {
		panic("stored deferred functions not []func(frontend.API) error")
	}
	return deferred
}
