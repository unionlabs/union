package db

import (
	dbm "github.com/cosmos/cosmos-sdk/db"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
)

var _ = (*storetypes.Iterator)(nil)

type AsStoreIter struct {
	dbm.Iterator
	valid bool
}

// DBToStoreIterator returns an iterator wrapping the given iterator so that it satisfies the
// (store/types).Iterator interface.
func ToStoreIterator(source dbm.Iterator) *AsStoreIter {
	ret := &AsStoreIter{Iterator: source}
	ret.Next() // The DB iterator must be primed before it can access the first element, because Next also returns the validity status
	return ret
}

func (it *AsStoreIter) Next()       { it.valid = it.Iterator.Next() }
func (it *AsStoreIter) Valid() bool { return it.valid }
