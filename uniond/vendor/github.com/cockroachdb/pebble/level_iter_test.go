// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
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

const (
	level = 1
)

func TestLevelIter(t *testing.T) {
	var iters []*fakeIter
	var files manifest.LevelSlice

	newIters := func(
		_ context.Context, file *manifest.FileMetadata, opts *IterOptions, _ internalIterOpts,
	) (internalIterator, keyspan.FragmentIterator, error) {
		f := *iters[file.FileNum]
		f.lower = opts.GetLowerBound()
		f.upper = opts.GetUpperBound()
		return &f, nil, nil
	}

	datadriven.RunTest(t, "testdata/level_iter", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			iters = nil
			var metas []*fileMetadata
			for _, line := range strings.Split(d.Input, "\n") {
				f := &fakeIter{}
				for _, key := range strings.Fields(line) {
					j := strings.Index(key, ":")
					f.keys = append(f.keys, base.ParseInternalKey(key[:j]))
					f.vals = append(f.vals, []byte(key[j+1:]))
				}
				iters = append(iters, f)

				meta := (&fileMetadata{
					FileNum: FileNum(len(metas)),
				}).ExtendPointKeyBounds(
					DefaultComparer.Compare,
					f.keys[0],
					f.keys[len(f.keys)-1],
				)
				meta.InitPhysicalBacking()
				metas = append(metas, meta)
			}
			files = manifest.NewLevelSliceKeySorted(base.DefaultComparer.Compare, metas)

			return ""

		case "iter":
			var opts IterOptions
			for _, arg := range d.CmdArgs {
				if len(arg.Vals) != 1 {
					return fmt.Sprintf("%s: %s=<value>", d.Cmd, arg.Key)
				}
				switch arg.Key {
				case "lower":
					opts.LowerBound = []byte(arg.Vals[0])
				case "upper":
					opts.UpperBound = []byte(arg.Vals[0])
				default:
					return fmt.Sprintf("%s: unknown arg: %s", d.Cmd, arg.Key)
				}
			}

			iter := newLevelIter(opts, testkeys.Comparer, newIters, files.Iter(), manifest.Level(level),
				internalIterOpts{})
			defer iter.Close()
			// Fake up the range deletion initialization.
			iter.initRangeDel(new(keyspan.FragmentIterator))
			iter.disableInvariants = true
			return runInternalIterCmd(t, d, iter, iterCmdVerboseKey)

		case "load":
			// The "load" command allows testing the iterator options passed to load
			// sstables.
			//
			// load <key> [lower=<key>] [upper=<key>]
			var opts IterOptions
			var key string
			for _, arg := range d.CmdArgs {
				if len(arg.Vals) == 0 {
					key = arg.Key
					continue
				}
				if len(arg.Vals) != 1 {
					return fmt.Sprintf("%s: %s=<value>", d.Cmd, arg.Key)
				}
				switch arg.Key {
				case "lower":
					opts.LowerBound = []byte(arg.Vals[0])
				case "upper":
					opts.UpperBound = []byte(arg.Vals[0])
				default:
					return fmt.Sprintf("%s: unknown arg: %s", d.Cmd, arg.Key)
				}
			}

			var tableOpts *IterOptions
			newIters2 := func(
				ctx context.Context, file *manifest.FileMetadata, opts *IterOptions,
				internalOpts internalIterOpts,
			) (internalIterator, keyspan.FragmentIterator, error) {
				tableOpts = opts
				return newIters(ctx, file, opts, internalOpts)
			}

			iter := newLevelIter(opts, testkeys.Comparer, newIters2, files.Iter(),
				manifest.Level(level), internalIterOpts{})
			iter.SeekGE([]byte(key), base.SeekGEFlagsNone)
			lower, upper := tableOpts.GetLowerBound(), tableOpts.GetUpperBound()
			return fmt.Sprintf("[%s,%s]\n", lower, upper)

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

type levelIterTest struct {
	cmp          base.Comparer
	mem          vfs.FS
	readers      []*sstable.Reader
	metas        []*fileMetadata
	itersCreated int
}

func newLevelIterTest() *levelIterTest {
	lt := &levelIterTest{
		cmp: *DefaultComparer,
		mem: vfs.NewMem(),
	}
	lt.cmp.Split = func(a []byte) int { return len(a) }
	return lt
}

func (lt *levelIterTest) newIters(
	ctx context.Context, file *manifest.FileMetadata, opts *IterOptions, iio internalIterOpts,
) (internalIterator, keyspan.FragmentIterator, error) {
	lt.itersCreated++
	iter, err := lt.readers[file.FileNum].NewIterWithBlockPropertyFiltersAndContextEtc(
		ctx, opts.LowerBound, opts.UpperBound, nil, false, true, iio.stats,
		sstable.TrivialReaderProvider{Reader: lt.readers[file.FileNum]})
	if err != nil {
		return nil, nil, err
	}
	rangeDelIter, err := lt.readers[file.FileNum].NewRawRangeDelIter()
	if err != nil {
		return nil, nil, err
	}
	return iter, rangeDelIter, nil
}

func (lt *levelIterTest) runClear(d *datadriven.TestData) string {
	lt.mem = vfs.NewMem()
	for _, r := range lt.readers {
		r.Close()
	}
	lt.readers = nil
	lt.metas = nil
	lt.itersCreated = 0
	return ""
}

func (lt *levelIterTest) runBuild(d *datadriven.TestData) string {
	fileNum := FileNum(len(lt.readers))
	name := fmt.Sprint(fileNum)
	f0, err := lt.mem.Create(name)
	if err != nil {
		return err.Error()
	}

	tableFormat := sstable.TableFormatRocksDBv2
	for _, arg := range d.CmdArgs {
		if arg.Key == "format" {
			switch arg.Vals[0] {
			case "rocksdbv2":
				tableFormat = sstable.TableFormatRocksDBv2
			case "pebblev2":
				tableFormat = sstable.TableFormatPebblev2
			}
		}
	}
	fp := bloom.FilterPolicy(10)
	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f0), sstable.WriterOptions{
		Comparer:     &lt.cmp,
		FilterPolicy: fp,
		TableFormat:  tableFormat,
	})
	var tombstones []keyspan.Span
	f := keyspan.Fragmenter{
		Cmp:    lt.cmp.Compare,
		Format: lt.cmp.FormatKey,
		Emit: func(fragmented keyspan.Span) {
			tombstones = append(tombstones, fragmented)
		},
	}
	for _, key := range strings.Split(d.Input, "\n") {
		j := strings.Index(key, ":")
		ikey := base.ParseInternalKey(key[:j])
		value := []byte(key[j+1:])
		switch ikey.Kind() {
		case InternalKeyKindRangeDelete:
			f.Add(rangedel.Decode(ikey, value, nil))
		case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
			if err := w.AddRangeKey(ikey, value); err != nil {
				return err.Error()
			}
		default:
			if err := w.Add(ikey, value); err != nil {
				return err.Error()
			}
		}
	}
	f.Finish()
	for _, v := range tombstones {
		if err := rangedel.Encode(&v, w.Add); err != nil {
			return err.Error()
		}
	}
	if err := w.Close(); err != nil {
		return err.Error()
	}
	meta, err := w.Metadata()
	if err != nil {
		return err.Error()
	}

	f1, err := lt.mem.Open(name)
	if err != nil {
		return err.Error()
	}
	readable, err := sstable.NewSimpleReadable(f1)
	if err != nil {
		return err.Error()
	}
	r, err := sstable.NewReader(readable, sstable.ReaderOptions{
		Filters: map[string]FilterPolicy{
			fp.Name(): fp,
		},
	})
	if err != nil {
		return err.Error()
	}
	lt.readers = append(lt.readers, r)
	m := &fileMetadata{FileNum: fileNum}
	if meta.HasPointKeys {
		m.ExtendPointKeyBounds(lt.cmp.Compare, meta.SmallestPoint, meta.LargestPoint)
	}
	if meta.HasRangeDelKeys {
		m.ExtendPointKeyBounds(lt.cmp.Compare, meta.SmallestRangeDel, meta.LargestRangeDel)
	}
	if meta.HasRangeKeys {
		m.ExtendRangeKeyBounds(lt.cmp.Compare, meta.SmallestRangeKey, meta.LargestRangeKey)
	}
	m.InitPhysicalBacking()
	lt.metas = append(lt.metas, m)

	var buf bytes.Buffer
	for _, f := range lt.metas {
		fmt.Fprintf(&buf, "%d: %s-%s\n", f.FileNum, f.Smallest, f.Largest)
	}
	return buf.String()
}

func TestLevelIterBoundaries(t *testing.T) {
	lt := newLevelIterTest()
	defer lt.runClear(nil)

	var iter *levelIter
	datadriven.RunTest(t, "testdata/level_iter_boundaries", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "clear":
			return lt.runClear(d)

		case "build":
			return lt.runBuild(d)

		case "iter":
			// The save and continue parameters allow us to save the iterator
			// for later continued use.
			save := false
			cont := false
			for _, arg := range d.CmdArgs {
				switch arg.Key {
				case "save":
					save = true
				case "continue":
					cont = true
				default:
					return fmt.Sprintf("%s: unknown arg: %s", d.Cmd, arg.Key)
				}
			}
			if !cont && iter != nil {
				return "preceding iter was not closed"
			}
			if cont && iter == nil {
				return "no existing iter"
			}
			if iter == nil {
				slice := manifest.NewLevelSliceKeySorted(lt.cmp.Compare, lt.metas)
				iter = newLevelIter(IterOptions{}, testkeys.Comparer, lt.newIters, slice.Iter(),
					manifest.Level(level), internalIterOpts{})
				// Fake up the range deletion initialization.
				iter.initRangeDel(new(keyspan.FragmentIterator))
			}
			if !save {
				defer func() {
					iter.Close()
					iter = nil
				}()
			}
			return runInternalIterCmd(t, d, iter, iterCmdVerboseKey)

		case "file-pos":
			// Returns the FileNum at which the iterator is positioned.
			if iter == nil {
				return "nil levelIter"
			}
			if iter.iterFile == nil {
				return "nil iterFile"
			}
			return fmt.Sprintf("file %d", iter.iterFile.FileNum)

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

// levelIterTestIter allows a datadriven test to use runInternalIterCmd and
// perform parallel operations on both both a levelIter and rangeDelIter.
type levelIterTestIter struct {
	*levelIter
	rangeDelIter keyspan.FragmentIterator
}

func (i *levelIterTestIter) rangeDelSeek(
	key []byte, ikey *InternalKey, val base.LazyValue, dir int,
) (*InternalKey, base.LazyValue) {
	var tombstone keyspan.Span
	if i.rangeDelIter != nil {
		var t *keyspan.Span
		if dir < 0 {
			t = keyspan.SeekLE(i.levelIter.cmp, i.rangeDelIter, key)
		} else {
			t = i.rangeDelIter.SeekGE(key)
		}
		if t != nil {
			tombstone = t.Visible(1000)
		}
	}
	if ikey == nil {
		return &InternalKey{
			UserKey: []byte(fmt.Sprintf("./%s", tombstone)),
		}, base.LazyValue{}
	}
	return &InternalKey{
		UserKey: []byte(fmt.Sprintf("%s/%s", ikey.UserKey, tombstone)),
		Trailer: ikey.Trailer,
	}, val
}

func (i *levelIterTestIter) String() string {
	return "level-iter-test"
}

func (i *levelIterTestIter) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	ikey, val := i.levelIter.SeekGE(key, flags)
	return i.rangeDelSeek(key, ikey, val, 1)
}

func (i *levelIterTestIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	ikey, val := i.levelIter.SeekPrefixGE(prefix, key, flags)
	return i.rangeDelSeek(key, ikey, val, 1)
}

func (i *levelIterTestIter) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	ikey, val := i.levelIter.SeekLT(key, flags)
	return i.rangeDelSeek(key, ikey, val, -1)
}

func TestLevelIterSeek(t *testing.T) {
	lt := newLevelIterTest()
	defer lt.runClear(nil)

	datadriven.RunTest(t, "testdata/level_iter_seek", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "clear":
			return lt.runClear(d)

		case "build":
			return lt.runBuild(d)

		case "iter":
			var stats base.InternalIteratorStats
			slice := manifest.NewLevelSliceKeySorted(lt.cmp.Compare, lt.metas)
			iter := &levelIterTestIter{levelIter: &levelIter{}}
			iter.init(context.Background(), IterOptions{}, testkeys.Comparer, lt.newIters, slice.Iter(),
				manifest.Level(level), internalIterOpts{stats: &stats})
			defer iter.Close()
			iter.initRangeDel(&iter.rangeDelIter)
			return runInternalIterCmd(t, d, iter, iterCmdVerboseKey, iterCmdStats(&stats))

		case "iters-created":
			return fmt.Sprintf("%d", lt.itersCreated)
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func buildLevelIterTables(
	b *testing.B, blockSize, restartInterval, count int,
) ([]*sstable.Reader, manifest.LevelSlice, [][]byte, func()) {
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

	var keys [][]byte
	var i int
	const targetSize = 2 << 20
	for _, w := range writers {
		for ; w.EstimatedSize() < targetSize; i++ {
			key := []byte(fmt.Sprintf("%08d", i))
			keys = append(keys, key)
			ikey := base.MakeInternalKey(key, 0, InternalKeyKindSet)
			w.Add(ikey, nil)
		}
		if err := w.Close(); err != nil {
			b.Fatal(err)
		}
	}

	opts := sstable.ReaderOptions{Cache: NewCache(128 << 20), Comparer: DefaultComparer}
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

	cleanup := func() {
		for _, r := range readers {
			require.NoError(b, r.Close())
		}
	}

	meta := make([]*fileMetadata, len(readers))
	for i := range readers {
		iter, err := readers[i].NewIter(nil /* lower */, nil /* upper */)
		require.NoError(b, err)
		smallest, _ := iter.First()
		meta[i] = &fileMetadata{}
		meta[i].FileNum = FileNum(i)
		largest, _ := iter.Last()
		meta[i].ExtendPointKeyBounds(opts.Comparer.Compare, (*smallest).Clone(), (*largest).Clone())
		meta[i].InitPhysicalBacking()
	}
	slice := manifest.NewLevelSliceKeySorted(base.DefaultComparer.Compare, meta)
	return readers, slice, keys, cleanup
}

func BenchmarkLevelIterSeekGE(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, metas, keys, cleanup := buildLevelIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							newIters := func(
								_ context.Context, file *manifest.FileMetadata, _ *IterOptions, _ internalIterOpts,
							) (internalIterator, keyspan.FragmentIterator, error) {
								iter, err := readers[file.FileNum].NewIter(nil /* lower */, nil /* upper */)
								return iter, nil, err
							}
							l := newLevelIter(IterOptions{}, DefaultComparer, newIters, metas.Iter(), manifest.Level(level), internalIterOpts{})
							rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								l.SeekGE(keys[rng.Intn(len(keys))], base.SeekGEFlagsNone)
							}
							l.Close()
						})
				}
			})
	}
}

// A benchmark that simulates the behavior of a levelIter being used as part
// of a mergingIter where narrow bounds are repeatedly set and used to Seek
// and then iterate over the keys within the bounds. This resembles MVCC
// scanning by CockroachDB when doing a lookup/index join with a large number
// of left rows, that are batched and reuse the same iterator, and which can
// have good locality of access. This results in the successive bounds being
// in the same file.
func BenchmarkLevelIterSeqSeekGEWithBounds(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, metas, keys, cleanup :=
								buildLevelIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							// This newIters is cheaper than in practice since it does not do
							// tableCacheShard.findNode.
							newIters := func(
								_ context.Context, file *manifest.FileMetadata, opts *IterOptions, _ internalIterOpts,
							) (internalIterator, keyspan.FragmentIterator, error) {
								iter, err := readers[file.FileNum].NewIter(
									opts.LowerBound, opts.UpperBound)
								return iter, nil, err
							}
							l := newLevelIter(IterOptions{}, DefaultComparer, newIters, metas.Iter(), manifest.Level(level), internalIterOpts{})
							// Fake up the range deletion initialization, to resemble the usage
							// in a mergingIter.
							l.initRangeDel(new(keyspan.FragmentIterator))
							keyCount := len(keys)
							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								pos := i % (keyCount - 1)
								l.SetBounds(keys[pos], keys[pos+1])
								// SeekGE will return keys[pos].
								k, _ := l.SeekGE(keys[pos], base.SeekGEFlagsNone)
								// Next() will get called once and return nil.
								for k != nil {
									k, _ = l.Next()
								}
							}
							l.Close()
						})
				}
			})
	}
}

// BenchmarkLevelIterSeqSeekPrefixGE simulates the behavior of a levelIter
// being used as part of a mergingIter where SeekPrefixGE is used to seek in a
// monotonically increasing manner. This resembles key-value lookups done by
// CockroachDB when evaluating Put operations.
func BenchmarkLevelIterSeqSeekPrefixGE(b *testing.B) {
	const blockSize = 32 << 10
	const restartInterval = 16
	readers, metas, keys, cleanup :=
		buildLevelIterTables(b, blockSize, restartInterval, 5)
	defer cleanup()
	// This newIters is cheaper than in practice since it does not do
	// tableCacheShard.findNode.
	newIters := func(
		_ context.Context, file *manifest.FileMetadata, opts *IterOptions, _ internalIterOpts,
	) (internalIterator, keyspan.FragmentIterator, error) {
		iter, err := readers[file.FileNum].NewIter(
			opts.LowerBound, opts.UpperBound)
		return iter, nil, err
	}

	for _, skip := range []int{1, 2, 4, 8, 16} {
		for _, useNext := range []bool{false, true} {
			b.Run(fmt.Sprintf("skip=%d/use-next=%t", skip, useNext),
				func(b *testing.B) {
					l := newLevelIter(IterOptions{}, testkeys.Comparer, newIters, metas.Iter(),
						manifest.Level(level), internalIterOpts{})
					// Fake up the range deletion initialization, to resemble the usage
					// in a mergingIter.
					l.initRangeDel(new(keyspan.FragmentIterator))
					keyCount := len(keys)
					pos := 0
					l.SeekPrefixGE(keys[pos], keys[pos], base.SeekGEFlagsNone)
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
						l.SeekPrefixGE(keys[pos], keys[pos], flags)
					}
					b.StopTimer()
					l.Close()
				})
		}
	}
}

func BenchmarkLevelIterNext(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, metas, _, cleanup := buildLevelIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							newIters := func(
								_ context.Context, file *manifest.FileMetadata, _ *IterOptions, _ internalIterOpts,
							) (internalIterator, keyspan.FragmentIterator, error) {
								iter, err := readers[file.FileNum].NewIter(nil /* lower */, nil /* upper */)
								return iter, nil, err
							}
							l := newLevelIter(IterOptions{}, testkeys.Comparer, newIters, metas.Iter(), manifest.Level(level), internalIterOpts{})

							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								key, _ := l.Next()
								if key == nil {
									key, _ = l.First()
								}
								_ = key
							}
							l.Close()
						})
				}
			})
	}
}

func BenchmarkLevelIterPrev(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				for _, count := range []int{5} {
					b.Run(fmt.Sprintf("count=%d", count),
						func(b *testing.B) {
							readers, metas, _, cleanup := buildLevelIterTables(b, blockSize, restartInterval, count)
							defer cleanup()
							newIters := func(
								_ context.Context, file *manifest.FileMetadata, _ *IterOptions, _ internalIterOpts,
							) (internalIterator, keyspan.FragmentIterator, error) {
								iter, err := readers[file.FileNum].NewIter(nil /* lower */, nil /* upper */)
								return iter, nil, err
							}
							l := newLevelIter(IterOptions{}, DefaultComparer, newIters, metas.Iter(), manifest.Level(level), internalIterOpts{})

							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								key, _ := l.Prev()
								if key == nil {
									key, _ = l.Last()
								}
								_ = key
							}
							l.Close()
						})
				}
			})
	}
}
