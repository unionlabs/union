package indexes

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/codec"
)

// Unique identifies an index that imposes uniqueness constraints on the reference key.
// It creates relationships between reference and primary key of the value.
type Unique[ReferenceKey, PrimaryKey, Value any] struct {
	getRefKey func(PrimaryKey, Value) (ReferenceKey, error)
	refKeys   collections.Map[ReferenceKey, PrimaryKey]
}

// NewUnique instantiates a new Unique index.
func NewUnique[ReferenceKey, PrimaryKey, Value any](
	schema *collections.SchemaBuilder,
	prefix collections.Prefix,
	name string,
	refCodec codec.KeyCodec[ReferenceKey],
	pkCodec codec.KeyCodec[PrimaryKey],
	getRefKeyFunc func(pk PrimaryKey, v Value) (ReferenceKey, error),
) *Unique[ReferenceKey, PrimaryKey, Value] {
	return &Unique[ReferenceKey, PrimaryKey, Value]{
		getRefKey: getRefKeyFunc,
		refKeys:   collections.NewMap(schema, prefix, name, refCodec, codec.KeyToValueCodec(pkCodec)),
	}
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) Reference(ctx context.Context, pk PrimaryKey, newValue Value, lazyOldValue func() (Value, error)) error {
	oldValue, err := lazyOldValue()
	switch {
	// if no error it means the value existed, and we need to remove the old indexes
	case err == nil:
		err = i.unreference(ctx, pk, oldValue)
		if err != nil {
			return err
		}
	// if error is ErrNotFound, it means that the object does not exist, so we're creating indexes for the first time.
	// we do nothing.
	case errors.Is(err, collections.ErrNotFound):
	// default case means that there was some other error
	default:
		return err
	}
	// create new indexes, asserting no uniqueness constraint violation
	refKey, err := i.getRefKey(pk, newValue)
	if err != nil {
		return err
	}
	has, err := i.refKeys.Has(ctx, refKey)
	if err != nil {
		return err
	}
	if has {
		return fmt.Errorf("%w: index uniqueness constrain violation: %s", collections.ErrConflict, i.refKeys.KeyCodec().Stringify(refKey))
	}
	return i.refKeys.Set(ctx, refKey, pk)
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) Unreference(ctx context.Context, pk PrimaryKey, getValue func() (Value, error)) error {
	value, err := getValue()
	if err != nil {
		return err
	}
	return i.unreference(ctx, pk, value)
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) unreference(ctx context.Context, pk PrimaryKey, value Value) error {
	refKey, err := i.getRefKey(pk, value)
	if err != nil {
		return err
	}
	return i.refKeys.Remove(ctx, refKey)
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) MatchExact(ctx context.Context, ref ReferenceKey) (PrimaryKey, error) {
	return i.refKeys.Get(ctx, ref)
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) Iterate(ctx context.Context, ranger collections.Ranger[ReferenceKey]) (UniqueIterator[ReferenceKey, PrimaryKey], error) {
	iter, err := i.refKeys.Iterate(ctx, ranger)
	return (UniqueIterator[ReferenceKey, PrimaryKey])(iter), err
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) Walk(
	ctx context.Context,
	ranger collections.Ranger[ReferenceKey],
	walkFunc func(indexingKey ReferenceKey, indexedKey PrimaryKey) (stop bool, err error),
) error {
	return i.refKeys.Walk(ctx, ranger, walkFunc)
}

func (i *Unique[ReferenceKey, PrimaryKey, Value]) IterateRaw(ctx context.Context, start, end []byte, order collections.Order) (u UniqueIterator[ReferenceKey, PrimaryKey], err error) {
	iter, err := i.refKeys.IterateRaw(ctx, start, end, order)
	if err != nil {
		return
	}
	return (UniqueIterator[ReferenceKey, PrimaryKey])(iter), nil
}

// UniqueIterator is an Iterator wrapper, that exposes only the functionality needed to work with Unique keys.
type UniqueIterator[ReferenceKey, PrimaryKey any] collections.Iterator[ReferenceKey, PrimaryKey]

// PrimaryKey returns the iterator's current primary key.
func (i UniqueIterator[ReferenceKey, PrimaryKey]) PrimaryKey() (PrimaryKey, error) {
	return (collections.Iterator[ReferenceKey, PrimaryKey])(i).Value()
}

// PrimaryKeys fully consumes the iterator, and returns all the primary keys.
func (i UniqueIterator[ReferenceKey, PrimaryKey]) PrimaryKeys() ([]PrimaryKey, error) {
	return (collections.Iterator[ReferenceKey, PrimaryKey])(i).Values()
}

// FullKey returns the iterator's current full reference key as Pair[ReferenceKey, PrimaryKey].
func (i UniqueIterator[ReferenceKey, PrimaryKey]) FullKey() (collections.Pair[ReferenceKey, PrimaryKey], error) {
	kv, err := (collections.Iterator[ReferenceKey, PrimaryKey])(i).KeyValue()
	return collections.Join(kv.Key, kv.Value), err
}

func (i UniqueIterator[ReferenceKey, PrimaryKey]) FullKeys() ([]collections.Pair[ReferenceKey, PrimaryKey], error) {
	kvs, err := (collections.Iterator[ReferenceKey, PrimaryKey])(i).KeyValues()
	if err != nil {
		return nil, err
	}
	pairKeys := make([]collections.Pair[ReferenceKey, PrimaryKey], len(kvs))
	for index := range kvs {
		kv := kvs[index]
		pairKeys[index] = collections.Join(kv.Key, kv.Value)
	}
	return pairKeys, nil
}

func (i UniqueIterator[ReferenceKey, PrimaryKey]) Next() {
	(collections.Iterator[ReferenceKey, PrimaryKey])(i).Next()
}

func (i UniqueIterator[ReferenceKey, PrimaryKey]) Valid() bool {
	return (collections.Iterator[ReferenceKey, PrimaryKey])(i).Valid()
}

func (i UniqueIterator[ReferenceKey, PrimaryKey]) Close() error {
	return (collections.Iterator[ReferenceKey, PrimaryKey])(i).Close()
}
