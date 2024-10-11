// Command main generates test fixtures for a DB containing a mixture of point
// and range keys.
//
// Upon running this command, the DB directory will contain:
//
//  1. A single SSTable (000005.sst), containing:
//     a. 26 point keys, a@1 through z@1.
//     b. a RANGEKEYSET [a, z)@1
//     c. a RANGEKEYUNSET [a, z)@2
//     d. a RANGEKEYDEL [a, b)
//  2. A WAL for an unflushed memtable containing:
//     a. A single point key a@2.
//     b. a RANGEKEYSET [a, z)@3
//     c. a RANGEKEYUNSET [a, z)@4
//     d. a RANGEKEYDEL [a, b)
package main

import (
	"io/fs"
	"os"
	"path/filepath"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/vfs"
)

const outDir = "./tool/testdata/mixed"

func main() {
	err := filepath.WalkDir(outDir, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if filepath.Base(path) == "main.go" || d.IsDir() {
			return nil
		}
		return os.Remove(path)
	})
	if err != nil {
		panic(err)
	}
	lel := pebble.MakeLoggingEventListener(pebble.DefaultLogger)

	opts := &pebble.Options{
		FS:                          vfs.Default,
		Comparer:                    testkeys.Comparer,
		FormatMajorVersion:          pebble.FormatNewest,
		EventListener:               &lel,
		DisableAutomaticCompactions: true,
	}
	db, err := pebble.Open(outDir, opts)
	if err != nil {
		panic(err)
	}

	writePoint := func(b *pebble.Batch, k []byte) {
		if err := b.Set(k, nil, nil); err != nil {
			panic(err)
		}
	}
	writeRangeKeySet := func(b *pebble.Batch, start, end, suffix []byte) {
		if err := b.RangeKeySet(start, end, suffix, nil, nil); err != nil {
			panic(err)
		}
	}
	writeRangeKeyUnset := func(b *pebble.Batch, start, end, suffix []byte) {
		if err := b.RangeKeyUnset(start, end, suffix, nil); err != nil {
			panic(err)
		}
	}
	writeRangeKeyDel := func(b *pebble.Batch, start, end []byte) {
		if err := b.RangeKeyDelete(start, end, nil); err != nil {
			panic(err)
		}
	}

	// Write some point and range keys.
	ks := testkeys.Alpha(1)
	b := db.NewBatch()
	for i := int64(0); i < ks.Count(); i++ {
		writePoint(b, testkeys.KeyAt(ks, i, 1))
	}
	start, end := testkeys.Key(ks, 0), testkeys.Key(ks, ks.Count()-1)
	writeRangeKeySet(b, start, end, testkeys.Suffix(1))   // SET   [a, z)@1
	writeRangeKeyUnset(b, start, end, testkeys.Suffix(2)) // UNSET [a, z)@2
	writeRangeKeyDel(b, start, testkeys.Key(ks, 1))       // DEL   [a, b)
	if err := b.Commit(nil); err != nil {
		panic(err)
	}

	// Flush memtables.
	if err := db.Flush(); err != nil {
		panic(err)
	}

	// Write one more point and range key into a memtable before closing, so the
	// WAL is not empty.
	b = db.NewBatch()
	writePoint(b, testkeys.KeyAt(ks, 0, 2))
	writeRangeKeySet(b, start, end, testkeys.Suffix(3))   // SET   [a, z)@3
	writeRangeKeyUnset(b, start, end, testkeys.Suffix(4)) // UNSET [a, z)@4
	writeRangeKeyDel(b, start, testkeys.Key(ks, 1))       // DEL   [a, b)
	if err := b.Commit(nil); err != nil {
		panic(err)
	}
}
