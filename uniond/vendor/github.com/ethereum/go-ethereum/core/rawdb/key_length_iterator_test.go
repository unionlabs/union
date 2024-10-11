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

package rawdb

import (
	"encoding/binary"
	"testing"
)

func TestKeyLengthIterator(t *testing.T) {
	db := NewMemoryDatabase()

	keyLen := 8
	expectedKeys := make(map[string]struct{})
	for i := 0; i < 100; i++ {
		key := make([]byte, keyLen)
		binary.BigEndian.PutUint64(key, uint64(i))
		if err := db.Put(key, []byte{0x1}); err != nil {
			t.Fatal(err)
		}
		expectedKeys[string(key)] = struct{}{}

		longerKey := make([]byte, keyLen*2)
		binary.BigEndian.PutUint64(longerKey, uint64(i))
		if err := db.Put(longerKey, []byte{0x1}); err != nil {
			t.Fatal(err)
		}
	}

	it := NewKeyLengthIterator(db.NewIterator(nil, nil), keyLen)
	for it.Next() {
		key := it.Key()
		_, exists := expectedKeys[string(key)]
		if !exists {
			t.Fatalf("Found unexpected key %d", binary.BigEndian.Uint64(key))
		}
		delete(expectedKeys, string(key))
		if len(key) != keyLen {
			t.Fatalf("Found unexpected key in key length iterator with length %d", len(key))
		}
	}

	if len(expectedKeys) != 0 {
		t.Fatalf("Expected all keys of length %d to be removed from expected keys during iteration", keyLen)
	}
}
