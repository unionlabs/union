/*
* Copyright 2019 Dgraph Labs, Inc. and Contributors
*
* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*     http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.
 */

package badger

import (
	"encoding/binary"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestGetMergeOperator(t *testing.T) {
	t.Run("Get before Add", func(t *testing.T) {
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			m := db.GetMergeOperator([]byte("merge"), add, 200*time.Millisecond)
			defer m.Stop()

			val, err := m.Get()
			require.Equal(t, ErrKeyNotFound, err)
			require.Nil(t, val)
		})
	})
	t.Run("Add and Get", func(t *testing.T) {
		key := []byte("merge")
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			m := db.GetMergeOperator(key, add, 200*time.Millisecond)
			defer m.Stop()

			require.NoError(t, m.Add(uint64ToBytes(1)))
			require.NoError(t, m.Add(uint64ToBytes(2)))
			require.NoError(t, m.Add(uint64ToBytes(3)))

			res, err := m.Get()
			require.NoError(t, err)
			require.Equal(t, uint64(6), bytesToUint64(res))
		})

	})
	t.Run("Add and Get slices", func(t *testing.T) {
		// Merge function to merge two byte slices
		add := func(originalValue, newValue []byte) []byte {
			return append(originalValue, newValue...)
		}
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			m := db.GetMergeOperator([]byte("fooprefix"), add, 2*time.Millisecond)
			defer m.Stop()

			require.NoError(t, m.Add([]byte("A")))
			require.NoError(t, m.Add([]byte("B")))
			require.NoError(t, m.Add([]byte("C")))

			value, err := m.Get()
			require.NoError(t, err)
			require.Equal(t, "ABC", string(value))
		})
	})
	t.Run("Get Before Compact", func(t *testing.T) {
		key := []byte("merge")
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			m := db.GetMergeOperator(key, add, 500*time.Millisecond)
			defer m.Stop()

			require.NoError(t, m.Add(uint64ToBytes(1)))
			require.NoError(t, m.Add(uint64ToBytes(2)))
			require.NoError(t, m.Add(uint64ToBytes(3)))

			res, err := m.Get()
			require.NoError(t, err)
			require.Equal(t, uint64(6), bytesToUint64(res))
		})
	})

	t.Run("Get after Delete", func(t *testing.T) {
		key := []byte("merge")
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			m := db.GetMergeOperator(key, add, 200*time.Millisecond)

			require.NoError(t, m.Add(uint64ToBytes(1)))
			require.NoError(t, m.Add(uint64ToBytes(2)))
			require.NoError(t, m.Add(uint64ToBytes(3)))

			m.Stop()
			res, err := m.Get()
			require.NoError(t, err)
			require.Equal(t, uint64(6), bytesToUint64(res))

			require.NoError(t, db.Update(func(txn *Txn) error {
				return txn.Delete(key)
			}))

			m = db.GetMergeOperator(key, add, 200*time.Millisecond)
			require.NoError(t, m.Add(uint64ToBytes(1)))
			m.Stop()

			res, err = m.Get()
			require.NoError(t, err)
			require.Equal(t, uint64(1), bytesToUint64(res))
		})
	})

	t.Run("Get after Stop", func(t *testing.T) {
		key := []byte("merge")
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			m := db.GetMergeOperator(key, add, 1*time.Second)

			require.NoError(t, m.Add(uint64ToBytes(1)))
			require.NoError(t, m.Add(uint64ToBytes(2)))
			require.NoError(t, m.Add(uint64ToBytes(3)))

			m.Stop()
			res, err := m.Get()
			require.NoError(t, err)
			require.Equal(t, uint64(6), bytesToUint64(res))
		})
	})
	t.Run("Old keys should be removed after compaction", func(t *testing.T) {
		dir, err := os.MkdirTemp("", "badger-test")
		require.NoError(t, err)
		defer removeDir(dir)

		// This test relies on CompactL0OnClose
		opts := getTestOptions(dir).WithCompactL0OnClose(true)
		db, err := Open(opts)
		require.NoError(t, err)
		mergeKey := []byte("foo")
		m := db.GetMergeOperator(mergeKey, add, 2*time.Millisecond)

		count := 5000 // This will cause compaction from L0->L1
		for i := 0; i < count; i++ {
			require.NoError(t, m.Add(uint64ToBytes(1)))
		}
		value, err := m.Get()
		require.Nil(t, err)
		require.Equal(t, uint64(count), bytesToUint64(value))
		m.Stop()

		// Force compaction by closing DB. The compaction should discard all the old merged values
		require.Nil(t, db.Close())
		db, err = Open(opts)
		require.NoError(t, err)
		defer db.Close()

		keyCount := 0
		txn := db.NewTransaction(false)
		defer txn.Discard()
		iopt := DefaultIteratorOptions
		iopt.AllVersions = true
		it := txn.NewKeyIterator(mergeKey, iopt)
		defer it.Close()
		for it.Rewind(); it.Valid(); it.Next() {
			keyCount++
		}
		// We should have only one key in badger. All the other keys should've been removed by
		// compaction
		require.Equal(t, 1, keyCount)
	})

}

func uint64ToBytes(i uint64) []byte {
	var buf [8]byte
	binary.BigEndian.PutUint64(buf[:], i)
	return buf[:]
}

func bytesToUint64(b []byte) uint64 {
	return binary.BigEndian.Uint64(b)
}

// Merge function to add two uint64 numbers
func add(existing, latest []byte) []byte {
	return uint64ToBytes(bytesToUint64(existing) + bytesToUint64(latest))
}
