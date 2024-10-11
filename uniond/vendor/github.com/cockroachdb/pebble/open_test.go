// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"reflect"
	"runtime/debug"
	"sort"
	"strconv"
	"strings"
	"sync/atomic"
	"syscall"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/atomicfs"
	"github.com/cockroachdb/pebble/vfs/errorfs"
	"github.com/cockroachdb/redact"
	"github.com/kr/pretty"
	"github.com/stretchr/testify/require"
)

func TestOpenSharedTableCache(t *testing.T) {
	c := cache.New(cacheDefaultSize)
	tc := NewTableCache(c, 16, 100)
	defer tc.Unref()
	defer c.Unref()

	d0, err := Open("", testingRandomized(t, &Options{
		FS:         vfs.NewMem(),
		Cache:      c,
		TableCache: tc,
	}))
	if err != nil {
		t.Errorf("d0 Open: %s", err.Error())
	}
	defer d0.Close()

	d1, err := Open("", testingRandomized(t, &Options{
		FS:         vfs.NewMem(),
		Cache:      c,
		TableCache: tc,
	}))
	if err != nil {
		t.Errorf("d1 Open: %s", err.Error())
	}
	defer d1.Close()

	// Make sure that the Open function is using the passed in table cache
	// when the TableCache option is set.
	require.Equalf(
		t, d0.tableCache.tableCache, d1.tableCache.tableCache,
		"expected tableCache for both d0 and d1 to be the same",
	)
}

func TestErrorIfExists(t *testing.T) {
	opts := testingRandomized(t, &Options{
		FS:            vfs.NewMem(),
		ErrorIfExists: true,
	})
	defer ensureFilesClosed(t, opts)()

	d0, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d0.Close())

	if _, err := Open("", opts); !errors.Is(err, ErrDBAlreadyExists) {
		t.Fatalf("expected db-already-exists error, got %v", err)
	}

	opts.ErrorIfExists = false
	d1, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d1.Close())
}

func TestErrorIfNotExists(t *testing.T) {
	opts := testingRandomized(t, &Options{
		FS:               vfs.NewMem(),
		ErrorIfNotExists: true,
	})
	defer ensureFilesClosed(t, opts)()

	_, err := Open("", opts)
	if !errors.Is(err, ErrDBDoesNotExist) {
		t.Fatalf("expected db-does-not-exist error, got %v", err)
	}

	// Create the DB and try again.
	opts.ErrorIfNotExists = false
	d0, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d0.Close())

	opts.ErrorIfNotExists = true
	d1, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d1.Close())
}

func TestErrorIfNotPristine(t *testing.T) {
	opts := testingRandomized(t, &Options{
		FS:                 vfs.NewMem(),
		ErrorIfNotPristine: true,
	})
	defer ensureFilesClosed(t, opts)()

	d0, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d0.Close())

	// Store is pristine; ok to open.
	d1, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d1.Set([]byte("foo"), []byte("bar"), Sync))
	require.NoError(t, d1.Close())

	if _, err := Open("", opts); !errors.Is(err, ErrDBNotPristine) {
		t.Fatalf("expected db-not-pristine error, got %v", err)
	}

	// Run compaction and make sure we're still not allowed to open.
	opts.ErrorIfNotPristine = false
	d2, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d2.Compact([]byte("a"), []byte("z"), false /* parallelize */))
	require.NoError(t, d2.Close())

	opts.ErrorIfNotPristine = true
	if _, err := Open("", opts); !errors.Is(err, ErrDBNotPristine) {
		t.Fatalf("expected db-already-exists error, got %v", err)
	}
}

func TestOpenAlreadyLocked(t *testing.T) {
	runTest := func(t *testing.T, dirname string, fs vfs.FS) {
		opts := testingRandomized(t, &Options{FS: fs})
		var err error
		opts.Lock, err = LockDirectory(dirname, fs)
		require.NoError(t, err)

		d, err := Open(dirname, opts)
		require.NoError(t, err)
		require.NoError(t, d.Set([]byte("foo"), []byte("bar"), Sync))

		// Try to open the same database reusing the Options containing the same
		// Lock. It should error when it observes that it's already referenced.
		_, err = Open(dirname, opts)
		require.Error(t, err)

		// Close the database.
		require.NoError(t, d.Close())

		// Now Opening should succeed again.
		d, err = Open(dirname, opts)
		require.NoError(t, err)
		require.NoError(t, d.Close())

		require.NoError(t, opts.Lock.Close())
		// There should be no more remaining references.
		require.Equal(t, int32(0), opts.Lock.refs.Load())
	}
	t.Run("memfs", func(t *testing.T) {
		runTest(t, "", vfs.NewMem())
	})
	t.Run("disk", func(t *testing.T) {
		runTest(t, t.TempDir(), vfs.Default)
	})
}

func TestNewDBFilenames(t *testing.T) {
	versions := map[FormatMajorVersion][]string{
		FormatMostCompatible: {
			"000002.log",
			"CURRENT",
			"LOCK",
			"MANIFEST-000001",
			"OPTIONS-000003",
		},
		internalFormatNewest: {
			"000002.log",
			"CURRENT",
			"LOCK",
			"MANIFEST-000001",
			"OPTIONS-000003",
			"marker.format-version.000015.016",
			"marker.manifest.000001.MANIFEST-000001",
		},
	}

	for formatVers, want := range versions {
		t.Run(fmt.Sprintf("vers=%s", formatVers), func(t *testing.T) {
			mem := vfs.NewMem()
			fooBar := mem.PathJoin("foo", "bar")
			d, err := Open(fooBar, &Options{
				FS:                 mem,
				FormatMajorVersion: formatVers,
			})
			if err != nil {
				t.Fatalf("Open: %v", err)
			}
			if err := d.Close(); err != nil {
				t.Fatalf("Close: %v", err)
			}
			got, err := mem.List(fooBar)
			if err != nil {
				t.Fatalf("List: %v", err)
			}
			sort.Strings(got)
			if !reflect.DeepEqual(got, want) {
				t.Errorf("\ngot  %v\nwant %v", got, want)
			}
		})
	}
}

func testOpenCloseOpenClose(t *testing.T, fs vfs.FS, root string) {
	opts := testingRandomized(t, &Options{FS: fs})

	for _, startFromEmpty := range []bool{false, true} {
		for _, walDirname := range []string{"", "wal"} {
			for _, length := range []int{-1, 0, 1, 1000, 10000, 100000} {
				dirname := "sharedDatabase" + walDirname
				if startFromEmpty {
					dirname = "startFromEmpty" + walDirname + strconv.Itoa(length)
				}
				dirname = fs.PathJoin(root, dirname)
				if walDirname == "" {
					opts.WALDir = ""
				} else {
					opts.WALDir = fs.PathJoin(dirname, walDirname)
				}

				got, xxx := []byte(nil), ""
				if length >= 0 {
					xxx = strings.Repeat("x", length)
				}

				d0, err := Open(dirname, opts)
				if err != nil {
					t.Fatalf("sfe=%t, length=%d: Open #0: %v",
						startFromEmpty, length, err)
					continue
				}
				if length >= 0 {
					err = d0.Set([]byte("key"), []byte(xxx), nil)
					if err != nil {
						t.Errorf("sfe=%t, length=%d: Set: %v",
							startFromEmpty, length, err)
						continue
					}
				}
				err = d0.Close()
				if err != nil {
					t.Errorf("sfe=%t, length=%d: Close #0: %v",
						startFromEmpty, length, err)
					continue
				}

				d1, err := Open(dirname, opts)
				if err != nil {
					t.Errorf("sfe=%t, length=%d: Open #1: %v",
						startFromEmpty, length, err)
					continue
				}
				if length >= 0 {
					var closer io.Closer
					got, closer, err = d1.Get([]byte("key"))
					if err != nil {
						t.Errorf("sfe=%t, length=%d: Get: %v",
							startFromEmpty, length, err)
						continue
					}
					got = append([]byte(nil), got...)
					closer.Close()
				}
				err = d1.Close()
				if err != nil {
					t.Errorf("sfe=%t, length=%d: Close #1: %v",
						startFromEmpty, length, err)
					continue
				}

				if length >= 0 && string(got) != xxx {
					t.Errorf("sfe=%t, length=%d: got value differs from set value",
						startFromEmpty, length)
					continue
				}

				{
					got, err := opts.FS.List(dirname)
					if err != nil {
						t.Fatalf("List: %v", err)
					}
					var optionsCount int
					for _, s := range got {
						if t, _, ok := base.ParseFilename(opts.FS, s); ok && t == fileTypeOptions {
							optionsCount++
						}
					}
					if optionsCount != 1 {
						t.Fatalf("expected 1 OPTIONS file, but found %d", optionsCount)
					}
				}
			}
		}
	}
}

func TestOpenCloseOpenClose(t *testing.T) {
	for _, fstype := range []string{"disk", "mem"} {
		t.Run(fstype, func(t *testing.T) {
			var fs vfs.FS
			var dir string
			switch fstype {
			case "disk":
				var err error
				dir, err = os.MkdirTemp("", "open-close")
				require.NoError(t, err)
				defer func() {
					_ = os.RemoveAll(dir)
				}()
				fs = vfs.Default
			case "mem":
				dir = ""
				fs = vfs.NewMem()
			}
			testOpenCloseOpenClose(t, fs, dir)
		})
	}
}

func TestOpenOptionsCheck(t *testing.T) {
	mem := vfs.NewMem()
	opts := &Options{FS: mem}

	d, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d.Close())

	opts = &Options{
		Comparer: &Comparer{Name: "foo"},
		FS:       mem,
	}
	_, err = Open("", opts)
	require.Regexp(t, `comparer name from file.*!=.*`, err)

	opts = &Options{
		Merger: &Merger{Name: "bar"},
		FS:     mem,
	}
	_, err = Open("", opts)
	require.Regexp(t, `merger name from file.*!=.*`, err)
}

func TestOpenCrashWritingOptions(t *testing.T) {
	memFS := vfs.NewMem()

	d, err := Open("", &Options{FS: memFS})
	require.NoError(t, err)
	require.NoError(t, d.Close())

	// Open the database again, this time with a mocked filesystem that
	// will only succeed in partially writing the OPTIONS file.
	fs := optionsTornWriteFS{FS: memFS}
	_, err = Open("", &Options{FS: fs})
	require.Error(t, err)

	// Re-opening the database must succeed.
	d, err = Open("", &Options{FS: memFS})
	require.NoError(t, err)
	require.NoError(t, d.Close())
}

type optionsTornWriteFS struct {
	vfs.FS
}

func (fs optionsTornWriteFS) Create(name string) (vfs.File, error) {
	file, err := fs.FS.Create(name)
	if file != nil {
		file = optionsTornWriteFile{File: file}
	}
	return file, err
}

type optionsTornWriteFile struct {
	vfs.File
}

func (f optionsTornWriteFile) Write(b []byte) (int, error) {
	// Look for the OPTIONS-XXXXXX file's `comparer=` field.
	comparerKey := []byte("comparer=")
	i := bytes.Index(b, comparerKey)
	if i == -1 {
		return f.File.Write(b)
	}
	// Write only the contents through `comparer=` and return an error.
	n, _ := f.File.Write(b[:i+len(comparerKey)])
	return n, syscall.EIO
}

func TestOpenReadOnly(t *testing.T) {
	mem := vfs.NewMem()

	{
		// Opening a non-existent DB in read-only mode should result in no mutable
		// filesystem operations.
		var memLog base.InMemLogger
		_, err := Open("non-existent", testingRandomized(t, &Options{
			FS:       vfs.WithLogging(mem, memLog.Infof),
			ReadOnly: true,
			WALDir:   "non-existent-waldir",
		}))
		if err == nil {
			t.Fatalf("expected error, but found success")
		}
		const expected = `open-dir: non-existent`
		if trimmed := strings.TrimSpace(memLog.String()); expected != trimmed {
			t.Fatalf("expected %q, but found %q", expected, trimmed)
		}
	}

	{
		// Opening a DB with a non-existent WAL dir in read-only mode should result
		// in no mutable filesystem operations other than the LOCK.
		var memLog base.InMemLogger
		_, err := Open("", testingRandomized(t, &Options{
			FS:       vfs.WithLogging(mem, memLog.Infof),
			ReadOnly: true,
			WALDir:   "non-existent-waldir",
		}))
		if err == nil {
			t.Fatalf("expected error, but found success")
		}
		const expected = "open-dir: \nopen-dir: non-existent-waldir\nclose:"
		if trimmed := strings.TrimSpace(memLog.String()); expected != trimmed {
			t.Fatalf("expected %q, but found %q", expected, trimmed)
		}
	}

	var contents []string
	{
		// Create a new DB and populate it with a small amount of data.
		d, err := Open("", testingRandomized(t, &Options{
			FS: mem,
		}))
		require.NoError(t, err)
		require.NoError(t, d.Set([]byte("test"), nil, nil))
		require.NoError(t, d.Close())
		contents, err = mem.List("")
		require.NoError(t, err)
		sort.Strings(contents)
	}

	{
		// Re-open the DB read-only. The directory contents should be unchanged.
		d, err := Open("", testingRandomized(t, &Options{
			FS:       mem,
			ReadOnly: true,
		}))
		require.NoError(t, err)

		// Verify various write operations fail in read-only mode.
		require.EqualValues(t, ErrReadOnly, d.Compact(nil, []byte("\xff"), false))
		require.EqualValues(t, ErrReadOnly, d.Flush())
		require.EqualValues(t, ErrReadOnly, func() error { _, err := d.AsyncFlush(); return err }())

		require.EqualValues(t, ErrReadOnly, d.Delete(nil, nil))
		require.EqualValues(t, ErrReadOnly, d.DeleteRange(nil, nil, nil))
		require.EqualValues(t, ErrReadOnly, d.Ingest(nil))
		require.EqualValues(t, ErrReadOnly, d.LogData(nil, nil))
		require.EqualValues(t, ErrReadOnly, d.Merge(nil, nil, nil))
		require.EqualValues(t, ErrReadOnly, d.Set(nil, nil, nil))

		// Verify we can still read in read-only mode.
		require.NoError(t, func() error {
			_, closer, err := d.Get([]byte("test"))
			if closer != nil {
				closer.Close()
			}
			return err
		}())

		checkIter := func(iter *Iterator, err error) {
			t.Helper()

			var keys []string
			for valid := iter.First(); valid; valid = iter.Next() {
				keys = append(keys, string(iter.Key()))
			}
			require.NoError(t, iter.Close())
			expectedKeys := []string{"test"}
			if diff := pretty.Diff(keys, expectedKeys); diff != nil {
				t.Fatalf("%s\n%s", strings.Join(diff, "\n"), keys)
			}
		}

		checkIter(d.NewIter(nil))

		b := d.NewIndexedBatch()
		checkIter(b.NewIter(nil))
		require.EqualValues(t, ErrReadOnly, b.Commit(nil))
		require.EqualValues(t, ErrReadOnly, d.Apply(b, nil))

		s := d.NewSnapshot()
		checkIter(s.NewIter(nil))
		require.NoError(t, s.Close())

		require.NoError(t, d.Close())

		newContents, err := mem.List("")
		require.NoError(t, err)

		sort.Strings(newContents)
		if diff := pretty.Diff(contents, newContents); diff != nil {
			t.Fatalf("%s", strings.Join(diff, "\n"))
		}
	}
}

func TestOpenWALReplay(t *testing.T) {
	largeValue := []byte(strings.Repeat("a", 100<<10))
	hugeValue := []byte(strings.Repeat("b", 10<<20))
	checkIter := func(iter *Iterator, err error) {
		t.Helper()

		var keys []string
		for valid := iter.First(); valid; valid = iter.Next() {
			keys = append(keys, string(iter.Key()))
		}
		require.NoError(t, iter.Close())
		expectedKeys := []string{"1", "2", "3", "4", "5"}
		if diff := pretty.Diff(keys, expectedKeys); diff != nil {
			t.Fatalf("%s\n%s", strings.Join(diff, "\n"), keys)
		}
	}

	for _, readOnly := range []bool{false, true} {
		t.Run(fmt.Sprintf("read-only=%t", readOnly), func(t *testing.T) {
			// Create a new DB and populate it with some data.
			const dir = ""
			mem := vfs.NewMem()
			d, err := Open(dir, testingRandomized(t, &Options{
				FS:           mem,
				MemTableSize: 32 << 20,
			}))
			require.NoError(t, err)
			// All these values will fit in a single memtable, so on closing the db there
			// will be no sst and all the data is in a single WAL.
			require.NoError(t, d.Set([]byte("1"), largeValue, nil))
			require.NoError(t, d.Set([]byte("2"), largeValue, nil))
			require.NoError(t, d.Set([]byte("3"), largeValue, nil))
			require.NoError(t, d.Set([]byte("4"), hugeValue, nil))
			require.NoError(t, d.Set([]byte("5"), largeValue, nil))
			checkIter(d.NewIter(nil))
			require.NoError(t, d.Close())
			files, err := mem.List(dir)
			require.NoError(t, err)
			sort.Strings(files)
			logCount, sstCount := 0, 0
			for _, fname := range files {
				if strings.HasSuffix(fname, ".sst") {
					sstCount++
				}
				if strings.HasSuffix(fname, ".log") {
					logCount++
				}
			}
			require.Equal(t, 0, sstCount)
			// The memtable size starts at 256KB and doubles up to 32MB so we expect 5
			// logs (one for each doubling).
			require.Equal(t, 7, logCount)

			// Re-open the DB with a smaller memtable. Values for 1, 2 will fit in the first memtable;
			// value for 3 will go in the next memtable; value for 4 will be in a flushable batch
			// which will cause the previous memtable to be flushed; value for 5 will go in the next
			// memtable
			d, err = Open(dir, testingRandomized(t, &Options{
				FS:           mem,
				MemTableSize: 300 << 10,
				ReadOnly:     readOnly,
			}))
			require.NoError(t, err)

			if readOnly {
				m := d.Metrics()
				require.Equal(t, int64(logCount), m.WAL.Files)
				d.mu.Lock()
				require.NotNil(t, d.mu.mem.mutable)
				d.mu.Unlock()
			}
			checkIter(d.NewIter(nil))
			require.NoError(t, d.Close())
		})
	}
}

// Reproduction for https://github.com/cockroachdb/pebble/issues/2234.
func TestWALReplaySequenceNumBug(t *testing.T) {
	mem := vfs.NewMem()
	d, err := Open("", testingRandomized(t, &Options{
		FS: mem,
	}))
	require.NoError(t, err)

	d.mu.Lock()
	// Disable any flushes.
	d.mu.compact.flushing = true
	d.mu.Unlock()

	require.NoError(t, d.Set([]byte("1"), nil, nil))
	require.NoError(t, d.Set([]byte("2"), nil, nil))

	// Write a large batch. This should go to a separate memtable.
	largeValue := []byte(strings.Repeat("a", int(d.largeBatchThreshold)))
	require.NoError(t, d.Set([]byte("1"), largeValue, nil))

	// This write should go the mutable memtable after the large batch in the
	// memtable queue.
	d.Set([]byte("1"), nil, nil)

	d.mu.Lock()
	d.mu.compact.flushing = false
	d.mu.Unlock()

	// Make sure none of the flushables have been flushed.
	require.Equal(t, 3, len(d.mu.mem.queue))

	// Close the db. This doesn't cause a flush of the memtables, so they'll
	// have to be replayed when the db is reopened.
	require.NoError(t, d.Close())

	files, err := mem.List("")
	require.NoError(t, err)
	sort.Strings(files)
	sstCount := 0
	for _, fname := range files {
		if strings.HasSuffix(fname, ".sst") {
			sstCount++
		}
	}
	require.Equal(t, 0, sstCount)

	// Reopen db in read only mode to force read only wal replay.
	d, err = Open("", &Options{
		FS:       mem,
		ReadOnly: true,
	})
	require.NoError(t, err)
	val, c, _ := d.Get([]byte("1"))
	require.Equal(t, []byte{}, val)
	c.Close()
	require.NoError(t, d.Close())
}

// Similar to TestOpenWALReplay, except we test replay behavior after a
// memtable has been flushed. We test all 3 reasons for flushing: forced, size,
// and large-batch.
func TestOpenWALReplay2(t *testing.T) {
	for _, readOnly := range []bool{false, true} {
		t.Run(fmt.Sprintf("read-only=%t", readOnly), func(t *testing.T) {
			for _, reason := range []string{"forced", "size", "large-batch"} {
				t.Run(reason, func(t *testing.T) {
					mem := vfs.NewMem()
					d, err := Open("", testingRandomized(t, &Options{
						FS:           mem,
						MemTableSize: 256 << 10,
					}))
					require.NoError(t, err)

					switch reason {
					case "forced":
						require.NoError(t, d.Set([]byte("1"), nil, nil))
						require.NoError(t, d.Flush())
						require.NoError(t, d.Set([]byte("2"), nil, nil))
					case "size":
						largeValue := []byte(strings.Repeat("a", 100<<10))
						require.NoError(t, d.Set([]byte("1"), largeValue, nil))
						require.NoError(t, d.Set([]byte("2"), largeValue, nil))
						require.NoError(t, d.Set([]byte("3"), largeValue, nil))
					case "large-batch":
						largeValue := []byte(strings.Repeat("a", int(d.largeBatchThreshold)))
						require.NoError(t, d.Set([]byte("1"), nil, nil))
						require.NoError(t, d.Set([]byte("2"), largeValue, nil))
						require.NoError(t, d.Set([]byte("3"), nil, nil))
					}
					require.NoError(t, d.Close())

					files, err := mem.List("")
					require.NoError(t, err)
					sort.Strings(files)
					sstCount := 0
					for _, fname := range files {
						if strings.HasSuffix(fname, ".sst") {
							sstCount++
						}
					}
					require.Equal(t, 1, sstCount)

					// Re-open the DB with a smaller memtable. Values for 1, 2 will fit in the first memtable;
					// value for 3 will go in the next memtable; value for 4 will be in a flushable batch
					// which will cause the previous memtable to be flushed; value for 5 will go in the next
					// memtable
					d, err = Open("", testingRandomized(t, &Options{
						FS:           mem,
						MemTableSize: 300 << 10,
						ReadOnly:     readOnly,
					}))
					require.NoError(t, err)
					require.NoError(t, d.Close())
				})
			}
		})
	}
}

// TestTwoWALReplayCorrupt tests WAL-replay behavior when the first of the two
// WALs is corrupted with an sstable checksum error. Replay must stop at the
// first WAL because otherwise we may violate point-in-time recovery
// semantics. See #864.
func TestTwoWALReplayCorrupt(t *testing.T) {
	// Use the real filesystem so that we can seek and overwrite WAL data
	// easily.
	dir, err := os.MkdirTemp("", "wal-replay")
	require.NoError(t, err)
	defer os.RemoveAll(dir)

	d, err := Open(dir, testingRandomized(t, &Options{
		MemTableStopWritesThreshold: 4,
		MemTableSize:                2048,
	}))
	require.NoError(t, err)
	d.mu.Lock()
	d.mu.compact.flushing = true
	d.mu.Unlock()
	require.NoError(t, d.Set([]byte("1"), []byte(strings.Repeat("a", 1024)), nil))
	require.NoError(t, d.Set([]byte("2"), nil, nil))
	d.mu.Lock()
	d.mu.compact.flushing = false
	d.mu.Unlock()
	require.NoError(t, d.Close())

	// We should have two WALs.
	var logs []string
	ls, err := vfs.Default.List(dir)
	require.NoError(t, err)
	for _, name := range ls {
		if filepath.Ext(name) == ".log" {
			logs = append(logs, name)
		}
	}
	sort.Strings(logs)
	if len(logs) < 2 {
		t.Fatalf("expected at least two log files, found %d", len(logs))
	}

	// Corrupt the (n-1)th WAL by zeroing four bytes, 100 bytes from the end
	// of the file.
	f, err := os.OpenFile(filepath.Join(dir, logs[len(logs)-2]), os.O_RDWR, os.ModePerm)
	require.NoError(t, err)
	off, err := f.Seek(-100, 2)
	require.NoError(t, err)
	_, err = f.Write([]byte{0, 0, 0, 0})
	require.NoError(t, err)
	require.NoError(t, f.Close())
	t.Logf("zeored four bytes in %s at offset %d\n", logs[len(logs)-2], off)

	// Re-opening the database should detect and report the corruption.
	_, err = Open(dir, nil)
	require.Error(t, err, "pebble: corruption")
}

// TestTwoWALReplayCorrupt tests WAL-replay behavior when the first of the two
// WALs is corrupted with an sstable checksum error and the OPTIONS file does
// not enable the private strict_wal_tail option, indicating that the WAL was
// produced by a database that did not guarantee clean WAL tails. See #864.
func TestTwoWALReplayPermissive(t *testing.T) {
	// Use the real filesystem so that we can seek and overwrite WAL data
	// easily.
	dir, err := os.MkdirTemp("", "wal-replay")
	require.NoError(t, err)
	defer os.RemoveAll(dir)

	opts := &Options{
		MemTableStopWritesThreshold: 4,
		MemTableSize:                2048,
	}
	opts.testingRandomized(t)
	opts.EnsureDefaults()
	d, err := Open(dir, opts)
	require.NoError(t, err)
	d.mu.Lock()
	d.mu.compact.flushing = true
	d.mu.Unlock()
	require.NoError(t, d.Set([]byte("1"), []byte(strings.Repeat("a", 1024)), nil))
	require.NoError(t, d.Set([]byte("2"), nil, nil))
	d.mu.Lock()
	d.mu.compact.flushing = false
	d.mu.Unlock()
	require.NoError(t, d.Close())

	// We should have two WALs.
	var logs []string
	var optionFilename string
	ls, err := vfs.Default.List(dir)
	require.NoError(t, err)
	for _, name := range ls {
		if filepath.Ext(name) == ".log" {
			logs = append(logs, name)
		}
		if strings.HasPrefix(filepath.Base(name), "OPTIONS") {
			optionFilename = name
		}
	}
	sort.Strings(logs)
	if len(logs) < 2 {
		t.Fatalf("expected at least two log files, found %d", len(logs))
	}

	// Corrupt the (n-1)th WAL by zeroing four bytes, 100 bytes from the end
	// of the file.
	f, err := os.OpenFile(filepath.Join(dir, logs[len(logs)-2]), os.O_RDWR, os.ModePerm)
	require.NoError(t, err)
	off, err := f.Seek(-100, 2)
	require.NoError(t, err)
	_, err = f.Write([]byte{0, 0, 0, 0})
	require.NoError(t, err)
	require.NoError(t, f.Close())
	t.Logf("zeored four bytes in %s at offset %d\n", logs[len(logs)-2], off)

	// Remove the OPTIONS file containing the strict_wal_tail option.
	require.NoError(t, vfs.Default.Remove(filepath.Join(dir, optionFilename)))

	// Re-opening the database should not report the corruption.
	d, err = Open(dir, nil)
	require.NoError(t, err)
	require.NoError(t, d.Close())
}

// TestCrashOpenCrashAfterWALCreation tests a database that exits
// ungracefully, begins recovery, creates the new WAL but promptly exits
// ungracefully again.
//
// This sequence has the potential to be problematic with the strict_wal_tail
// behavior because the first crash's WAL has an unclean tail. By the time the
// new WAL is created, the current manifest's MinUnflushedLogNum must be
// higher than the previous WAL.
func TestCrashOpenCrashAfterWALCreation(t *testing.T) {
	fs := vfs.NewStrictMem()

	getLogs := func() (logs []string) {
		ls, err := fs.List("")
		require.NoError(t, err)
		for _, name := range ls {
			if filepath.Ext(name) == ".log" {
				logs = append(logs, name)
			}
		}
		return logs
	}

	{
		d, err := Open("", testingRandomized(t, &Options{FS: fs}))
		require.NoError(t, err)
		require.NoError(t, d.Set([]byte("abc"), nil, Sync))

		// Ignore syncs during close to simulate a crash. This will leave the WAL
		// without an EOF trailer. It won't be an 'unclean tail' yet since the
		// log file was not recycled, but we'll fix that down below.
		fs.SetIgnoreSyncs(true)
		require.NoError(t, d.Close())
		fs.ResetToSyncedState()
		fs.SetIgnoreSyncs(false)
	}

	// There should be one WAL.
	logs := getLogs()
	if len(logs) != 1 {
		t.Fatalf("expected one log file, found %d", len(logs))
	}

	// The one WAL file doesn't have an EOF trailer, but since it wasn't
	// recycled it won't have garbage at the end. Rewrite it so that it has
	// the same contents it currently has, followed by garbage.
	{
		f, err := fs.Open(logs[0])
		require.NoError(t, err)
		b, err := io.ReadAll(f)
		require.NoError(t, err)
		require.NoError(t, f.Close())
		f, err = fs.Create(logs[0])
		require.NoError(t, err)
		_, err = f.Write(b)
		require.NoError(t, err)
		_, err = f.Write([]byte{0xde, 0xad, 0xbe, 0xef})
		require.NoError(t, err)
		require.NoError(t, f.Sync())
		require.NoError(t, f.Close())
		dir, err := fs.OpenDir("")
		require.NoError(t, err)
		require.NoError(t, dir.Sync())
		require.NoError(t, dir.Close())
	}

	// Open the database again (with syncs respected again). Wrap the
	// filesystem with an errorfs that will turn off syncs after a new .log
	// file is created and after a subsequent directory sync occurs. This
	// simulates a crash after the new log file is created and synced.
	{
		var walCreated, dirSynced atomic.Bool
		d, err := Open("", &Options{
			FS: errorfs.Wrap(fs, errorfs.InjectorFunc(func(op errorfs.Op, path string) error {
				if dirSynced.Load() {
					fs.SetIgnoreSyncs(true)
				}
				if op == errorfs.OpCreate && filepath.Ext(path) == ".log" {
					walCreated.Store(true)
				}
				// Record when there's a sync of the data directory after the
				// WAL was created. The data directory will have an empty
				// path because that's what we passed into Open.
				if op == errorfs.OpFileSync && path == "" && walCreated.Load() {
					dirSynced.Store(true)
				}
				return nil
			})),
		})
		require.NoError(t, err)
		require.NoError(t, d.Close())
	}

	fs.ResetToSyncedState()
	fs.SetIgnoreSyncs(false)

	if n := len(getLogs()); n != 2 {
		t.Fatalf("expected two logs, found %d\n", n)
	}

	// Finally, open the database with syncs enabled.
	d, err := Open("", testingRandomized(t, &Options{FS: fs}))
	require.NoError(t, err)
	require.NoError(t, d.Close())
}

// TestOpenWALReplayReadOnlySeqNums tests opening a database:
//   - in read-only mode
//   - with multiple unflushed log files that must replayed
//   - a MANIFEST that sets the last sequence number to a number greater than
//     the unflushed log files
//
// See cockroachdb/cockroach#48660.
func TestOpenWALReplayReadOnlySeqNums(t *testing.T) {
	const root = ""
	mem := vfs.NewMem()

	copyFiles := func(srcDir, dstDir string) {
		files, err := mem.List(srcDir)
		require.NoError(t, err)
		for _, f := range files {
			require.NoError(t, vfs.Copy(mem, mem.PathJoin(srcDir, f), mem.PathJoin(dstDir, f)))
		}
	}

	// Create a new database under `/original` with a couple sstables.
	dir := mem.PathJoin(root, "original")
	d, err := Open(dir, testingRandomized(t, &Options{FS: mem}))
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("a"), nil, nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Set([]byte("a"), nil, nil))
	require.NoError(t, d.Flush())

	// Prevent flushes so that multiple unflushed log files build up.
	d.mu.Lock()
	d.mu.compact.flushing = true
	d.mu.Unlock()

	require.NoError(t, d.Set([]byte("b"), nil, nil))
	d.AsyncFlush()
	require.NoError(t, d.Set([]byte("c"), nil, nil))
	d.AsyncFlush()
	require.NoError(t, d.Set([]byte("e"), nil, nil))

	// Manually compact some of the key space so that the latest `logSeqNum` is
	// written to the MANIFEST. This produces a MANIFEST where the `logSeqNum`
	// is greater than the sequence numbers contained in the
	// `minUnflushedLogNum` log file
	require.NoError(t, d.Compact([]byte("a"), []byte("a\x00"), false))
	d.mu.Lock()
	for d.mu.compact.compactingCount > 0 {
		d.mu.compact.cond.Wait()
	}
	d.mu.Unlock()

	d.TestOnlyWaitForCleaning()
	// While the MANIFEST is still in this state, copy all the files in the
	// database to a new directory.
	replayDir := mem.PathJoin(root, "replay")
	require.NoError(t, mem.MkdirAll(replayDir, os.ModePerm))
	copyFiles(dir, replayDir)

	d.mu.Lock()
	d.mu.compact.flushing = false
	d.mu.Unlock()
	require.NoError(t, d.Close())

	// Open the copy of the database in read-only mode. Since we copied all
	// the files before the flushes were allowed to complete, there should be
	// multiple unflushed log files that need to replay. Since the manual
	// compaction completed, the `logSeqNum` read from the manifest should be
	// greater than the unflushed log files' sequence numbers.
	d, err = Open(replayDir, testingRandomized(t, &Options{
		FS:       mem,
		ReadOnly: true,
	}))
	require.NoError(t, err)
	require.NoError(t, d.Close())
}

func TestOpenWALReplayMemtableGrowth(t *testing.T) {
	mem := vfs.NewMem()
	const memTableSize = 64 * 1024 * 1024
	opts := &Options{
		MemTableSize: memTableSize,
		FS:           mem,
	}
	opts.testingRandomized(t)
	func() {
		db, err := Open("", opts)
		require.NoError(t, err)
		defer db.Close()
		b := db.NewBatch()
		defer b.Close()
		key := make([]byte, 8)
		val := make([]byte, 16*1024*1024)
		b.Set(key, val, nil)
		require.NoError(t, db.Apply(b, Sync))
	}()
	db, err := Open("", opts)
	require.NoError(t, err)
	db.Close()
}

func TestGetVersion(t *testing.T) {
	mem := vfs.NewMem()
	opts := &Options{
		FS: mem,
	}
	opts.testingRandomized(t)

	// Case 1: No options file.
	version, err := GetVersion("", mem)
	require.NoError(t, err)
	require.Empty(t, version)

	// Case 2: Pebble created file.
	db, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, db.Close())
	version, err = GetVersion("", mem)
	require.NoError(t, err)
	require.Equal(t, "0.1", version)

	// Case 3: Manually created OPTIONS file with a higher number.
	highestOptionsNum := FileNum(0)
	ls, err := mem.List("")
	require.NoError(t, err)
	for _, filename := range ls {
		ft, fn, ok := base.ParseFilename(mem, filename)
		if !ok {
			continue
		}
		switch ft {
		case fileTypeOptions:
			if fn.FileNum() > highestOptionsNum {
				highestOptionsNum = fn.FileNum()
			}
		}
	}
	f, _ := mem.Create(fmt.Sprintf("OPTIONS-%d", highestOptionsNum+1))
	_, err = f.Write([]byte("[Version]\n  pebble_version=0.2\n"))
	require.NoError(t, err)
	err = f.Close()
	require.NoError(t, err)
	version, err = GetVersion("", mem)
	require.NoError(t, err)
	require.Equal(t, "0.2", version)

	// Case 4: Manually created OPTIONS file with a RocksDB number.
	f, _ = mem.Create(fmt.Sprintf("OPTIONS-%d", highestOptionsNum+2))
	_, err = f.Write([]byte("[Version]\n  rocksdb_version=6.2.1\n"))
	require.NoError(t, err)
	err = f.Close()
	require.NoError(t, err)
	version, err = GetVersion("", mem)
	require.NoError(t, err)
	require.Equal(t, "rocksdb v6.2.1", version)
}

func TestRocksDBNoFlushManifest(t *testing.T) {
	mem := vfs.NewMem()
	// Have the comparer and merger names match what's in the testdata
	// directory.
	comparer := *DefaultComparer
	merger := *DefaultMerger
	comparer.Name = "cockroach_comparator"
	merger.Name = "cockroach_merge_operator"
	opts := &Options{
		FS:       mem,
		Comparer: &comparer,
		Merger:   &merger,
	}

	// rocksdb-ingest-only is a RocksDB-generated db directory that has not had
	// a single flush yet, only ingestion operations. The manifest contains
	// a next-log-num but no log-num entry. Ensure that pebble can read these
	// directories without an issue.
	_, err := vfs.Clone(vfs.Default, mem, "testdata/rocksdb-ingest-only", "testdata")
	require.NoError(t, err)

	db, err := Open("testdata", opts)
	require.NoError(t, err)
	defer db.Close()

	val, closer, err := db.Get([]byte("ajulxeiombjiyw\x00\x00\x00\x00\x00\x00\x00\x01\x12\x09"))
	require.NoError(t, err)
	require.NotEmpty(t, val)
	require.NoError(t, closer.Close())
}

func TestOpen_ErrorIfUnknownFormatVersion(t *testing.T) {
	fs := vfs.NewMem()
	d, err := Open("", &Options{
		FS:                 fs,
		FormatMajorVersion: FormatVersioned,
	})
	require.NoError(t, err)
	require.NoError(t, d.Close())

	// Move the marker to a version that does not exist.
	m, _, err := atomicfs.LocateMarker(fs, "", formatVersionMarkerName)
	require.NoError(t, err)
	require.NoError(t, m.Move("999999"))
	require.NoError(t, m.Close())

	_, err = Open("", &Options{
		FS:                 fs,
		FormatMajorVersion: FormatVersioned,
	})
	require.Error(t, err)
	require.EqualError(t, err, `pebble: database "" written in format major version 999999`)
}

// ensureFilesClosed updates the provided Options to wrap the filesystem. It
// returns a closure that when invoked fails the test if any files opened by the
// filesystem are not closed.
//
// This function is intended to be used in tests with defer.
//
//	opts := &Options{FS: vfs.NewMem()}
//	defer ensureFilesClosed(t, opts)()
//	/* test code */
func ensureFilesClosed(t *testing.T, o *Options) func() {
	fs := &closeTrackingFS{
		FS:    o.FS,
		files: map[*closeTrackingFile]struct{}{},
	}
	o.FS = fs
	return func() {
		// fs.files should be empty if all the files were closed.
		for f := range fs.files {
			t.Errorf("An open file was never closed. Opened at:\n%s", f.stack)
		}
	}
}

type closeTrackingFS struct {
	vfs.FS
	files map[*closeTrackingFile]struct{}
}

func (fs *closeTrackingFS) wrap(file vfs.File, err error) (vfs.File, error) {
	if err != nil {
		return nil, err
	}
	f := &closeTrackingFile{
		File:  file,
		fs:    fs,
		stack: debug.Stack(),
	}
	fs.files[f] = struct{}{}
	return f, err
}

func (fs *closeTrackingFS) Create(name string) (vfs.File, error) {
	return fs.wrap(fs.FS.Create(name))
}

func (fs *closeTrackingFS) Open(name string, opts ...vfs.OpenOption) (vfs.File, error) {
	return fs.wrap(fs.FS.Open(name))
}

func (fs *closeTrackingFS) OpenDir(name string) (vfs.File, error) {
	return fs.wrap(fs.FS.OpenDir(name))
}

func (fs *closeTrackingFS) ReuseForWrite(oldname, newname string) (vfs.File, error) {
	return fs.wrap(fs.FS.ReuseForWrite(oldname, newname))
}

type closeTrackingFile struct {
	vfs.File
	fs    *closeTrackingFS
	stack []byte
}

func (f *closeTrackingFile) Close() error {
	delete(f.fs.files, f)
	return f.File.Close()
}

func TestCheckConsistency(t *testing.T) {
	const dir = "./test"
	mem := vfs.NewMem()
	mem.MkdirAll(dir, 0755)

	provider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(mem, dir))
	require.NoError(t, err)
	defer provider.Close()

	cmp := base.DefaultComparer.Compare
	fmtKey := base.DefaultComparer.FormatKey
	parseMeta := func(s string) (*manifest.FileMetadata, error) {
		if len(s) == 0 {
			return nil, nil
		}
		parts := strings.Split(s, ":")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %q", s)
		}
		fileNum, err := strconv.Atoi(strings.TrimSpace(parts[0]))
		if err != nil {
			return nil, err
		}
		size, err := strconv.Atoi(strings.TrimSpace(parts[1]))
		if err != nil {
			return nil, err
		}
		m := &manifest.FileMetadata{
			FileNum: base.FileNum(fileNum),
			Size:    uint64(size),
		}
		m.InitPhysicalBacking()
		return m, nil
	}

	datadriven.RunTest(t, "testdata/version_check_consistency",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "check-consistency":
				var filesByLevel [manifest.NumLevels][]*manifest.FileMetadata
				var files *[]*manifest.FileMetadata

				for _, data := range strings.Split(d.Input, "\n") {
					switch data {
					case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
						level, err := strconv.Atoi(data[1:])
						if err != nil {
							return err.Error()
						}
						files = &filesByLevel[level]

					default:
						m, err := parseMeta(data)
						if err != nil {
							return err.Error()
						}
						if m != nil {
							*files = append(*files, m)
						}
					}
				}

				redactErr := false
				for _, arg := range d.CmdArgs {
					switch v := arg.String(); v {
					case "redact":
						redactErr = true
					default:
						return fmt.Sprintf("unknown argument: %q", v)
					}
				}

				v := manifest.NewVersion(cmp, fmtKey, 0, filesByLevel)
				err := checkConsistency(v, dir, provider)
				if err != nil {
					if redactErr {
						redacted := redact.Sprint(err).Redact()
						return string(redacted)
					}
					return err.Error()
				}
				return "OK"

			case "build":
				for _, data := range strings.Split(d.Input, "\n") {
					m, err := parseMeta(data)
					if err != nil {
						return err.Error()
					}
					path := base.MakeFilepath(mem, dir, base.FileTypeTable, m.FileBacking.DiskFileNum)
					_ = mem.Remove(path)
					f, err := mem.Create(path)
					if err != nil {
						return err.Error()
					}
					_, err = f.Write(make([]byte, m.Size))
					if err != nil {
						return err.Error()
					}
					f.Close()
				}
				return ""

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestOpenRatchetsNextFileNum(t *testing.T) {
	mem := vfs.NewMem()
	memShared := remote.NewInMem()

	opts := &Options{FS: mem}
	opts.Experimental.CreateOnShared = remote.CreateOnSharedAll
	opts.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
		"": memShared,
	})
	d, err := Open("", opts)
	require.NoError(t, err)
	d.SetCreatorID(1)

	require.NoError(t, d.Set([]byte("foo"), []byte("value"), nil))
	require.NoError(t, d.Set([]byte("bar"), []byte("value"), nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Compact([]byte("a"), []byte("z"), false))

	// Create a shared file with the newest file num and then close the db.
	d.mu.Lock()
	nextFileNum := d.mu.versions.getNextFileNum()
	w, _, err := d.objProvider.Create(context.TODO(), fileTypeTable, nextFileNum.DiskFileNum(), objstorage.CreateOptions{PreferSharedStorage: true})
	require.NoError(t, err)
	require.NoError(t, w.Write([]byte("foobar")))
	require.NoError(t, w.Finish())
	require.NoError(t, d.objProvider.Sync())
	d.mu.Unlock()

	// Write one key and then close the db. This write will stay in the memtable,
	// forcing the reopen to do a compaction on open.
	require.NoError(t, d.Set([]byte("foo1"), []byte("value"), nil))
	require.NoError(t, d.Close())

	// Reopen db. Compactions should happen without error.
	d, err = Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("foo2"), []byte("value"), nil))
	require.NoError(t, d.Set([]byte("bar2"), []byte("value"), nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Compact([]byte("a"), []byte("z"), false))

}
