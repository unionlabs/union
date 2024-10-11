// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestMergingIter(t *testing.T) {
	var stats base.InternalIteratorStats
	newFunc := func(iters ...internalIterator) internalIterator {
		return newMergingIter(nil /* logger */, &stats, DefaultComparer.Compare,
			func(a []byte) int { return len(a) }, iters...)
	}
	testIterator(t, newFunc, func(r *rand.Rand) [][]string {
		// Shuffle testKeyValuePairs into one or more splits. Each individual
		// split is in increasing order, but different splits may overlap in
		// range. Some of the splits may be empty.
		splits := make([][]string, 1+r.Intn(2+len(testKeyValuePairs)))
		for _, kv := range testKeyValuePairs {
			j := r.Intn(len(splits))
			splits[j] = append(splits[j], kv)
		}
		return splits
	})
}

func TestMergingIterSeek(t *testing.T) {
	var def string
	datadriven.RunTest(t, "testdata/merging_iter_seek", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			def = d.Input
			return ""

		case "iter":
			var iters []internalIterator
			for _, line := range strings.Split(def, "\n") {
				f := &fakeIter{}
				for _, key := range strings.Fields(line) {
					j := strings.Index(key, ":")
					f.keys = append(f.keys, base.ParseInternalKey(key[:j]))
					f.vals = append(f.vals, []byte(key[j+1:]))
				}
				iters = append(iters, f)
			}

			var stats base.InternalIteratorStats
			iter := newMergingIter(nil /* logger */, &stats, DefaultComparer.Compare,
				func(a []byte) int { return len(a) }, iters...)
			defer iter.Close()
			return runInternalIterCmd(t, d, iter)

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestMergingIterNextPrev(t *testing.T) {
	// The data is the same in each of these cases, but divided up amongst the
	// iterators differently. This data must match the definition in
	// testdata/internal_iter_next.
	iterCases := [][]string{
		{
			"a.SET.2:2 a.SET.1:1 b.SET.2:2 b.SET.1:1 c.SET.2:2 c.SET.1:1",
		},
		{
			"a.SET.2:2 b.SET.2:2 c.SET.2:2",
			"a.SET.1:1 b.SET.1:1 c.SET.1:1",
		},
		{
			"a.SET.2:2 b.SET.2:2",
			"a.SET.1:1 b.SET.1:1",
			"c.SET.2:2 c.SET.1:1",
		},
		{
			"a.SET.2:2",
			"a.SET.1:1",
			"b.SET.2:2",
			"b.SET.1:1",
			"c.SET.2:2",
			"c.SET.1:1",
		},
	}

	for _, c := range iterCases {
		t.Run("", func(t *testing.T) {
			datadriven.RunTest(t, "testdata/internal_iter_next", func(t *testing.T, d *datadriven.TestData) string {
				switch d.Cmd {
				case "define":
					// Ignore. We've defined the iterator data above.
					return ""

				case "iter":
					iters := make([]internalIterator, len(c))
					for i := range c {
						f := &fakeIter{}
						iters[i] = f
						for _, key := range strings.Fields(c[i]) {
							j := strings.Index(key, ":")
							f.keys = append(f.keys, base.ParseInternalKey(key[:j]))
							f.vals = append(f.vals, []byte(key[j+1:]))
						}
					}

					var stats base.InternalIteratorStats
					iter := newMergingIter(nil /* logger */, &stats, DefaultComparer.Compare,
						func(a []byte) int { return len(a) }, iters...)
					defer iter.Close()
					return runInternalIterCmd(t, d, iter)

				default:
					return fmt.Sprintf("unknown command: %s", d.Cmd)
				}
			})
		})
	}
}

func TestMergingIterCornerCases(t *testing.T) {
	memFS := vfs.NewMem()
	cmp := DefaultComparer.Compare
	fmtKey := DefaultComparer.FormatKey
	opts := (*Options)(nil).EnsureDefaults()
	var v *version

	// Indexed by fileNum.
	var readers []*sstable.Reader
	defer func() {
		for _, r := range readers {
			r.Close()
		}
	}()

	var fileNum base.FileNum
	newIters :=
		func(_ context.Context, file *manifest.FileMetadata, opts *IterOptions, iio internalIterOpts,
		) (internalIterator, keyspan.FragmentIterator, error) {
			r := readers[file.FileNum]
			rangeDelIter, err := r.NewRawRangeDelIter()
			if err != nil {
				return nil, nil, err
			}
			iter, err := r.NewIterWithBlockPropertyFilters(
				opts.GetLowerBound(), opts.GetUpperBound(), nil, true /* useFilterBlock */, iio.stats,
				sstable.TrivialReaderProvider{Reader: r})
			if err != nil {
				return nil, nil, err
			}
			return iter, rangeDelIter, nil
		}

	datadriven.RunTest(t, "testdata/merging_iter", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			lines := strings.Split(d.Input, "\n")

			var files [numLevels][]*fileMetadata
			var level int
			for i := 0; i < len(lines); i++ {
				line := lines[i]
				line = strings.TrimSpace(line)
				if line == "L" || line == "L0" {
					// start next level
					level++
					continue
				}
				keys := strings.Fields(line)
				smallestKey := base.ParseInternalKey(keys[0])
				largestKey := base.ParseInternalKey(keys[1])
				m := (&fileMetadata{
					FileNum: fileNum,
				}).ExtendPointKeyBounds(cmp, smallestKey, largestKey)
				m.InitPhysicalBacking()
				files[level] = append(files[level], m)

				i++
				line = lines[i]
				line = strings.TrimSpace(line)
				name := fmt.Sprint(fileNum)
				fileNum++
				f, err := memFS.Create(name)
				if err != nil {
					return err.Error()
				}
				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
				var tombstones []keyspan.Span
				frag := keyspan.Fragmenter{
					Cmp:    cmp,
					Format: fmtKey,
					Emit: func(fragmented keyspan.Span) {
						tombstones = append(tombstones, fragmented)
					},
				}
				keyvalues := strings.Fields(line)
				for _, kv := range keyvalues {
					j := strings.Index(kv, ":")
					ikey := base.ParseInternalKey(kv[:j])
					value := []byte(kv[j+1:])
					switch ikey.Kind() {
					case InternalKeyKindRangeDelete:
						frag.Add(keyspan.Span{Start: ikey.UserKey, End: value, Keys: []keyspan.Key{{Trailer: ikey.Trailer}}})
					default:
						if err := w.Add(ikey, value); err != nil {
							return err.Error()
						}
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
				r, err := sstable.NewReader(readable, sstable.ReaderOptions{})
				if err != nil {
					return err.Error()
				}
				readers = append(readers, r)
			}

			v = newVersion(opts, files)
			return v.String()
		case "iter":
			levelIters := make([]mergingIterLevel, 0, len(v.Levels))
			var stats base.InternalIteratorStats
			for i, l := range v.Levels {
				slice := l.Slice()
				if slice.Empty() {
					continue
				}
				li := &levelIter{}
				li.init(context.Background(), IterOptions{}, testkeys.Comparer,
					newIters, slice.Iter(), manifest.Level(i), internalIterOpts{stats: &stats})
				i := len(levelIters)
				levelIters = append(levelIters, mergingIterLevel{iter: li})
				li.initRangeDel(&levelIters[i].rangeDelIter)
				li.initBoundaryContext(&levelIters[i].levelIterBoundaryContext)
			}
			miter := &mergingIter{}
			miter.init(nil /* opts */, &stats, cmp, func(a []byte) int { return len(a) }, levelIters...)
			defer miter.Close()
			miter.forceEnableSeekOpt = true
			return runInternalIterCmd(t, d, miter, iterCmdVerboseKey, iterCmdStats(&stats))
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func buildMergingIterTables(
	b *testing.B, blockSize, restartInterval, count int,
) ([]*sstable.Reader, [][]byte, func()) {
	mem := vfs.NewMem()
	files := make([]vfs.File, count)
	for i := range files {
		f, err := mem.Create(fmt.Sprintf("bench%d", i))
		if err != nil {
			b.Fatal(err)
		}
		files[i] = f
	}

	writers := make([]*sstable.Writer, len(files))
	for i := range files {
		writers[i] = sstable.NewWriter(objstorageprovider.NewFileWritable(files[i]), sstable.WriterOptions{
			BlockRestartInterval: restartInterval,
			BlockSize:            blockSize,
			Compression:          NoCompression,
		})
	}

	estimatedSize := func() uint64 {
		var sum uint64
		for _, w := range writers {
			sum += w.EstimatedSize()
		}
		return sum
	}

	var keys [][]byte
	var ikey InternalKey
	targetSize := uint64(count * (2 << 20))
	for i := 0; estimatedSize() < targetSize; i++ {
		key := []byte(fmt.Sprintf("%08d", i))
		keys = append(keys, key)
		ikey.UserKey = key
		j := rand.Intn(len(writers))
		w := writers[j]
		w.Add(ikey, nil)
	}

	for _, w := range writers {
		if err := w.Close(); err != nil {
			b.Fatal(err)
		}
	}

	opts := sstable.ReaderOptions{Cache: NewCache(128 << 20)}
	defer opts.Cache.Unref()

	readers := make([]*sstable.Reader, len(files))
	for i := range files {
		f, err := mem.Open(fmt.Sprintf("bench%d", i))
		if err != nil {
			b.Fatal(err)
		}
		readable, err := sstable.NewSimpleReadable(f)
		if err != nil {
			b.Fatal(err)
		}
		readers[i], err = sstable.NewReader(readable, opts)
		if err != nil {
			b.Fatal(err)
		}
	}
	return readers, keys, func() {
		for _, r := range readers {
			require.NoError(b, r.Close())
		}
	}
}

func BenchmarkMergingIterSeekGE(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{1, 2, 3, 4, 5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, keys, cleanup := buildMergingIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							iters := make([]internalIterator, len(readers))
							for i := range readers {
								var err error
								iters[i], err = readers[i].NewIter(nil /* lower */, nil /* upper */)
								require.NoError(b, err)
							}
							var stats base.InternalIteratorStats
							m := newMergingIter(nil /* logger */, &stats, DefaultComparer.Compare,
								func(a []byte) int { return len(a) }, iters...)
							rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								m.SeekGE(keys[rng.Intn(len(keys))], base.SeekGEFlagsNone)
							}
							m.Close()
						})
				}
			})
	}
}

func BenchmarkMergingIterNext(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{1, 2, 3, 4, 5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, _, cleanup := buildMergingIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							iters := make([]internalIterator, len(readers))
							for i := range readers {
								var err error
								iters[i], err = readers[i].NewIter(nil /* lower */, nil /* upper */)
								require.NoError(b, err)
							}
							var stats base.InternalIteratorStats
							m := newMergingIter(nil /* logger */, &stats, DefaultComparer.Compare,
								func(a []byte) int { return len(a) }, iters...)

							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								key, _ := m.Next()
								if key == nil {
									key, _ = m.First()
								}
								_ = key
							}
							m.Close()
						})
				}
			})
	}
}

func BenchmarkMergingIterPrev(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{1, 2, 3, 4, 5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, _, cleanup := buildMergingIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							iters := make([]internalIterator, len(readers))
							for i := range readers {
								var err error
								iters[i], err = readers[i].NewIter(nil /* lower */, nil /* upper */)
								require.NoError(b, err)
							}
							var stats base.InternalIteratorStats
							m := newMergingIter(nil /* logger */, &stats, DefaultComparer.Compare,
								func(a []byte) int { return len(a) }, iters...)

							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								key, _ := m.Prev()
								if key == nil {
									key, _ = m.Last()
								}
								_ = key
							}
							m.Close()
						})
				}
			})
	}
}

// Builds levels for BenchmarkMergingIterSeqSeekGEWithBounds. The lowest level,
// index 0 here, contains most of the data. Each level has 2 files, to allow for
// stepping into the second file if needed. The lowest level has all the keys in
// the file 0, and a single "lastIKey" in file 1. File 0 in all other levels have
// only the first and last key of file 0 of the aforementioned level -- this
// simulates sparseness of data, but not necessarily of file width, in higher
// levels. File 1 in other levels is similar to File 1 in the aforementioned level
// since it is only for stepping into. If writeRangeTombstoneToLowestLevel is
// true, a range tombstone is written to the first lowest level file that
// deletes all the keys in it, and no other levels should be written.
func buildLevelsForMergingIterSeqSeek(
	b *testing.B,
	blockSize, restartInterval, levelCount int,
	keyOffset int,
	writeRangeTombstoneToLowestLevel bool,
	writeBloomFilters bool,
	forceTwoLevelIndex bool,
) (readers [][]*sstable.Reader, levelSlices []manifest.LevelSlice, keys [][]byte) {
	mem := vfs.NewMem()
	if writeRangeTombstoneToLowestLevel && levelCount != 1 {
		panic("expect to write only 1 level")
	}
	files := make([][]vfs.File, levelCount)
	for i := range files {
		for j := 0; j < 2; j++ {
			f, err := mem.Create(fmt.Sprintf("bench%d_%d", i, j))
			if err != nil {
				b.Fatal(err)
			}
			files[i] = append(files[i], f)
		}
	}

	const targetL6FirstFileSize = 2 << 20
	writers := make([][]*sstable.Writer, levelCount)
	// A policy unlikely to have false positives.
	filterPolicy := bloom.FilterPolicy(100)
	for i := range files {
		for j := range files[i] {
			writerOptions := sstable.WriterOptions{
				BlockRestartInterval: restartInterval,
				BlockSize:            blockSize,
				Compression:          NoCompression,
			}
			if writeBloomFilters {
				writerOptions.FilterPolicy = filterPolicy
				writerOptions.FilterType = base.TableFilter
			}
			if forceTwoLevelIndex {
				if i == 0 && j == 0 {
					// Ignoring compression, approximate number of blocks
					numDataBlocks := targetL6FirstFileSize / blockSize
					if numDataBlocks < 4 {
						b.Fatalf("cannot produce two level index")
					}
					// Produce ~2 lower-level index blocks.
					writerOptions.IndexBlockSize = (numDataBlocks / 2) * 8
				} else if j == 0 {
					// Only 2 keys in these files, so to produce two level indexes we
					// set the block sizes to 1.
					writerOptions.BlockSize = 1
					writerOptions.IndexBlockSize = 1
				}
			}
			writers[i] = append(writers[i], sstable.NewWriter(objstorageprovider.NewFileWritable(files[i][j]), writerOptions))
		}
	}

	i := keyOffset
	w := writers[0][0]
	for ; w.EstimatedSize() < targetL6FirstFileSize; i++ {
		key := []byte(fmt.Sprintf("%08d", i))
		keys = append(keys, key)
		ikey := base.MakeInternalKey(key, 0, InternalKeyKindSet)
		w.Add(ikey, nil)
	}
	if writeRangeTombstoneToLowestLevel {
		tombstoneKey := base.MakeInternalKey(keys[0], 1, InternalKeyKindRangeDelete)
		w.Add(tombstoneKey, []byte(fmt.Sprintf("%08d", i)))
	}
	for j := 1; j < len(files); j++ {
		for _, k := range []int{0, len(keys) - 1} {
			ikey := base.MakeInternalKey(keys[k], uint64(j), InternalKeyKindSet)
			writers[j][0].Add(ikey, nil)
		}
	}
	lastKey := []byte(fmt.Sprintf("%08d", i))
	keys = append(keys, lastKey)
	for j := 0; j < len(files); j++ {
		lastIKey := base.MakeInternalKey(lastKey, uint64(j), InternalKeyKindSet)
		writers[j][1].Add(lastIKey, nil)
	}
	for _, levelWriters := range writers {
		for j, w := range levelWriters {
			if err := w.Close(); err != nil {
				b.Fatal(err)
			}
			meta, err := w.Metadata()
			require.NoError(b, err)
			if forceTwoLevelIndex && j == 0 && meta.Properties.IndexType != 2 {
				b.Fatalf("did not produce two level index")
			}
		}
	}

	opts := sstable.ReaderOptions{Cache: NewCache(128 << 20), Comparer: DefaultComparer}
	if writeBloomFilters {
		opts.Filters = make(map[string]FilterPolicy)
		opts.Filters[filterPolicy.Name()] = filterPolicy
	}
	defer opts.Cache.Unref()

	readers = make([][]*sstable.Reader, levelCount)
	for i := range files {
		for j := range files[i] {
			f, err := mem.Open(fmt.Sprintf("bench%d_%d", i, j))
			if err != nil {
				b.Fatal(err)
			}
			readable, err := sstable.NewSimpleReadable(f)
			if err != nil {
				b.Fatal(err)
			}
			r, err := sstable.NewReader(readable, opts)
			if err != nil {
				b.Fatal(err)
			}
			readers[i] = append(readers[i], r)
		}
	}
	levelSlices = make([]manifest.LevelSlice, levelCount)
	for i := range readers {
		meta := make([]*fileMetadata, len(readers[i]))
		for j := range readers[i] {
			iter, err := readers[i][j].NewIter(nil /* lower */, nil /* upper */)
			require.NoError(b, err)
			smallest, _ := iter.First()
			meta[j] = &fileMetadata{}
			// The same FileNum is being reused across different levels, which
			// is harmless for the benchmark since each level has its own iterator
			// creation func.
			meta[j].FileNum = FileNum(j)
			largest, _ := iter.Last()
			meta[j].ExtendPointKeyBounds(opts.Comparer.Compare, smallest.Clone(), largest.Clone())
			meta[j].InitPhysicalBacking()
		}
		levelSlices[i] = manifest.NewLevelSliceSpecificOrder(meta)
	}
	return readers, levelSlices, keys
}

func buildMergingIter(readers [][]*sstable.Reader, levelSlices []manifest.LevelSlice) *mergingIter {
	mils := make([]mergingIterLevel, len(levelSlices))
	for i := len(readers) - 1; i >= 0; i-- {
		levelIndex := i
		level := len(readers) - 1 - i
		newIters := func(
			_ context.Context, file *manifest.FileMetadata, opts *IterOptions, _ internalIterOpts,
		) (internalIterator, keyspan.FragmentIterator, error) {
			iter, err := readers[levelIndex][file.FileNum].NewIter(
				opts.LowerBound, opts.UpperBound)
			if err != nil {
				return nil, nil, err
			}
			rdIter, err := readers[levelIndex][file.FileNum].NewRawRangeDelIter()
			if err != nil {
				iter.Close()
				return nil, nil, err
			}
			return iter, rdIter, err
		}
		l := newLevelIter(IterOptions{}, testkeys.Comparer, newIters, levelSlices[i].Iter(),
			manifest.Level(level), internalIterOpts{})
		l.initRangeDel(&mils[level].rangeDelIter)
		l.initBoundaryContext(&mils[level].levelIterBoundaryContext)
		mils[level].iter = l
	}
	var stats base.InternalIteratorStats
	m := &mergingIter{}
	m.init(nil /* logger */, &stats, testkeys.Comparer.Compare,
		func(a []byte) int { return len(a) }, mils...)
	return m
}

// A benchmark that simulates the behavior of a mergingIter where
// monotonically increasing narrow bounds are repeatedly set and used to Seek
// and then iterate over the keys within the bounds. This resembles MVCC
// scanning by CockroachDB when doing a lookup/index join with a large number
// of left rows, that are batched and reuse the same iterator, and which can
// have good locality of access. This results in the successive bounds being
// in the same file.
func BenchmarkMergingIterSeqSeekGEWithBounds(b *testing.B) {
	const blockSize = 32 << 10

	restartInterval := 16
	for _, levelCount := range []int{5} {
		b.Run(fmt.Sprintf("levelCount=%d", levelCount),
			func(b *testing.B) {
				readers, levelSlices, keys := buildLevelsForMergingIterSeqSeek(
					b, blockSize, restartInterval, levelCount, 0 /* keyOffset */, false, false, false)
				m := buildMergingIter(readers, levelSlices)
				keyCount := len(keys)
				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					pos := i % (keyCount - 1)
					m.SetBounds(keys[pos], keys[pos+1])
					// SeekGE will return keys[pos].
					k, _ := m.SeekGE(keys[pos], base.SeekGEFlagsNone)
					for k != nil {
						k, _ = m.Next()
					}
				}
				m.Close()
				for i := range readers {
					for j := range readers[i] {
						readers[i][j].Close()
					}
				}
			})
	}
}

func BenchmarkMergingIterSeqSeekPrefixGE(b *testing.B) {
	const blockSize = 32 << 10
	const restartInterval = 16
	const levelCount = 5
	readers, levelSlices, keys := buildLevelsForMergingIterSeqSeek(
		b, blockSize, restartInterval, levelCount, 0 /* keyOffset */, false, false, false)

	for _, skip := range []int{1, 2, 4, 8, 16} {
		for _, useNext := range []bool{false, true} {
			b.Run(fmt.Sprintf("skip=%d/use-next=%t", skip, useNext),
				func(b *testing.B) {
					m := buildMergingIter(readers, levelSlices)
					keyCount := len(keys)
					pos := 0

					m.SeekPrefixGE(keys[pos], keys[pos], base.SeekGEFlagsNone)
					b.ResetTimer()
					for i := 0; i < b.N; i++ {
						pos += skip
						var flags base.SeekGEFlags
						if useNext {
							flags = flags.EnableTrySeekUsingNext()
						}
						if pos >= keyCount {
							pos = 0
							flags = flags.DisableTrySeekUsingNext()
						}
						// SeekPrefixGE will return keys[pos].
						m.SeekPrefixGE(keys[pos], keys[pos], flags)
					}
					b.StopTimer()
					m.Close()
				})
		}
	}
	for i := range readers {
		for j := range readers[i] {
			readers[i][j].Close()
		}
	}
}
