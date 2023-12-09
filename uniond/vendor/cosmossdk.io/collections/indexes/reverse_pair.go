package indexes

import (
	"context"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/codec"
)

type reversePairOptions struct {
	uncheckedValue bool
}

// WithReversePairUncheckedValue is an option that can be passed to NewReversePair to
// ignore index values different from '[]byte{}' and continue with the operation.
// This should be used only if you are migrating to collections and have used a different
// placeholder value in your storage index keys.
// Refer to WithKeySetUncheckedValue for more information.
func WithReversePairUncheckedValue() func(*reversePairOptions) {
	return func(o *reversePairOptions) {
		o.uncheckedValue = true
	}
}

// ReversePair is an index that is used with collections.Pair keys. It indexes objects by their second part of the key.
// When the value is being indexed by collections.IndexedMap then ReversePair will create a relationship between
// the second part of the primary key and the first part.
type ReversePair[K1, K2, Value any] struct {
	refKeys collections.KeySet[collections.Pair[K2, K1]] // refKeys has the relationships between Join(K2, K1)
}

// TODO(tip): this is an interface to cast a collections.KeyCodec
// to a pair codec. currently we return it as a KeyCodec[Pair[K1, K2]]
// to improve dev experience with type inference, which means we cannot
// get the concrete implementation which exposes KeyCodec1 and KeyCodec2.
type pairKeyCodec[K1, K2 any] interface {
	KeyCodec1() codec.KeyCodec[K1]
	KeyCodec2() codec.KeyCodec[K2]
}

// NewReversePair instantiates a new ReversePair index.
// NOTE: when using this function you will need to type hint: doing NewReversePair[Value]()
// Example: if the value of the indexed map is string, you need to do NewReversePair[string](...)
func NewReversePair[Value, K1, K2 any](
	sb *collections.SchemaBuilder,
	prefix collections.Prefix,
	name string,
	pairCodec codec.KeyCodec[collections.Pair[K1, K2]],
	options ...func(*reversePairOptions),
) *ReversePair[K1, K2, Value] {
	pkc := pairCodec.(pairKeyCodec[K1, K2])
	o := new(reversePairOptions)
	for _, option := range options {
		option(o)
	}
	if o.uncheckedValue {
		return &ReversePair[K1, K2, Value]{
			refKeys: collections.NewKeySet(sb, prefix, name, collections.PairKeyCodec(pkc.KeyCodec2(), pkc.KeyCodec1()), collections.WithKeySetUncheckedValue()),
		}
	}

	mi := &ReversePair[K1, K2, Value]{
		refKeys: collections.NewKeySet(sb, prefix, name, collections.PairKeyCodec(pkc.KeyCodec2(), pkc.KeyCodec1())),
	}

	return mi
}

// Iterate exposes the raw iterator API.
func (i *ReversePair[K1, K2, Value]) Iterate(ctx context.Context, ranger collections.Ranger[collections.Pair[K2, K1]]) (iter ReversePairIterator[K2, K1], err error) {
	sIter, err := i.refKeys.Iterate(ctx, ranger)
	if err != nil {
		return
	}
	return (ReversePairIterator[K2, K1])(sIter), nil
}

// MatchExact will return an iterator containing only the primary keys starting with the provided second part of the multipart pair key.
func (i *ReversePair[K1, K2, Value]) MatchExact(ctx context.Context, key K2) (ReversePairIterator[K2, K1], error) {
	return i.Iterate(ctx, collections.NewPrefixedPairRange[K2, K1](key))
}

// Reference implements collections.Index
func (i *ReversePair[K1, K2, Value]) Reference(ctx context.Context, pk collections.Pair[K1, K2], _ Value, _ func() (Value, error)) error {
	return i.refKeys.Set(ctx, collections.Join(pk.K2(), pk.K1()))
}

// Unreference implements collections.Index
func (i *ReversePair[K1, K2, Value]) Unreference(ctx context.Context, pk collections.Pair[K1, K2], _ func() (Value, error)) error {
	return i.refKeys.Remove(ctx, collections.Join(pk.K2(), pk.K1()))
}

func (i *ReversePair[K1, K2, Value]) Walk(
	ctx context.Context,
	ranger collections.Ranger[collections.Pair[K2, K1]],
	walkFunc func(indexingKey K2, indexedKey K1) (stop bool, err error),
) error {
	return i.refKeys.Walk(ctx, ranger, func(key collections.Pair[K2, K1]) (bool, error) {
		return walkFunc(key.K1(), key.K2())
	})
}

func (i *ReversePair[K1, K2, Value]) IterateRaw(
	ctx context.Context, start, end []byte, order collections.Order,
) (
	iter collections.Iterator[collections.Pair[K2, K1], collections.NoValue], err error,
) {
	return i.refKeys.IterateRaw(ctx, start, end, order)
}

func (i *ReversePair[K1, K2, Value]) KeyCodec() codec.KeyCodec[collections.Pair[K2, K1]] {
	return i.refKeys.KeyCodec()
}

// ReversePairIterator is a helper type around a collections.KeySetIterator when used to work
// with ReversePair indexes iterations.
type ReversePairIterator[K2, K1 any] collections.KeySetIterator[collections.Pair[K2, K1]]

// PrimaryKey returns the primary key from the index. The index is composed like a reverse
// pair key. So we just fetch the pair key from the index and return the reverse.
func (m ReversePairIterator[K2, K1]) PrimaryKey() (pair collections.Pair[K1, K2], err error) {
	reversePair, err := m.FullKey()
	if err != nil {
		return pair, err
	}
	pair = collections.Join(reversePair.K2(), reversePair.K1())
	return pair, nil
}

// PrimaryKeys returns all the primary keys contained in the iterator.
func (m ReversePairIterator[K2, K1]) PrimaryKeys() (pairs []collections.Pair[K1, K2], err error) {
	defer m.Close()
	for ; m.Valid(); m.Next() {
		pair, err := m.PrimaryKey()
		if err != nil {
			return nil, err
		}
		pairs = append(pairs, pair)
	}
	return pairs, err
}

func (m ReversePairIterator[K2, K1]) FullKey() (p collections.Pair[K2, K1], err error) {
	return (collections.KeySetIterator[collections.Pair[K2, K1]])(m).Key()
}

func (m ReversePairIterator[K2, K1]) Next() {
	(collections.KeySetIterator[collections.Pair[K2, K1]])(m).Next()
}

func (m ReversePairIterator[K2, K1]) Valid() bool {
	return (collections.KeySetIterator[collections.Pair[K2, K1]])(m).Valid()
}

func (m ReversePairIterator[K2, K1]) Close() error {
	return (collections.KeySetIterator[collections.Pair[K2, K1]])(m).Close()
}
