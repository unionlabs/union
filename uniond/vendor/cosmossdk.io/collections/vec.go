package collections

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections/codec"
)

var (
	// ErrEmptyVec is returned when trying to pop an element from an empty Vec.
	ErrEmptyVec = errors.New("vec is empty")
	// ErrOutOfBounds is returned when trying to do an operation on an index that is out of bounds.
	ErrOutOfBounds = errors.New("vec index is out of bounds")
)

const (
	VecElementsNameSuffix   = "_elements"
	VecLengthNameSuffix     = "_length"
	VecElementsPrefixSuffix = 0x0
	VecLengthPrefixSuffix   = 0x1
)

// NewVec creates a new Vec instance. Since Vec relies on two collections, one for the length
// and the other for the elements, it will register two state objects on the schema builder.
// The first is the length which is an item, whose prefix is the provided prefix with a suffix
// which equals to VecLengthPrefixSuffix, the name is also suffixed with VecLengthNameSuffix.
// The second is the elements which is a map, whose prefix is the provided prefix with a suffix
// which equals to VecElementsPrefixSuffix, the name is also suffixed with VecElementsNameSuffix.
func NewVec[T any](sb *SchemaBuilder, prefix Prefix, name string, vc codec.ValueCodec[T]) Vec[T] {
	return Vec[T]{
		length:   NewItem(sb, append(prefix, VecLengthPrefixSuffix), name+VecLengthNameSuffix, Uint64Value),
		elements: NewMap(sb, append(prefix, VecElementsPrefixSuffix), name+VecElementsNameSuffix, Uint64Key, vc),
	}
}

// Vec works like a slice sitting on top of a KVStore.
// It relies on two collections, one for the length which is an Item[uint64],
// the other for the elements which is a Map[uint64, T].
type Vec[T any] struct {
	length   Item[uint64]
	elements Map[uint64, T]
}

// Push adds an element to the end of the Vec.
func (v Vec[T]) Push(ctx context.Context, elem T) error {
	length, err := v.length.Get(ctx)
	if err != nil && !errors.Is(err, ErrNotFound) {
		return err
	}
	err = v.elements.Set(ctx, length, elem)
	if err != nil {
		return err
	}
	err = v.length.Set(ctx, length+1)
	if err != nil {
		return err
	}
	return nil
}

// Pop removes an element from the end of the Vec and returns it. Fails
// if the Vec is empty.
func (v Vec[T]) Pop(ctx context.Context) (elem T, err error) {
	length, err := v.length.Get(ctx)
	if err != nil && !errors.Is(err, ErrNotFound) {
		return elem, err
	}
	if length == 0 {
		return elem, ErrEmptyVec
	}
	length -= 1
	elem, err = v.elements.Get(ctx, length)
	if err != nil {
		return elem, err
	}
	err = v.elements.Remove(ctx, length)
	if err != nil {
		return elem, err
	}
	err = v.length.Set(ctx, length)
	if err != nil {
		return elem, err
	}
	return elem, nil
}

// Replace replaces an element at a given index. Fails if the index is out of bounds.
func (v Vec[T]) Replace(ctx context.Context, index uint64, elem T) error {
	length, err := v.length.Get(ctx)
	if err != nil && !errors.Is(err, ErrNotFound) {
		return err
	}
	if index >= length {
		return fmt.Errorf("%w: length %d", ErrOutOfBounds, length)
	}
	return v.elements.Set(ctx, index, elem)
}

// Get returns an element at a given index. Returns ErrOutOfBounds
// if the index is out of bounds.
func (v Vec[T]) Get(ctx context.Context, index uint64) (elem T, err error) {
	elem, err = v.elements.Get(ctx, index)
	switch {
	case err == nil:
		return elem, nil
	case errors.Is(err, ErrNotFound):
		return elem, fmt.Errorf("%w: index %d", ErrOutOfBounds, index)
	default:
		return elem, err
	}
}

// Len returns the length of the Vec.
func (v Vec[T]) Len(ctx context.Context) (uint64, error) {
	length, err := v.length.Get(ctx)
	switch {
	// no error, return length as the vec is populated
	case err == nil:
		return length, nil
	// not found, return 0 as the vec is empty
	case errors.Is(err, ErrNotFound):
		return 0, nil
	// something else happened
	default:
		return 0, err
	}
}

// Iterate iterates over the Vec. It returns an Iterator whose key is the index
// and the value is the element at that index.
func (v Vec[T]) Iterate(ctx context.Context, rng Ranger[uint64]) (Iterator[uint64, T], error) {
	return v.elements.Iterate(ctx, rng)
}

// Walk walks over the Vec. It calls the walkFn for each element in the Vec,
// where the key is the index and the value is the element at that index.
func (v Vec[T]) Walk(ctx context.Context, rng Ranger[uint64], walkFn func(index uint64, elem T) (stop bool, err error)) error {
	return v.elements.Walk(ctx, rng, walkFn)
}
