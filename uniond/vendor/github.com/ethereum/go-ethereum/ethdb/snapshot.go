// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package ethdb

type Snapshot interface {
	// Has retrieves if a key is present in the snapshot backing by a key-value
	// data store.
	Has(key []byte) (bool, error)

	// Get retrieves the given key if it's present in the snapshot backing by
	// key-value data store.
	Get(key []byte) ([]byte, error)

	// Release releases associated resources. Release should always succeed and can
	// be called multiple times without causing error.
	Release()
}

// Snapshotter wraps the Snapshot method of a backing data store.
type Snapshotter interface {
	// NewSnapshot creates a database snapshot based on the current state.
	// The created snapshot will not be affected by all following mutations
	// happened on the database.
	// Note don't forget to release the snapshot once it's used up, otherwise
	// the stale data will never be cleaned up by the underlying compactor.
	NewSnapshot() (Snapshot, error)
}
