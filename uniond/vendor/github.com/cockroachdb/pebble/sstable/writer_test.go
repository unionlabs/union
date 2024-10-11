// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math/rand"
	"strconv"
	"strings"
	"sync"
	"testing"
	"unsafe"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func testWriterParallelism(t *testing.T, parallelism bool) {
	for _, format := range []TableFormat{TableFormatPebblev2, TableFormatPebblev3} {
		tdFile := "testdata/writer"
		if format == TableFormatPebblev3 {
			tdFile = "testdata/writer_v3"
		}
		t.Run(format.String(), func(t *testing.T) { runDataDriven(t, tdFile, format, parallelism) })
	}
}
func TestWriter(t *testing.T) {
	testWriterParallelism(t, false)
}

func testRewriterParallelism(t *testing.T, parallelism bool) {
	for _, format := range []TableFormat{TableFormatPebblev2, TableFormatPebblev3} {
		tdFile := "testdata/rewriter"
		if format == TableFormatPebblev3 {
			tdFile = "testdata/rewriter_v3"
		}
		t.Run(format.String(), func(t *testing.T) { runDataDriven(t, tdFile, format, parallelism) })
	}
}

func TestRewriter(t *testing.T) {
	testRewriterParallelism(t, false)
}

func TestWriterParallel(t *testing.T) {
	testWriterParallelism(t, true)
}

func TestRewriterParallel(t *testing.T) {
	testRewriterParallelism(t, true)
}

func runDataDriven(t *testing.T, file string, tableFormat TableFormat, parallelism bool) {
	var r *Reader
	defer func() {
		if r != nil {
			require.NoError(t, r.Close())
		}
	}()

	format := func(td *datadriven.TestData, m *WriterMetadata) string {
		var requestedProps []string
		for _, cmdArg := range td.CmdArgs {
			switch cmdArg.Key {
			case "props":
				requestedProps = cmdArg.Vals
			}
		}

		var b bytes.Buffer
		if m.HasPointKeys {
			fmt.Fprintf(&b, "point:    [%s-%s]\n", m.SmallestPoint, m.LargestPoint)
		}
		if m.HasRangeDelKeys {
			fmt.Fprintf(&b, "rangedel: [%s-%s]\n", m.SmallestRangeDel, m.LargestRangeDel)
		}
		if m.HasRangeKeys {
			fmt.Fprintf(&b, "rangekey: [%s-%s]\n", m.SmallestRangeKey, m.LargestRangeKey)
		}
		fmt.Fprintf(&b, "seqnums:  [%d-%d]\n", m.SmallestSeqNum, m.LargestSeqNum)

		if len(requestedProps) > 0 {
			props := strings.Split(r.Properties.String(), "\n")
			for _, requestedProp := range requestedProps {
				fmt.Fprintf(&b, "props %q:\n", requestedProp)
				for _, prop := range props {
					if strings.Contains(prop, requestedProp) {
						fmt.Fprintf(&b, "  %s\n", prop)
					}
				}
			}
		}

		return b.String()
	}

	datadriven.RunTest(t, file, func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "build":
			if r != nil {
				_ = r.Close()
				r = nil
			}
			var meta *WriterMetadata
			var err error
			meta, r, err = runBuildCmd(td, &WriterOptions{
				TableFormat: tableFormat,
				Parallelism: parallelism,
			}, 0)
			if err != nil {
				return err.Error()
			}
			return format(td, meta)

		case "build-raw":
			if r != nil {
				_ = r.Close()
				r = nil
			}
			var meta *WriterMetadata
			var err error
			meta, r, err = runBuildRawCmd(td, &WriterOptions{
				TableFormat: tableFormat,
			})
			if err != nil {
				return err.Error()
			}
			return format(td, meta)

		case "scan":
			origIter, err := r.NewIter(nil /* lower */, nil /* upper */)
			if err != nil {
				return err.Error()
			}
			iter := newIterAdapter(origIter)
			defer iter.Close()

			var buf bytes.Buffer
			for valid := iter.First(); valid; valid = iter.Next() {
				fmt.Fprintf(&buf, "%s:%s\n", iter.Key(), iter.Value())
			}
			return buf.String()

		case "get":
			var buf bytes.Buffer
			for _, k := range strings.Split(td.Input, "\n") {
				value, err := r.get([]byte(k))
				if err != nil {
					fmt.Fprintf(&buf, "get %s: %s\n", k, err.Error())
				} else {
					fmt.Fprintf(&buf, "%s\n", value)
				}
			}
			return buf.String()

		case "scan-range-del":
			iter, err := r.NewRawRangeDelIter()
			if err != nil {
				return err.Error()
			}
			if iter == nil {
				return ""
			}
			defer iter.Close()

			var buf bytes.Buffer
			for s := iter.First(); s != nil; s = iter.Next() {
				fmt.Fprintf(&buf, "%s\n", s)
			}
			return buf.String()

		case "scan-range-key":
			iter, err := r.NewRawRangeKeyIter()
			if err != nil {
				return err.Error()
			}
			if iter == nil {
				return ""
			}
			defer iter.Close()

			var buf bytes.Buffer
			for s := iter.First(); s != nil; s = iter.Next() {
				fmt.Fprintf(&buf, "%s\n", s)
			}
			return buf.String()

		case "layout":
			l, err := r.Layout()
			if err != nil {
				return err.Error()
			}
			verbose := false
			if len(td.CmdArgs) > 0 {
				if td.CmdArgs[0].Key == "verbose" {
					verbose = true
				} else {
					return "unknown arg"
				}
			}
			var buf bytes.Buffer
			l.Describe(&buf, verbose, r, nil)
			return buf.String()

		case "rewrite":
			var meta *WriterMetadata
			var err error
			meta, r, err = runRewriteCmd(td, r, WriterOptions{
				TableFormat: tableFormat,
			})
			if err != nil {
				return err.Error()
			}
			if err != nil {
				return err.Error()
			}
			return format(td, meta)

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestWriterWithValueBlocks(t *testing.T) {
	var r *Reader
	defer func() {
		if r != nil {
			require.NoError(t, r.Close())
		}
	}()
	formatVersion := TableFormatMax
	formatMeta := func(m *WriterMetadata) string {
		return fmt.Sprintf("value-blocks: num-values %d, num-blocks: %d, size: %d",
			m.Properties.NumValuesInValueBlocks, m.Properties.NumValueBlocks,
			m.Properties.ValueBlocksSize)
	}

	parallelism := false
	if rand.Intn(2) == 0 {
		parallelism = true
	}
	t.Logf("writer parallelism %t", parallelism)
	attributeExtractor := func(
		key []byte, keyPrefixLen int, value []byte) (base.ShortAttribute, error) {
		require.NotNil(t, key)
		require.Less(t, 0, keyPrefixLen)
		attribute := base.ShortAttribute(len(value) & '\x07')
		return attribute, nil
	}

	datadriven.RunTest(t, "testdata/writer_value_blocks", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "build":
			if r != nil {
				_ = r.Close()
				r = nil
			}
			var meta *WriterMetadata
			var err error
			var blockSize int
			if td.HasArg("block-size") {
				td.ScanArgs(t, "block-size", &blockSize)
			}
			var inPlaceValueBound UserKeyPrefixBound
			if td.HasArg("in-place-bound") {
				var l, u string
				td.ScanArgs(t, "in-place-bound", &l, &u)
				inPlaceValueBound.Lower = []byte(l)
				inPlaceValueBound.Upper = []byte(u)
			}
			meta, r, err = runBuildCmd(td, &WriterOptions{
				BlockSize:                 blockSize,
				Comparer:                  testkeys.Comparer,
				TableFormat:               formatVersion,
				Parallelism:               parallelism,
				RequiredInPlaceValueBound: inPlaceValueBound,
				ShortAttributeExtractor:   attributeExtractor,
			}, 0)
			if err != nil {
				return err.Error()
			}
			return formatMeta(meta)

		case "layout":
			l, err := r.Layout()
			if err != nil {
				return err.Error()
			}
			var buf bytes.Buffer
			l.Describe(&buf, true, r, func(key *base.InternalKey, value []byte) {
				fmt.Fprintf(&buf, "  %s:%s\n", key.String(), string(value))
			})
			return buf.String()

		case "scan-raw":
			// Raw scan does not fetch from value blocks.
			origIter, err := r.NewIter(nil /* lower */, nil /* upper */)
			if err != nil {
				return err.Error()
			}
			forceIgnoreValueBlocks := func(i *singleLevelIterator) {
				i.vbReader = nil
				i.data.lazyValueHandling.vbr = nil
				i.data.lazyValueHandling.hasValuePrefix = false
			}
			switch i := origIter.(type) {
			case *twoLevelIterator:
				forceIgnoreValueBlocks(&i.singleLevelIterator)
			case *singleLevelIterator:
				forceIgnoreValueBlocks(i)
			}
			iter := newIterAdapter(origIter)
			defer iter.Close()

			var buf bytes.Buffer
			for valid := iter.First(); valid; valid = iter.Next() {
				v := iter.Value()
				if iter.Key().Kind() == InternalKeyKindSet {
					prefix := valuePrefix(v[0])
					setWithSamePrefix := setHasSamePrefix(prefix)
					if isValueHandle(prefix) {
						attribute := getShortAttribute(prefix)
						vh := decodeValueHandle(v[1:])
						fmt.Fprintf(&buf, "%s:value-handle len %d block %d offset %d, att %d, same-pre %t\n",
							iter.Key(), vh.valueLen, vh.blockNum, vh.offsetInBlock, attribute, setWithSamePrefix)
					} else {
						fmt.Fprintf(&buf, "%s:in-place %s, same-pre %t\n", iter.Key(), v[1:], setWithSamePrefix)
					}
				} else {
					fmt.Fprintf(&buf, "%s:%s\n", iter.Key(), v)
				}
			}
			return buf.String()

		case "scan":
			origIter, err := r.NewIter(nil /* lower */, nil /* upper */)
			if err != nil {
				return err.Error()
			}
			iter := newIterAdapter(origIter)
			defer iter.Close()
			var buf bytes.Buffer
			for valid := iter.First(); valid; valid = iter.Next() {
				fmt.Fprintf(&buf, "%s:%s\n", iter.Key(), iter.Value())
			}
			return buf.String()

		case "scan-cloned-lazy-values":
			iter, err := r.NewIter(nil /* lower */, nil /* upper */)
			if err != nil {
				return err.Error()
			}
			var fetchers [100]base.LazyFetcher
			var values []base.LazyValue
			n := 0
			var b []byte
			for k, lv := iter.First(); k != nil; k, lv = iter.Next() {
				var lvClone base.LazyValue
				lvClone, b = lv.Clone(b, &fetchers[n])
				if lv.Fetcher != nil {
					_, callerOwned, err := lv.Value(nil)
					require.False(t, callerOwned)
					require.NoError(t, err)
				}
				n++
				values = append(values, lvClone)
			}
			require.NoError(t, iter.Error())
			iter.Close()
			var buf bytes.Buffer
			for i := range values {
				fmt.Fprintf(&buf, "%d", i)
				v, callerOwned, err := values[i].Value(nil)
				require.NoError(t, err)
				if values[i].Fetcher != nil {
					require.True(t, callerOwned)
					fmt.Fprintf(&buf, "(lazy: len %d, attr: %d): %s\n",
						values[i].Len(), values[i].Fetcher.Attribute.ShortAttribute, string(v))
					v2, callerOwned, err := values[i].Value(nil)
					require.NoError(t, err)
					require.True(t, callerOwned)
					require.Equal(t, &v[0], &v2[0])

				} else {
					require.False(t, callerOwned)
					fmt.Fprintf(&buf, "(in-place: len %d): %s\n", values[i].Len(), string(v))
				}
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func testBlockBufClear(t *testing.T, b1, b2 *blockBuf) {
	require.Equal(t, b1.tmp, b2.tmp)
}

func TestBlockBufClear(t *testing.T) {
	b1 := &blockBuf{}
	b1.tmp[0] = 1
	b1.compressedBuf = make([]byte, 1)
	b1.clear()
	testBlockBufClear(t, b1, &blockBuf{})
}

func TestClearDataBlockBuf(t *testing.T) {
	d := newDataBlockBuf(1, ChecksumTypeCRC32c)
	d.blockBuf.compressedBuf = make([]byte, 1)
	d.dataBlock.add(ikey("apple"), nil)
	d.dataBlock.add(ikey("banana"), nil)

	d.clear()
	testBlockCleared(t, &d.dataBlock, &blockWriter{})
	testBlockBufClear(t, &d.blockBuf, &blockBuf{})

	dataBlockBufPool.Put(d)
}

func TestClearIndexBlockBuf(t *testing.T) {
	i := newIndexBlockBuf(false)
	i.block.add(ikey("apple"), nil)
	i.block.add(ikey("banana"), nil)
	i.clear()

	testBlockCleared(t, &i.block, &blockWriter{})
	require.Equal(
		t, i.size.estimate, sizeEstimate{emptySize: emptyBlockSize},
	)
	indexBlockBufPool.Put(i)
}

func TestClearWriteTask(t *testing.T) {
	w := writeTaskPool.Get().(*writeTask)
	ch := make(chan bool, 1)
	w.compressionDone = ch
	w.buf = &dataBlockBuf{}
	w.flushableIndexBlock = &indexBlockBuf{}
	w.currIndexBlock = &indexBlockBuf{}
	w.indexEntrySep = ikey("apple")
	w.indexInflightSize = 1
	w.finishedIndexProps = []byte{'a', 'v'}

	w.clear()

	var nilDataBlockBuf *dataBlockBuf
	var nilIndexBlockBuf *indexBlockBuf
	// Channels should be the same(no new channel should be allocated)
	require.Equal(t, w.compressionDone, ch)
	require.Equal(t, w.buf, nilDataBlockBuf)
	require.Equal(t, w.flushableIndexBlock, nilIndexBlockBuf)
	require.Equal(t, w.currIndexBlock, nilIndexBlockBuf)
	require.Equal(t, w.indexEntrySep, base.InvalidInternalKey)
	require.Equal(t, w.indexInflightSize, 0)
	require.Equal(t, w.finishedIndexProps, []byte(nil))

	writeTaskPool.Put(w)
}

func TestDoubleClose(t *testing.T) {
	// There is code in Cockroach land which relies on Writer.Close being
	// idempotent. We should test this in Pebble, so that we don't cause
	// Cockroach test failures.
	f := &discardFile{}
	w := NewWriter(f, WriterOptions{
		BlockSize:   1,
		TableFormat: TableFormatPebblev1,
	})
	w.Set(ikey("a").UserKey, nil)
	w.Set(ikey("b").UserKey, nil)
	err := w.Close()
	require.NoError(t, err)
	err = w.Close()
	require.Equal(t, err, errWriterClosed)
}

func TestParallelWriterErrorProp(t *testing.T) {
	fs := vfs.NewMem()
	f, err := fs.Create("test")
	require.NoError(t, err)
	opts := WriterOptions{
		TableFormat: TableFormatPebblev1, BlockSize: 1, Parallelism: true,
	}

	w := NewWriter(objstorageprovider.NewFileWritable(f), opts)
	// Directly testing this, because it's difficult to get the Writer to
	// encounter an error, precisely when the writeQueue is doing block writes.
	w.coordination.writeQueue.err = errors.New("write queue write error")
	w.Set(ikey("a").UserKey, nil)
	w.Set(ikey("b").UserKey, nil)
	err = w.Close()
	require.Equal(t, err.Error(), "write queue write error")
}

func TestSizeEstimate(t *testing.T) {
	var sizeEstimate sizeEstimate
	datadriven.RunTest(t, "testdata/size_estimate",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "init":
				if len(td.CmdArgs) != 1 {
					return "init <empty size>"
				}
				emptySize, err := strconv.Atoi(td.CmdArgs[0].String())
				if err != nil {
					return "invalid empty size"
				}
				sizeEstimate.init(uint64(emptySize))
				return "success"
			case "clear":
				sizeEstimate.clear()
				return fmt.Sprintf("%d", sizeEstimate.size())
			case "size":
				return fmt.Sprintf("%d", sizeEstimate.size())
			case "add_inflight":
				if len(td.CmdArgs) != 1 {
					return "add_inflight <inflight size estimate>"
				}
				inflightSize, err := strconv.Atoi(td.CmdArgs[0].String())
				if err != nil {
					return "invalid inflight size"
				}
				sizeEstimate.addInflight(inflightSize)
				return fmt.Sprintf("%d", sizeEstimate.size())
			case "entry_written":
				if len(td.CmdArgs) != 2 {
					return "entry_written <new_total_size> <prev_inflight_size>"
				}
				newTotalSize, err := strconv.Atoi(td.CmdArgs[0].String())
				if err != nil {
					return "invalid inflight size"
				}
				inflightSize, err := strconv.Atoi(td.CmdArgs[1].String())
				if err != nil {
					return "invalid inflight size"
				}
				sizeEstimate.writtenWithTotal(uint64(newTotalSize), inflightSize)
				return fmt.Sprintf("%d", sizeEstimate.size())
			case "num_written_entries":
				return fmt.Sprintf("%d", sizeEstimate.numWrittenEntries)
			case "num_inflight_entries":
				return fmt.Sprintf("%d", sizeEstimate.numInflightEntries)
			case "num_entries":
				return fmt.Sprintf("%d", sizeEstimate.numWrittenEntries+sizeEstimate.numInflightEntries)
			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		})
}

func TestWriterClearCache(t *testing.T) {
	// Verify that Writer clears the cache of blocks that it writes.
	mem := vfs.NewMem()
	opts := ReaderOptions{
		Cache:    cache.New(64 << 20),
		Comparer: testkeys.Comparer,
	}
	defer opts.Cache.Unref()

	writerOpts := WriterOptions{
		Cache:       opts.Cache,
		Comparer:    testkeys.Comparer,
		TableFormat: TableFormatPebblev3,
	}
	cacheOpts := &cacheOpts{cacheID: 1, fileNum: base.FileNum(1).DiskFileNum()}
	invalidData := func() *cache.Value {
		invalid := []byte("invalid data")
		v := cache.Alloc(len(invalid))
		copy(v.Buf(), invalid)
		return v
	}

	build := func(name string) {
		f, err := mem.Create(name)
		require.NoError(t, err)

		w := NewWriter(objstorageprovider.NewFileWritable(f), writerOpts, cacheOpts)
		require.NoError(t, w.Set([]byte("hello"), []byte("world")))
		require.NoError(t, w.Set([]byte("hello@42"), []byte("world@42")))
		require.NoError(t, w.Set([]byte("hello@5"), []byte("world@5")))
		require.NoError(t, w.Close())
	}

	// Build the sstable a first time so that we can determine the locations of
	// all of the blocks.
	build("test")

	f, err := mem.Open("test")
	require.NoError(t, err)

	r, err := newReader(f, opts)
	require.NoError(t, err)

	layout, err := r.Layout()
	require.NoError(t, err)

	foreachBH := func(layout *Layout, f func(bh BlockHandle)) {
		for _, bh := range layout.Data {
			f(bh.BlockHandle)
		}
		for _, bh := range layout.Index {
			f(bh)
		}
		f(layout.TopIndex)
		f(layout.Filter)
		f(layout.RangeDel)
		for _, bh := range layout.ValueBlock {
			f(bh)
		}
		if layout.ValueIndex.Length != 0 {
			f(layout.ValueIndex)
		}
		f(layout.Properties)
		f(layout.MetaIndex)
	}

	// Poison the cache for each of the blocks.
	poison := func(bh BlockHandle) {
		opts.Cache.Set(cacheOpts.cacheID, cacheOpts.fileNum, bh.Offset, invalidData()).Release()
	}
	foreachBH(layout, poison)

	// Build the table a second time. This should clear the cache for the blocks
	// that are written.
	build("test")

	// Verify that the written blocks have been cleared from the cache.
	check := func(bh BlockHandle) {
		h := opts.Cache.Get(cacheOpts.cacheID, cacheOpts.fileNum, bh.Offset)
		if h.Get() != nil {
			t.Fatalf("%d: expected cache to be cleared, but found %q", bh.Offset, h.Get())
		}
	}
	foreachBH(layout, check)

	require.NoError(t, r.Close())
}

type discardFile struct {
	wrote int64
}

var _ objstorage.Writable = (*discardFile)(nil)

func (f *discardFile) Finish() error {
	return nil
}

func (f *discardFile) Abort() {}

func (f *discardFile) Write(p []byte) error {
	f.wrote += int64(len(p))
	return nil
}

type blockPropErrSite uint

const (
	errSiteAdd blockPropErrSite = iota
	errSiteFinishBlock
	errSiteFinishIndex
	errSiteFinishTable
	errSiteNone
)

type testBlockPropCollector struct {
	errSite blockPropErrSite
	err     error
}

func (c *testBlockPropCollector) Name() string { return "testBlockPropCollector" }

func (c *testBlockPropCollector) Add(_ InternalKey, _ []byte) error {
	if c.errSite == errSiteAdd {
		return c.err
	}
	return nil
}

func (c *testBlockPropCollector) FinishDataBlock(_ []byte) ([]byte, error) {
	if c.errSite == errSiteFinishBlock {
		return nil, c.err
	}
	return nil, nil
}

func (c *testBlockPropCollector) AddPrevDataBlockToIndexBlock() {}

func (c *testBlockPropCollector) FinishIndexBlock(_ []byte) ([]byte, error) {
	if c.errSite == errSiteFinishIndex {
		return nil, c.err
	}
	return nil, nil
}

func (c *testBlockPropCollector) FinishTable(_ []byte) ([]byte, error) {
	if c.errSite == errSiteFinishTable {
		return nil, c.err
	}
	return nil, nil
}

func TestWriterBlockPropertiesErrors(t *testing.T) {
	blockPropErr := errors.Newf("block property collector failed")
	testCases := []blockPropErrSite{
		errSiteAdd,
		errSiteFinishBlock,
		errSiteFinishIndex,
		errSiteFinishTable,
		errSiteNone,
	}

	var (
		k1 = base.MakeInternalKey([]byte("a"), 0, base.InternalKeyKindSet)
		v1 = []byte("apples")
		k2 = base.MakeInternalKey([]byte("b"), 0, base.InternalKeyKindSet)
		v2 = []byte("bananas")
		k3 = base.MakeInternalKey([]byte("c"), 0, base.InternalKeyKindSet)
		v3 = []byte("carrots")
	)

	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			fs := vfs.NewMem()
			f, err := fs.Create("test")
			require.NoError(t, err)

			w := NewWriter(objstorageprovider.NewFileWritable(f), WriterOptions{
				BlockSize: 1,
				BlockPropertyCollectors: []func() BlockPropertyCollector{
					func() BlockPropertyCollector {
						return &testBlockPropCollector{
							errSite: tc,
							err:     blockPropErr,
						}
					},
				},
				TableFormat: TableFormatPebblev1,
			})

			err = w.Add(k1, v1)
			switch tc {
			case errSiteAdd:
				require.Error(t, err)
				require.Equal(t, blockPropErr, err)
				return
			case errSiteFinishBlock:
				require.NoError(t, err)
				// Addition of a second key completes the first block.
				err = w.Add(k2, v2)
				require.Error(t, err)
				require.Equal(t, blockPropErr, err)
				return
			case errSiteFinishIndex:
				require.NoError(t, err)
				// Addition of a second key completes the first block.
				err = w.Add(k2, v2)
				require.NoError(t, err)
				// The index entry for the first block is added after the completion of
				// the second block, which is triggered by adding a third key.
				err = w.Add(k3, v3)
				require.Error(t, err)
				require.Equal(t, blockPropErr, err)
				return
			}

			err = w.Close()
			if tc == errSiteFinishTable {
				require.Error(t, err)
				require.Equal(t, blockPropErr, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestWriter_TableFormatCompatibility(t *testing.T) {
	testCases := []struct {
		name        string
		minFormat   TableFormat
		configureFn func(opts *WriterOptions)
		writeFn     func(w *Writer) error
	}{
		{
			name:      "block properties",
			minFormat: TableFormatPebblev1,
			configureFn: func(opts *WriterOptions) {
				opts.BlockPropertyCollectors = []func() BlockPropertyCollector{
					func() BlockPropertyCollector {
						return NewBlockIntervalCollector(
							"collector", &valueCharBlockIntervalCollector{charIdx: 0}, nil,
						)
					},
				}
			},
		},
		{
			name:      "range keys",
			minFormat: TableFormatPebblev2,
			writeFn: func(w *Writer) error {
				return w.RangeKeyDelete([]byte("a"), []byte("b"))
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			for tf := TableFormatLevelDB; tf <= TableFormatMax; tf++ {
				t.Run(tf.String(), func(t *testing.T) {
					fs := vfs.NewMem()
					f, err := fs.Create("sst")
					require.NoError(t, err)

					opts := WriterOptions{TableFormat: tf}
					if tc.configureFn != nil {
						tc.configureFn(&opts)
					}

					w := NewWriter(objstorageprovider.NewFileWritable(f), opts)
					if tc.writeFn != nil {
						err = tc.writeFn(w)
						require.NoError(t, err)
					}

					err = w.Close()
					if tf < tc.minFormat {
						require.Error(t, err)
					} else {
						require.NoError(t, err)
					}
				})
			}
		})
	}
}

// Tests for races, such as https://github.com/cockroachdb/cockroach/issues/77194,
// in the Writer.
func TestWriterRace(t *testing.T) {
	ks := testkeys.Alpha(5)
	ks = ks.EveryN(ks.Count() / 1_000)
	keys := make([][]byte, ks.Count())
	for ki := 0; ki < len(keys); ki++ {
		keys[ki] = testkeys.Key(ks, int64(ki))
	}
	readerOpts := ReaderOptions{
		Comparer: testkeys.Comparer,
		Filters:  map[string]base.FilterPolicy{},
	}

	var wg sync.WaitGroup
	for i := 0; i < 16; i++ {
		wg.Add(1)
		go func() {
			val := make([]byte, rand.Intn(1000))
			opts := WriterOptions{
				Comparer:    testkeys.Comparer,
				BlockSize:   rand.Intn(1 << 10),
				Compression: NoCompression,
			}
			defer wg.Done()
			f := &memFile{}
			w := NewWriter(f, opts)
			for ki := 0; ki < len(keys); ki++ {
				require.NoError(
					t,
					w.Add(base.MakeInternalKey(keys[ki], uint64(ki), InternalKeyKindSet), val),
				)
				require.Equal(
					t, w.dataBlockBuf.dataBlock.getCurKey().UserKey, keys[ki],
				)
			}
			require.NoError(t, w.Close())
			require.Equal(t, w.meta.LargestPoint.UserKey, keys[len(keys)-1])
			r, err := NewMemReader(f.Data(), readerOpts)
			require.NoError(t, err)
			defer r.Close()
			it, err := r.NewIter(nil, nil)
			require.NoError(t, err)
			defer it.Close()
			ki := 0
			for k, v := it.First(); k != nil; k, v = it.Next() {
				require.Equal(t, k.UserKey, keys[ki])
				vBytes, _, err := v.Value(nil)
				require.NoError(t, err)
				require.Equal(t, vBytes, val)
				ki++
			}
		}()
	}
	wg.Wait()
}

func TestObsoleteBlockPropertyCollectorFilter(t *testing.T) {
	var c obsoleteKeyBlockPropertyCollector
	var f obsoleteKeyBlockPropertyFilter
	require.Equal(t, c.Name(), f.Name())
	// Data block with 1 obsolete and 1 non-obsolete point.
	c.AddPoint(false)
	c.AddPoint(true)
	finishAndCheck := func(finishFunc func([]byte) ([]byte, error), expectedIntersects bool) {
		var buf [1]byte
		prop, err := finishFunc(buf[:0:1])
		require.NoError(t, err)
		expectedLength := 1
		if expectedIntersects {
			// The common case is encoded in 0 bytes
			expectedLength = 0
		}
		require.Equal(t, expectedLength, len(prop))
		// Confirm that the collector used the slice.
		require.Equal(t, unsafe.Pointer(&buf[0]), unsafe.Pointer(&prop[:1][0]))
		intersects, err := f.Intersects(prop)
		require.NoError(t, err)
		require.Equal(t, expectedIntersects, intersects)
	}
	finishAndCheck(c.FinishDataBlock, true)
	c.AddPrevDataBlockToIndexBlock()
	// Data block with only obsolete points.
	c.AddPoint(true)
	c.AddPoint(true)
	finishAndCheck(c.FinishDataBlock, false)
	c.AddPrevDataBlockToIndexBlock()
	// Index block has one obsolete block and one non-obsolete block.
	finishAndCheck(c.FinishIndexBlock, true)

	// Data block with obsolete point.
	c.AddPoint(true)
	finishAndCheck(c.FinishDataBlock, false)
	c.AddPrevDataBlockToIndexBlock()
	// Data block with obsolete point.
	c.AddPoint(true)
	finishAndCheck(c.FinishDataBlock, false)
	c.AddPrevDataBlockToIndexBlock()
	// Index block has only obsolete blocks.
	finishAndCheck(c.FinishIndexBlock, false)
	// Table is not obsolete.
	finishAndCheck(c.FinishTable, true)

	// Reset the collector state.
	c = obsoleteKeyBlockPropertyCollector{}
	// Table with only obsolete blocks.

	// Data block with obsolete point.
	c.AddPoint(true)
	finishAndCheck(c.FinishDataBlock, false)
	c.AddPrevDataBlockToIndexBlock()
	// Data block with obsolete point.
	c.AddPoint(true)
	finishAndCheck(c.FinishDataBlock, false)
	c.AddPrevDataBlockToIndexBlock()
	// Index block has only obsolete blocks.
	finishAndCheck(c.FinishIndexBlock, false)
	// Table is obsolete.
	finishAndCheck(c.FinishTable, false)
}

func BenchmarkWriter(b *testing.B) {
	keys := make([][]byte, 1e6)
	const keyLen = 24
	keySlab := make([]byte, keyLen*len(keys))
	for i := range keys {
		key := keySlab[i*keyLen : i*keyLen+keyLen]
		binary.BigEndian.PutUint64(key[:8], 123) // 16-byte shared prefix
		binary.BigEndian.PutUint64(key[8:16], 456)
		binary.BigEndian.PutUint64(key[16:], uint64(i))
		keys[i] = key
	}
	for _, format := range []TableFormat{TableFormatPebblev2, TableFormatPebblev3} {
		b.Run(fmt.Sprintf("format=%s", format.String()), func(b *testing.B) {
			runWriterBench(b, keys, nil, format)
		})
	}
}

func BenchmarkWriterWithVersions(b *testing.B) {
	keys := make([][]byte, 1e6)
	const keyLen = 26
	keySlab := make([]byte, keyLen*len(keys))
	for i := range keys {
		key := keySlab[i*keyLen : i*keyLen+keyLen]
		binary.BigEndian.PutUint64(key[:8], 123) // 16-byte shared prefix
		binary.BigEndian.PutUint64(key[8:16], 456)
		// @ is ascii value 64. Placing any byte with value 64 in these 8 bytes
		// will confuse testkeys.Comparer, when we pass it a key after splitting
		// of the suffix, since Comparer thinks this prefix is also a key with a
		// suffix. Hence, we print as a base 10 string.
		require.Equal(b, 8, copy(key[16:], fmt.Sprintf("%8d", i/2)))
		key[24] = '@'
		// Ascii representation of single digit integer 2-(i%2).
		key[25] = byte(48 + 2 - (i % 2))
		keys[i] = key
	}
	// TableFormatPebblev3 can sometimes be ~50% slower than
	// TableFormatPebblev2, since testkeys.Compare is expensive (mainly due to
	// split) and with v3 we have to call it twice for 50% of the Set calls,
	// since they have the same prefix as the preceding key.
	for _, format := range []TableFormat{TableFormatPebblev2, TableFormatPebblev3} {
		b.Run(fmt.Sprintf("format=%s", format.String()), func(b *testing.B) {
			runWriterBench(b, keys, testkeys.Comparer, format)
		})
	}
}

func runWriterBench(b *testing.B, keys [][]byte, comparer *base.Comparer, format TableFormat) {
	for _, bs := range []int{base.DefaultBlockSize, 32 << 10} {
		b.Run(fmt.Sprintf("block=%s", humanize.Bytes.Int64(int64(bs))), func(b *testing.B) {
			for _, filter := range []bool{true, false} {
				b.Run(fmt.Sprintf("filter=%t", filter), func(b *testing.B) {
					for _, comp := range []Compression{NoCompression, SnappyCompression, ZstdCompression} {
						b.Run(fmt.Sprintf("compression=%s", comp), func(b *testing.B) {
							opts := WriterOptions{
								BlockRestartInterval: 16,
								BlockSize:            bs,
								Comparer:             comparer,
								Compression:          comp,
								TableFormat:          format,
							}
							if filter {
								opts.FilterPolicy = bloom.FilterPolicy(10)
							}
							f := &discardFile{}
							b.ResetTimer()
							for i := 0; i < b.N; i++ {
								f.wrote = 0
								w := NewWriter(f, opts)

								for j := range keys {
									if err := w.Set(keys[j], keys[j]); err != nil {
										b.Fatal(err)
									}
								}
								if err := w.Close(); err != nil {
									b.Fatal(err)
								}
								b.SetBytes(int64(f.wrote))
							}
						})
					}
				})
			}
		})
	}
}

var test4bSuffixComparer = &base.Comparer{
	Compare:   base.DefaultComparer.Compare,
	Equal:     base.DefaultComparer.Equal,
	Separator: base.DefaultComparer.Separator,
	Successor: base.DefaultComparer.Successor,
	Split: func(key []byte) int {
		if len(key) > 4 {
			return len(key) - 4
		}
		return len(key)
	},
	Name: "comparer-split-4b-suffix",
}
