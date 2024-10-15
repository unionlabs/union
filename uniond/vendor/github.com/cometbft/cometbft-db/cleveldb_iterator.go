//go:build cleveldb
// +build cleveldb

package db

import (
	"bytes"

	"github.com/jmhodges/levigo"
)

// cLevelDBIterator is a cLevelDB iterator.
type cLevelDBIterator struct {
	source     *levigo.Iterator
	start, end []byte
	isReverse  bool
	isInvalid  bool
}

var _ Iterator = (*cLevelDBIterator)(nil)

func newCLevelDBIterator(source *levigo.Iterator, start, end []byte, isReverse bool) *cLevelDBIterator {
	if isReverse {
		if end == nil || len(end) == 0 {
			source.SeekToLast()
		} else {
			source.Seek(end)
			if source.Valid() {
				eoakey := source.Key() // end or after key
				if bytes.Compare(end, eoakey) <= 0 {
					source.Prev()
				}
			} else {
				source.SeekToLast()
			}
		}
	} else {
		if start == nil || len(start) == 0 {
			source.SeekToFirst()
		} else {
			source.Seek(start)
		}
	}
	return &cLevelDBIterator{
		source:    source,
		start:     start,
		end:       end,
		isReverse: isReverse,
		isInvalid: false,
	}
}

// Domain implements Iterator.
func (itr cLevelDBIterator) Domain() ([]byte, []byte) {
	return itr.start, itr.end
}

// Valid implements Iterator.
func (itr cLevelDBIterator) Valid() bool {
	// Once invalid, forever invalid.
	if itr.isInvalid {
		return false
	}

	// If source errors, invalid.
	if itr.source.GetError() != nil {
		itr.isInvalid = true
		return false
	}

	// If source is invalid, invalid.
	if !itr.source.Valid() {
		itr.isInvalid = true
		return false
	}

	// If key is end or past it, invalid.
	start := itr.start
	end := itr.end
	key := itr.source.Key()
	if itr.isReverse {
		if start != nil && bytes.Compare(key, start) < 0 {
			itr.isInvalid = true
			return false
		}
	} else {
		if end != nil && bytes.Compare(end, key) <= 0 {
			itr.isInvalid = true
			return false
		}
	}

	// It's valid.
	return true
}

// Key implements Iterator.
// The caller should not modify the contents of the returned slice.
// Instead, the caller should make a copy and work on the copy.
func (itr cLevelDBIterator) Key() []byte {
	itr.assertIsValid()
	return itr.source.Key()
}

// Value implements Iterator.
// The caller should not modify the contents of the returned slice.
// Instead, the caller should make a copy and work on the copy.
func (itr cLevelDBIterator) Value() []byte {
	itr.assertIsValid()
	return itr.source.Value()
}

// Next implements Iterator.
func (itr cLevelDBIterator) Next() {
	itr.assertIsValid()
	if itr.isReverse {
		itr.source.Prev()
	} else {
		itr.source.Next()
	}
}

// Error implements Iterator.
func (itr cLevelDBIterator) Error() error {
	return itr.source.GetError()
}

// Close implements Iterator.
func (itr cLevelDBIterator) Close() error {
	itr.source.Close()
	return nil
}

func (itr cLevelDBIterator) assertIsValid() {
	if !itr.Valid() {
		panic("iterator is invalid")
	}
}
