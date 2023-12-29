package collections

import (
	"bytes"
	"context"
	"fmt"

	"cosmossdk.io/collections/codec"
	"cosmossdk.io/core/store"
)

// Map represents the basic collections object.
// It is used to map arbitrary keys to arbitrary
// objects.
type Map[K, V any] struct {
	kc codec.KeyCodec[K]
	vc codec.ValueCodec[V]

	// store accessor
	sa     func(context.Context) store.KVStore
	prefix []byte
	name   string
}

// NewMap returns a Map given a StoreKey, a Prefix, human-readable name and the relative value and key encoders.
// Name and prefix must be unique within the schema and name must match the format specified by NameRegex, or
// else this method will panic.
func NewMap[K, V any](
	schemaBuilder *SchemaBuilder,
	prefix Prefix,
	name string,
	keyCodec codec.KeyCodec[K],
	valueCodec codec.ValueCodec[V],
) Map[K, V] {
	m := Map[K, V]{
		kc:     keyCodec,
		vc:     valueCodec,
		sa:     schemaBuilder.schema.storeAccessor,
		prefix: prefix.Bytes(),
		name:   name,
	}
	schemaBuilder.addCollection(collectionImpl[K, V]{m})
	return m
}

func (m Map[K, V]) GetName() string {
	return m.name
}

func (m Map[K, V]) GetPrefix() []byte {
	return m.prefix
}

// Set maps the provided value to the provided key in the store.
// Errors with ErrEncoding if key or value encoding fails.
func (m Map[K, V]) Set(ctx context.Context, key K, value V) error {
	bytesKey, err := EncodeKeyWithPrefix(m.prefix, m.kc, key)
	if err != nil {
		return err
	}

	valueBytes, err := m.vc.Encode(value)
	if err != nil {
		return fmt.Errorf("%w: value encode: %s", ErrEncoding, err) // TODO: use multi err wrapping in go1.20: https://github.com/golang/go/issues/53435
	}

	kvStore := m.sa(ctx)
	return kvStore.Set(bytesKey, valueBytes)
}

// Get returns the value associated with the provided key,
// errors with ErrNotFound if the key does not exist, or
// with ErrEncoding if the key or value decoding fails.
func (m Map[K, V]) Get(ctx context.Context, key K) (v V, err error) {
	bytesKey, err := EncodeKeyWithPrefix(m.prefix, m.kc, key)
	if err != nil {
		return v, err
	}

	kvStore := m.sa(ctx)
	valueBytes, err := kvStore.Get(bytesKey)
	if err != nil {
		return v, err
	}
	if valueBytes == nil {
		return v, fmt.Errorf("%w: key '%s' of type %s", ErrNotFound, m.kc.Stringify(key), m.vc.ValueType())
	}

	v, err = m.vc.Decode(valueBytes)
	if err != nil {
		return v, fmt.Errorf("%w: value decode: %s", ErrEncoding, err) // TODO: use multi err wrapping in go1.20: https://github.com/golang/go/issues/53435
	}
	return v, nil
}

// Has reports whether the key is present in storage or not.
// Errors with ErrEncoding if key encoding fails.
func (m Map[K, V]) Has(ctx context.Context, key K) (bool, error) {
	bytesKey, err := EncodeKeyWithPrefix(m.prefix, m.kc, key)
	if err != nil {
		return false, err
	}
	kvStore := m.sa(ctx)
	return kvStore.Has(bytesKey)
}

// Remove removes the key from the storage.
// Errors with ErrEncoding if key encoding fails.
// If the key does not exist then this is a no-op.
func (m Map[K, V]) Remove(ctx context.Context, key K) error {
	bytesKey, err := EncodeKeyWithPrefix(m.prefix, m.kc, key)
	if err != nil {
		return err
	}
	kvStore := m.sa(ctx)
	return kvStore.Delete(bytesKey)
}

// Iterate provides an Iterator over K and V. It accepts a Ranger interface.
// A nil ranger equals to iterate over all the keys in ascending order.
func (m Map[K, V]) Iterate(ctx context.Context, ranger Ranger[K]) (Iterator[K, V], error) {
	return iteratorFromRanger(ctx, m, ranger)
}

// Walk iterates over the Map with the provided range, calls the provided
// walk function with the decoded key and value. If the callback function
// returns true then the walking is stopped.
// A nil ranger equals to walking over the entire key and value set.
func (m Map[K, V]) Walk(ctx context.Context, ranger Ranger[K], walkFunc func(key K, value V) (stop bool, err error)) error {
	iter, err := m.Iterate(ctx, ranger)
	if err != nil {
		return err
	}
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		kv, err := iter.KeyValue()
		if err != nil {
			return err
		}
		stop, err := walkFunc(kv.Key, kv.Value)
		if err != nil {
			return err
		}
		if stop {
			return nil
		}
	}
	return nil
}

// Clear clears the collection contained within the provided key range.
// A nil ranger equals to clearing the whole collection.
// NOTE: this API needs to be used with care, considering that as of today
// cosmos-sdk stores the deletion records to be committed in a memory cache,
// clearing a lot of data might make the node go OOM.
func (m Map[K, V]) Clear(ctx context.Context, ranger Ranger[K]) error {
	startBytes, endBytes, _, err := parseRangeInstruction(m.prefix, m.kc, ranger)
	if err != nil {
		return err
	}
	return deleteDomain(m.sa(ctx), startBytes, endBytes)
}

const clearBatchSize = 10000

// deleteDomain deletes the domain of an iterator, the key difference
// is that it uses batches to clear the store meaning that it will read
// the keys within the domain close the iterator and then delete them.
func deleteDomain(s store.KVStore, start, end []byte) error {
	for {
		iter, err := s.Iterator(start, end)
		if err != nil {
			return err
		}

		keys := make([][]byte, 0, clearBatchSize)
		for ; iter.Valid() && len(keys) < clearBatchSize; iter.Next() {
			keys = append(keys, iter.Key())
		}

		// we close the iterator here instead of deferring
		err = iter.Close()
		if err != nil {
			return err
		}

		for _, key := range keys {
			err = s.Delete(key)
			if err != nil {
				return err
			}
		}

		// If we've retrieved less than the batchSize, we're done.
		if len(keys) < clearBatchSize {
			break
		}
	}

	return nil
}

// IterateRaw iterates over the collection. The iteration range is untyped, it uses raw
// bytes. The resulting Iterator is typed.
// A nil start iterates from the first key contained in the collection.
// A nil end iterates up to the last key contained in the collection.
// A nil start and a nil end iterates over every key contained in the collection.
// TODO(tip): simplify after https://github.com/cosmos/cosmos-sdk/pull/14310 is merged
func (m Map[K, V]) IterateRaw(ctx context.Context, start, end []byte, order Order) (Iterator[K, V], error) {
	prefixedStart := append(m.prefix, start...)
	var prefixedEnd []byte
	if end == nil {
		prefixedEnd = nextBytesPrefixKey(m.prefix)
	} else {
		prefixedEnd = append(m.prefix, end...)
	}

	if bytes.Compare(prefixedStart, prefixedEnd) == 1 {
		return Iterator[K, V]{}, ErrInvalidIterator
	}

	s := m.sa(ctx)
	var (
		storeIter store.Iterator
		err       error
	)
	switch order {
	case OrderAscending:
		storeIter, err = s.Iterator(prefixedStart, prefixedEnd)
	case OrderDescending:
		storeIter, err = s.ReverseIterator(prefixedStart, prefixedEnd)
	default:
		return Iterator[K, V]{}, errOrder
	}
	if err != nil {
		return Iterator[K, V]{}, err
	}

	return Iterator[K, V]{
		kc:           m.kc,
		vc:           m.vc,
		iter:         storeIter,
		prefixLength: len(m.prefix),
	}, nil
}

// KeyCodec returns the Map's KeyCodec.
func (m Map[K, V]) KeyCodec() codec.KeyCodec[K] { return m.kc }

// ValueCodec returns the Map's ValueCodec.
func (m Map[K, V]) ValueCodec() codec.ValueCodec[V] { return m.vc }

// EncodeKeyWithPrefix returns how the collection would store the key in storage given
// prefix, key codec and the concrete key.
func EncodeKeyWithPrefix[K any](prefix []byte, kc codec.KeyCodec[K], key K) ([]byte, error) {
	prefixLen := len(prefix)
	// preallocate buffer
	keyBytes := make([]byte, prefixLen+kc.Size(key))
	// put prefix
	copy(keyBytes, prefix)
	// put key
	_, err := kc.Encode(keyBytes[prefixLen:], key)
	if err != nil {
		return nil, fmt.Errorf("%w: key encode: %s", ErrEncoding, err) // TODO: use multi err wrapping in go1.20: https://github.com/golang/go/issues/53435
	}
	return keyBytes, nil
}
