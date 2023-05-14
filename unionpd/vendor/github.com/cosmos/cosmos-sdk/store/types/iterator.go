package types

import (
	"fmt"
)

// KVStorePrefixIteratorPaginated returns iterator over items in the selected page.
// Items iterated and skipped in ascending order.
func KVStorePrefixIteratorPaginated(kvs KVStore, prefix []byte, page, limit uint) Iterator {
	pi := &PaginatedIterator{
		Iterator: KVStorePrefixIterator(kvs, prefix),
		page:     page,
		limit:    limit,
	}
	pi.skip()
	return pi
}

// KVStoreReversePrefixIteratorPaginated returns iterator over items in the selected page.
// Items iterated and skipped in descending order.
func KVStoreReversePrefixIteratorPaginated(kvs KVStore, prefix []byte, page, limit uint) Iterator {
	pi := &PaginatedIterator{
		Iterator: KVStoreReversePrefixIterator(kvs, prefix),
		page:     page,
		limit:    limit,
	}
	pi.skip()
	return pi
}

// PaginatedIterator is a wrapper around Iterator that iterates over values starting for given page and limit.
type PaginatedIterator struct {
	Iterator

	page, limit uint // provided during initialization
	iterated    uint // incremented in a call to Next
}

func (pi *PaginatedIterator) skip() {
	for i := (pi.page - 1) * pi.limit; i > 0 && pi.Iterator.Valid(); i-- {
		pi.Iterator.Next()
	}
}

// Next will panic after limit is reached.
func (pi *PaginatedIterator) Next() {
	if !pi.Valid() {
		panic(fmt.Sprintf("PaginatedIterator reached limit %d", pi.limit))
	}
	pi.Iterator.Next()
	pi.iterated++
}

// Valid if below limit and underlying iterator is valid.
func (pi *PaginatedIterator) Valid() bool {
	if pi.iterated >= pi.limit {
		return false
	}
	return pi.Iterator.Valid()
}
