package json

import (
	"errors"
	"fmt"
	"reflect"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

var (
	// typeRegistry contains globally registered types for JSON encoding/decoding.
	typeRegistry = newTypes()
)

// RegisterType registers a type for Amino-compatible interface encoding in the global type
// registry. These types will be encoded with a type wrapper `{"type":"<type>","value":<value>}`
// regardless of which interface they are wrapped in (if any). If the type is a pointer, it will
// still be valid both for value and pointer types, but decoding into an interface will generate
// the a value or pointer based on the registered type.
//
// Should only be called in init() functions, as it panics on error.
func RegisterType(_type interface{}, name string) {
	if _type == nil {
		panic("cannot register nil type")
	}
	err := typeRegistry.register(name, reflect.ValueOf(_type).Type())
	if err != nil {
		panic(err)
	}
}

// typeInfo contains type information.
type typeInfo struct {
	name      string
	rt        reflect.Type
	returnPtr bool
}

// types is a type registry. It is safe for concurrent use.
type types struct {
	cmtsync.RWMutex
	byType map[reflect.Type]*typeInfo
	byName map[string]*typeInfo
}

// newTypes creates a new type registry.
func newTypes() types {
	return types{
		byType: map[reflect.Type]*typeInfo{},
		byName: map[string]*typeInfo{},
	}
}

// registers the given type with the given name. The name and type must not be registered already.
func (t *types) register(name string, rt reflect.Type) error {
	if name == "" {
		return errors.New("name cannot be empty")
	}
	// If this is a pointer type, we recursively resolve until we get a bare type, but register that
	// we should return pointers.
	returnPtr := false
	for rt.Kind() == reflect.Ptr {
		returnPtr = true
		rt = rt.Elem()
	}
	tInfo := &typeInfo{
		name:      name,
		rt:        rt,
		returnPtr: returnPtr,
	}

	t.Lock()
	defer t.Unlock()
	if _, ok := t.byName[tInfo.name]; ok {
		return fmt.Errorf("a type with name %q is already registered", name)
	}
	if _, ok := t.byType[tInfo.rt]; ok {
		return fmt.Errorf("the type %v is already registered", rt)
	}
	t.byName[name] = tInfo
	t.byType[rt] = tInfo
	return nil
}

// lookup looks up a type from a name, or nil if not registered.
func (t *types) lookup(name string) (reflect.Type, bool) {
	t.RLock()
	defer t.RUnlock()
	tInfo := t.byName[name]
	if tInfo == nil {
		return nil, false
	}
	return tInfo.rt, tInfo.returnPtr
}

// name looks up the name of a type, or empty if not registered. Unwraps pointers as necessary.
func (t *types) name(rt reflect.Type) string {
	for rt.Kind() == reflect.Ptr {
		rt = rt.Elem()
	}
	t.RLock()
	defer t.RUnlock()
	tInfo := t.byType[rt]
	if tInfo == nil {
		return ""
	}
	return tInfo.name
}
