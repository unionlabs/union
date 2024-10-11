/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
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
	"expvar"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"
)

func clearAllMetrics() {
	expvar.Do(func(kv expvar.KeyValue) {
		// Reset the value of each expvar variable based on its type
		switch v := kv.Value.(type) {
		case *expvar.Int:
			v.Set(0)
		case *expvar.Float:
			v.Set(0)
		case *expvar.Map:
			v.Init()
		case *expvar.String:
			v.Set("")
		}
	})
}

func TestWriteMetrics(t *testing.T) {
	opt := getTestOptions("")
	opt.managedTxns = true
	opt.CompactL0OnClose = true
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		clearAllMetrics()
		num := 10
		val := make([]byte, 1<<12)
		key := make([]byte, 40)
		for i := 0; i < num; i++ {
			_, err := rand.Read(key)
			require.NoError(t, err)
			_, err = rand.Read(val)
			require.NoError(t, err)

			writer := db.NewManagedWriteBatch()
			require.NoError(t, writer.SetEntryAt(NewEntry(key, val), 1))
			writer.Flush()
		}

		expectedSize := int64(len(val)) + 48 + 2 // 48 := size of key (40 + 8(ts)), 2 := meta
		write_metric := expvar.Get("badger_write_bytes_user")
		require.Equal(t, expectedSize*int64(num), write_metric.(*expvar.Int).Value())

		put_metric := expvar.Get("badger_put_num_user")
		require.Equal(t, int64(num), put_metric.(*expvar.Int).Value())

		lsm_metric := expvar.Get("badger_write_bytes_l0")
		require.Equal(t, expectedSize*int64(num), lsm_metric.(*expvar.Int).Value())

		compactionMetric := expvar.Get("badger_write_bytes_compaction").(*expvar.Map)
		require.Equal(t, nil, compactionMetric.Get("l6"))

		// Force compaction
		db.Close()

		_, err := OpenManaged(opt)
		require.NoError(t, err)

		compactionMetric = expvar.Get("badger_write_bytes_compaction").(*expvar.Map)
		require.GreaterOrEqual(t, expectedSize*int64(num)+int64(num*200), compactionMetric.Get("l6").(*expvar.Int).Value())
		// Because we have random values, compression is not able to do much, so we incur a cost on total size
	})
}

func TestVlogMetrics(t *testing.T) {
	opt := getTestOptions("")
	opt.managedTxns = true
	opt.CompactL0OnClose = true
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		clearAllMetrics()
		num := 10
		val := make([]byte, 1<<20) // Large Value
		key := make([]byte, 40)
		for i := 0; i < num; i++ {
			_, err := rand.Read(key)
			require.NoError(t, err)
			_, err = rand.Read(val)
			require.NoError(t, err)

			writer := db.NewManagedWriteBatch()
			require.NoError(t, writer.SetEntryAt(NewEntry(key, val), 1))
			writer.Flush()
		}

		expectedSize := int64(len(val)) + 200 // vlog expected size

		totalWrites := expvar.Get("badger_write_num_vlog")
		require.Equal(t, int64(num), totalWrites.(*expvar.Int).Value())

		bytesWritten := expvar.Get("badger_write_bytes_vlog")
		require.GreaterOrEqual(t, expectedSize*int64(num), bytesWritten.(*expvar.Int).Value())

		txn := db.NewTransactionAt(2, false)
		item, err := txn.Get(key)
		require.NoError(t, err)
		require.Equal(t, uint64(1), item.Version())

		err = item.Value(func(val []byte) error {
			totalReads := expvar.Get("badger_read_num_vlog")
			bytesRead := expvar.Get("badger_read_bytes_vlog")
			require.Equal(t, int64(1), totalReads.(*expvar.Int).Value())
			require.GreaterOrEqual(t, expectedSize, bytesRead.(*expvar.Int).Value())
			return nil
		})

		require.NoError(t, err)
	})
}

func TestReadMetrics(t *testing.T) {
	opt := getTestOptions("")
	opt.managedTxns = true
	opt.CompactL0OnClose = true
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		clearAllMetrics()
		num := 10
		val := make([]byte, 1<<15)
		keys := [][]byte{}
		writer := db.NewManagedWriteBatch()
		for i := 0; i < num; i++ {
			keyB := key("byte", 1)
			keys = append(keys, []byte(keyB))

			_, err := rand.Read(val)
			require.NoError(t, err)

			require.NoError(t, writer.SetEntryAt(NewEntry([]byte(keyB), val), 1))
		}
		writer.Flush()

		txn := db.NewTransactionAt(2, false)
		item, err := txn.Get(keys[0])
		require.NoError(t, err)
		require.Equal(t, uint64(1), item.Version())

		totalGets := expvar.Get("badger_get_num_user")
		require.Equal(t, int64(1), totalGets.(*expvar.Int).Value())

		totalMemtableReads := expvar.Get("badger_get_num_memtable")
		require.Equal(t, int64(1), totalMemtableReads.(*expvar.Int).Value())

		totalLSMGets := expvar.Get("badger_get_num_lsm")
		require.Nil(t, totalLSMGets.(*expvar.Map).Get("l6"))

		// Force compaction
		db.Close()

		db, err = OpenManaged(opt)
		require.NoError(t, err)

		txn = db.NewTransactionAt(2, false)
		item, err = txn.Get(keys[0])
		require.NoError(t, err)
		require.Equal(t, uint64(1), item.Version())

		_, err = txn.Get([]byte(key("abdbyte", 1000))) // val should be far enough that bloom filter doesn't hit
		require.Error(t, err)

		totalLSMGets = expvar.Get("badger_get_num_lsm")
		require.Equal(t, int64(0x1), totalLSMGets.(*expvar.Map).Get("l6").(*expvar.Int).Value())

		totalBloom := expvar.Get("badger_hit_num_lsm_bloom_filter")
		require.Equal(t, int64(0x1), totalBloom.(*expvar.Map).Get("l6").(*expvar.Int).Value())
		require.Equal(t, int64(0x1), totalBloom.(*expvar.Map).Get("DoesNotHave_HIT").(*expvar.Int).Value())
		require.Equal(t, int64(0x2), totalBloom.(*expvar.Map).Get("DoesNotHave_ALL").(*expvar.Int).Value())

		bytesLSM := expvar.Get("badger_read_bytes_lsm")
		require.Equal(t, int64(len(val)), bytesLSM.(*expvar.Int).Value())

		getWithResult := expvar.Get("badger_get_with_result_num_user")
		require.Equal(t, int64(2), getWithResult.(*expvar.Int).Value())

		iterOpts := DefaultIteratorOptions
		iter := txn.NewKeyIterator(keys[0], iterOpts)
		iter.Seek(keys[0])

		rangeQueries := expvar.Get("badger_iterator_num_user")
		require.Equal(t, int64(1), rangeQueries.(*expvar.Int).Value())
	})
}
