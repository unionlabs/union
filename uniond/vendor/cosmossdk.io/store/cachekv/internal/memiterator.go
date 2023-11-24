package internal

import (
	"bytes"
	"errors"

	"github.com/tidwall/btree"

	"cosmossdk.io/store/types"
)

var _ types.Iterator = (*memIterator)(nil)

// memIterator iterates over iterKVCache items.
// if value is nil, means it was deleted.
// Implements Iterator.
type memIterator struct {
	iter btree.IterG[item]

	start     []byte
	end       []byte
	ascending bool
	valid     bool
}

func newMemIterator(start, end []byte, items BTree, ascending bool) *memIterator {
	iter := items.tree.Iter()
	var valid bool
	if ascending {
		if start != nil {
			valid = iter.Seek(newItem(start, nil))
		} else {
			valid = iter.First()
		}
	} else {
		if end != nil {
			valid = iter.Seek(newItem(end, nil))
			if !valid {
				valid = iter.Last()
			} else {
				// end is exclusive
				valid = iter.Prev()
			}
		} else {
			valid = iter.Last()
		}
	}

	mi := &memIterator{
		iter:      iter,
		start:     start,
		end:       end,
		ascending: ascending,
		valid:     valid,
	}

	if mi.valid {
		mi.valid = mi.keyInRange(mi.Key())
	}

	return mi
}

func (mi *memIterator) Domain() (start, end []byte) {
	return mi.start, mi.end
}

func (mi *memIterator) Close() error {
	mi.iter.Release()
	return nil
}

func (mi *memIterator) Error() error {
	if !mi.Valid() {
		return errors.New("invalid memIterator")
	}
	return nil
}

func (mi *memIterator) Valid() bool {
	return mi.valid
}

func (mi *memIterator) Next() {
	mi.assertValid()

	if mi.ascending {
		mi.valid = mi.iter.Next()
	} else {
		mi.valid = mi.iter.Prev()
	}

	if mi.valid {
		mi.valid = mi.keyInRange(mi.Key())
	}
}

func (mi *memIterator) keyInRange(key []byte) bool {
	if mi.ascending && mi.end != nil && bytes.Compare(key, mi.end) >= 0 {
		return false
	}
	if !mi.ascending && mi.start != nil && bytes.Compare(key, mi.start) < 0 {
		return false
	}
	return true
}

func (mi *memIterator) Key() []byte {
	return mi.iter.Item().key
}

func (mi *memIterator) Value() []byte {
	return mi.iter.Item().value
}

func (mi *memIterator) assertValid() {
	if err := mi.Error(); err != nil {
		panic(err)
	}
}
