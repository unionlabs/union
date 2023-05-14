package db

import (
	"bytes"
	"fmt"
)

// IteratePrefix is a convenience function for iterating over a key domain
// restricted by prefix.
func IteratePrefix(db DB, prefix []byte) (Iterator, error) {
	var start, end []byte
	if len(prefix) == 0 {
		start = nil
		end = nil
	} else {
		start = cp(prefix)
		end = cpIncr(prefix)
	}
	itr, err := db.Iterator(start, end)
	if err != nil {
		return nil, err
	}
	return itr, nil
}

// Strips prefix while iterating from Iterator.
type prefixDBIterator struct {
	prefix []byte
	start  []byte
	end    []byte
	source Iterator
	valid  bool
	err    error
}

var _ Iterator = (*prefixDBIterator)(nil)

func newPrefixIterator(prefix, start, end []byte, source Iterator) (*prefixDBIterator, error) {
	pitrInvalid := &prefixDBIterator{
		prefix: prefix,
		start:  start,
		end:    end,
		source: source,
		valid:  false,
	}

	// Empty keys are not allowed, so if a key exists in the database that exactly matches the
	// prefix we need to skip it.
	if source.Valid() && bytes.Equal(source.Key(), prefix) {
		source.Next()
	}

	if !source.Valid() || !bytes.HasPrefix(source.Key(), prefix) {
		return pitrInvalid, nil
	}

	return &prefixDBIterator{
		prefix: prefix,
		start:  start,
		end:    end,
		source: source,
		valid:  true,
	}, nil
}

// Domain implements Iterator.
func (itr *prefixDBIterator) Domain() (start []byte, end []byte) {
	return itr.start, itr.end
}

// Valid implements Iterator.
func (itr *prefixDBIterator) Valid() bool {
	if !itr.valid || itr.err != nil || !itr.source.Valid() {
		return false
	}

	key := itr.source.Key()
	if len(key) < len(itr.prefix) || !bytes.Equal(key[:len(itr.prefix)], itr.prefix) {
		itr.err = fmt.Errorf("received invalid key from backend: %x (expected prefix %x)",
			key, itr.prefix)
		return false
	}

	return true
}

// Next implements Iterator.
func (itr *prefixDBIterator) Next() {
	itr.assertIsValid()
	itr.source.Next()

	if !itr.source.Valid() || !bytes.HasPrefix(itr.source.Key(), itr.prefix) {
		itr.valid = false

	} else if bytes.Equal(itr.source.Key(), itr.prefix) {
		// Empty keys are not allowed, so if a key exists in the database that exactly matches the
		// prefix we need to skip it.
		itr.Next()
	}
}

// Next implements Iterator.
func (itr *prefixDBIterator) Key() []byte {
	itr.assertIsValid()
	key := itr.source.Key()
	return key[len(itr.prefix):] // we have checked the key in Valid()
}

// Value implements Iterator.
func (itr *prefixDBIterator) Value() []byte {
	itr.assertIsValid()
	return itr.source.Value()
}

// Error implements Iterator.
func (itr *prefixDBIterator) Error() error {
	if err := itr.source.Error(); err != nil {
		return err
	}
	return itr.err
}

// Close implements Iterator.
func (itr *prefixDBIterator) Close() error {
	return itr.source.Close()
}

func (itr *prefixDBIterator) assertIsValid() {
	if !itr.Valid() {
		panic("iterator is invalid")
	}
}
