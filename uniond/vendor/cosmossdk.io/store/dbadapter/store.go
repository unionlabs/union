package dbadapter

import (
	"io"

	dbm "github.com/cosmos/cosmos-db"

	"cosmossdk.io/store/cachekv"
	"cosmossdk.io/store/tracekv"
	"cosmossdk.io/store/types"
)

// Wrapper type for dbm.Db with implementation of KVStore
type Store struct {
	dbm.DB
}

// Get wraps the underlying DB's Get method panicing on error.
func (dsa Store) Get(key []byte) []byte {
	v, err := dsa.DB.Get(key)
	if err != nil {
		panic(err)
	}

	return v
}

// Has wraps the underlying DB's Has method panicing on error.
func (dsa Store) Has(key []byte) bool {
	ok, err := dsa.DB.Has(key)
	if err != nil {
		panic(err)
	}

	return ok
}

// Set wraps the underlying DB's Set method panicing on error.
func (dsa Store) Set(key, value []byte) {
	types.AssertValidKey(key)
	types.AssertValidValue(value)
	if err := dsa.DB.Set(key, value); err != nil {
		panic(err)
	}
}

// Delete wraps the underlying DB's Delete method panicing on error.
func (dsa Store) Delete(key []byte) {
	if err := dsa.DB.Delete(key); err != nil {
		panic(err)
	}
}

// Iterator wraps the underlying DB's Iterator method panicing on error.
func (dsa Store) Iterator(start, end []byte) types.Iterator {
	iter, err := dsa.DB.Iterator(start, end)
	if err != nil {
		panic(err)
	}

	return iter
}

// ReverseIterator wraps the underlying DB's ReverseIterator method panicing on error.
func (dsa Store) ReverseIterator(start, end []byte) types.Iterator {
	iter, err := dsa.DB.ReverseIterator(start, end)
	if err != nil {
		panic(err)
	}

	return iter
}

// GetStoreType returns the type of the store.
func (Store) GetStoreType() types.StoreType {
	return types.StoreTypeDB
}

// CacheWrap branches the underlying store.
func (dsa Store) CacheWrap() types.CacheWrap {
	return cachekv.NewStore(dsa)
}

// CacheWrapWithTrace implements KVStore.
func (dsa Store) CacheWrapWithTrace(w io.Writer, tc types.TraceContext) types.CacheWrap {
	return cachekv.NewStore(tracekv.NewStore(dsa, w, tc))
}

// dbm.DB implements KVStore so we can CacheKVStore it.
var _ types.KVStore = Store{}
