// Copyright 2020 The go-ethereum Authors
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
	"bytes"
	"testing"

	"github.com/ethereum/go-ethereum/ethdb"
)

func TestTableDatabase(t *testing.T)            { testTableDatabase(t, "prefix") }
func TestEmptyPrefixTableDatabase(t *testing.T) { testTableDatabase(t, "") }

type testReplayer struct {
	puts [][]byte
	dels [][]byte
}

func (r *testReplayer) Put(key []byte, value []byte) error {
	r.puts = append(r.puts, key)
	return nil
}

func (r *testReplayer) Delete(key []byte) error {
	r.dels = append(r.dels, key)
	return nil
}

func testTableDatabase(t *testing.T, prefix string) {
	db := NewTable(NewMemoryDatabase(), prefix)

	var entries = []struct {
		key   []byte
		value []byte
	}{
		{[]byte{0x01, 0x02}, []byte{0x0a, 0x0b}},
		{[]byte{0x03, 0x04}, []byte{0x0c, 0x0d}},
		{[]byte{0x05, 0x06}, []byte{0x0e, 0x0f}},

		{[]byte{0xff, 0xff, 0x01}, []byte{0x1a, 0x1b}},
		{[]byte{0xff, 0xff, 0x02}, []byte{0x1c, 0x1d}},
		{[]byte{0xff, 0xff, 0x03}, []byte{0x1e, 0x1f}},
	}

	// Test Put/Get operation
	for _, entry := range entries {
		db.Put(entry.key, entry.value)
	}
	for _, entry := range entries {
		got, err := db.Get(entry.key)
		if err != nil {
			t.Fatalf("Failed to get value: %v", err)
		}
		if !bytes.Equal(got, entry.value) {
			t.Fatalf("Value mismatch: want=%v, got=%v", entry.value, got)
		}
	}

	// Test batch operation
	db = NewTable(NewMemoryDatabase(), prefix)
	batch := db.NewBatch()
	for _, entry := range entries {
		batch.Put(entry.key, entry.value)
	}
	batch.Write()
	for _, entry := range entries {
		got, err := db.Get(entry.key)
		if err != nil {
			t.Fatalf("Failed to get value: %v", err)
		}
		if !bytes.Equal(got, entry.value) {
			t.Fatalf("Value mismatch: want=%v, got=%v", entry.value, got)
		}
	}

	// Test batch replayer
	r := &testReplayer{}
	batch.Replay(r)
	for index, entry := range entries {
		got := r.puts[index]
		if !bytes.Equal(got, entry.key) {
			t.Fatalf("Key mismatch: want=%v, got=%v", entry.key, got)
		}
	}

	check := func(iter ethdb.Iterator, expCount, index int) {
		count := 0
		for iter.Next() {
			key, value := iter.Key(), iter.Value()
			if !bytes.Equal(key, entries[index].key) {
				t.Fatalf("Key mismatch: want=%v, got=%v", entries[index].key, key)
			}
			if !bytes.Equal(value, entries[index].value) {
				t.Fatalf("Value mismatch: want=%v, got=%v", entries[index].value, value)
			}
			index += 1
			count++
		}
		if count != expCount {
			t.Fatalf("Wrong number of elems, exp %d got %d", expCount, count)
		}
		iter.Release()
	}
	// Test iterators
	check(db.NewIterator(nil, nil), 6, 0)
	// Test iterators with prefix
	check(db.NewIterator([]byte{0xff, 0xff}, nil), 3, 3)
	// Test iterators with start point
	check(db.NewIterator(nil, []byte{0xff, 0xff, 0x02}), 2, 4)
	// Test iterators with prefix and start point
	check(db.NewIterator([]byte{0xee}, nil), 0, 0)
	check(db.NewIterator(nil, []byte{0x00}), 6, 0)
}
