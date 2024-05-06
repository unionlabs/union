package wrapper

import (
	dbm "github.com/cosmos/cosmos-db"
	idb "github.com/cosmos/iavl/db"
)

var _ idb.DB = &DBWrapper{}

// DBwrapper is a simple wrapper of dbm.DB that implements the iavl.DB interface.
type DBWrapper struct {
	dbm.DB
}

// NewDBWrapper creates a new DBWrapper instance.
func NewDBWrapper(db dbm.DB) *DBWrapper {
	return &DBWrapper{db}
}

func (dbw *DBWrapper) NewBatch() idb.Batch {
	return dbw.DB.NewBatch()
}

func (dbw *DBWrapper) NewBatchWithSize(size int) idb.Batch {
	return dbw.DB.NewBatchWithSize(size)
}

func (dbw *DBWrapper) Iterator(start, end []byte) (idb.Iterator, error) {
	return dbw.DB.Iterator(start, end)
}

func (dbw *DBWrapper) ReverseIterator(start, end []byte) (idb.Iterator, error) {
	return dbw.DB.ReverseIterator(start, end)
}
