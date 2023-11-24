package prefix

import (
	"bytes"
	"errors"
	"io"

	"cosmossdk.io/store/cachekv"
	"cosmossdk.io/store/tracekv"
	"cosmossdk.io/store/types"
)

var _ types.KVStore = Store{}

// Store is similar with cometbft/cometbft/libs/db/prefix_db
// both gives access only to the limited subset of the store
// for convinience or safety
type Store struct {
	parent types.KVStore
	prefix []byte
}

func NewStore(parent types.KVStore, prefix []byte) Store {
	return Store{
		parent: parent,
		prefix: prefix,
	}
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

// Implements Store
func (s Store) GetStoreType() types.StoreType {
	return s.parent.GetStoreType()
}

// Implements CacheWrap
func (s Store) CacheWrap() types.CacheWrap {
	return cachekv.NewStore(s)
}

// CacheWrapWithTrace implements the KVStore interface.
func (s Store) CacheWrapWithTrace(w io.Writer, tc types.TraceContext) types.CacheWrap {
	return cachekv.NewStore(tracekv.NewStore(s, w, tc))
}

// Implements KVStore
func (s Store) Get(key []byte) []byte {
	res := s.parent.Get(s.key(key))
	return res
}

// Implements KVStore
func (s Store) Has(key []byte) bool {
	return s.parent.Has(s.key(key))
}

// Implements KVStore
func (s Store) Set(key, value []byte) {
	types.AssertValidKey(key)
	types.AssertValidValue(value)
	s.parent.Set(s.key(key), value)
}

// Implements KVStore
func (s Store) Delete(key []byte) {
	s.parent.Delete(s.key(key))
}

// Implements KVStore
// Check https://github.com/cometbft/cometbft/blob/master/libs/db/prefix_db.go#L106
func (s Store) Iterator(start, end []byte) types.Iterator {
	newstart := cloneAppend(s.prefix, start)

	var newend []byte
	if end == nil {
		newend = cpIncr(s.prefix)
	} else {
		newend = cloneAppend(s.prefix, end)
	}

	iter := s.parent.Iterator(newstart, newend)

	return newPrefixIterator(s.prefix, start, end, iter)
}

// ReverseIterator implements KVStore
// Check https://github.com/cometbft/cometbft/blob/master/libs/db/prefix_db.go#L129
func (s Store) ReverseIterator(start, end []byte) types.Iterator {
	newstart := cloneAppend(s.prefix, start)

	var newend []byte
	if end == nil {
		newend = cpIncr(s.prefix)
	} else {
		newend = cloneAppend(s.prefix, end)
	}

	iter := s.parent.ReverseIterator(newstart, newend)

	return newPrefixIterator(s.prefix, start, end, iter)
}

var _ types.Iterator = (*prefixIterator)(nil)

type prefixIterator struct {
	prefix []byte
	start  []byte
	end    []byte
	iter   types.Iterator
	valid  bool
}

func newPrefixIterator(prefix, start, end []byte, parent types.Iterator) *prefixIterator {
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

// copied from github.com/cometbft/cometbft/libs/db/prefix_db.go
func stripPrefix(key, prefix []byte) []byte {
	if len(key) < len(prefix) || !bytes.Equal(key[:len(prefix)], prefix) {
		panic("should not happen")
	}

	return key[len(prefix):]
}

// wrapping types.PrefixEndBytes
func cpIncr(bz []byte) []byte {
	return types.PrefixEndBytes(bz)
}
