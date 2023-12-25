package collections

import (
	"bytes"
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections/codec"
	"cosmossdk.io/core/store"
)

// ErrInvalidIterator is returned when an Iterate call resulted in an invalid iterator.
var ErrInvalidIterator = errors.New("collections: invalid iterator")

// Order defines the key order.
type Order uint8

const (
	// OrderAscending instructs the Iterator to provide keys from the smallest to the greatest.
	OrderAscending Order = 0
	// OrderDescending instructs the Iterator to provide keys from the greatest to the smallest.
	OrderDescending Order = 1
)

type rangeKeyKind uint8

const (
	rangeKeyExact rangeKeyKind = iota
	rangeKeyNext
	rangeKeyPrefixEnd
)

// RangeKey wraps a generic range key K, acts as an enum which defines different
// ways to encode the wrapped key to bytes when it's being used in an iteration.
type RangeKey[K any] struct {
	kind rangeKeyKind
	key  K
}

// RangeKeyNext instantiates a RangeKey that when encoded to bytes
// identifies the next key after the provided key K.
// Example: given a string key "ABCD" the next key is bytes("ABCD\0")
// It's useful when defining inclusivity or exclusivity of a key
// in store iteration. Specifically: to make an Iterator start exclude key K
// I would return a RangeKeyNext(key) in the Ranger start.
func RangeKeyNext[K any](key K) *RangeKey[K] {
	return &RangeKey[K]{key: key, kind: rangeKeyNext}
}

// RangeKeyPrefixEnd instantiates a RangeKey that when encoded to bytes
// identifies the key that would end the prefix of the key K.
// Example: if the string key "ABCD" is provided, it would be encoded as bytes("ABCE").
func RangeKeyPrefixEnd[K any](key K) *RangeKey[K] {
	return &RangeKey[K]{key: key, kind: rangeKeyPrefixEnd}
}

// RangeKeyExact instantiates a RangeKey that applies no modifications
// to the key K. So its bytes representation will not be altered.
func RangeKeyExact[K any](key K) *RangeKey[K] {
	return &RangeKey[K]{key: key, kind: rangeKeyExact}
}

// Ranger defines a generic interface that provides a range of keys.
type Ranger[K any] interface {
	// RangeValues is defined by Ranger implementers.
	// The implementer can optionally return a start and an end.
	// If start is nil and end is not, the iteration will include all the keys
	// in the collection up until the provided end.
	// If start is defined and end is nil, the iteration will include all the keys
	// in the collection starting from the provided start.
	// If both are nil then the iteration will include all the possible keys in the
	// collection.
	// Order defines the order of the iteration, if order is OrderAscending then the
	// iteration will yield keys from the smallest to the biggest, if order
	// is OrderDescending then the iteration will yield keys from the biggest to the smallest.
	// Ordering is defined by the keys bytes representation, which is dependent on the KeyCodec used.
	RangeValues() (start, end *RangeKey[K], order Order, err error)
}

// Range is a Ranger implementer.
type Range[K any] struct {
	start *RangeKey[K]
	end   *RangeKey[K]
	order Order
}

// Prefix sets a fixed prefix for the key range.
func (r *Range[K]) Prefix(key K) *Range[K] {
	r.start = RangeKeyExact(key)
	r.end = RangeKeyPrefixEnd(key)
	return r
}

// StartInclusive makes the range contain only keys which are bigger or equal to the provided start K.
func (r *Range[K]) StartInclusive(start K) *Range[K] {
	r.start = RangeKeyExact(start)
	return r
}

// StartExclusive makes the range contain only keys which are bigger to the provided start K.
func (r *Range[K]) StartExclusive(start K) *Range[K] {
	r.start = RangeKeyNext(start)
	return r
}

// EndInclusive makes the range contain only keys which are smaller or equal to the provided end K.
func (r *Range[K]) EndInclusive(end K) *Range[K] {
	r.end = RangeKeyNext(end)
	return r
}

// EndExclusive makes the range contain only keys which are smaller to the provided end K.
func (r *Range[K]) EndExclusive(end K) *Range[K] {
	r.end = RangeKeyExact(end)
	return r
}

func (r *Range[K]) Descending() *Range[K] {
	r.order = OrderDescending
	return r
}

// test sentinel error
var (
	errOrder = errors.New("collections: invalid order")
)

func (r *Range[K]) RangeValues() (start, end *RangeKey[K], order Order, err error) {
	return r.start, r.end, r.order, nil
}

// parseRangeInstruction converts a Ranger into start bytes, end bytes and order of a store iteration.
func parseRangeInstruction[K any](prefix []byte, keyCodec codec.KeyCodec[K], r Ranger[K]) ([]byte, []byte, Order, error) {
	var (
		start *RangeKey[K]
		end   *RangeKey[K]
		order = OrderAscending
		err   error
	)

	if r != nil {
		start, end, order, err = r.RangeValues()
		if err != nil {
			return nil, nil, 0, err
		}
	}

	startBytes := prefix
	if start != nil {
		startBytes, err = encodeRangeBound(prefix, keyCodec, start)
		if err != nil {
			return nil, nil, 0, err
		}
	}
	var endBytes []byte
	if end != nil {
		endBytes, err = encodeRangeBound(prefix, keyCodec, end)
		if err != nil {
			return nil, nil, 0, err
		}
	} else {
		endBytes = nextBytesPrefixKey(prefix)
	}
	if bytes.Compare(startBytes, endBytes) == 1 {
		return nil, nil, 0, ErrInvalidIterator
	}
	return startBytes, endBytes, order, nil
}

// iteratorFromRanger generates an Iterator instance, with the proper prefixing and ranging.
// a nil Ranger can be seen as an ascending iteration over all the possible keys.
func iteratorFromRanger[K, V any](ctx context.Context, m Map[K, V], r Ranger[K]) (iter Iterator[K, V], err error) {
	startBytes, endBytes, order, err := parseRangeInstruction(m.prefix, m.kc, r)
	if err != nil {
		return Iterator[K, V]{}, err
	}
	return newIterator(ctx, startBytes, endBytes, order, m)
}

func newIterator[K, V any](ctx context.Context, start, end []byte, order Order, m Map[K, V]) (Iterator[K, V], error) {
	kv := m.sa(ctx)
	var (
		iter store.Iterator
		err  error
	)
	switch order {
	case OrderAscending:
		iter, err = kv.Iterator(start, end)
	case OrderDescending:
		iter, err = kv.ReverseIterator(start, end)
	default:
		return Iterator[K, V]{}, errOrder
	}
	if err != nil {
		return Iterator[K, V]{}, err
	}

	return Iterator[K, V]{
		kc:           m.kc,
		vc:           m.vc,
		iter:         iter,
		prefixLength: len(m.prefix),
	}, nil
}

// Iterator defines a generic wrapper around an storetypes.Iterator.
// This iterator provides automatic key and value encoding,
// it assumes all the keys and values contained within the storetypes.Iterator
// range are the same.
type Iterator[K, V any] struct {
	kc codec.KeyCodec[K]
	vc codec.ValueCodec[V]

	iter store.Iterator

	prefixLength int // prefixLength refers to the bytes provided by Prefix.Bytes, not Ranger.RangeValues() prefix.
}

// Value returns the current iterator value bytes decoded.
func (i Iterator[K, V]) Value() (V, error) {
	return i.vc.Decode(i.iter.Value())
}

// Key returns the current storetypes.Iterator decoded key.
func (i Iterator[K, V]) Key() (K, error) {
	bytesKey := i.iter.Key()[i.prefixLength:] // strip prefix namespace

	read, key, err := i.kc.Decode(bytesKey)
	if err != nil {
		var k K
		return k, err
	}
	if read != len(bytesKey) {
		var k K
		return k, fmt.Errorf("%w: key decoder didn't fully consume the key: %T %x %d", ErrEncoding, i.kc, bytesKey, read)
	}
	return key, nil
}

// Values fully consumes the iterator and returns all the decoded values contained within the range.
func (i Iterator[K, V]) Values() ([]V, error) {
	defer i.Close()

	var values []V
	for ; i.iter.Valid(); i.iter.Next() {
		value, err := i.Value()
		if err != nil {
			return nil, err
		}
		values = append(values, value)
	}
	return values, nil
}

// Keys fully consumes the iterator and returns all the decoded keys contained within the range.
func (i Iterator[K, V]) Keys() ([]K, error) {
	defer i.Close()

	var keys []K
	for ; i.iter.Valid(); i.iter.Next() {
		key, err := i.Key()
		if err != nil {
			return nil, err
		}
		keys = append(keys, key)
	}
	return keys, nil
}

// KeyValue returns the current key and value decoded.
func (i Iterator[K, V]) KeyValue() (kv KeyValue[K, V], err error) {
	key, err := i.Key()
	if err != nil {
		return kv, err
	}
	value, err := i.Value()
	if err != nil {
		return kv, err
	}
	kv.Key = key
	kv.Value = value
	return kv, nil
}

// KeyValues fully consumes the iterator and returns the list of key and values within the iterator range.
func (i Iterator[K, V]) KeyValues() ([]KeyValue[K, V], error) {
	defer i.Close()

	var kvs []KeyValue[K, V]
	for ; i.iter.Valid(); i.iter.Next() {
		kv, err := i.KeyValue()
		if err != nil {
			return nil, err
		}
		kvs = append(kvs, kv)
	}

	return kvs, nil
}

func (i Iterator[K, V]) Close() error { return i.iter.Close() }
func (i Iterator[K, V]) Next()        { i.iter.Next() }
func (i Iterator[K, V]) Valid() bool  { return i.iter.Valid() }

// KeyValue represent a Key and Value pair of an iteration.
type KeyValue[K, V any] struct {
	Key   K
	Value V
}

// encodeRangeBound encodes a range bound, modifying the key bytes to adhere to bound semantics.
func encodeRangeBound[T any](prefix []byte, keyCodec codec.KeyCodec[T], bound *RangeKey[T]) ([]byte, error) {
	key, err := EncodeKeyWithPrefix(prefix, keyCodec, bound.key)
	if err != nil {
		return nil, err
	}
	switch bound.kind {
	case rangeKeyExact:
		return key, nil
	case rangeKeyNext:
		return nextBytesKey(key), nil
	case rangeKeyPrefixEnd:
		return nextBytesPrefixKey(key), nil
	default:
		panic("undefined bound kind")
	}
}

// nextBytesKey returns the next byte key after this one.
func nextBytesKey(b []byte) []byte {
	return append(b, 0)
}

// nextBytesPrefixKey returns the []byte that would end a
// range query for all []byte with a certain prefix
// Deals with last byte of prefix being FF without overflowing
func nextBytesPrefixKey(prefix []byte) []byte {
	if len(prefix) == 0 {
		return nil
	}

	end := make([]byte, len(prefix))
	copy(end, prefix)

	for {
		if end[len(end)-1] != byte(255) {
			end[len(end)-1]++
			break
		}

		end = end[:len(end)-1]

		if len(end) == 0 {
			end = nil
			break
		}
	}

	return end
}
