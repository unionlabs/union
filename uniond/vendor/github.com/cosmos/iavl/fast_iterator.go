package iavl

import (
	"errors"

	dbm "github.com/cosmos/cosmos-db"

	"github.com/cosmos/iavl/fastnode"
)

var errFastIteratorNilNdbGiven = errors.New("fast iterator must be created with a nodedb but it was nil")

// FastIterator is a dbm.Iterator for ImmutableTree
// it iterates over the latest state via fast nodes,
// taking advantage of keys being located in sequence in the underlying database.
type FastIterator struct {
	start, end []byte

	valid bool

	ascending bool

	err error

	ndb *nodeDB

	nextFastNode *fastnode.Node

	fastIterator dbm.Iterator
}

var _ dbm.Iterator = (*FastIterator)(nil)

func NewFastIterator(start, end []byte, ascending bool, ndb *nodeDB) *FastIterator {
	iter := &FastIterator{
		start:        start,
		end:          end,
		err:          nil,
		ascending:    ascending,
		ndb:          ndb,
		nextFastNode: nil,
		fastIterator: nil,
	}
	// Move iterator before the first element
	iter.Next()
	return iter
}

// Domain implements dbm.Iterator.
// Maps the underlying nodedb iterator domain, to the 'logical' keys involved.
func (iter *FastIterator) Domain() ([]byte, []byte) {
	if iter.fastIterator == nil {
		return iter.start, iter.end
	}

	start, end := iter.fastIterator.Domain()

	if start != nil {
		start = start[1:]
		if len(start) == 0 {
			start = nil
		}
	}

	if end != nil {
		end = end[1:]
		if len(end) == 0 {
			end = nil
		}
	}

	return start, end
}

// Valid implements dbm.Iterator.
func (iter *FastIterator) Valid() bool {
	return iter.fastIterator != nil && iter.fastIterator.Valid() && iter.valid
}

// Key implements dbm.Iterator
func (iter *FastIterator) Key() []byte {
	if iter.valid {
		return iter.nextFastNode.GetKey()
	}
	return nil
}

// Value implements dbm.Iterator
func (iter *FastIterator) Value() []byte {
	if iter.valid {
		return iter.nextFastNode.GetValue()
	}
	return nil
}

// Next implements dbm.Iterator
func (iter *FastIterator) Next() {
	if iter.ndb == nil {
		iter.err = errFastIteratorNilNdbGiven
		iter.valid = false
		return
	}

	if iter.fastIterator == nil {
		iter.fastIterator, iter.err = iter.ndb.getFastIterator(iter.start, iter.end, iter.ascending)
		iter.valid = true
	} else {
		iter.fastIterator.Next()
	}

	if iter.err == nil {
		iter.err = iter.fastIterator.Error()
	}

	iter.valid = iter.valid && iter.fastIterator.Valid()
	if iter.valid {
		iter.nextFastNode, iter.err = fastnode.DeserializeNode(iter.fastIterator.Key()[1:], iter.fastIterator.Value())
		iter.valid = iter.err == nil
	}
}

// Close implements dbm.Iterator
func (iter *FastIterator) Close() error {
	if iter.fastIterator != nil {
		iter.err = iter.fastIterator.Close()
	}
	iter.valid = false
	iter.fastIterator = nil
	return iter.err
}

// Error implements dbm.Iterator
func (iter *FastIterator) Error() error {
	return iter.err
}
