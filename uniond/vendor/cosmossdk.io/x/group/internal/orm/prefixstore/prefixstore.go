// Package prefixstore provides a store that prefixes all keys with a given
// prefix.
// Implementation taken from cosmossdk.io/store/prefix, and adapted to
// the cosmossdk.io/core/store.KVStore interface.
package prefixstore

import (
	"bytes"
	"errors"

	"cosmossdk.io/core/store"
)

// New creates a new prefix store using the provided bytes prefix.
func New(store store.KVStore, prefix []byte) store.KVStore {
	return Store{
		parent: store,
		prefix: prefix,
	}
}

var _ store.KVStore = Store{}

// Store is similar with cometbft/cometbft-db/blob/v1.0.1/prefixdb.go
// both gives access only to the limited subset of the store
// for convenience or safety
type Store struct {
	parent store.KVStore
	prefix []byte
}

func cloneAppend(bz, tail []byte) (res []byte) {
	res = make([]byte, len(bz)+len(tail))
	copy(res, bz)
	copy(res[len(bz):], tail)
	return
}

func (s Store) key(key []byte) (res []byte) {
	if key == nil {
		panic("nil key on Store")
	}
	res = cloneAppend(s.prefix, key)
	return
}

// Implements KVStore
func (s Store) Get(key []byte) ([]byte, error) {
	return s.parent.Get(s.key(key))
}

// Implements KVStore
func (s Store) Has(key []byte) (bool, error) {
	return s.parent.Has(s.key(key))
}

// Implements KVStore
func (s Store) Set(key, value []byte) error {
	return s.parent.Set(s.key(key), value)
}

// Implements KVStore
func (s Store) Delete(key []byte) error { return s.parent.Delete(s.key(key)) }

// Implements KVStore
// Check https://github.com/cometbft/cometbft-db/blob/v1.0.1/prefixdb.go#L109
func (s Store) Iterator(start, end []byte) (store.Iterator, error) {
	newstart := cloneAppend(s.prefix, start)

	var newend []byte
	if end == nil {
		newend = cpIncr(s.prefix)
	} else {
		newend = cloneAppend(s.prefix, end)
	}

	iter, err := s.parent.Iterator(newstart, newend)
	if err != nil {
		return nil, err
	}

	return newPrefixIterator(s.prefix, start, end, iter), nil
}

// ReverseIterator implements KVStore
// Check https://github.com/cometbft/cometbft-db/blob/v1.0.1/prefixdb.go#L132
func (s Store) ReverseIterator(start, end []byte) (store.Iterator, error) {
	newstart := cloneAppend(s.prefix, start)

	var newend []byte
	if end == nil {
		newend = cpIncr(s.prefix)
	} else {
		newend = cloneAppend(s.prefix, end)
	}

	iter, err := s.parent.ReverseIterator(newstart, newend)
	if err != nil {
		return nil, err
	}

	return newPrefixIterator(s.prefix, start, end, iter), nil
}

var _ store.Iterator = (*prefixIterator)(nil)

type prefixIterator struct {
	prefix []byte
	start  []byte
	end    []byte
	iter   store.Iterator
	valid  bool
}

func newPrefixIterator(prefix, start, end []byte, parent store.Iterator) *prefixIterator {
	return &prefixIterator{
		prefix: prefix,
		start:  start,
		end:    end,
		iter:   parent,
		valid:  parent.Valid() && bytes.HasPrefix(parent.Key(), prefix),
	}
}

// Implements Iterator
func (pi *prefixIterator) Domain() ([]byte, []byte) {
	return pi.start, pi.end
}

// Implements Iterator
func (pi *prefixIterator) Valid() bool {
	return pi.valid && pi.iter.Valid()
}

// Implements Iterator
func (pi *prefixIterator) Next() {
	if !pi.valid {
		panic("prefixIterator invalid, cannot call Next()")
	}

	if pi.iter.Next(); !pi.iter.Valid() || !bytes.HasPrefix(pi.iter.Key(), pi.prefix) {
		// TODO: shouldn't pi be set to nil instead?
		pi.valid = false
	}
}

// Implements Iterator
func (pi *prefixIterator) Key() (key []byte) {
	if !pi.valid {
		panic("prefixIterator invalid, cannot call Key()")
	}

	key = pi.iter.Key()
	key = stripPrefix(key, pi.prefix)

	return
}

// Implements Iterator
func (pi *prefixIterator) Value() []byte {
	if !pi.valid {
		panic("prefixIterator invalid, cannot call Value()")
	}

	return pi.iter.Value()
}

// Implements Iterator
func (pi *prefixIterator) Close() error {
	return pi.iter.Close()
}

// Error returns an error if the prefixIterator is invalid defined by the Valid
// method.
func (pi *prefixIterator) Error() error {
	if !pi.Valid() {
		return errors.New("invalid prefixIterator")
	}

	return nil
}

// copied from github.com/cometbft/cometbft-db/blob/v1.0.1/prefixdb.go
func stripPrefix(key, prefix []byte) []byte {
	if len(key) < len(prefix) || !bytes.Equal(key[:len(prefix)], prefix) {
		panic("should not happen")
	}

	return key[len(prefix):]
}

// wrapping types.PrefixEndBytes
func cpIncr(bz []byte) []byte {
	return prefixEndBytes(bz)
}

// prefixEndBytes returns the []byte that would end a
// range query for all []byte with a certain prefix
// Deals with last byte of prefix being FF without overflowing
func prefixEndBytes(prefix []byte) []byte {
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
