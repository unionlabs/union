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
	"bytes"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"reflect"
	"strconv"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/pb"
)

func TestBackupRestore1(t *testing.T) {
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)
	db, err := Open(getTestOptions(dir))
	require.NoError(t, err)

	// Write some stuff
	entries := []struct {
		key      []byte
		val      []byte
		userMeta byte
		version  uint64
	}{
		{key: []byte("answer1"), val: []byte("42"), version: 1},
		{key: []byte("answer2"), val: []byte("43"), userMeta: 1, version: 2},
	}

	err = db.Update(func(txn *Txn) error {
		e := entries[0]
		err := txn.SetEntry(NewEntry(e.key, e.val).WithMeta(e.userMeta))
		if err != nil {
			return err
		}
		return nil
	})
	require.NoError(t, err)

	err = db.Update(func(txn *Txn) error {
		e := entries[1]
		err := txn.SetEntry(NewEntry(e.key, e.val).WithMeta(e.userMeta))
		if err != nil {
			return err
		}
		return nil
	})
	require.NoError(t, err)

	// Use different directory.
	dir, err = os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)
	bak, err := os.CreateTemp(dir, "badgerbak")
	require.NoError(t, err)
	_, err = db.Backup(bak, 0)
	require.NoError(t, err)
	require.NoError(t, bak.Close())
	require.NoError(t, db.Close())

	db, err = Open(getTestOptions(dir))
	require.NoError(t, err)
	defer db.Close()
	bak, err = os.Open(bak.Name())
	require.NoError(t, err)
	defer bak.Close()

	require.NoError(t, db.Load(bak, 16))

	err = db.View(func(txn *Txn) error {
		opts := DefaultIteratorOptions
		opts.AllVersions = true
		it := txn.NewIterator(opts)
		defer it.Close()
		var count int
		for it.Rewind(); it.Valid(); it.Next() {
			item := it.Item()
			val, err := item.ValueCopy(nil)
			if err != nil {
				return err
			}
			t.Logf("Got entry: %v\n", item.Version())
			require.Equal(t, entries[count].key, item.Key())
			require.Equal(t, entries[count].val, val)
			require.Equal(t, entries[count].version, item.Version())
			require.Equal(t, entries[count].userMeta, item.UserMeta())
			count++
		}
		require.Equal(t, count, 2)
		return nil
	})
	require.NoError(t, err)
	require.Equal(t, 3, int(db.orc.nextTs()))
}

func TestBackupRestore2(t *testing.T) {
	tmpdir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)

	defer removeDir(tmpdir)

	s1Path := filepath.Join(tmpdir, "test1")
	s2Path := filepath.Join(tmpdir, "test2")
	s3Path := filepath.Join(tmpdir, "test3")

	db1, err := Open(getTestOptions(s1Path))
	require.NoError(t, err)

	defer db1.Close()
	key1 := []byte("key1")
	key2 := []byte("key2")
	rawValue := []byte("NotLongValue")
	N := byte(251)
	err = db1.Update(func(tx *Txn) error {
		if err := tx.SetEntry(NewEntry(key1, rawValue)); err != nil {
			return err
		}
		return tx.SetEntry(NewEntry(key2, rawValue))
	})
	require.NoError(t, err)

	for i := byte(1); i < N; i++ {
		err = db1.Update(func(tx *Txn) error {
			if err := tx.SetEntry(NewEntry(append(key1, i), rawValue)); err != nil {
				return err
			}
			return tx.SetEntry(NewEntry(append(key2, i), rawValue))
		})
		require.NoError(t, err)

	}
	var backup bytes.Buffer
	_, err = db1.Backup(&backup, 0)
	require.NoError(t, err)

	fmt.Println("backup1 length:", backup.Len())

	db2, err := Open(getTestOptions(s2Path))
	require.NoError(t, err)

	defer db2.Close()
	err = db2.Load(&backup, 16)
	require.NoError(t, err)

	// Check nextTs is correctly set.
	require.Equal(t, db1.orc.nextTs(), db2.orc.nextTs())

	for i := byte(1); i < N; i++ {
		err = db2.View(func(tx *Txn) error {
			k := append(key1, i)
			item, err := tx.Get(k)
			if err != nil {
				if err == ErrKeyNotFound {
					return fmt.Errorf("Key %q has been not found, but was set\n", k)
				}
				return err
			}
			v, err := item.ValueCopy(nil)
			if err != nil {
				return err
			}
			if !reflect.DeepEqual(v, rawValue) {
				return fmt.Errorf("Values not match, got %v, expected %v", v, rawValue)
			}
			return nil
		})
		require.NoError(t, err)

	}

	for i := byte(1); i < N; i++ {
		err = db2.Update(func(tx *Txn) error {
			if err := tx.SetEntry(NewEntry(append(key1, i), rawValue)); err != nil {
				return err
			}
			return tx.SetEntry(NewEntry(append(key2, i), rawValue))
		})
		require.NoError(t, err)

	}

	backup.Reset()
	_, err = db2.Backup(&backup, 0)
	require.NoError(t, err)

	fmt.Println("backup2 length:", backup.Len())
	db3, err := Open(getTestOptions(s3Path))
	require.NoError(t, err)

	defer db3.Close()

	err = db3.Load(&backup, 16)
	require.NoError(t, err)

	// Check nextTs is correctly set.
	require.Equal(t, db2.orc.nextTs(), db3.orc.nextTs())

	for i := byte(1); i < N; i++ {
		err = db3.View(func(tx *Txn) error {
			k := append(key1, i)
			item, err := tx.Get(k)
			if err != nil {
				if err == ErrKeyNotFound {
					return fmt.Errorf("Key %q has been not found, but was set\n", k)
				}
				return err
			}
			v, err := item.ValueCopy(nil)
			if err != nil {
				return err
			}
			if !reflect.DeepEqual(v, rawValue) {
				return fmt.Errorf("Values not match, got %v, expected %v", v, rawValue)
			}
			return nil
		})
		require.NoError(t, err)

	}

}

var randSrc = rand.NewSource(time.Now().UnixNano())

func createEntries(n int) []*pb.KV {
	entries := make([]*pb.KV, n)
	for i := 0; i < n; i++ {
		entries[i] = &pb.KV{
			Key:      []byte(fmt.Sprint("key", i)),
			Value:    []byte{1},
			UserMeta: []byte{0},
			Meta:     []byte{0},
		}
	}
	return entries
}

func populateEntries(db *DB, entries []*pb.KV) error {
	return db.Update(func(txn *Txn) error {
		var err error
		for i, e := range entries {
			if err = txn.SetEntry(NewEntry(e.Key, e.Value)); err != nil {
				return err
			}
			entries[i].Version = 1
		}
		return nil
	})
}

func TestBackup(t *testing.T) {
	test := func(t *testing.T, db *DB) {
		var bb bytes.Buffer
		N := 1000
		entries := createEntries(N)
		require.NoError(t, populateEntries(db, entries))

		_, err := db.Backup(&bb, 0)
		require.NoError(t, err)

		err = db.View(func(txn *Txn) error {
			opts := DefaultIteratorOptions
			it := txn.NewIterator(opts)
			defer it.Close()
			var count int
			for it.Rewind(); it.Valid(); it.Next() {
				item := it.Item()
				idx, err := strconv.Atoi(string(item.Key())[3:])
				if err != nil {
					return err
				}
				if idx > N || !bytes.Equal(entries[idx].Key, item.Key()) {
					return fmt.Errorf("%s: %s", string(item.Key()), ErrKeyNotFound)
				}
				count++
			}
			if N != count {
				return fmt.Errorf("wrong number of items: %d expected, %d actual", N, count)
			}
			return nil
		})
		require.NoError(t, err)
	}
	t.Run("disk mode", func(t *testing.T) {
		tmpdir, err := os.MkdirTemp("", "badger-test")
		require.NoError(t, err)

		defer removeDir(tmpdir)
		opt := DefaultOptions(filepath.Join(tmpdir, "backup0"))
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			test(t, db)
		})
	})
	t.Run("InMemory mode", func(t *testing.T) {
		opt := DefaultOptions("")
		opt.InMemory = true
		runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
			test(t, db)
		})
	})
}

func TestBackupRestore3(t *testing.T) {
	var bb bytes.Buffer
	tmpdir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)

	defer removeDir(tmpdir)

	N := 1000
	entries := createEntries(N)

	var db1NextTs uint64
	// backup
	{
		db1, err := Open(DefaultOptions(filepath.Join(tmpdir, "backup1")))
		require.NoError(t, err)

		defer db1.Close()
		require.NoError(t, populateEntries(db1, entries))

		_, err = db1.Backup(&bb, 0)
		require.NoError(t, err)

		db1NextTs = db1.orc.nextTs()
		require.NoError(t, db1.Close())
	}
	require.True(t, len(entries) == N)
	require.True(t, bb.Len() > 0)

	// restore
	db2, err := Open(DefaultOptions(filepath.Join(tmpdir, "restore1")))
	require.NoError(t, err)

	defer db2.Close()
	require.NotEqual(t, db1NextTs, db2.orc.nextTs())
	require.NoError(t, db2.Load(&bb, 16))
	require.Equal(t, db1NextTs, db2.orc.nextTs())

	// verify
	err = db2.View(func(txn *Txn) error {
		opts := DefaultIteratorOptions
		it := txn.NewIterator(opts)
		defer it.Close()
		var count int
		for it.Rewind(); it.Valid(); it.Next() {
			item := it.Item()
			idx, err := strconv.Atoi(string(item.Key())[3:])
			if err != nil {
				return err
			}
			if idx > N || !bytes.Equal(entries[idx].Key, item.Key()) {
				return fmt.Errorf("%s: %s", string(item.Key()), ErrKeyNotFound)
			}
			count++
		}
		if N != count {
			return fmt.Errorf("wrong number of items: %d expected, %d actual", N, count)
		}
		return nil
	})
	require.NoError(t, err)
}

func TestBackupLoadIncremental(t *testing.T) {
	tmpdir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)

	defer removeDir(tmpdir)

	N := 100
	entries := createEntries(N)
	updates := make(map[int]byte)
	var bb bytes.Buffer

	var db1NextTs uint64
	// backup
	{
		db1, err := Open(DefaultOptions(filepath.Join(tmpdir, "backup2")))
		require.NoError(t, err)

		defer db1.Close()

		require.NoError(t, populateEntries(db1, entries))
		since, err := db1.Backup(&bb, 0)
		require.NoError(t, err)

		ints := rand.New(randSrc).Perm(N)

		// pick 10 items to mark as deleted.
		err = db1.Update(func(txn *Txn) error {
			for _, i := range ints[:10] {
				if err := txn.Delete(entries[i].Key); err != nil {
					return err
				}
				updates[i] = bitDelete
			}
			return nil
		})
		require.NoError(t, err)
		since, err = db1.Backup(&bb, since)
		require.NoError(t, err)

		// pick 5 items to mark as expired.
		err = db1.Update(func(txn *Txn) error {
			for _, i := range (ints)[10:15] {
				entry := NewEntry(entries[i].Key, entries[i].Value).WithTTL(-time.Hour)
				if err := txn.SetEntry(entry); err != nil {
					return err
				}
				updates[i] = bitDelete // expired
			}
			return nil
		})
		require.NoError(t, err)
		since, err = db1.Backup(&bb, since)
		require.NoError(t, err)

		// pick 5 items to mark as discard.
		err = db1.Update(func(txn *Txn) error {
			for _, i := range ints[15:20] {
				entry := NewEntry(entries[i].Key, entries[i].Value).WithDiscard()
				if err := txn.SetEntry(entry); err != nil {
					return err
				}
				updates[i] = bitDiscardEarlierVersions
			}
			return nil
		})
		require.NoError(t, err)
		_, err = db1.Backup(&bb, since)
		require.NoError(t, err)

		db1NextTs = db1.orc.nextTs()

		require.NoError(t, db1.Close())
	}
	require.True(t, len(entries) == N)
	require.True(t, bb.Len() > 0)

	// restore
	db2, err := Open(getTestOptions(filepath.Join(tmpdir, "restore2")))
	require.NoError(t, err)

	defer db2.Close()

	require.NotEqual(t, db1NextTs, db2.orc.nextTs())
	require.NoError(t, db2.Load(&bb, 16))
	require.Equal(t, db1NextTs, db2.orc.nextTs())

	// verify
	actual := make(map[int]byte)
	err = db2.View(func(txn *Txn) error {
		opts := DefaultIteratorOptions
		opts.AllVersions = true
		it := txn.NewIterator(opts)
		defer it.Close()
		var count int
		for it.Rewind(); it.Valid(); it.Next() {
			item := it.Item()
			idx, err := strconv.Atoi(string(item.Key())[3:])
			if err != nil {
				return err
			}
			if item.IsDeletedOrExpired() {
				_, ok := updates[idx]
				if !ok {
					return fmt.Errorf("%s: not expected to be updated but it is",
						string(item.Key()))
				}
				actual[idx] = item.meta
				count++
				continue
			}
		}
		if len(updates) != count {
			return fmt.Errorf("mismatched updated items: %d expected, %d actual",
				len(updates), count)
		}
		return nil
	})
	require.NoError(t, err, "%v %v", updates, actual)
}

func TestBackupBitClear(t *testing.T) {
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	opt := getTestOptions(dir)
	opt.ValueThreshold = 10 // This is important
	db, err := Open(opt)
	require.NoError(t, err)

	key := []byte("foo")
	val := []byte(fmt.Sprintf("%0100d", 1))
	require.Greater(t, int64(len(val)), db.valueThreshold())

	err = db.Update(func(txn *Txn) error {
		e := NewEntry(key, val)
		// Value > valueTheshold so bitValuePointer will be set.
		return txn.SetEntry(e)
	})
	require.NoError(t, err)

	// Use different directory.
	dir, err = os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	bak, err := os.CreateTemp(dir, "badgerbak")
	require.NoError(t, err)
	_, err = db.Backup(bak, 0)
	require.NoError(t, err)
	require.NoError(t, bak.Close())

	oldValue := db.orc.nextTs()
	require.NoError(t, db.Close())

	opt = getTestOptions(dir)
	opt.ValueThreshold = 200 // This is important.
	db, err = Open(opt)
	require.NoError(t, err)
	defer db.Close()

	bak, err = os.Open(bak.Name())
	require.NoError(t, err)
	defer bak.Close()

	require.NoError(t, db.Load(bak, 16))
	// Ensure nextTs is still the same.
	require.Equal(t, oldValue, db.orc.nextTs())

	require.NoError(t, db.View(func(txn *Txn) error {
		e, err := txn.Get(key)
		require.NoError(t, err)
		v, err := e.ValueCopy(nil)
		require.NoError(t, err)
		require.Equal(t, val, v)
		return nil
	}))
}
