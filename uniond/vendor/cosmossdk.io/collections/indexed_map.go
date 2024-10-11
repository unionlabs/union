package collections

import (
	"context"
	"errors"
	"fmt"
	"reflect"

	"cosmossdk.io/collections/codec"
)

// Indexes represents a type which groups multiple Index
// of one Value saved with the provided PrimaryKey.
// Indexes is just meant to be a struct containing all
// the indexes to maintain relationship for.
type Indexes[PrimaryKey, Value any] interface {
	// IndexesList is implemented by the Indexes type
	// and returns all the grouped Index of Value.
	IndexesList() []Index[PrimaryKey, Value]
}

// Index represents an index of the Value indexed using the type PrimaryKey.
type Index[PrimaryKey, Value any] interface {
	// Reference creates a reference between the provided primary key and value.
	// It provides a lazyOldValue function that if called will attempt to fetch
	// the previous old value, returns ErrNotFound if no value existed.
	Reference(ctx context.Context, pk PrimaryKey, newValue Value, lazyOldValue func() (Value, error)) error
	// Unreference removes the reference between the primary key and value.
	// If error is ErrNotFound then it means that the value did not exist before.
	Unreference(ctx context.Context, pk PrimaryKey, lazyOldValue func() (Value, error)) error
}

// IndexedMap works like a Map but creates references between fields of Value and its PrimaryKey.
// These relationships are expressed and maintained using the Indexes type.
// Internally IndexedMap can be seen as a partitioned collection, one partition
// is a Map[PrimaryKey, Value], that maintains the object, the second
// are the Indexes.
type IndexedMap[PrimaryKey, Value, Idx any] struct {
	Indexes         Idx
	computedIndexes []Index[PrimaryKey, Value]
	m               Map[PrimaryKey, Value]
}

// NewIndexedMapSafe behaves like NewIndexedMap but returns errors.
func NewIndexedMapSafe[K, V, I any](
	schema *SchemaBuilder,
	prefix Prefix,
	name string,
	pkCodec codec.KeyCodec[K],
	valueCodec codec.ValueCodec[V],
	indexes I,
) (im *IndexedMap[K, V, I], err error) {
	var indexesList []Index[K, V]
	indexesImpl, ok := any(indexes).(Indexes[K, V])
	if ok {
		indexesList = indexesImpl.IndexesList()
	} else {
		// if does not implement Indexes, then we try to infer using reflection
		indexesList, err = tryInferIndexes[I, K, V](indexes)
		if err != nil {
			return nil, fmt.Errorf("unable to infer indexes using reflection, consider implementing Indexes interface: %w", err)
		}
	}

	return &IndexedMap[K, V, I]{
		computedIndexes: indexesList,
		Indexes:         indexes,
		m:               NewMap(schema, prefix, name, pkCodec, valueCodec),
	}, nil
}

var (
	// testing sentinel errors
	errNotStruct = errors.New("wanted struct or pointer to a struct")
	errNotIndex  = errors.New("field is not an index implementation")
)

func tryInferIndexes[I, K, V any](indexes I) ([]Index[K, V], error) {
	typ := reflect.TypeOf(indexes)
	v := reflect.ValueOf(indexes)
	// check if struct or pointer to a struct
	if typ.Kind() != reflect.Struct && (typ.Kind() != reflect.Pointer || typ.Elem().Kind() != reflect.Struct) {
		return nil, fmt.Errorf("%w: type %v", errNotStruct, typ)
	}
	// dereference
	if typ.Kind() == reflect.Pointer {
		v = v.Elem()
	}
	indexesImpl := make([]Index[K, V], v.NumField())
	for i := 0; i < v.NumField(); i++ {
		field := v.Field(i)
		index, ok := field.Interface().(Index[K, V])
		if !ok {
			return nil, fmt.Errorf("%w: field number %d", errNotIndex, i)
		}
		indexesImpl[i] = index
	}
	return indexesImpl, nil
}

// NewIndexedMap instantiates a new IndexedMap. Accepts a SchemaBuilder, a Prefix,
// a humanized name that defines the name of the collection, the primary key codec
// which is basically what IndexedMap uses to encode the primary key to bytes,
// the value codec which is what the IndexedMap uses to encode the value.
// Then it expects the initialized indexes. Reflection is used to infer the
// indexes, Indexes can optionally be implemented to be explicit. Panics
// on failure to create indexes. If you want an erroring API use NewIndexedMapSafe.
func NewIndexedMap[PrimaryKey, Value, Idx any](
	schema *SchemaBuilder,
	prefix Prefix,
	name string,
	pkCodec codec.KeyCodec[PrimaryKey],
	valueCodec codec.ValueCodec[Value],
	indexes Idx,
) *IndexedMap[PrimaryKey, Value, Idx] {
	im, err := NewIndexedMapSafe(schema, prefix, name, pkCodec, valueCodec, indexes)
	if err != nil {
		panic(err)
	}
	return im
}

// Get gets the object given its primary key.
func (m *IndexedMap[PrimaryKey, Value, Idx]) Get(ctx context.Context, pk PrimaryKey) (Value, error) {
	return m.m.Get(ctx, pk)
}

// Iterate allows to iterate over the objects given a Ranger of the primary key.
func (m *IndexedMap[PrimaryKey, Value, Idx]) Iterate(ctx context.Context, ranger Ranger[PrimaryKey]) (Iterator[PrimaryKey, Value], error) {
	return m.m.Iterate(ctx, ranger)
}

// Has reports if exists a value with the provided primary key.
func (m *IndexedMap[PrimaryKey, Value, Idx]) Has(ctx context.Context, pk PrimaryKey) (bool, error) {
	return m.m.Has(ctx, pk)
}

// Set maps the value using the primary key. It will also iterate every index and instruct them to
// add or update the indexes.
func (m *IndexedMap[PrimaryKey, Value, Idx]) Set(ctx context.Context, pk PrimaryKey, value Value) error {
	err := m.ref(ctx, pk, value)
	if err != nil {
		return err
	}
	return m.m.Set(ctx, pk, value)
}

// Remove removes the value associated with the primary key from the map. Then
// it iterates over all the indexes and instructs them to remove all the references
// associated with the removed value.
func (m *IndexedMap[PrimaryKey, Value, Idx]) Remove(ctx context.Context, pk PrimaryKey) error {
	err := m.unref(ctx, pk)
	if err != nil {
		return err
	}
	return m.m.Remove(ctx, pk)
}

// Walk applies the same semantics as Map.Walk.
func (m *IndexedMap[PrimaryKey, Value, Idx]) Walk(ctx context.Context, ranger Ranger[PrimaryKey], walkFunc func(key PrimaryKey, value Value) (stop bool, err error)) error {
	return m.m.Walk(ctx, ranger, walkFunc)
}

// IterateRaw iterates the IndexedMap using raw bytes keys. Follows the same semantics as Map.IterateRaw
func (m *IndexedMap[PrimaryKey, Value, Idx]) IterateRaw(ctx context.Context, start, end []byte, order Order) (Iterator[PrimaryKey, Value], error) {
	return m.m.IterateRaw(ctx, start, end, order)
}

func (m *IndexedMap[PrimaryKey, Value, Idx]) KeyCodec() codec.KeyCodec[PrimaryKey] {
	return m.m.KeyCodec()
}

func (m *IndexedMap[PrimaryKey, Value, Idx]) ValueCodec() codec.ValueCodec[Value] {
	return m.m.ValueCodec()
}

func (m *IndexedMap[PrimaryKey, Value, Idx]) ref(ctx context.Context, pk PrimaryKey, value Value) error {
	for _, index := range m.computedIndexes {
		err := index.Reference(ctx, pk, value, cachedGet[PrimaryKey, Value](ctx, m, pk))
		if err != nil {
			return err
		}
	}
	return nil
}

func (m *IndexedMap[PrimaryKey, Value, Idx]) unref(ctx context.Context, pk PrimaryKey) error {
	for _, index := range m.computedIndexes {
		err := index.Unreference(ctx, pk, cachedGet[PrimaryKey, Value](ctx, m, pk))
		if err != nil {
			return err
		}
	}
	return nil
}

// cachedGet returns a function that gets the value V, given the key K but
// returns always the same result on multiple calls.
func cachedGet[K, V any, M interface {
	Get(ctx context.Context, key K) (V, error)
}](ctx context.Context, m M, key K,
) func() (V, error) {
	var (
		value      V
		err        error
		calledOnce bool
	)

	return func() (V, error) {
		if calledOnce {
			return value, err
		}
		value, err = m.Get(ctx, key)
		calledOnce = true
		return value, err
	}
}
