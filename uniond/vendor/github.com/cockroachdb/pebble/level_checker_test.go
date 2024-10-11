// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"path/filepath"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func TestCheckLevelsBasics(t *testing.T) {
	testCases := []string{"db-stage-1", "db-stage-2", "db-stage-3", "db-stage-4"}
	for _, tc := range testCases {
		t.Run(tc, func(t *testing.T) {
			t.Logf("%s", t.Name())
			fs := vfs.NewMem()
			_, err := vfs.Clone(vfs.Default, fs, filepath.Join("testdata", tc), tc)
			if err != nil {
				t.Fatalf("%s: cloneFileSystem failed: %v", tc, err)
			}
			d, err := Open(tc, &Options{
				FS: fs,
			})
			if err != nil {
				t.Fatalf("%s: Open failed: %v", tc, err)
			}
			require.NoError(t, d.CheckLevels(nil))
			require.NoError(t, d.Close())
		})
	}
}

type failMerger struct {
	lastBuf    []byte
	closeCount int
}

func (f *failMerger) MergeNewer(value []byte) error {
	return nil
}

func (f *failMerger) MergeOlder(value []byte) error {
	if string(value) == "fail-merge" {
		f.lastBuf = nil
		return errors.New("merge failed")
	}
	f.lastBuf = append(f.lastBuf[:0], value...)
	return nil
}

func (f *failMerger) Finish(includesBase bool) ([]byte, io.Closer, error) {
	if string(f.lastBuf) == "fail-finish" {
		f.lastBuf = nil
		return nil, nil, errors.New("finish failed")
	}
	f.closeCount++
	return nil, f, nil
}

func (f *failMerger) Close() error {
	f.closeCount--
	f.lastBuf = nil
	return nil
}

func TestCheckLevelsCornerCases(t *testing.T) {
	memFS := vfs.NewMem()
	var levels [][]*fileMetadata
	formatKey := DefaultComparer.FormatKey
	// Indexed by fileNum
	var readers []*sstable.Reader
	defer func() {
		for _, r := range readers {
			r.Close()
		}
	}()

	var fileNum FileNum
	newIters :=
		func(_ context.Context, file *manifest.FileMetadata, _ *IterOptions, _ internalIterOpts) (internalIterator, keyspan.FragmentIterator, error) {
			r := readers[file.FileNum]
			rangeDelIter, err := r.NewRawRangeDelIter()
			if err != nil {
				return nil, nil, err
			}
			iter, err := r.NewIter(nil /* lower */, nil /* upper */)
			if err != nil {
				return nil, nil, err
			}
			return iter, rangeDelIter, nil
		}

	fm := &failMerger{}
	defer require.Equal(t, 0, fm.closeCount)

	failMerger := &Merger{
		Merge: func(key, value []byte) (ValueMerger, error) {
			fm.lastBuf = append(fm.lastBuf[:0], value...)
			return fm, nil
		},

		Name: "fail-merger",
	}

	datadriven.RunTest(t, "testdata/level_checker", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			lines := strings.Split(d.Input, "\n")
			levels = levels[:0]
			for i := 0; i < len(lines); i++ {
				line := lines[i]
				line = strings.TrimSpace(line)
				if line == "L" {
					// start next level
					levels = append(levels, nil)
					continue
				}
				li := &levels[len(levels)-1]
				keys := strings.Fields(line)
				smallestKey := base.ParseInternalKey(keys[0])
				largestKey := base.ParseInternalKey(keys[1])
				m := (&fileMetadata{
					FileNum: fileNum,
				}).ExtendPointKeyBounds(DefaultComparer.Compare, smallestKey, largestKey)
				m.InitPhysicalBacking()
				*li = append(*li, m)

				i++
				line = lines[i]
				line = strings.TrimSpace(line)
				name := fmt.Sprint(fileNum)
				fileNum++
				f, err := memFS.Create(name)
				if err != nil {
					return err.Error()
				}
				writeUnfragmented := false
				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
				for _, arg := range d.CmdArgs {
					switch arg.Key {
					case "disable-key-order-checks":
						private.SSTableWriterDisableKeyOrderChecks(w)
					case "write-unfragmented":
						writeUnfragmented = true
					default:
						return fmt.Sprintf("unknown arg: %s", arg.Key)
					}
				}
				var tombstones []keyspan.Span
				frag := keyspan.Fragmenter{
					Cmp:    DefaultComparer.Compare,
					Format: formatKey,
					Emit: func(fragmented keyspan.Span) {
						tombstones = append(tombstones, fragmented)
					},
				}
				keyvalues := strings.Fields(line)
				for _, kv := range keyvalues {
					j := strings.Index(kv, ":")
					ikey := base.ParseInternalKey(kv[:j])
					value := []byte(kv[j+1:])
					var err error
					switch ikey.Kind() {
					case InternalKeyKindRangeDelete:
						if writeUnfragmented {
							err = w.Add(ikey, value)
							break
						}
						frag.Add(rangedel.Decode(ikey, value, nil))
					default:
						err = w.Add(ikey, value)
					}
					if err != nil {
						return err.Error()
					}
				}
				frag.Finish()
				for _, v := range tombstones {
					if err := rangedel.Encode(&v, w.Add); err != nil {
						return err.Error()
					}
				}
				if err := w.Close(); err != nil {
					return err.Error()
				}
				f, err = memFS.Open(name)
				if err != nil {
					return err.Error()
				}
				readable, err := sstable.NewSimpleReadable(f)
				if err != nil {
					return err.Error()
				}
				cacheOpts := private.SSTableCacheOpts(0, base.FileNum(uint64(fileNum)-1).DiskFileNum()).(sstable.ReaderOption)
				r, err := sstable.NewReader(readable, sstable.ReaderOptions{}, cacheOpts)
				if err != nil {
					return err.Error()
				}
				readers = append(readers, r)
			}
			// TODO(sbhola): clean this up by wrapping levels in a Version and using
			// Version.DebugString().
			var buf bytes.Buffer
			for i, l := range levels {
				fmt.Fprintf(&buf, "Level %d\n", i+1)
				for j, f := range l {
					fmt.Fprintf(&buf, "  file %d: [%s-%s]\n", j, f.Smallest.String(), f.Largest.String())
				}
			}
			return buf.String()
		case "check":
			merge := DefaultMerger.Merge
			for _, arg := range d.CmdArgs {
				switch arg.Key {
				case "merger":
					if len(arg.Vals) != 1 {
						return fmt.Sprintf("expected one arg value, got %d", len(arg.Vals))
					}
					if arg.Vals[0] != failMerger.Name {
						return "unsupported merger"
					}
					merge = failMerger.Merge
				default:
					return fmt.Sprintf("unknown arg: %s", arg.Key)
				}
			}

			var files [numLevels][]*fileMetadata
			for i := range levels {
				// Start from level 1 in this test.
				files[i+1] = levels[i]
			}
			version := manifest.NewVersion(
				base.DefaultComparer.Compare,
				base.DefaultFormatter,
				0,
				files)
			readState := &readState{current: version}
			c := &checkConfig{
				comparer:  DefaultComparer,
				readState: readState,
				newIters:  newIters,
				seqNum:    InternalKeySeqNumMax,
				merge:     merge,
				formatKey: formatKey,
			}
			if err := checkLevelsInternal(c); err != nil {
				return err.Error()
			}
			return ""
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}
