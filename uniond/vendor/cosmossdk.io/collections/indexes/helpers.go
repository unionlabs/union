package indexes

import (
	"context"

	"cosmossdk.io/collections"
)

// iterator defines the minimum set of methods of an index iterator
// required to work with the helpers.
type iterator[K any] interface {
	// PrimaryKey returns the iterator current primary key.
	PrimaryKey() (K, error)
	// Next advances the iterator by one element.
	Next()
	// Valid asserts if the Iterator is valid.
	Valid() bool
	// Close closes the iterator.
	Close() error
}

// CollectKeyValues collects all the keys and the values of an indexed map index iterator.
// The Iterator is fully consumed and closed.
func CollectKeyValues[K, V any, I iterator[K], Idx collections.Indexes[K, V]](
	ctx context.Context,
	indexedMap *collections.IndexedMap[K, V, Idx],
	iter I,
) (kvs []collections.KeyValue[K, V], err error) {
	err = ScanKeyValues(ctx, indexedMap, iter, func(kv collections.KeyValue[K, V]) bool {
		kvs = append(kvs, kv)
		return false
	})
	return
}

// ScanKeyValues calls the do function on every record found, in the indexed map
// from the index iterator. Returning true stops the iteration.
// The Iterator is closed when this function exits.
func ScanKeyValues[K, V any, I iterator[K], Idx collections.Indexes[K, V]](
	ctx context.Context,
	indexedMap *collections.IndexedMap[K, V, Idx],
	iter I,
	do func(kv collections.KeyValue[K, V]) (stop bool),
) (err error) {
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		pk, err := iter.PrimaryKey()
		if err != nil {
			return err
		}

		value, err := indexedMap.Get(ctx, pk)
		if err != nil {
			return err
		}

		kv := collections.KeyValue[K, V]{
			Key:   pk,
			Value: value,
		}

		if do(kv) {
			break
		}
	}

	return nil
}

// CollectValues collects all the values from an Index iterator and the IndexedMap.
// Closes the Iterator.
func CollectValues[K, V any, I iterator[K], Idx collections.Indexes[K, V]](
	ctx context.Context,
	indexedMap *collections.IndexedMap[K, V, Idx],
	iter I,
) (values []V, err error) {
	err = ScanValues(ctx, indexedMap, iter, func(value V) (stop bool) {
		values = append(values, value)
		return false
	})
	return
}

// ScanValues collects all the values from an Index iterator and the IndexedMap in a lazy way.
// The iterator is closed when this function exits.
func ScanValues[K, V any, I iterator[K], Idx collections.Indexes[K, V]](
	ctx context.Context,
	indexedMap *collections.IndexedMap[K, V, Idx],
	iter I,
	f func(value V) (stop bool),
) error {
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		key, err := iter.PrimaryKey()
		if err != nil {
			return err
		}

		value, err := indexedMap.Get(ctx, key)
		if err != nil {
			return err
		}

		stop := f(value)
		if stop {
			return nil
		}
	}

	return nil
}
