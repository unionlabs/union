// Package kvstore implements simple key-value store
//
// It is without synchronization and allows any comparable keys. The main use of
// this package is for sharing singletons when building a circuit.
package kvstore

import (
	"reflect"
)

type Store interface {
	SetKeyValue(key, value any)
	GetKeyValue(key any) (value any)
}

type impl struct {
	db map[any]any
}

func New() Store {
	return &impl{
		db: make(map[any]any),
	}
}

func (c *impl) SetKeyValue(key, value any) {
	if !reflect.TypeOf(key).Comparable() {
		panic("key type not comparable")
	}
	c.db[key] = value
}

func (c *impl) GetKeyValue(key any) any {
	if !reflect.TypeOf(key).Comparable() {
		panic("key type not comparable")
	}
	return c.db[key]
}
