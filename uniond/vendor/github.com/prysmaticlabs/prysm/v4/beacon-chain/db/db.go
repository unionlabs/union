// Package db defines the ability to create a new database
// for an Ethereum Beacon Node.
package db

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/kv"
)

// NewFileName uses the KVStoreDatafilePath so that if this layer of
// indirection between db.NewDB->kv.NewKVStore ever changes, it will be easy to remember
// to also change this filename indirection at the same time.
func NewFileName(dirPath string) string {
	return kv.StoreDatafilePath(dirPath)
}
