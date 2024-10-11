// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"math"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/errorfs"
	"github.com/kr/pretty"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestSSTableKeyCompare(t *testing.T) {
	var buf bytes.Buffer
	datadriven.RunTest(t, "testdata/sstable_key_compare", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "cmp":
			buf.Reset()
			for _, line := range strings.Split(td.Input, "\n") {
				fields := strings.Fields(line)
				a := base.ParseInternalKey(fields[0])
				b := base.ParseInternalKey(fields[1])
				got := sstableKeyCompare(testkeys.Comparer.Compare, a, b)
				fmt.Fprintf(&buf, "%38s", fmt.Sprint(a.Pretty(base.DefaultFormatter)))
				switch got {
				case -1:
					fmt.Fprint(&buf, " < ")
				case +1:
					fmt.Fprint(&buf, " > ")
				case 0:
					fmt.Fprint(&buf, " = ")
				}
				fmt.Fprintf(&buf, "%s\n", fmt.Sprint(b.Pretty(base.DefaultFormatter)))
			}
			return buf.String()
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func TestIngestLoad(t *testing.T) {
	mem := vfs.NewMem()

	datadriven.RunTest(t, "testdata/ingest_load", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "load":
			writerOpts := sstable.WriterOptions{}
			var dbVersion FormatMajorVersion
			for _, cmdArgs := range td.CmdArgs {
				v, err := strconv.Atoi(cmdArgs.Vals[0])
				if err != nil {
					return err.Error()
				}
				switch k := cmdArgs.Key; k {
				case "writer-version":
					fmv := FormatMajorVersion(v)
					writerOpts.TableFormat = fmv.MaxTableFormat()
				case "db-version":
					dbVersion = FormatMajorVersion(v)
				default:
					return fmt.Sprintf("unknown cmd %s\n", k)
				}
			}
			f, err := mem.Create("ext")
			if err != nil {
				return err.Error()
			}
			w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), writerOpts)
			for _, data := range strings.Split(td.Input, "\n") {
				if strings.HasPrefix(data, "rangekey: ") {
					data = strings.TrimPrefix(data, "rangekey: ")
					s := keyspan.ParseSpan(data)
					err := rangekey.Encode(&s, w.AddRangeKey)
					if err != nil {
						return err.Error()
					}
					continue
				}

				j := strings.Index(data, ":")
				if j < 0 {
					return fmt.Sprintf("malformed input: %s\n", data)
				}
				key := base.ParseInternalKey(data[:j])
				value := []byte(data[j+1:])
				if err := w.Add(key, value); err != nil {
					return err.Error()
				}
			}
			if err := w.Close(); err != nil {
				return err.Error()
			}

			opts := (&Options{
				Comparer: DefaultComparer,
				FS:       mem,
			}).WithFSDefaults()
			lr, err := ingestLoad(opts, dbVersion, []string{"ext"}, nil, nil, 0, []base.DiskFileNum{base.FileNum(1).DiskFileNum()}, nil, 0)
			if err != nil {
				return err.Error()
			}
			var buf bytes.Buffer
			for _, m := range lr.localMeta {
				fmt.Fprintf(&buf, "%d: %s-%s\n", m.FileNum, m.Smallest, m.Largest)
				fmt.Fprintf(&buf, "  points: %s-%s\n", m.SmallestPointKey, m.LargestPointKey)
				fmt.Fprintf(&buf, "  ranges: %s-%s\n", m.SmallestRangeKey, m.LargestRangeKey)
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestIngestLoadRand(t *testing.T) {
	mem := vfs.NewMem()
	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
	cmp := DefaultComparer.Compare
	version := internalFormatNewest

	randBytes := func(size int) []byte {
		data := make([]byte, size)
		for i := range data {
			data[i] = byte(rng.Int() & 0xff)
		}
		return data
	}

	paths := make([]string, 1+rng.Intn(10))
	pending := make([]base.DiskFileNum, len(paths))
	expected := make([]*fileMetadata, len(paths))
	for i := range paths {
		paths[i] = fmt.Sprint(i)
		pending[i] = base.FileNum(rng.Uint64()).DiskFileNum()
		expected[i] = &fileMetadata{
			FileNum: pending[i].FileNum(),
		}
		expected[i].StatsMarkValid()

		func() {
			f, err := mem.Create(paths[i])
			require.NoError(t, err)

			keys := make([]InternalKey, 1+rng.Intn(100))
			for i := range keys {
				keys[i] = base.MakeInternalKey(
					randBytes(1+rng.Intn(10)),
					0,
					InternalKeyKindSet)
			}
			sort.Slice(keys, func(i, j int) bool {
				return base.InternalCompare(cmp, keys[i], keys[j]) < 0
			})

			expected[i].ExtendPointKeyBounds(cmp, keys[0], keys[len(keys)-1])

			w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
				TableFormat: version.MaxTableFormat(),
			})
			var count uint64
			for i := range keys {
				if i > 0 && base.InternalCompare(cmp, keys[i-1], keys[i]) == 0 {
					// Duplicate key, ignore.
					continue
				}
				w.Add(keys[i], nil)
				count++
			}
			expected[i].Stats.NumEntries = count
			require.NoError(t, w.Close())

			meta, err := w.Metadata()
			require.NoError(t, err)

			expected[i].Size = meta.Size
			expected[i].InitPhysicalBacking()
		}()
	}

	opts := (&Options{
		Comparer: DefaultComparer,
		FS:       mem,
	}).WithFSDefaults()
	lr, err := ingestLoad(opts, version, paths, nil, nil, 0, pending, nil, 0)
	require.NoError(t, err)

	for _, m := range lr.localMeta {
		m.CreationTime = 0
	}
	t.Log(strings.Join(pretty.Diff(expected, lr.localMeta), "\n"))
	require.Equal(t, expected, lr.localMeta)
}

func TestIngestLoadInvalid(t *testing.T) {
	mem := vfs.NewMem()
	f, err := mem.Create("invalid")
	require.NoError(t, err)
	require.NoError(t, f.Close())

	opts := (&Options{
		Comparer: DefaultComparer,
		FS:       mem,
	}).WithFSDefaults()
	if _, err := ingestLoad(opts, internalFormatNewest, []string{"invalid"}, nil, nil, 0, []base.DiskFileNum{base.FileNum(1).DiskFileNum()}, nil, 0); err == nil {
		t.Fatalf("expected error, but found success")
	}
}

func TestIngestSortAndVerify(t *testing.T) {
	comparers := map[string]Compare{
		"default": DefaultComparer.Compare,
		"reverse": func(a, b []byte) int {
			return DefaultComparer.Compare(b, a)
		},
	}

	t.Run("", func(t *testing.T) {
		datadriven.RunTest(t, "testdata/ingest_sort_and_verify", func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "ingest":
				var buf bytes.Buffer
				var meta []*fileMetadata
				var paths []string
				var cmpName string
				d.ScanArgs(t, "cmp", &cmpName)
				cmp := comparers[cmpName]
				if cmp == nil {
					return fmt.Sprintf("%s unknown comparer: %s", d.Cmd, cmpName)
				}
				for i, data := range strings.Split(d.Input, "\n") {
					parts := strings.Split(data, "-")
					if len(parts) != 2 {
						return fmt.Sprintf("malformed test case: %s", d.Input)
					}
					smallest := base.ParseInternalKey(parts[0])
					largest := base.ParseInternalKey(parts[1])
					if cmp(smallest.UserKey, largest.UserKey) > 0 {
						return fmt.Sprintf("range %v-%v is not valid", smallest, largest)
					}
					m := (&fileMetadata{}).ExtendPointKeyBounds(cmp, smallest, largest)
					m.InitPhysicalBacking()
					meta = append(meta, m)
					paths = append(paths, strconv.Itoa(i))
				}
				lr := ingestLoadResult{localPaths: paths, localMeta: meta}
				err := ingestSortAndVerify(cmp, lr, KeyRange{})
				if err != nil {
					return fmt.Sprintf("%v\n", err)
				}
				for i := range meta {
					fmt.Fprintf(&buf, "%s: %v-%v\n", paths[i], meta[i].Smallest, meta[i].Largest)
				}
				return buf.String()

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
	})
}

func TestIngestLink(t *testing.T) {
	// Test linking of tables into the DB directory. Test cleanup when one of the
	// tables cannot be linked.

	const dir = "db"
	const count = 10
	for i := 0; i <= count; i++ {
		t.Run("", func(t *testing.T) {
			opts := &Options{FS: vfs.NewMem()}
			opts.EnsureDefaults().WithFSDefaults()
			require.NoError(t, opts.FS.MkdirAll(dir, 0755))
			objProvider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(opts.FS, dir))
			require.NoError(t, err)
			defer objProvider.Close()

			paths := make([]string, 10)
			meta := make([]*fileMetadata, len(paths))
			contents := make([][]byte, len(paths))
			for j := range paths {
				paths[j] = fmt.Sprintf("external%d", j)
				meta[j] = &fileMetadata{}
				meta[j].FileNum = FileNum(j)
				meta[j].InitPhysicalBacking()
				f, err := opts.FS.Create(paths[j])
				require.NoError(t, err)

				contents[j] = []byte(fmt.Sprintf("data%d", j))
				// memFile.Write will modify the supplied buffer when invariants are
				// enabled, so provide a throw-away copy.
				_, err = f.Write(append([]byte(nil), contents[j]...))
				require.NoError(t, err)
				require.NoError(t, f.Close())
			}

			if i < count {
				opts.FS.Remove(paths[i])
			}

			lr := ingestLoadResult{localMeta: meta, localPaths: paths}
			err = ingestLink(0 /* jobID */, opts, objProvider, lr, nil /* shared */)
			if i < count {
				if err == nil {
					t.Fatalf("expected error, but found success")
				}
			} else {
				require.NoError(t, err)
			}

			files, err := opts.FS.List(dir)
			require.NoError(t, err)

			sort.Strings(files)

			if i < count {
				if len(files) > 0 {
					t.Fatalf("expected all of the files to be cleaned up, but found:\n%s",
						strings.Join(files, "\n"))
				}
			} else {
				if len(files) != count {
					t.Fatalf("expected %d files, but found:\n%s", count, strings.Join(files, "\n"))
				}
				for j := range files {
					ftype, fileNum, ok := base.ParseFilename(opts.FS, files[j])
					if !ok {
						t.Fatalf("unable to parse filename: %s", files[j])
					}
					if fileTypeTable != ftype {
						t.Fatalf("expected table, but found %d", ftype)
					}
					if j != int(fileNum.FileNum()) {
						t.Fatalf("expected table %d, but found %d", j, fileNum)
					}
					f, err := opts.FS.Open(opts.FS.PathJoin(dir, files[j]))
					require.NoError(t, err)

					data, err := io.ReadAll(f)
					require.NoError(t, err)
					require.NoError(t, f.Close())
					if !bytes.Equal(contents[j], data) {
						t.Fatalf("expected %s, but found %s", contents[j], data)
					}
				}
			}
		})
	}
}

func TestIngestLinkFallback(t *testing.T) {
	// Verify that ingestLink succeeds if linking fails by falling back to
	// copying.
	mem := vfs.NewMem()
	src, err := mem.Create("source")
	require.NoError(t, err)

	opts := &Options{FS: errorfs.Wrap(mem, errorfs.OnIndex(1))}
	opts.EnsureDefaults().WithFSDefaults()
	objSettings := objstorageprovider.DefaultSettings(opts.FS, "")
	// Prevent the provider from listing the dir (where we may get an injected error).
	objSettings.FSDirInitialListing = []string{}
	objProvider, err := objstorageprovider.Open(objSettings)
	require.NoError(t, err)
	defer objProvider.Close()

	meta := []*fileMetadata{{FileNum: 1}}
	meta[0].InitPhysicalBacking()
	lr := ingestLoadResult{localMeta: meta, localPaths: []string{"source"}}
	err = ingestLink(0, opts, objProvider, lr, nil /* shared */)
	require.NoError(t, err)

	dest, err := mem.Open("000001.sst")
	require.NoError(t, err)

	// We should be able to write bytes to src, and not have them show up in
	// dest.
	_, _ = src.Write([]byte("test"))
	data, err := io.ReadAll(dest)
	require.NoError(t, err)
	if len(data) != 0 {
		t.Fatalf("expected copy, but files appear to be hard linked: [%s] unexpectedly found", data)
	}
}

func TestOverlappingIngestedSSTs(t *testing.T) {
	dir := ""
	var (
		mem        vfs.FS
		d          *DB
		opts       *Options
		closed     = false
		blockFlush = false
	)
	defer func() {
		if !closed {
			require.NoError(t, d.Close())
		}
	}()

	reset := func(strictMem bool) {
		if d != nil && !closed {
			require.NoError(t, d.Close())
		}
		blockFlush = false

		if strictMem {
			mem = vfs.NewStrictMem()
		} else {
			mem = vfs.NewMem()
		}

		require.NoError(t, mem.MkdirAll("ext", 0755))
		opts = (&Options{
			FS:                          mem,
			MemTableStopWritesThreshold: 4,
			L0CompactionThreshold:       100,
			L0StopWritesThreshold:       100,
			DebugCheck:                  DebugCheckLevels,
			FormatMajorVersion:          internalFormatNewest,
		}).WithFSDefaults()
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts.DisableAutomaticCompactions = true

		var err error
		d, err = Open(dir, opts)
		require.NoError(t, err)
		d.TestOnlyWaitForCleaning()
	}
	waitForFlush := func() {
		if d == nil {
			return
		}
		d.mu.Lock()
		for d.mu.compact.flushing {
			d.mu.compact.cond.Wait()
		}
		d.mu.Unlock()
	}
	reset(false)

	datadriven.RunTest(t, "testdata/flushable_ingest", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset(td.HasArg("strictMem"))
			return ""

		case "ignoreSyncs":
			var ignoreSyncs bool
			if len(td.CmdArgs) == 1 && td.CmdArgs[0].String() == "true" {
				ignoreSyncs = true
			}
			mem.(*vfs.MemFS).SetIgnoreSyncs(ignoreSyncs)
			return ""

		case "resetToSynced":
			mem.(*vfs.MemFS).ResetToSyncedState()
			files, err := mem.List(dir)
			sort.Strings(files)
			require.NoError(t, err)
			return strings.Join(files, "\n")

		case "batch":
			b := d.NewIndexedBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""

		case "build":
			if err := runBuildCmd(td, d, mem); err != nil {
				return err.Error()
			}
			return ""

		case "ingest":
			if err := runIngestCmd(td, d, mem); err != nil {
				return err.Error()
			}
			if !blockFlush {
				waitForFlush()
			}
			return ""

		case "iter":
			iter, _ := d.NewIter(nil)
			return runIterCmd(td, iter, true)

		case "lsm":
			return runLSMCmd(td, d)

		case "close":
			if closed {
				return "already closed"
			}
			require.NoError(t, d.Close())
			closed = true
			return ""

		case "ls":
			files, err := mem.List(dir)
			sort.Strings(files)
			require.NoError(t, err)
			return strings.Join(files, "\n")

		case "open":
			opts.ReadOnly = td.HasArg("readOnly")
			var err error
			d, err = Open(dir, opts)
			closed = false
			require.NoError(t, err)
			waitForFlush()
			d.TestOnlyWaitForCleaning()
			return ""

		case "blockFlush":
			blockFlush = true
			d.mu.Lock()
			d.mu.compact.flushing = true
			d.mu.Unlock()
			return ""

		case "allowFlush":
			blockFlush = false
			d.mu.Lock()
			d.mu.compact.flushing = false
			d.mu.Unlock()
			return ""

		case "flush":
			d.maybeScheduleFlush()
			waitForFlush()
			d.TestOnlyWaitForCleaning()
			return ""

		case "get":
			return runGetCmd(t, td, d)

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestExcise(t *testing.T) {
	var mem vfs.FS
	var d *DB
	var flushed bool
	defer func() {
		require.NoError(t, d.Close())
	}()

	var opts *Options
	reset := func() {
		if d != nil {
			require.NoError(t, d.Close())
		}

		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))
		opts = &Options{
			FS:                    mem,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
			DebugCheck:            DebugCheckLevels,
			EventListener: &EventListener{FlushEnd: func(info FlushInfo) {
				flushed = true
			}},
			FormatMajorVersion: FormatVirtualSSTables,
			Comparer:           testkeys.Comparer,
		}
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts.DisableAutomaticCompactions = true
		// Set this to true to add some testing for the virtual sstable validation
		// code paths.
		opts.Experimental.ValidateOnIngest = true

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
	}
	reset()

	datadriven.RunTest(t, "testdata/excise", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset()
			return ""
		case "reopen":
			require.NoError(t, d.Close())
			var err error
			d, err = Open("", opts)
			require.NoError(t, err)

			return ""
		case "batch":
			b := d.NewIndexedBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "build":
			if err := runBuildCmd(td, d, mem); err != nil {
				return err.Error()
			}
			return ""

		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return ""

		case "ingest":
			flushed = false
			if err := runIngestCmd(td, d, mem); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			if flushed {
				return "memtable flushed"
			}
			return ""

		case "ingest-and-excise":
			flushed = false
			if err := runIngestAndExciseCmd(td, d, mem); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			if flushed {
				return "memtable flushed"
			}
			return ""

		case "get":
			return runGetCmd(t, td, d)

		case "iter":
			iter, _ := d.NewIter(&IterOptions{
				KeyTypes: IterKeyTypePointsAndRanges,
			})
			return runIterCmd(td, iter, true)

		case "lsm":
			return runLSMCmd(td, d)

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "wait-pending-table-stats":
			return runTableStatsCmd(td, d)

		case "excise":
			ve := &versionEdit{
				DeletedFiles: map[deletedFileEntry]*fileMetadata{},
			}
			var exciseSpan KeyRange
			if len(td.CmdArgs) != 2 {
				panic("insufficient args for compact command")
			}
			exciseSpan.Start = []byte(td.CmdArgs[0].Key)
			exciseSpan.End = []byte(td.CmdArgs[1].Key)

			d.mu.Lock()
			d.mu.versions.logLock()
			d.mu.Unlock()
			current := d.mu.versions.currentVersion()
			for level := range current.Levels {
				iter := current.Levels[level].Iter()
				for m := iter.SeekGE(d.cmp, exciseSpan.Start); m != nil && d.cmp(m.Smallest.UserKey, exciseSpan.End) < 0; m = iter.Next() {
					_, err := d.excise(exciseSpan, m, ve, level)
					if err != nil {
						d.mu.Lock()
						d.mu.versions.logUnlock()
						d.mu.Unlock()
						return fmt.Sprintf("error when excising %s: %s", m.FileNum, err.Error())
					}
				}
			}
			d.mu.Lock()
			d.mu.versions.logUnlock()
			d.mu.Unlock()
			return fmt.Sprintf("would excise %d files, use ingest-and-excise to excise.\n%s", len(ve.DeletedFiles), ve.DebugString(base.DefaultFormatter))

		case "confirm-backing":
			// Confirms that the files have the same FileBacking.
			fileNums := make(map[base.FileNum]struct{})
			for i := range td.CmdArgs {
				fNum, err := strconv.Atoi(td.CmdArgs[i].Key)
				if err != nil {
					panic("invalid file number")
				}
				fileNums[base.FileNum(fNum)] = struct{}{}
			}
			d.mu.Lock()
			currVersion := d.mu.versions.currentVersion()
			var ptr *manifest.FileBacking
			for _, level := range currVersion.Levels {
				lIter := level.Iter()
				for f := lIter.First(); f != nil; f = lIter.Next() {
					if _, ok := fileNums[f.FileNum]; ok {
						if ptr == nil {
							ptr = f.FileBacking
							continue
						}
						if f.FileBacking != ptr {
							d.mu.Unlock()
							return "file backings are not the same"
						}
					}
				}
			}
			d.mu.Unlock()
			return "file backings are the same"
		case "compact":
			if len(td.CmdArgs) != 2 {
				panic("insufficient args for compact command")
			}
			l := td.CmdArgs[0].Key
			r := td.CmdArgs[1].Key
			err := d.Compact([]byte(l), []byte(r), false)
			if err != nil {
				return err.Error()
			}
			return ""
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func testIngestSharedImpl(
	t *testing.T, createOnShared remote.CreateOnSharedStrategy, fileName string,
) {
	var d, d1, d2 *DB
	var efos map[string]*EventuallyFileOnlySnapshot
	defer func() {
		for _, e := range efos {
			require.NoError(t, e.Close())
		}
		if d1 != nil {
			require.NoError(t, d1.Close())
		}
		if d2 != nil {
			require.NoError(t, d2.Close())
		}
	}()
	creatorIDCounter := uint64(1)
	replicateCounter := 1

	reset := func() {
		for _, e := range efos {
			require.NoError(t, e.Close())
		}
		if d1 != nil {
			require.NoError(t, d1.Close())
		}
		if d2 != nil {
			require.NoError(t, d2.Close())
		}
		efos = make(map[string]*EventuallyFileOnlySnapshot)

		sstorage := remote.NewInMem()
		mem1 := vfs.NewMem()
		mem2 := vfs.NewMem()
		require.NoError(t, mem1.MkdirAll("ext", 0755))
		require.NoError(t, mem2.MkdirAll("ext", 0755))
		opts1 := &Options{
			Comparer:              testkeys.Comparer,
			FS:                    mem1,
			LBaseMaxBytes:         1,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
			DebugCheck:            DebugCheckLevels,
			FormatMajorVersion:    FormatVirtualSSTables,
		}
		// lel.
		lel := MakeLoggingEventListener(DefaultLogger)
		opts1.EventListener = &lel
		opts1.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
			"": sstorage,
		})
		opts1.Experimental.CreateOnShared = createOnShared
		opts1.Experimental.CreateOnSharedLocator = ""
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts1.DisableAutomaticCompactions = true

		opts2 := &Options{}
		*opts2 = *opts1
		opts2.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
			"": sstorage,
		})
		opts2.Experimental.CreateOnShared = createOnShared
		opts2.Experimental.CreateOnSharedLocator = ""
		opts2.FS = mem2

		var err error
		d1, err = Open("", opts1)
		require.NoError(t, err)
		require.NoError(t, d1.SetCreatorID(creatorIDCounter))
		creatorIDCounter++
		d2, err = Open("", opts2)
		require.NoError(t, err)
		require.NoError(t, d2.SetCreatorID(creatorIDCounter))
		creatorIDCounter++
		d = d1
	}
	reset()

	datadriven.RunTest(t, fmt.Sprintf("testdata/%s", fileName), func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset()
			return ""
		case "switch":
			if len(td.CmdArgs) != 1 {
				return "usage: switch <1 or 2>"
			}
			switch td.CmdArgs[0].Key {
			case "1":
				d = d1
			case "2":
				d = d2
			default:
				return "usage: switch <1 or 2>"
			}
			return "ok"
		case "batch":
			b := d.NewIndexedBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "build":
			if err := runBuildCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			return ""

		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return ""

		case "ingest":
			if err := runIngestCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			return ""

		case "ingest-and-excise":
			if err := runIngestAndExciseCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			return ""

		case "replicate":
			if len(td.CmdArgs) != 4 {
				return "usage: replicate <from-db-num> <to-db-num> <start-key> <end-key>"
			}
			var from, to *DB
			switch td.CmdArgs[0].Key {
			case "1":
				from = d1
			case "2":
				from = d2
			default:
				return "usage: replicate <from-db-num> <to-db-num> <start-key> <end-key>"
			}
			switch td.CmdArgs[1].Key {
			case "1":
				to = d1
			case "2":
				to = d2
			default:
				return "usage: replicate <from-db-num> <to-db-num> <start-key> <end-key>"
			}
			startKey := []byte(td.CmdArgs[2].Key)
			endKey := []byte(td.CmdArgs[3].Key)

			writeOpts := d.opts.MakeWriterOptions(0 /* level */, to.opts.FormatMajorVersion.MaxTableFormat())
			sstPath := fmt.Sprintf("ext/replicate%d.sst", replicateCounter)
			f, err := to.opts.FS.Create(sstPath)
			require.NoError(t, err)
			replicateCounter++
			w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), writeOpts)

			var sharedSSTs []SharedSSTMeta
			err = from.ScanInternal(context.TODO(), startKey, endKey,
				func(key *InternalKey, value LazyValue, _ IteratorLevel) error {
					val, _, err := value.Value(nil)
					require.NoError(t, err)
					require.NoError(t, w.Add(base.MakeInternalKey(key.UserKey, 0, key.Kind()), val))
					return nil
				},
				func(start, end []byte, seqNum uint64) error {
					require.NoError(t, w.DeleteRange(start, end))
					return nil
				},
				func(start, end []byte, keys []keyspan.Key) error {
					s := keyspan.Span{
						Start:     start,
						End:       end,
						Keys:      keys,
						KeysOrder: 0,
					}
					require.NoError(t, rangekey.Encode(&s, func(k base.InternalKey, v []byte) error {
						return w.AddRangeKey(base.MakeInternalKey(k.UserKey, 0, k.Kind()), v)
					}))
					return nil
				},
				func(sst *SharedSSTMeta) error {
					sharedSSTs = append(sharedSSTs, *sst)
					return nil
				},
			)
			require.NoError(t, err)
			require.NoError(t, w.Close())

			_, err = to.IngestAndExcise([]string{sstPath}, sharedSSTs, KeyRange{Start: startKey, End: endKey})
			require.NoError(t, err)
			return fmt.Sprintf("replicated %d shared SSTs", len(sharedSSTs))

		case "get":
			return runGetCmd(t, td, d)

		case "iter":
			o := &IterOptions{KeyTypes: IterKeyTypePointsAndRanges}
			var reader Reader
			reader = d
			for _, arg := range td.CmdArgs {
				switch arg.Key {
				case "mask-suffix":
					o.RangeKeyMasking.Suffix = []byte(arg.Vals[0])
				case "mask-filter":
					o.RangeKeyMasking.Filter = func() BlockPropertyFilterMask {
						return sstable.NewTestKeysMaskingFilter()
					}
				case "snapshot":
					reader = efos[arg.Vals[0]]
				}
			}
			iter, err := reader.NewIter(o)
			if err != nil {
				return err.Error()
			}
			return runIterCmd(td, iter, true)

		case "lsm":
			return runLSMCmd(td, d)

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "wait-pending-table-stats":
			return runTableStatsCmd(td, d)

		case "excise":
			ve := &versionEdit{
				DeletedFiles: map[deletedFileEntry]*fileMetadata{},
			}
			var exciseSpan KeyRange
			if len(td.CmdArgs) != 2 {
				panic("insufficient args for excise command")
			}
			exciseSpan.Start = []byte(td.CmdArgs[0].Key)
			exciseSpan.End = []byte(td.CmdArgs[1].Key)

			d.mu.Lock()
			d.mu.versions.logLock()
			d.mu.Unlock()
			current := d.mu.versions.currentVersion()
			for level := range current.Levels {
				iter := current.Levels[level].Iter()
				for m := iter.SeekGE(d.cmp, exciseSpan.Start); m != nil && d.cmp(m.Smallest.UserKey, exciseSpan.End) < 0; m = iter.Next() {
					_, err := d.excise(exciseSpan, m, ve, level)
					if err != nil {
						d.mu.Lock()
						d.mu.versions.logUnlock()
						d.mu.Unlock()
						return fmt.Sprintf("error when excising %s: %s", m.FileNum, err.Error())
					}
				}
			}
			d.mu.Lock()
			d.mu.versions.logUnlock()
			d.mu.Unlock()
			return fmt.Sprintf("would excise %d files, use ingest-and-excise to excise.\n%s", len(ve.DeletedFiles), ve.String())

		case "file-only-snapshot":
			if len(td.CmdArgs) != 1 {
				panic("insufficient args for file-only-snapshot command")
			}
			name := td.CmdArgs[0].Key
			var keyRanges []KeyRange
			for _, line := range strings.Split(td.Input, "\n") {
				fields := strings.Fields(line)
				if len(fields) != 2 {
					return "expected two fields for file-only snapshot KeyRanges"
				}
				kr := KeyRange{Start: []byte(fields[0]), End: []byte(fields[1])}
				keyRanges = append(keyRanges, kr)
			}

			s := d.NewEventuallyFileOnlySnapshot(keyRanges)
			efos[name] = s
			return "ok"

		case "wait-for-file-only-snapshot":
			if len(td.CmdArgs) != 1 {
				panic("insufficient args for file-only-snapshot command")
			}
			name := td.CmdArgs[0].Key
			err := efos[name].WaitForFileOnlySnapshot(context.TODO(), 1*time.Millisecond)
			if err != nil {
				return err.Error()
			}
			return "ok"

		case "compact":
			err := runCompactCmd(td, d)
			if err != nil {
				return err.Error()
			}
			return "ok"
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestIngestShared(t *testing.T) {
	for _, strategy := range []remote.CreateOnSharedStrategy{remote.CreateOnSharedAll, remote.CreateOnSharedLower} {
		strategyStr := "all"
		if strategy == remote.CreateOnSharedLower {
			strategyStr = "lower"
		}
		t.Run(fmt.Sprintf("createOnShared=%s", strategyStr), func(t *testing.T) {
			fileName := "ingest_shared"
			if strategy == remote.CreateOnSharedLower {
				fileName = "ingest_shared_lower"
			}
			testIngestSharedImpl(t, strategy, fileName)
		})
	}
}

func TestSimpleIngestShared(t *testing.T) {
	mem := vfs.NewMem()
	var d *DB
	var provider2 objstorage.Provider
	opts2 := Options{FS: vfs.NewMem(), FormatMajorVersion: FormatVirtualSSTables}
	opts2.EnsureDefaults()

	// Create an objProvider where we will fake-create some sstables that can
	// then be shared back to the db instance.
	providerSettings := objstorageprovider.Settings{
		Logger:              opts2.Logger,
		FS:                  opts2.FS,
		FSDirName:           "",
		FSDirInitialListing: nil,
		FSCleaner:           opts2.Cleaner,
		NoSyncOnClose:       opts2.NoSyncOnClose,
		BytesPerSync:        opts2.BytesPerSync,
	}
	providerSettings.Remote.StorageFactory = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
		"": remote.NewInMem(),
	})
	providerSettings.Remote.CreateOnShared = remote.CreateOnSharedAll
	providerSettings.Remote.CreateOnSharedLocator = ""

	provider2, err := objstorageprovider.Open(providerSettings)
	require.NoError(t, err)
	creatorIDCounter := uint64(1)
	provider2.SetCreatorID(objstorage.CreatorID(creatorIDCounter))
	creatorIDCounter++

	defer func() {
		require.NoError(t, d.Close())
	}()

	reset := func() {
		if d != nil {
			require.NoError(t, d.Close())
		}

		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))
		opts := &Options{
			FormatMajorVersion:    FormatVirtualSSTables,
			FS:                    mem,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
		}
		opts.Experimental.RemoteStorage = providerSettings.Remote.StorageFactory
		opts.Experimental.CreateOnShared = providerSettings.Remote.CreateOnShared
		opts.Experimental.CreateOnSharedLocator = providerSettings.Remote.CreateOnSharedLocator

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
		require.NoError(t, d.SetCreatorID(creatorIDCounter))
		creatorIDCounter++
	}
	reset()

	metaMap := map[base.DiskFileNum]objstorage.ObjectMetadata{}

	require.NoError(t, d.Set([]byte("d"), []byte("unexpected"), nil))
	require.NoError(t, d.Set([]byte("e"), []byte("unexpected"), nil))
	require.NoError(t, d.Set([]byte("a"), []byte("unexpected"), nil))
	require.NoError(t, d.Set([]byte("f"), []byte("unexpected"), nil))
	d.Flush()

	{
		// Create a shared file.
		fn := base.FileNum(2)
		f, meta, err := provider2.Create(context.TODO(), fileTypeTable, fn.DiskFileNum(), objstorage.CreateOptions{PreferSharedStorage: true})
		require.NoError(t, err)
		w := sstable.NewWriter(f, d.opts.MakeWriterOptions(0, d.opts.FormatMajorVersion.MaxTableFormat()))
		w.Set([]byte("d"), []byte("shared"))
		w.Set([]byte("e"), []byte("shared"))
		w.Close()
		metaMap[fn.DiskFileNum()] = meta
	}

	m := metaMap[base.FileNum(2).DiskFileNum()]
	handle, err := provider2.RemoteObjectBacking(&m)
	require.NoError(t, err)
	size, err := provider2.Size(m)
	require.NoError(t, err)

	sharedSSTMeta := SharedSSTMeta{
		Backing:          handle,
		Smallest:         base.MakeInternalKey([]byte("d"), 0, InternalKeyKindSet),
		Largest:          base.MakeInternalKey([]byte("e"), 0, InternalKeyKindSet),
		SmallestPointKey: base.MakeInternalKey([]byte("d"), 0, InternalKeyKindSet),
		LargestPointKey:  base.MakeInternalKey([]byte("e"), 0, InternalKeyKindSet),
		Level:            6,
		Size:             uint64(size + 5),
	}
	_, err = d.IngestAndExcise([]string{}, []SharedSSTMeta{sharedSSTMeta}, KeyRange{Start: []byte("d"), End: []byte("ee")})
	require.NoError(t, err)

	// TODO(bilal): Once reading of shared sstables is in, verify that the values
	// of d and e have been updated.
}

type blockedCompaction struct {
	startBlock, unblock chan struct{}
}

func TestConcurrentExcise(t *testing.T) {
	var d, d1, d2 *DB
	var efos map[string]*EventuallyFileOnlySnapshot
	backgroundErrs := make(chan error, 5)
	var compactions map[string]*blockedCompaction
	defer func() {
		for _, e := range efos {
			require.NoError(t, e.Close())
		}
		if d1 != nil {
			require.NoError(t, d1.Close())
		}
		if d2 != nil {
			require.NoError(t, d2.Close())
		}
	}()
	creatorIDCounter := uint64(1)
	replicateCounter := 1

	var wg sync.WaitGroup
	defer wg.Wait()
	var blockNextCompaction bool
	var blockedJobID int
	var blockedCompactionName string
	var blockedCompactionsMu sync.Mutex // protects the above three variables.

	reset := func() {
		wg.Wait()
		for _, e := range efos {
			require.NoError(t, e.Close())
		}
		if d1 != nil {
			require.NoError(t, d1.Close())
		}
		if d2 != nil {
			require.NoError(t, d2.Close())
		}
		efos = make(map[string]*EventuallyFileOnlySnapshot)
		compactions = make(map[string]*blockedCompaction)
		backgroundErrs = make(chan error, 5)

		var el EventListener
		el.EnsureDefaults(testLogger{t: t})
		el.FlushBegin = func(info FlushInfo) {
			// Don't block flushes
		}
		el.BackgroundError = func(err error) {
			backgroundErrs <- err
		}
		el.CompactionBegin = func(info CompactionInfo) {
			if info.Reason == "move" {
				return
			}
			blockedCompactionsMu.Lock()
			defer blockedCompactionsMu.Unlock()
			if blockNextCompaction {
				blockNextCompaction = false
				blockedJobID = info.JobID
			}
		}
		el.TableCreated = func(info TableCreateInfo) {
			blockedCompactionsMu.Lock()
			if info.JobID != blockedJobID {
				blockedCompactionsMu.Unlock()
				return
			}
			blockedJobID = 0
			c := compactions[blockedCompactionName]
			blockedCompactionName = ""
			blockedCompactionsMu.Unlock()
			c.startBlock <- struct{}{}
			<-c.unblock
		}

		sstorage := remote.NewInMem()
		mem1 := vfs.NewMem()
		mem2 := vfs.NewMem()
		require.NoError(t, mem1.MkdirAll("ext", 0755))
		require.NoError(t, mem2.MkdirAll("ext", 0755))
		opts1 := &Options{
			Comparer:              testkeys.Comparer,
			LBaseMaxBytes:         1,
			FS:                    mem1,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
			DebugCheck:            DebugCheckLevels,
			FormatMajorVersion:    FormatVirtualSSTables,
		}
		// lel.
		lel := MakeLoggingEventListener(DefaultLogger)
		tel := TeeEventListener(lel, el)
		opts1.EventListener = &tel
		opts1.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
			"": sstorage,
		})
		opts1.Experimental.CreateOnShared = remote.CreateOnSharedAll
		opts1.Experimental.CreateOnSharedLocator = ""
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts1.DisableAutomaticCompactions = true

		opts2 := &Options{}
		*opts2 = *opts1
		opts2.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
			"": sstorage,
		})
		opts2.Experimental.CreateOnShared = remote.CreateOnSharedAll
		opts2.Experimental.CreateOnSharedLocator = ""
		opts2.FS = mem2

		var err error
		d1, err = Open("", opts1)
		require.NoError(t, err)
		require.NoError(t, d1.SetCreatorID(creatorIDCounter))
		creatorIDCounter++
		d2, err = Open("", opts2)
		require.NoError(t, err)
		require.NoError(t, d2.SetCreatorID(creatorIDCounter))
		creatorIDCounter++
		d = d1
	}
	reset()

	datadriven.RunTest(t, "testdata/concurrent_excise", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset()
			return ""
		case "switch":
			if len(td.CmdArgs) != 1 {
				return "usage: switch <1 or 2>"
			}
			switch td.CmdArgs[0].Key {
			case "1":
				d = d1
			case "2":
				d = d2
			default:
				return "usage: switch <1 or 2>"
			}
			return "ok"
		case "batch":
			b := d.NewIndexedBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "build":
			if err := runBuildCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			return ""

		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return ""

		case "ingest":
			if err := runIngestCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			return ""

		case "ingest-and-excise":
			if err := runIngestAndExciseCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			return ""

		case "replicate":
			if len(td.CmdArgs) != 4 {
				return "usage: replicate <from-db-num> <to-db-num> <start-key> <end-key>"
			}
			var from, to *DB
			switch td.CmdArgs[0].Key {
			case "1":
				from = d1
			case "2":
				from = d2
			default:
				return "usage: replicate <from-db-num> <to-db-num> <start-key> <end-key>"
			}
			switch td.CmdArgs[1].Key {
			case "1":
				to = d1
			case "2":
				to = d2
			default:
				return "usage: replicate <from-db-num> <to-db-num> <start-key> <end-key>"
			}
			startKey := []byte(td.CmdArgs[2].Key)
			endKey := []byte(td.CmdArgs[3].Key)

			writeOpts := d.opts.MakeWriterOptions(0 /* level */, to.opts.FormatMajorVersion.MaxTableFormat())
			sstPath := fmt.Sprintf("ext/replicate%d.sst", replicateCounter)
			f, err := to.opts.FS.Create(sstPath)
			require.NoError(t, err)
			replicateCounter++
			w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), writeOpts)

			var sharedSSTs []SharedSSTMeta
			err = from.ScanInternal(context.TODO(), startKey, endKey,
				func(key *InternalKey, value LazyValue, _ IteratorLevel) error {
					val, _, err := value.Value(nil)
					require.NoError(t, err)
					require.NoError(t, w.Add(base.MakeInternalKey(key.UserKey, 0, key.Kind()), val))
					return nil
				},
				func(start, end []byte, seqNum uint64) error {
					require.NoError(t, w.DeleteRange(start, end))
					return nil
				},
				func(start, end []byte, keys []keyspan.Key) error {
					s := keyspan.Span{
						Start:     start,
						End:       end,
						Keys:      keys,
						KeysOrder: 0,
					}
					require.NoError(t, rangekey.Encode(&s, func(k base.InternalKey, v []byte) error {
						return w.AddRangeKey(base.MakeInternalKey(k.UserKey, 0, k.Kind()), v)
					}))
					return nil
				},
				func(sst *SharedSSTMeta) error {
					sharedSSTs = append(sharedSSTs, *sst)
					return nil
				},
			)
			require.NoError(t, err)
			require.NoError(t, w.Close())

			_, err = to.IngestAndExcise([]string{sstPath}, sharedSSTs, KeyRange{Start: startKey, End: endKey})
			require.NoError(t, err)
			return fmt.Sprintf("replicated %d shared SSTs", len(sharedSSTs))

		case "get":
			return runGetCmd(t, td, d)

		case "iter":
			o := &IterOptions{KeyTypes: IterKeyTypePointsAndRanges}
			var reader Reader
			reader = d
			for _, arg := range td.CmdArgs {
				switch arg.Key {
				case "mask-suffix":
					o.RangeKeyMasking.Suffix = []byte(arg.Vals[0])
				case "mask-filter":
					o.RangeKeyMasking.Filter = func() BlockPropertyFilterMask {
						return sstable.NewTestKeysMaskingFilter()
					}
				case "snapshot":
					reader = efos[arg.Vals[0]]
				}
			}
			iter, err := reader.NewIter(o)
			if err != nil {
				return err.Error()
			}
			return runIterCmd(td, iter, true)

		case "lsm":
			return runLSMCmd(td, d)

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "wait-pending-table-stats":
			return runTableStatsCmd(td, d)

		case "excise":
			ve := &versionEdit{
				DeletedFiles: map[deletedFileEntry]*fileMetadata{},
			}
			var exciseSpan KeyRange
			if len(td.CmdArgs) != 2 {
				panic("insufficient args for excise command")
			}
			exciseSpan.Start = []byte(td.CmdArgs[0].Key)
			exciseSpan.End = []byte(td.CmdArgs[1].Key)

			d.mu.Lock()
			d.mu.versions.logLock()
			d.mu.Unlock()
			current := d.mu.versions.currentVersion()
			for level := range current.Levels {
				iter := current.Levels[level].Iter()
				for m := iter.SeekGE(d.cmp, exciseSpan.Start); m != nil && d.cmp(m.Smallest.UserKey, exciseSpan.End) < 0; m = iter.Next() {
					_, err := d.excise(exciseSpan, m, ve, level)
					if err != nil {
						d.mu.Lock()
						d.mu.versions.logUnlock()
						d.mu.Unlock()
						return fmt.Sprintf("error when excising %s: %s", m.FileNum, err.Error())
					}
				}
			}
			d.mu.Lock()
			d.mu.versions.logUnlock()
			d.mu.Unlock()
			return fmt.Sprintf("would excise %d files, use ingest-and-excise to excise.\n%s", len(ve.DeletedFiles), ve.String())

		case "file-only-snapshot":
			if len(td.CmdArgs) != 1 {
				panic("insufficient args for file-only-snapshot command")
			}
			name := td.CmdArgs[0].Key
			var keyRanges []KeyRange
			for _, line := range strings.Split(td.Input, "\n") {
				fields := strings.Fields(line)
				if len(fields) != 2 {
					return "expected two fields for file-only snapshot KeyRanges"
				}
				kr := KeyRange{Start: []byte(fields[0]), End: []byte(fields[1])}
				keyRanges = append(keyRanges, kr)
			}

			s := d.NewEventuallyFileOnlySnapshot(keyRanges)
			efos[name] = s
			return "ok"

		case "wait-for-file-only-snapshot":
			if len(td.CmdArgs) != 1 {
				panic("insufficient args for file-only-snapshot command")
			}
			name := td.CmdArgs[0].Key
			err := efos[name].WaitForFileOnlySnapshot(context.TODO(), 1*time.Millisecond)
			if err != nil {
				return err.Error()
			}
			return "ok"

		case "unblock":
			name := td.CmdArgs[0].Key
			blockedCompactionsMu.Lock()
			c := compactions[name]
			delete(compactions, name)
			blockedCompactionsMu.Unlock()
			c.unblock <- struct{}{}
			return "ok"

		case "compact":
			async := false
			var otherArgs []datadriven.CmdArg
			var bc *blockedCompaction
			for i := range td.CmdArgs {
				switch td.CmdArgs[i].Key {
				case "block":
					name := td.CmdArgs[i].Vals[0]
					bc = &blockedCompaction{startBlock: make(chan struct{}), unblock: make(chan struct{})}
					blockedCompactionsMu.Lock()
					compactions[name] = bc
					blockNextCompaction = true
					blockedCompactionName = name
					blockedCompactionsMu.Unlock()
					async = true
				default:
					otherArgs = append(otherArgs, td.CmdArgs[i])
				}
			}
			var tdClone datadriven.TestData
			tdClone = *td
			tdClone.CmdArgs = otherArgs
			if !async {
				err := runCompactCmd(td, d)
				if err != nil {
					return err.Error()
				}
			} else {
				wg.Add(1)
				go func() {
					defer wg.Done()
					_ = runCompactCmd(&tdClone, d)
				}()
				<-bc.startBlock
				return "spun off in separate goroutine"
			}
			return "ok"
		case "wait-for-background-error":
			err := <-backgroundErrs
			return err.Error()
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestIngestExternal(t *testing.T) {
	var mem vfs.FS
	var d *DB
	var flushed bool
	defer func() {
		require.NoError(t, d.Close())
	}()

	var remoteStorage remote.Storage

	reset := func() {
		if d != nil {
			require.NoError(t, d.Close())
		}

		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))
		remoteStorage = remote.NewInMem()
		opts := &Options{
			FS:                    mem,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
			DebugCheck:            DebugCheckLevels,
			EventListener: &EventListener{FlushEnd: func(info FlushInfo) {
				flushed = true
			}},
			FormatMajorVersion: FormatVirtualSSTables,
		}
		opts.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
			"external-locator": remoteStorage,
		})
		opts.Experimental.CreateOnShared = remote.CreateOnSharedNone
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts.DisableAutomaticCompactions = true

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
		require.NoError(t, d.SetCreatorID(1))
	}
	reset()

	datadriven.RunTest(t, "testdata/ingest_external", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset()
			return ""
		case "batch":
			b := d.NewIndexedBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "build-remote":
			if err := runBuildRemoteCmd(td, d, remoteStorage); err != nil {
				return err.Error()
			}
			return ""

		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return ""

		case "ingest-external":
			flushed = false
			if err := runIngestExternalCmd(td, d, "external-locator"); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			if flushed {
				return "memtable flushed"
			}
			return ""

		case "get":
			return runGetCmd(t, td, d)

		case "iter":
			iter, _ := d.NewIter(&IterOptions{
				KeyTypes: IterKeyTypePointsAndRanges,
			})
			return runIterCmd(td, iter, true)

		case "lsm":
			return runLSMCmd(td, d)

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "wait-pending-table-stats":
			return runTableStatsCmd(td, d)

		case "compact":
			if len(td.CmdArgs) != 2 {
				panic("insufficient args for compact command")
			}
			l := td.CmdArgs[0].Key
			r := td.CmdArgs[1].Key
			err := d.Compact([]byte(l), []byte(r), false)
			if err != nil {
				return err.Error()
			}
			return ""
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestIngestMemtableOverlaps(t *testing.T) {
	comparers := []Comparer{
		{Name: "default", Compare: DefaultComparer.Compare, FormatKey: DefaultComparer.FormatKey},
		{
			Name:      "reverse",
			Compare:   func(a, b []byte) int { return DefaultComparer.Compare(b, a) },
			FormatKey: DefaultComparer.FormatKey,
		},
	}
	m := make(map[string]*Comparer)
	for i := range comparers {
		c := &comparers[i]
		m[c.Name] = c
	}

	for _, comparer := range comparers {
		t.Run(comparer.Name, func(t *testing.T) {
			var mem *memTable

			parseMeta := func(s string) *fileMetadata {
				parts := strings.Split(s, "-")
				meta := &fileMetadata{}
				if len(parts) != 2 {
					t.Fatalf("malformed table spec: %s", s)
				}
				var smallest, largest base.InternalKey
				if strings.Contains(parts[0], ".") {
					if !strings.Contains(parts[1], ".") {
						t.Fatalf("malformed table spec: %s", s)
					}
					smallest = base.ParseInternalKey(parts[0])
					largest = base.ParseInternalKey(parts[1])
				} else {
					smallest = InternalKey{UserKey: []byte(parts[0])}
					largest = InternalKey{UserKey: []byte(parts[1])}
				}
				// If we're using a reverse comparer, flip the file bounds.
				if mem.cmp(smallest.UserKey, largest.UserKey) > 0 {
					smallest, largest = largest, smallest
				}
				meta.ExtendPointKeyBounds(comparer.Compare, smallest, largest)
				meta.InitPhysicalBacking()
				return meta
			}

			datadriven.RunTest(t, "testdata/ingest_memtable_overlaps", func(t *testing.T, d *datadriven.TestData) string {
				switch d.Cmd {
				case "define":
					b := newBatch(nil)
					if err := runBatchDefineCmd(d, b); err != nil {
						return err.Error()
					}

					opts := &Options{
						Comparer: &comparer,
					}
					opts.EnsureDefaults().WithFSDefaults()
					if len(d.CmdArgs) > 1 {
						return fmt.Sprintf("%s expects at most 1 argument", d.Cmd)
					}
					if len(d.CmdArgs) == 1 {
						opts.Comparer = m[d.CmdArgs[0].String()]
						if opts.Comparer == nil {
							return fmt.Sprintf("%s unknown comparer: %s", d.Cmd, d.CmdArgs[0].String())
						}
					}

					mem = newMemTable(memTableOptions{Options: opts})
					if err := mem.apply(b, 0); err != nil {
						return err.Error()
					}
					return ""

				case "overlaps":
					var buf bytes.Buffer
					for _, data := range strings.Split(d.Input, "\n") {
						var keyRanges []internalKeyRange
						for _, part := range strings.Fields(data) {
							meta := parseMeta(part)
							keyRanges = append(keyRanges, internalKeyRange{smallest: meta.Smallest, largest: meta.Largest})
						}
						fmt.Fprintf(&buf, "%t\n", ingestMemtableOverlaps(mem.cmp, mem, keyRanges))
					}
					return buf.String()

				default:
					return fmt.Sprintf("unknown command: %s", d.Cmd)
				}
			})
		})
	}
}

func TestKeyRangeBasic(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	k1 := KeyRange{Start: []byte("b"), End: []byte("c")}

	// Tests for Contains()
	require.True(t, k1.Contains(cmp, base.MakeInternalKey([]byte("b"), 1, InternalKeyKindSet)))
	require.False(t, k1.Contains(cmp, base.MakeInternalKey([]byte("c"), 1, InternalKeyKindSet)))
	require.True(t, k1.Contains(cmp, base.MakeInternalKey([]byte("bb"), 1, InternalKeyKindSet)))
	require.True(t, k1.Contains(cmp, base.MakeExclusiveSentinelKey(InternalKeyKindRangeDelete, []byte("c"))))

	m1 := &fileMetadata{
		Smallest: base.MakeInternalKey([]byte("b"), 1, InternalKeyKindSet),
		Largest:  base.MakeInternalKey([]byte("c"), 1, InternalKeyKindSet),
	}
	require.True(t, k1.Overlaps(cmp, m1))
	m2 := &fileMetadata{
		Smallest: base.MakeInternalKey([]byte("c"), 1, InternalKeyKindSet),
		Largest:  base.MakeInternalKey([]byte("d"), 1, InternalKeyKindSet),
	}
	require.False(t, k1.Overlaps(cmp, m2))
	m3 := &fileMetadata{
		Smallest: base.MakeInternalKey([]byte("a"), 1, InternalKeyKindSet),
		Largest:  base.MakeExclusiveSentinelKey(InternalKeyKindRangeDelete, []byte("b")),
	}
	require.False(t, k1.Overlaps(cmp, m3))
	m4 := &fileMetadata{
		Smallest: base.MakeInternalKey([]byte("a"), 1, InternalKeyKindSet),
		Largest:  base.MakeInternalKey([]byte("b"), 1, InternalKeyKindSet),
	}
	require.True(t, k1.Overlaps(cmp, m4))
}

func BenchmarkIngestOverlappingMemtable(b *testing.B) {
	assertNoError := func(err error) {
		b.Helper()
		if err != nil {
			b.Fatal(err)
		}
	}

	for count := 1; count < 6; count++ {
		b.Run(fmt.Sprintf("memtables=%d", count), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				b.StopTimer()
				mem := vfs.NewMem()
				d, err := Open("", &Options{
					FS: mem,
				})
				assertNoError(err)

				// Create memtables.
				for {
					assertNoError(d.Set([]byte("a"), nil, nil))
					d.mu.Lock()
					done := len(d.mu.mem.queue) == count
					d.mu.Unlock()
					if done {
						break
					}
				}

				// Create the overlapping sstable that will force a flush when ingested.
				f, err := mem.Create("ext")
				assertNoError(err)
				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
				assertNoError(w.Set([]byte("a"), nil))
				assertNoError(w.Close())

				b.StartTimer()
				assertNoError(d.Ingest([]string{"ext"}))
			}
		})
	}
}

func TestIngestTargetLevel(t *testing.T) {
	var d *DB
	defer func() {
		if d != nil {
			// Ignore errors because this test defines fake in-progress transactions
			// that prohibit clean shutdown.
			_ = d.Close()
		}
	}()

	parseMeta := func(s string) *fileMetadata {
		var rkey bool
		if len(s) >= 4 && s[0:4] == "rkey" {
			rkey = true
			s = s[5:]
		}
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		var m *fileMetadata
		if rkey {
			m = (&fileMetadata{}).ExtendRangeKeyBounds(
				d.cmp,
				InternalKey{UserKey: []byte(parts[0])},
				InternalKey{UserKey: []byte(parts[1])},
			)
		} else {
			m = (&fileMetadata{}).ExtendPointKeyBounds(
				d.cmp,
				InternalKey{UserKey: []byte(parts[0])},
				InternalKey{UserKey: []byte(parts[1])},
			)
		}
		m.InitPhysicalBacking()
		return m
	}

	datadriven.RunTest(t, "testdata/ingest_target_level", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			if d != nil {
				// Ignore errors because this test defines fake in-progress
				// transactions that prohibit clean shutdown.
				_ = d.Close()
			}

			var err error
			opts := Options{
				FormatMajorVersion: internalFormatNewest,
			}
			opts.WithFSDefaults()
			if d, err = runDBDefineCmd(td, &opts); err != nil {
				return err.Error()
			}

			readState := d.loadReadState()
			c := &checkConfig{
				logger:    d.opts.Logger,
				comparer:  d.opts.Comparer,
				readState: readState,
				newIters:  d.newIters,
				// TODO: runDBDefineCmd doesn't properly update the visible
				// sequence number. So we have to explicitly configure level checker with a very large
				// sequence number, otherwise the DB appears empty.
				seqNum: InternalKeySeqNumMax,
			}
			if err := checkLevelsInternal(c); err != nil {
				return err.Error()
			}
			readState.unref()

			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "target":
			var buf bytes.Buffer
			suggestSplit := false
			for _, cmd := range td.CmdArgs {
				switch cmd.Key {
				case "suggest-split":
					suggestSplit = true
				}
			}
			for _, target := range strings.Split(td.Input, "\n") {
				meta := parseMeta(target)
				level, overlapFile, err := ingestTargetLevel(
					d.newIters, d.tableNewRangeKeyIter, IterOptions{logger: d.opts.Logger},
					d.opts.Comparer, d.mu.versions.currentVersion(), 1, d.mu.compact.inProgress, meta,
					suggestSplit)
				if err != nil {
					return err.Error()
				}
				if overlapFile != nil {
					fmt.Fprintf(&buf, "%d (split file: %s)\n", level, overlapFile.FileNum)
				} else {
					fmt.Fprintf(&buf, "%d\n", level)
				}
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestIngest(t *testing.T) {
	var mem vfs.FS
	var d *DB
	var flushed bool
	if runtime.GOARCH == "386" {
		t.Skip("skipped on 32-bit due to slightly varied output")
	}
	defer func() {
		require.NoError(t, d.Close())
	}()

	reset := func(split bool) {
		if d != nil {
			require.NoError(t, d.Close())
		}

		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))
		opts := &Options{
			FS:                    mem,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
			DebugCheck:            DebugCheckLevels,
			EventListener: &EventListener{FlushEnd: func(info FlushInfo) {
				flushed = true
			}},
			FormatMajorVersion: internalFormatNewest,
		}
		opts.Experimental.IngestSplit = func() bool {
			return split
		}
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts.DisableAutomaticCompactions = true

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
	}
	reset(false /* split */)

	datadriven.RunTest(t, "testdata/ingest", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			split := false
			for _, cmd := range td.CmdArgs {
				switch cmd.Key {
				case "enable-split":
					split = true
				default:
					return fmt.Sprintf("unexpected key: %s", cmd.Key)
				}
			}
			reset(split)
			return ""
		case "batch":
			b := d.NewIndexedBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""

		case "build":
			if err := runBuildCmd(td, d, mem); err != nil {
				return err.Error()
			}
			return ""

		case "ingest":
			flushed = false
			if err := runIngestCmd(td, d, mem); err != nil {
				return err.Error()
			}
			// Wait for a possible flush.
			d.mu.Lock()
			for d.mu.compact.flushing {
				d.mu.compact.cond.Wait()
			}
			d.mu.Unlock()
			if flushed {
				return "memtable flushed"
			}
			return ""

		case "get":
			return runGetCmd(t, td, d)

		case "iter":
			iter, _ := d.NewIter(&IterOptions{
				KeyTypes: IterKeyTypePointsAndRanges,
			})
			return runIterCmd(td, iter, true)

		case "lsm":
			return runLSMCmd(td, d)

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "wait-pending-table-stats":
			return runTableStatsCmd(td, d)

		case "compact":
			if len(td.CmdArgs) != 2 {
				panic("insufficient args for compact command")
			}
			l := td.CmdArgs[0].Key
			r := td.CmdArgs[1].Key
			err := d.Compact([]byte(l), []byte(r), false)
			if err != nil {
				return err.Error()
			}
			return ""
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestIngestError(t *testing.T) {
	for i := int32(0); ; i++ {
		mem := vfs.NewMem()

		f0, err := mem.Create("ext0")
		require.NoError(t, err)
		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f0), sstable.WriterOptions{})
		require.NoError(t, w.Set([]byte("d"), nil))
		require.NoError(t, w.Close())
		f1, err := mem.Create("ext1")
		require.NoError(t, err)
		w = sstable.NewWriter(objstorageprovider.NewFileWritable(f1), sstable.WriterOptions{})
		require.NoError(t, w.Set([]byte("d"), nil))
		require.NoError(t, w.Close())

		inj := errorfs.OnIndex(-1)
		d, err := Open("", &Options{
			FS:                    errorfs.Wrap(mem, inj),
			Logger:                panicLogger{},
			L0CompactionThreshold: 8,
		})
		require.NoError(t, err)
		// Force the creation of an L0 sstable that overlaps with the tables
		// we'll attempt to ingest. This ensures that we exercise filesystem
		// codepaths when determining the ingest target level.
		require.NoError(t, d.Set([]byte("a"), nil, nil))
		require.NoError(t, d.Set([]byte("d"), nil, nil))
		require.NoError(t, d.Flush())

		t.Run(fmt.Sprintf("index-%d", i), func(t *testing.T) {
			defer func() {
				if r := recover(); r != nil {
					if e, ok := r.(error); ok && errors.Is(e, errorfs.ErrInjected) {
						return
					}
					// d.opts.Logger.Fatalf won't propagate ErrInjected
					// itself, but should contain the error message.
					if strings.HasSuffix(fmt.Sprint(r), errorfs.ErrInjected.Error()) {
						return
					}
					t.Fatal(r)
				}
			}()

			inj.SetIndex(i)
			err1 := d.Ingest([]string{"ext0"})
			err2 := d.Ingest([]string{"ext1"})
			err := firstError(err1, err2)
			if err != nil && !errors.Is(err, errorfs.ErrInjected) {
				t.Fatal(err)
			}
		})

		// d.Close may error if we failed to flush the manifest.
		_ = d.Close()

		// If the injector's index is non-negative, the i-th filesystem
		// operation was never executed.
		if inj.Index() >= 0 {
			break
		}
	}
}

func TestIngestIdempotence(t *testing.T) {
	// Use an on-disk filesystem, because Ingest with a MemFS will copy, not
	// link the ingested file.
	dir, err := os.MkdirTemp("", "ingest-idempotence")
	require.NoError(t, err)
	defer os.RemoveAll(dir)
	fs := vfs.Default

	path := fs.PathJoin(dir, "ext")
	f, err := fs.Create(fs.PathJoin(dir, "ext"))
	require.NoError(t, err)
	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
	require.NoError(t, w.Set([]byte("d"), nil))
	require.NoError(t, w.Close())

	d, err := Open(dir, &Options{
		FS: fs,
	})
	require.NoError(t, err)
	const count = 4
	for i := 0; i < count; i++ {
		ingestPath := fs.PathJoin(dir, fmt.Sprintf("ext%d", i))
		require.NoError(t, fs.Link(path, ingestPath))
		require.NoError(t, d.Ingest([]string{ingestPath}))
	}
	require.NoError(t, d.Close())
}

func TestIngestCompact(t *testing.T) {
	mem := vfs.NewMem()
	lel := MakeLoggingEventListener(&base.InMemLogger{})
	d, err := Open("", &Options{
		EventListener:         &lel,
		FS:                    mem,
		L0CompactionThreshold: 1,
		L0StopWritesThreshold: 1,
	})
	require.NoError(t, err)

	src := func(i int) string {
		return fmt.Sprintf("ext%d", i)
	}
	f, err := mem.Create(src(0))
	require.NoError(t, err)

	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
	key := []byte("a")
	require.NoError(t, w.Add(base.MakeInternalKey(key, 0, InternalKeyKindSet), nil))
	require.NoError(t, w.Close())

	// Make N copies of the sstable.
	const count = 20
	for i := 1; i < count; i++ {
		require.NoError(t, vfs.Copy(d.opts.FS, src(0), src(i)))
	}

	// Ingest the same sstable multiple times. Compaction should take place as
	// ingestion happens, preventing an indefinite write stall from occurring.
	for i := 0; i < count; i++ {
		if i == 10 {
			// Half-way through the ingestions, set a key in the memtable to force
			// overlap with the memtable which will require the memtable to be
			// flushed.
			require.NoError(t, d.Set(key, nil, nil))
		}
		require.NoError(t, d.Ingest([]string{src(i)}))
	}

	require.NoError(t, d.Close())
}

func TestConcurrentIngest(t *testing.T) {
	mem := vfs.NewMem()
	d, err := Open("", &Options{
		FS: mem,
	})
	require.NoError(t, err)

	// Create an sstable with 2 keys. This is necessary to trigger the overlap
	// bug because an sstable with a single key will not have overlap in internal
	// key space and the sequence number assignment had already guaranteed
	// correct ordering.
	src := func(i int) string {
		return fmt.Sprintf("ext%d", i)
	}
	f, err := mem.Create(src(0))
	require.NoError(t, err)

	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
	require.NoError(t, w.Set([]byte("a"), nil))
	require.NoError(t, w.Set([]byte("b"), nil))
	require.NoError(t, w.Close())

	// Make N copies of the sstable.
	errCh := make(chan error, 5)
	for i := 1; i < cap(errCh); i++ {
		require.NoError(t, vfs.Copy(d.opts.FS, src(0), src(i)))
	}

	// Perform N ingestions concurrently.
	for i := 0; i < cap(errCh); i++ {
		go func(i int) {
			err := d.Ingest([]string{src(i)})
			if err == nil {
				if _, err = d.opts.FS.Stat(src(i)); oserror.IsNotExist(err) {
					err = nil
				}
			}
			errCh <- err
		}(i)
	}
	for i := 0; i < cap(errCh); i++ {
		require.NoError(t, <-errCh)
	}

	require.NoError(t, d.Close())
}

func TestConcurrentIngestCompact(t *testing.T) {
	for i := 0; i < 2; i++ {
		t.Run("", func(t *testing.T) {
			mem := vfs.NewMem()
			compactionReady := make(chan struct{})
			compactionBegin := make(chan struct{})
			d, err := Open("", &Options{
				FS: mem,
				EventListener: &EventListener{
					TableCreated: func(info TableCreateInfo) {
						if info.Reason == "compacting" {
							close(compactionReady)
							<-compactionBegin
						}
					},
				},
			})
			require.NoError(t, err)

			ingest := func(keys ...string) {
				t.Helper()
				f, err := mem.Create("ext")
				require.NoError(t, err)

				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
				for _, k := range keys {
					require.NoError(t, w.Set([]byte(k), nil))
				}
				require.NoError(t, w.Close())
				require.NoError(t, d.Ingest([]string{"ext"}))
			}

			compact := func(start, end string) {
				t.Helper()
				require.NoError(t, d.Compact([]byte(start), []byte(end), false))
			}

			lsm := func() string {
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s
			}

			expectLSM := func(expected string) {
				t.Helper()
				expected = strings.TrimSpace(expected)
				actual := strings.TrimSpace(lsm())
				if expected != actual {
					t.Fatalf("expected\n%s\nbut found\n%s", expected, actual)
				}
			}

			ingest("a")
			ingest("a")
			ingest("c")
			ingest("c")

			expectLSM(`
0.0:
  000005:[a#11,SET-a#11,SET]
  000007:[c#13,SET-c#13,SET]
6:
  000004:[a#10,SET-a#10,SET]
  000006:[c#12,SET-c#12,SET]
`)

			// At this point ingestion of an sstable containing only key "b" will be
			// targeted at L6. Yet a concurrent compaction of sstables 5 and 7 will
			// create a new sstable in L6 spanning ["a"-"c"]. So the ingestion must
			// actually target L5.

			switch i {
			case 0:
				// Compact, then ingest.
				go func() {
					<-compactionReady

					ingest("b")

					close(compactionBegin)
				}()

				compact("a", "z")

				expectLSM(`
0.0:
  000009:[b#14,SET-b#14,SET]
6:
  000008:[a#0,SET-c#0,SET]
`)

			case 1:
				// Ingest, then compact
				var wg sync.WaitGroup
				wg.Add(1)
				go func() {
					defer wg.Done()
					close(compactionBegin)
					compact("a", "z")
				}()

				ingest("b")
				wg.Wait()

				// Because we're performing the ingestion and compaction concurrently,
				// we can't guarantee any particular LSM structure at this point. The
				// test will fail with an assertion error due to overlapping sstables
				// if there is insufficient synchronization between ingestion and
				// compaction.
			}

			require.NoError(t, d.Close())
		})
	}
}

func TestIngestFlushQueuedMemTable(t *testing.T) {
	// Verify that ingestion forces a flush of a queued memtable.

	// Test with a format major version prior to FormatFlushableIngest and one
	// after. Both should result in the same statistic calculations.
	for _, fmv := range []FormatMajorVersion{FormatFlushableIngest - 1, internalFormatNewest} {
		func(fmv FormatMajorVersion) {
			mem := vfs.NewMem()
			d, err := Open("", &Options{
				FS:                 mem,
				FormatMajorVersion: fmv,
			})
			require.NoError(t, err)

			// Add the key "a" to the memtable, then fill up the memtable with the key
			// "b". The ingested sstable will only overlap with the queued memtable.
			require.NoError(t, d.Set([]byte("a"), nil, nil))
			for {
				require.NoError(t, d.Set([]byte("b"), nil, nil))
				d.mu.Lock()
				done := len(d.mu.mem.queue) == 2
				d.mu.Unlock()
				if done {
					break
				}
			}

			ingest := func(keys ...string) {
				t.Helper()
				f, err := mem.Create("ext")
				require.NoError(t, err)

				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
					TableFormat: fmv.MinTableFormat(),
				})
				for _, k := range keys {
					require.NoError(t, w.Set([]byte(k), nil))
				}
				require.NoError(t, w.Close())
				stats, err := d.IngestWithStats([]string{"ext"})
				require.NoError(t, err)
				require.Equal(t, stats.ApproxIngestedIntoL0Bytes, stats.Bytes)
				require.Equal(t, stats.MemtableOverlappingFiles, 1)
				require.Less(t, uint64(0), stats.Bytes)
			}

			ingest("a")

			require.NoError(t, d.Close())
		}(fmv)
	}
}

func TestIngestStats(t *testing.T) {
	mem := vfs.NewMem()
	d, err := Open("", &Options{
		FS: mem,
	})
	require.NoError(t, err)

	ingest := func(expectedLevel int, keys ...string) {
		t.Helper()
		f, err := mem.Create("ext")
		require.NoError(t, err)

		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
		for _, k := range keys {
			require.NoError(t, w.Set([]byte(k), nil))
		}
		require.NoError(t, w.Close())
		stats, err := d.IngestWithStats([]string{"ext"})
		require.NoError(t, err)
		if expectedLevel == 0 {
			require.Equal(t, stats.ApproxIngestedIntoL0Bytes, stats.Bytes)
		} else {
			require.EqualValues(t, 0, stats.ApproxIngestedIntoL0Bytes)
		}
		require.Less(t, uint64(0), stats.Bytes)
	}
	ingest(6, "a")
	ingest(0, "a")
	ingest(6, "b", "g")
	ingest(0, "c")
	require.NoError(t, d.Close())
}

func TestIngestFlushQueuedLargeBatch(t *testing.T) {
	// Verify that ingestion forces a flush of a queued large batch.

	mem := vfs.NewMem()
	d, err := Open("", &Options{
		FS: mem,
	})
	require.NoError(t, err)

	// The default large batch threshold is slightly less than 1/2 of the
	// memtable size which makes triggering a problem with flushing queued large
	// batches irritating. Manually adjust the threshold to 1/8 of the memtable
	// size in order to more easily create a situation where a large batch is
	// queued but not automatically flushed.
	d.mu.Lock()
	d.largeBatchThreshold = d.opts.MemTableSize / 8
	d.mu.Unlock()

	// Set a record with a large value. This will be transformed into a large
	// batch and placed in the flushable queue.
	require.NoError(t, d.Set([]byte("a"), bytes.Repeat([]byte("v"), int(d.largeBatchThreshold)), nil))

	ingest := func(keys ...string) {
		t.Helper()
		f, err := mem.Create("ext")
		require.NoError(t, err)

		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
		for _, k := range keys {
			require.NoError(t, w.Set([]byte(k), nil))
		}
		require.NoError(t, w.Close())
		require.NoError(t, d.Ingest([]string{"ext"}))
	}

	ingest("a")

	require.NoError(t, d.Close())
}

func TestIngestMemtablePendingOverlap(t *testing.T) {
	mem := vfs.NewMem()
	d, err := Open("", &Options{
		FS: mem,
	})
	require.NoError(t, err)

	d.mu.Lock()
	// Use a custom commit pipeline apply function to give us control over
	// timing of events.
	assignedBatch := make(chan struct{})
	applyBatch := make(chan struct{})
	originalApply := d.commit.env.apply
	d.commit.env.apply = func(b *Batch, mem *memTable) error {
		assignedBatch <- struct{}{}
		applyBatch <- struct{}{}
		return originalApply(b, mem)
	}
	d.mu.Unlock()

	ingest := func(keys ...string) {
		t.Helper()
		f, err := mem.Create("ext")
		require.NoError(t, err)

		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
		for _, k := range keys {
			require.NoError(t, w.Set([]byte(k), nil))
		}
		require.NoError(t, w.Close())
		require.NoError(t, d.Ingest([]string{"ext"}))
	}

	var wg sync.WaitGroup
	wg.Add(2)

	// First, Set('c') begins. This call will:
	//
	// * enqueue the batch to the pending queue.
	// * allocate a sequence number `x`.
	// * write the batch to the WAL.
	//
	// and then block until we read from the `applyBatch` channel down below.
	go func() {
		err := d.Set([]byte("c"), nil, nil)
		if err != nil {
			t.Error(err)
		}
		wg.Done()
	}()

	// When the above Set('c') is ready to apply, it sends on the
	// `assignedBatch` channel. Once that happens, we start Ingest('a', 'c').
	// The Ingest('a', 'c') allocates sequence number `x + 1`.
	go func() {
		// Wait until the Set has grabbed a sequence number before ingesting.
		<-assignedBatch
		ingest("a", "c")
		wg.Done()
	}()

	// The Set('c')#1 and Ingest('a', 'c')#2 are both pending. To maintain
	// sequence number invariants, the Set needs to be applied and flushed
	// before the Ingest determines its target level.
	//
	// Sleep a bit to ensure that the Ingest has time to call into
	// AllocateSeqNum. Once it allocates its sequence number, it should see
	// that there are unpublished sequence numbers below it and spin until the
	// Set's sequence number is published. After sleeping, read from
	// `applyBatch` to actually allow the Set to apply and publish its
	// sequence number.
	time.Sleep(100 * time.Millisecond)
	<-applyBatch

	// Wait for both calls to complete.
	wg.Wait()
	require.NoError(t, d.Flush())
	require.NoError(t, d.CheckLevels(nil))
	require.NoError(t, d.Close())
}

type testLogger struct {
	t testing.TB
}

func (l testLogger) Infof(format string, args ...interface{}) {
	l.t.Logf(format, args...)
}

func (l testLogger) Fatalf(format string, args ...interface{}) {
	l.t.Fatalf(format, args...)
}

// TestIngestMemtableOverlapRace is a regression test for the race described in
// #2196. If an ingest that checks for overlap with the mutable memtable and
// finds no overlap, it must not allow overlapping keys with later sequence
// numbers to be applied to the memtable and the memtable to be flushed before
// the ingest completes.
//
// This test operates by committing the same key concurrently:
//   - 1 goroutine repeatedly ingests the same sstable writing the key `foo`
//   - n goroutines repeatedly apply batches writing the key `foo` and trigger
//     flushes.
//
// After a while, the database is closed and the manifest is verified. Version
// edits should contain new files with monotonically increasing sequence
// numbers, since every flush and every ingest conflicts with one another.
func TestIngestMemtableOverlapRace(t *testing.T) {
	mem := vfs.NewMem()
	el := MakeLoggingEventListener(testLogger{t: t})
	d, err := Open("", &Options{
		FS: mem,
		// Disable automatic compactions to keep the manifest clean; only
		// flushes and ingests.
		DisableAutomaticCompactions: true,
		// Disable the WAL to speed up batch commits.
		DisableWAL:    true,
		EventListener: &el,
		// We're endlessly appending to L0 without clearing it, so set a maximal
		// stop writes threshold.
		L0StopWritesThreshold: math.MaxInt,
		// Accumulating more than 1 immutable memtable doesn't help us exercise
		// the bug, since the committed keys need to be flushed promptly.
		MemTableStopWritesThreshold: 2,
	})
	require.NoError(t, err)

	// Prepare a sstable `ext` deleting foo.
	f, err := mem.Create("ext")
	require.NoError(t, err)
	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
	require.NoError(t, w.Delete([]byte("foo")))
	require.NoError(t, w.Close())

	var done atomic.Bool
	const numSetters = 2
	var wg sync.WaitGroup
	wg.Add(numSetters + 1)

	untilDone := func(fn func()) {
		defer wg.Done()
		for !done.Load() {
			fn()
		}
	}

	// Ingest in the background.
	totalIngests := 0
	go untilDone(func() {
		filename := fmt.Sprintf("ext%d", totalIngests)
		require.NoError(t, mem.Link("ext", filename))
		require.NoError(t, d.Ingest([]string{filename}))
		totalIngests++
	})

	// Apply batches and trigger flushes in the background.
	wo := &WriteOptions{Sync: false}
	var localCommits [numSetters]int
	for i := 0; i < numSetters; i++ {
		i := i
		v := []byte(fmt.Sprintf("v%d", i+1))
		go untilDone(func() {
			// Commit a batch setting foo=vN.
			b := d.NewBatch()
			require.NoError(t, b.Set([]byte("foo"), v, nil))
			require.NoError(t, b.Commit(wo))
			localCommits[i]++
			d.AsyncFlush()
		})
	}
	time.Sleep(100 * time.Millisecond)
	done.Store(true)
	wg.Wait()

	var totalCommits int
	for i := 0; i < numSetters; i++ {
		totalCommits += localCommits[i]
	}
	m := d.Metrics()
	tot := m.Total()
	t.Logf("Committed %d batches.", totalCommits)
	t.Logf("Flushed %d times.", m.Flush.Count)
	t.Logf("Ingested %d sstables.", tot.TablesIngested)
	require.NoError(t, d.CheckLevels(nil))
	require.NoError(t, d.Close())

	// Replay the manifest. Every flush and ingest is a separate version edit.
	// Since they all write the same key and compactions are disabled, sequence
	// numbers of new files should be monotonically increasing.
	//
	// This check is necessary because most of these sstables are ingested into
	// L0. The L0 sublevels construction will order them by LargestSeqNum, even
	// if they're added to L0 out-of-order. The CheckLevels call at the end of
	// the test may find that the sublevels are all appropriately ordered, but
	// the manifest may reveal they were added to the LSM out-of-order.
	dbDesc, err := Peek("", mem)
	require.NoError(t, err)
	require.True(t, dbDesc.Exists)
	f, err = mem.Open(dbDesc.ManifestFilename)
	require.NoError(t, err)
	defer f.Close()
	rr := record.NewReader(f, 0 /* logNum */)
	var largest *fileMetadata
	for {
		r, err := rr.Next()
		if err == io.EOF || err == record.ErrInvalidChunk {
			break
		}
		require.NoError(t, err)
		var ve manifest.VersionEdit
		require.NoError(t, ve.Decode(r))
		t.Log(ve.String())
		for _, f := range ve.NewFiles {
			if largest != nil {
				require.Equal(t, 0, f.Level)
				if largest.LargestSeqNum > f.Meta.LargestSeqNum {
					t.Fatalf("previous largest file %s has sequence number > next file %s", largest, f.Meta)
				}
			}
			largest = f.Meta
		}
	}
}

type ingestCrashFS struct {
	vfs.FS
}

func (fs ingestCrashFS) Link(oldname, newname string) error {
	if err := fs.FS.Link(oldname, newname); err != nil {
		return err
	}
	panic(errorfs.ErrInjected)
}

type noRemoveFS struct {
	vfs.FS
}

func (fs noRemoveFS) Remove(string) error {
	return errorfs.ErrInjected
}

func TestIngestFileNumReuseCrash(t *testing.T) {
	const count = 10
	// Use an on-disk filesystem, because Ingest with a MemFS will copy, not
	// link the ingested file.
	dir, err := os.MkdirTemp("", "ingest-filenum-reuse")
	require.NoError(t, err)
	defer os.RemoveAll(dir)
	fs := vfs.Default

	readFile := func(s string) []byte {
		f, err := fs.Open(fs.PathJoin(dir, s))
		require.NoError(t, err)
		b, err := io.ReadAll(f)
		require.NoError(t, err)
		require.NoError(t, f.Close())
		return b
	}

	// Create sstables to ingest.
	var files []string
	var fileBytes [][]byte
	for i := 0; i < count; i++ {
		name := fmt.Sprintf("ext%d", i)
		f, err := fs.Create(fs.PathJoin(dir, name))
		require.NoError(t, err)
		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
		require.NoError(t, w.Set([]byte(fmt.Sprintf("foo%d", i)), nil))
		require.NoError(t, w.Close())
		files = append(files, name)
		fileBytes = append(fileBytes, readFile(name))
	}

	// Open a database with a filesystem that will successfully link the
	// ingested files but then panic. This is an approximation of what a crash
	// after linking but before updating the manifest would look like.
	d, err := Open(dir, &Options{
		FS: ingestCrashFS{FS: fs},
	})
	// A flush here ensures the file num bumps from creating OPTIONS files,
	// etc get recorded in the manifest. We want the nextFileNum after the
	// restart to be the same as one of our ingested sstables.
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("boop"), nil, nil))
	require.NoError(t, d.Flush())
	for _, f := range files {
		func() {
			defer func() { err = recover().(error) }()
			err = d.Ingest([]string{fs.PathJoin(dir, f)})
		}()
		if err == nil || !errors.Is(err, errorfs.ErrInjected) {
			t.Fatalf("expected injected error, got %v", err)
		}
	}
	// Leave something in the WAL so that Open will flush while replaying the
	// WAL.
	require.NoError(t, d.Set([]byte("wal"), nil, nil))
	require.NoError(t, d.Close())

	// There are now two links to each external file: the original extX link
	// and a numbered sstable link. The sstable files are still not a part of
	// the manifest and so they may be overwritten. Open will detect the
	// obsolete number sstables and try to remove them. The FS here is wrapped
	// to induce errors on Remove calls. Even if we're unsuccessful in
	// removing the obsolete files, the external files should not be
	// overwritten.
	d, err = Open(dir, &Options{FS: noRemoveFS{FS: fs}})
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("bar"), nil, nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Close())

	// None of the external files should change despite modifying the linked
	// versions.
	for i, f := range files {
		afterBytes := readFile(f)
		require.Equal(t, fileBytes[i], afterBytes)
	}
}

func TestIngest_UpdateSequenceNumber(t *testing.T) {
	mem := vfs.NewMem()
	cmp := base.DefaultComparer.Compare
	parse := func(input string) (*sstable.Writer, error) {
		f, err := mem.Create("ext")
		if err != nil {
			return nil, err
		}
		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
			TableFormat: sstable.TableFormatMax,
		})
		for _, data := range strings.Split(input, "\n") {
			if strings.HasPrefix(data, "rangekey: ") {
				data = strings.TrimPrefix(data, "rangekey: ")
				s := keyspan.ParseSpan(data)
				err := rangekey.Encode(&s, w.AddRangeKey)
				if err != nil {
					return nil, err
				}
				continue
			}
			j := strings.Index(data, ":")
			if j < 0 {
				return nil, errors.Newf("malformed input: %s\n", data)
			}
			key := base.ParseInternalKey(data[:j])
			value := []byte(data[j+1:])
			if err := w.Add(key, value); err != nil {
				return nil, err
			}
		}
		return w, nil
	}

	var (
		seqnum uint64
		err    error
		metas  []*fileMetadata
	)
	datadriven.RunTest(t, "testdata/ingest_update_seqnums", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "starting-seqnum":
			seqnum, err = strconv.ParseUint(td.Input, 10, 64)
			if err != nil {
				return err.Error()
			}
			return ""

		case "reset":
			metas = metas[:0]
			return ""

		case "load":
			w, err := parse(td.Input)
			if err != nil {
				return err.Error()
			}
			if err = w.Close(); err != nil {
				return err.Error()
			}
			defer w.Close()

			// Format the bounds of the table.
			wm, err := w.Metadata()
			if err != nil {
				return err.Error()
			}

			// Upper bounds for range dels and range keys are expected to be sentinel
			// keys.
			maybeUpdateUpperBound := func(key base.InternalKey) base.InternalKey {
				switch k := key.Kind(); {
				case k == base.InternalKeyKindRangeDelete:
					key.Trailer = base.InternalKeyRangeDeleteSentinel
				case rangekey.IsRangeKey(k):
					return base.MakeExclusiveSentinelKey(k, key.UserKey)
				}
				return key
			}

			// Construct the file metadata from the writer metadata.
			m := &fileMetadata{
				SmallestSeqNum: 0, // Simulate an ingestion.
				LargestSeqNum:  0,
			}
			if wm.HasPointKeys {
				m.ExtendPointKeyBounds(cmp, wm.SmallestPoint, wm.LargestPoint)
			}
			if wm.HasRangeDelKeys {
				m.ExtendPointKeyBounds(
					cmp,
					wm.SmallestRangeDel,
					maybeUpdateUpperBound(wm.LargestRangeDel),
				)
			}
			if wm.HasRangeKeys {
				m.ExtendRangeKeyBounds(
					cmp,
					wm.SmallestRangeKey,
					maybeUpdateUpperBound(wm.LargestRangeKey),
				)
			}
			m.InitPhysicalBacking()
			if err := m.Validate(cmp, base.DefaultFormatter); err != nil {
				return err.Error()
			}

			// Collect this file.
			metas = append(metas, m)

			// Return an index number for the file.
			return fmt.Sprintf("file %d\n", len(metas)-1)

		case "update-files":
			// Update the bounds across all files.
			if err = ingestUpdateSeqNum(cmp, base.DefaultFormatter, seqnum, ingestLoadResult{localMeta: metas}); err != nil {
				return err.Error()
			}

			var buf bytes.Buffer
			for i, m := range metas {
				fmt.Fprintf(&buf, "file %d:\n", i)
				fmt.Fprintf(&buf, "  combined: %s-%s\n", m.Smallest, m.Largest)
				fmt.Fprintf(&buf, "    points: %s-%s\n", m.SmallestPointKey, m.LargestPointKey)
				fmt.Fprintf(&buf, "    ranges: %s-%s\n", m.SmallestRangeKey, m.LargestRangeKey)
			}

			return buf.String()

		default:
			return fmt.Sprintf("unknown command %s\n", td.Cmd)
		}
	})
}

func TestIngestCleanup(t *testing.T) {
	fns := []base.FileNum{0, 1, 2}

	testCases := []struct {
		closeFiles   []base.FileNum
		cleanupFiles []base.FileNum
		wantErr      string
	}{
		// Close and remove all files.
		{
			closeFiles:   fns,
			cleanupFiles: fns,
		},
		// Remove a non-existent file.
		{
			closeFiles:   fns,
			cleanupFiles: []base.FileNum{3},
			wantErr:      "unknown to the objstorage provider",
		},
		// Remove a file that has not been closed.
		{
			closeFiles:   []base.FileNum{0, 2},
			cleanupFiles: fns,
			wantErr:      oserror.ErrInvalid.Error(),
		},
		// Remove all files, one of which is still open, plus a file that does not exist.
		{
			closeFiles:   []base.FileNum{0, 2},
			cleanupFiles: []base.FileNum{0, 1, 2, 3},
			wantErr:      oserror.ErrInvalid.Error(), // The first error encountered is due to the open file.
		},
	}

	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			mem := vfs.NewMem()
			mem.UseWindowsSemantics(true)
			objProvider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(mem, ""))
			require.NoError(t, err)
			defer objProvider.Close()

			// Create the files in the VFS.
			metaMap := make(map[base.FileNum]objstorage.Writable)
			for _, fn := range fns {
				w, _, err := objProvider.Create(context.Background(), base.FileTypeTable, fn.DiskFileNum(), objstorage.CreateOptions{})
				require.NoError(t, err)

				metaMap[fn] = w
			}

			// Close a select number of files.
			for _, m := range tc.closeFiles {
				w, ok := metaMap[m]
				if !ok {
					continue
				}
				require.NoError(t, w.Finish())
			}

			// Cleanup the set of files in the FS.
			var toRemove []*fileMetadata
			for _, fn := range tc.cleanupFiles {
				m := &fileMetadata{FileNum: fn}
				m.InitPhysicalBacking()
				toRemove = append(toRemove, m)
			}

			err = ingestCleanup(objProvider, toRemove)
			if tc.wantErr != "" {
				require.Error(t, err, "got no error, expected %s", tc.wantErr)
				require.Contains(t, err.Error(), tc.wantErr)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

// fatalCapturingLogger captures a fatal error instead of panicking.
type fatalCapturingLogger struct {
	t   testing.TB
	err error
}

// Infof implements the Logger interface.
func (l *fatalCapturingLogger) Infof(fmt string, args ...interface{}) {
	l.t.Logf(fmt, args...)
}

// Fatalf implements the Logger interface.
func (l *fatalCapturingLogger) Fatalf(_ string, args ...interface{}) {
	l.err = args[0].(error)
}

func TestIngestValidation(t *testing.T) {
	type keyVal struct {
		key, val []byte
	}
	type corruptionLocation int
	const (
		corruptionLocationNone corruptionLocation = iota
		corruptionLocationStart
		corruptionLocationEnd
		corruptionLocationInternal
	)
	type errLocation int
	const (
		errLocationNone errLocation = iota
		errLocationIngest
		errLocationValidation
	)
	const (
		nKeys     = 1_000
		keySize   = 16
		valSize   = 100
		blockSize = 100

		ingestTableName = "ext"
	)
	ingestPath := filepath.Join(t.TempDir(), ingestTableName)

	seed := uint64(time.Now().UnixNano())
	rng := rand.New(rand.NewSource(seed))
	t.Logf("rng seed = %d", seed)

	testCases := []struct {
		description string
		cLoc        corruptionLocation
		wantErrType errLocation
	}{
		{
			description: "no corruption",
			cLoc:        corruptionLocationNone,
			wantErrType: errLocationNone,
		},
		{
			description: "start block",
			cLoc:        corruptionLocationStart,
			wantErrType: errLocationIngest,
		},
		{
			description: "end block",
			cLoc:        corruptionLocationEnd,
			wantErrType: errLocationIngest,
		},
		{
			description: "non-end block",
			cLoc:        corruptionLocationInternal,
			wantErrType: errLocationValidation,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.description, func(t *testing.T) {
			var wg sync.WaitGroup
			wg.Add(1)

			fs := vfs.NewMem()
			logger := &fatalCapturingLogger{t: t}
			opts := &Options{
				FS:     fs,
				Logger: logger,
				EventListener: &EventListener{
					TableValidated: func(i TableValidatedInfo) {
						wg.Done()
					},
				},
			}
			opts.Experimental.ValidateOnIngest = true
			d, err := Open("", opts)
			require.NoError(t, err)
			defer func() { require.NoError(t, d.Close()) }()

			corrupt := func(f vfs.File) {
				readable, err := sstable.NewSimpleReadable(f)
				require.NoError(t, err)
				// Compute the layout of the sstable in order to find the
				// appropriate block locations to corrupt.
				r, err := sstable.NewReader(readable, sstable.ReaderOptions{})
				require.NoError(t, err)
				l, err := r.Layout()
				require.NoError(t, err)
				require.NoError(t, r.Close())

				// Select an appropriate data block to corrupt.
				var blockIdx int
				switch tc.cLoc {
				case corruptionLocationStart:
					blockIdx = 0
				case corruptionLocationEnd:
					blockIdx = len(l.Data) - 1
				case corruptionLocationInternal:
					blockIdx = 1 + rng.Intn(len(l.Data)-2)
				default:
					t.Fatalf("unknown corruptionLocation: %T", tc.cLoc)
				}
				bh := l.Data[blockIdx]

				osF, err := os.OpenFile(ingestPath, os.O_RDWR, 0600)
				require.NoError(t, err)
				defer func() { require.NoError(t, osF.Close()) }()

				// Corrupting a key will cause the ingestion to fail due to a
				// malformed key, rather than a block checksum mismatch.
				// Instead, we corrupt the last byte in the selected block,
				// before the trailer, which corresponds to a value.
				offset := bh.Offset + bh.Length - 1
				_, err = osF.WriteAt([]byte("\xff"), int64(offset))
				require.NoError(t, err)
			}

			type errT struct {
				errLoc errLocation
				err    error
			}
			runIngest := func(keyVals []keyVal) (et errT) {
				// The vfs.File does not allow for random reads and writes.
				// Create a disk-backed file outside of the DB FS that we can
				// open as a regular os.File, if required.
				tmpFS := vfs.Default
				f, err := tmpFS.Create(ingestPath)
				require.NoError(t, err)
				defer func() { _ = tmpFS.Remove(ingestPath) }()

				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
					BlockSize:   blockSize,     // Create many smaller blocks.
					Compression: NoCompression, // For simpler debugging.
				})
				for _, kv := range keyVals {
					require.NoError(t, w.Set(kv.key, kv.val))
				}
				require.NoError(t, w.Close())

				// Possibly corrupt the file.
				if tc.cLoc != corruptionLocationNone {
					f, err = tmpFS.Open(ingestPath)
					require.NoError(t, err)
					corrupt(f)
				}

				// Copy the file into the DB's FS.
				_, err = vfs.Clone(tmpFS, fs, ingestPath, ingestTableName)
				require.NoError(t, err)

				// Ingest the external table.
				err = d.Ingest([]string{ingestTableName})
				if err != nil {
					et.errLoc = errLocationIngest
					et.err = err
					return
				}

				// Wait for the validation on the sstable to complete.
				wg.Wait()

				// Return any error encountered during validation.
				if logger.err != nil {
					et.errLoc = errLocationValidation
					et.err = logger.err
				}

				return
			}

			// Construct a set of keys to ingest.
			var keyVals []keyVal
			for i := 0; i < nKeys; i++ {
				key := make([]byte, keySize)
				_, err = rng.Read(key)
				require.NoError(t, err)

				val := make([]byte, valSize)
				_, err = rng.Read(val)
				require.NoError(t, err)

				keyVals = append(keyVals, keyVal{key, val})
			}

			// Keys must be sorted.
			sort.Slice(keyVals, func(i, j int) bool {
				return d.cmp(keyVals[i].key, keyVals[j].key) <= 0
			})

			// Run the ingestion.
			et := runIngest(keyVals)

			// Assert we saw the errors we expect.
			switch tc.wantErrType {
			case errLocationNone:
				require.Equal(t, errLocationNone, et.errLoc)
				require.NoError(t, et.err)
			case errLocationIngest:
				require.Equal(t, errLocationIngest, et.errLoc)
				require.Error(t, et.err)
				require.True(t, errors.Is(et.err, base.ErrCorruption))
			case errLocationValidation:
				require.Equal(t, errLocationValidation, et.errLoc)
				require.Error(t, et.err)
				require.True(t, errors.Is(et.err, base.ErrCorruption))
			default:
				t.Fatalf("unknown wantErrType %T", tc.wantErrType)
			}
		})
	}
}

// BenchmarkManySSTables measures the cost of various operations with various
// counts of SSTables within the database.
func BenchmarkManySSTables(b *testing.B) {
	counts := []int{10, 1_000, 10_000, 100_000, 1_000_000}
	ops := []string{"ingest", "calculateInuseKeyRanges"}
	for _, op := range ops {
		b.Run(op, func(b *testing.B) {
			for _, count := range counts {
				b.Run(fmt.Sprintf("sstables=%d", count), func(b *testing.B) {
					mem := vfs.NewMem()
					d, err := Open("", &Options{
						FS: mem,
					})
					require.NoError(b, err)

					var paths []string
					for i := 0; i < count; i++ {
						n := fmt.Sprintf("%07d", i)
						f, err := mem.Create(n)
						require.NoError(b, err)
						w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
						require.NoError(b, w.Set([]byte(n), nil))
						require.NoError(b, w.Close())
						paths = append(paths, n)
					}
					require.NoError(b, d.Ingest(paths))

					{
						const broadIngest = "broad.sst"
						f, err := mem.Create(broadIngest)
						require.NoError(b, err)
						w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
						require.NoError(b, w.Set([]byte("0"), nil))
						require.NoError(b, w.Set([]byte("Z"), nil))
						require.NoError(b, w.Close())
						require.NoError(b, d.Ingest([]string{broadIngest}))
					}

					switch op {
					case "ingest":
						runBenchmarkManySSTablesIngest(b, d, mem, count)
					case "calculateInuseKeyRanges":
						runBenchmarkManySSTablesInUseKeyRanges(b, d, count)
					}
					require.NoError(b, d.Close())
				})
			}
		})
	}
}

func runBenchmarkManySSTablesIngest(b *testing.B, d *DB, fs vfs.FS, count int) {
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		n := fmt.Sprintf("%07d", count+i)
		f, err := fs.Create(n)
		require.NoError(b, err)
		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
		require.NoError(b, w.Set([]byte(n), nil))
		require.NoError(b, w.Close())
		require.NoError(b, d.Ingest([]string{n}))
	}
}

func runBenchmarkManySSTablesInUseKeyRanges(b *testing.B, d *DB, count int) {
	// This benchmark is pretty contrived, but it's not easy to write a
	// microbenchmark for this in a more natural way. L6 has many files, and
	// L5 has 1 file spanning the entire breadth of L5.
	d.mu.Lock()
	defer d.mu.Unlock()
	v := d.mu.versions.currentVersion()
	b.ResetTimer()

	smallest := []byte("0")
	largest := []byte("z")
	for i := 0; i < b.N; i++ {
		_ = calculateInuseKeyRanges(v, d.cmp, 0, numLevels-1, smallest, largest)
	}
}
