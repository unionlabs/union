// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"io"
	"math"
	"math/rand"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"
	"unicode"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/batchskl"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func TestBatch(t *testing.T) {
	testBatch(t, 0)
	testBatch(t, batchInitialSize)
}

func testBatch(t *testing.T, size int) {
	type testCase struct {
		kind       InternalKeyKind
		key, value string
		valueInt   uint32
	}

	verifyTestCases := func(b *Batch, testCases []testCase, indexedPointKindsOnly bool) {
		r := b.Reader()

		for _, tc := range testCases {
			if indexedPointKindsOnly && (tc.kind == InternalKeyKindLogData || tc.kind == InternalKeyKindIngestSST ||
				tc.kind == InternalKeyKindRangeKeyUnset || tc.kind == InternalKeyKindRangeKeySet ||
				tc.kind == InternalKeyKindRangeKeyDelete || tc.kind == InternalKeyKindRangeDelete) {
				continue
			}
			kind, k, v, ok, err := r.Next()
			if !ok {
				if err != nil {
					t.Fatal(err)
				}
				t.Fatalf("next returned !ok: test case = %v", tc)
			}
			key, value := string(k), string(v)
			if kind != tc.kind || key != tc.key || value != tc.value {
				t.Errorf("got (%d, %q, %q), want (%d, %q, %q)",
					kind, key, value, tc.kind, tc.key, tc.value)
			}
		}
		if len(r) != 0 {
			t.Errorf("reader was not exhausted: remaining bytes = %q", r)
		}
	}

	encodeFileNum := func(n base.FileNum) string {
		return string(binary.AppendUvarint(nil, uint64(n)))
	}
	decodeFileNum := func(d []byte) base.FileNum {
		val, n := binary.Uvarint(d)
		if n <= 0 {
			t.Fatalf("invalid filenum encoding")
		}
		return base.FileNum(val)
	}

	// RangeKeySet and RangeKeyUnset are untested here because they don't expose
	// deferred variants. This is a consequence of these keys' more complex
	// value encodings.
	testCases := []testCase{
		{InternalKeyKindIngestSST, encodeFileNum(1), "", 0},
		{InternalKeyKindSet, "roses", "red", 0},
		{InternalKeyKindSet, "violets", "blue", 0},
		{InternalKeyKindDelete, "roses", "", 0},
		{InternalKeyKindSingleDelete, "roses", "", 0},
		{InternalKeyKindSet, "", "", 0},
		{InternalKeyKindSet, "", "non-empty", 0},
		{InternalKeyKindDelete, "", "", 0},
		{InternalKeyKindSingleDelete, "", "", 0},
		{InternalKeyKindSet, "grass", "green", 0},
		{InternalKeyKindSet, "grass", "greener", 0},
		{InternalKeyKindSet, "eleventy", strings.Repeat("!!11!", 100), 0},
		{InternalKeyKindDelete, "nosuchkey", "", 0},
		{InternalKeyKindDeleteSized, "eleventy", string(binary.AppendUvarint([]byte(nil), 508)), 500},
		{InternalKeyKindSingleDelete, "nosuchkey", "", 0},
		{InternalKeyKindSet, "binarydata", "\x00", 0},
		{InternalKeyKindSet, "binarydata", "\xff", 0},
		{InternalKeyKindMerge, "merge", "mergedata", 0},
		{InternalKeyKindMerge, "merge", "", 0},
		{InternalKeyKindMerge, "", "", 0},
		{InternalKeyKindRangeDelete, "a", "b", 0},
		{InternalKeyKindRangeDelete, "", "", 0},
		{InternalKeyKindLogData, "logdata", "", 0},
		{InternalKeyKindLogData, "", "", 0},
		{InternalKeyKindRangeKeyDelete, "grass", "green", 0},
		{InternalKeyKindRangeKeyDelete, "", "", 0},
		{InternalKeyKindDeleteSized, "nosuchkey", string(binary.AppendUvarint([]byte(nil), 11)), 2},
	}
	b := newBatchWithSize(nil, size)
	for _, tc := range testCases {
		switch tc.kind {
		case InternalKeyKindSet:
			_ = b.Set([]byte(tc.key), []byte(tc.value), nil)
		case InternalKeyKindMerge:
			_ = b.Merge([]byte(tc.key), []byte(tc.value), nil)
		case InternalKeyKindDelete:
			_ = b.Delete([]byte(tc.key), nil)
		case InternalKeyKindDeleteSized:
			_ = b.DeleteSized([]byte(tc.key), tc.valueInt, nil)
		case InternalKeyKindSingleDelete:
			_ = b.SingleDelete([]byte(tc.key), nil)
		case InternalKeyKindRangeDelete:
			_ = b.DeleteRange([]byte(tc.key), []byte(tc.value), nil)
		case InternalKeyKindLogData:
			_ = b.LogData([]byte(tc.key), nil)
		case InternalKeyKindRangeKeyDelete:
			_ = b.RangeKeyDelete([]byte(tc.key), []byte(tc.value), nil)
		case InternalKeyKindIngestSST:
			b.ingestSST(decodeFileNum([]byte(tc.key)))
		}
	}
	verifyTestCases(b, testCases, false /* indexedKindsOnly */)

	b.Reset()
	// Run the same operations, this time using the Deferred variants of each
	// operation (eg. SetDeferred).
	for _, tc := range testCases {
		key := []byte(tc.key)
		value := []byte(tc.value)
		switch tc.kind {
		case InternalKeyKindSet:
			d := b.SetDeferred(len(key), len(value))
			copy(d.Key, key)
			copy(d.Value, value)
			d.Finish()
		case InternalKeyKindMerge:
			d := b.MergeDeferred(len(key), len(value))
			copy(d.Key, key)
			copy(d.Value, value)
			d.Finish()
		case InternalKeyKindDelete:
			d := b.DeleteDeferred(len(key))
			copy(d.Key, key)
			copy(d.Value, value)
			d.Finish()
		case InternalKeyKindDeleteSized:
			d := b.DeleteSizedDeferred(len(tc.key), tc.valueInt)
			copy(d.Key, key)
			d.Finish()
		case InternalKeyKindSingleDelete:
			d := b.SingleDeleteDeferred(len(key))
			copy(d.Key, key)
			copy(d.Value, value)
			d.Finish()
		case InternalKeyKindRangeDelete:
			d := b.DeleteRangeDeferred(len(key), len(value))
			copy(d.Key, key)
			copy(d.Value, value)
			d.Finish()
		case InternalKeyKindLogData:
			_ = b.LogData([]byte(tc.key), nil)
		case InternalKeyKindIngestSST:
			b.ingestSST(decodeFileNum([]byte(tc.key)))
		case InternalKeyKindRangeKeyDelete:
			d := b.RangeKeyDeleteDeferred(len(key), len(value))
			copy(d.Key, key)
			copy(d.Value, value)
			d.Finish()
		}
	}
	verifyTestCases(b, testCases, false /* indexedKindsOnly */)

	b.Reset()
	// Run the same operations, this time using AddInternalKey instead of the
	// Kind-specific methods.
	for _, tc := range testCases {
		if tc.kind == InternalKeyKindLogData || tc.kind == InternalKeyKindIngestSST ||
			tc.kind == InternalKeyKindRangeKeyUnset || tc.kind == InternalKeyKindRangeKeySet ||
			tc.kind == InternalKeyKindRangeKeyDelete || tc.kind == InternalKeyKindRangeDelete {
			continue
		}
		key := []byte(tc.key)
		value := []byte(tc.value)
		b.AddInternalKey(&InternalKey{UserKey: key, Trailer: base.MakeTrailer(0, tc.kind)}, value, nil)
	}
	verifyTestCases(b, testCases, true /* indexedKindsOnly */)
}

func TestBatchPreAlloc(t *testing.T) {
	var cases = []struct {
		size int
		exp  int
	}{
		{0, batchInitialSize},
		{batchInitialSize, batchInitialSize},
		{2 * batchInitialSize, 2 * batchInitialSize},
	}
	for _, c := range cases {
		b := newBatchWithSize(nil, c.size)
		b.Set([]byte{0x1}, []byte{0x2}, nil)
		if cap(b.data) != c.exp {
			t.Errorf("Unexpected memory space, required: %d, got: %d", c.exp, cap(b.data))
		}
	}
}

func TestBatchIngestSST(t *testing.T) {
	// Verify that Batch.IngestSST has the correct batch count and memtable
	// size.
	var b Batch
	b.ingestSST(1)
	require.Equal(t, int(b.Count()), 1)
	b.ingestSST(2)
	require.Equal(t, int(b.Count()), 2)
	require.Equal(t, int(b.memTableSize), 0)
	require.Equal(t, b.ingestedSSTBatch, true)
}

func TestBatchLen(t *testing.T) {
	var b Batch

	requireLenAndReprEq := func(size int) {
		require.Equal(t, size, b.Len())
		require.Equal(t, size, len(b.Repr()))
	}

	requireLenAndReprEq(batchHeaderLen)

	key := "test-key"
	value := "test-value"

	err := b.Set([]byte(key), []byte(value), nil)
	require.NoError(t, err)

	requireLenAndReprEq(33)

	err = b.Delete([]byte(key), nil)
	require.NoError(t, err)

	requireLenAndReprEq(43)
}

func TestBatchEmpty(t *testing.T) {
	testBatchEmpty(t, 0)
	testBatchEmpty(t, batchInitialSize)
}

func testBatchEmpty(t *testing.T, size int) {
	b := newBatchWithSize(nil, size)
	require.True(t, b.Empty())

	ops := []func(*Batch) error{
		func(b *Batch) error { return b.Set(nil, nil, nil) },
		func(b *Batch) error { return b.Merge(nil, nil, nil) },
		func(b *Batch) error { return b.Delete(nil, nil) },
		func(b *Batch) error { return b.DeleteRange(nil, nil, nil) },
		func(b *Batch) error { return b.LogData(nil, nil) },
		func(b *Batch) error { return b.RangeKeySet(nil, nil, nil, nil, nil) },
		func(b *Batch) error { return b.RangeKeyUnset(nil, nil, nil, nil) },
		func(b *Batch) error { return b.RangeKeyDelete(nil, nil, nil) },
	}

	for _, op := range ops {
		require.NoError(t, op(b))
		require.False(t, b.Empty())
		b.Reset()
		require.True(t, b.Empty())
		// Reset may choose to reuse b.data, so clear it to the zero value in
		// order to test the lazy initialization of b.data.
		b = newBatchWithSize(nil, size)
	}

	_ = b.Reader()
	require.True(t, b.Empty())
	b.Reset()
	require.True(t, b.Empty())
	b = newBatchWithSize(nil, size)

	require.Equal(t, uint64(0), b.SeqNum())
	require.True(t, b.Empty())
	b.Reset()
	require.True(t, b.Empty())
	b = &Batch{}

	d, err := Open("", &Options{
		FS: vfs.NewMem(),
	})
	require.NoError(t, err)
	defer d.Close()
	ib := newIndexedBatch(d, DefaultComparer)
	iter, _ := ib.NewIter(nil)
	require.False(t, iter.First())
	iter2, err := iter.Clone(CloneOptions{})
	require.NoError(t, err)
	require.NoError(t, iter.Close())
	_, err = iter.Clone(CloneOptions{})
	require.True(t, err != nil)
	require.False(t, iter2.First())
	require.NoError(t, iter2.Close())
}

func TestBatchApplyNoSyncWait(t *testing.T) {
	db, err := Open("", &Options{
		FS: vfs.NewMem(),
	})
	require.NoError(t, err)
	defer db.Close()
	var batches []*Batch
	options := &WriteOptions{Sync: true}
	for i := 0; i < 10000; i++ {
		b := db.NewBatch()
		str := fmt.Sprintf("a%d", i)
		require.NoError(t, b.Set([]byte(str), []byte(str), nil))
		require.NoError(t, db.ApplyNoSyncWait(b, options))
		// k-v pair is visible even if not yet synced.
		val, closer, err := db.Get([]byte(str))
		require.NoError(t, err)
		require.Equal(t, str, string(val))
		closer.Close()
		batches = append(batches, b)
	}
	for _, b := range batches {
		require.NoError(t, b.SyncWait())
		b.Close()
	}
}

func TestBatchReset(t *testing.T) {
	db, err := Open("", &Options{
		FS: vfs.NewMem(),
	})
	require.NoError(t, err)
	defer db.Close()
	key := "test-key"
	value := "test-value"
	b := db.NewBatch()
	require.NoError(t, b.Set([]byte(key), []byte(value), nil))
	dd := b.DeleteRangeDeferred(len(key), len(value))
	copy(dd.Key, key)
	copy(dd.Value, value)
	dd.Finish()

	require.NoError(t, b.RangeKeySet([]byte(key), []byte(value), []byte(value), []byte(value), nil))

	b.setSeqNum(100)
	b.applied.Store(true)
	b.commitErr = errors.New("test-error")
	b.commit.Add(1)
	b.fsyncWait.Add(1)
	require.Equal(t, uint32(3), b.Count())
	require.Equal(t, uint64(1), b.countRangeDels)
	require.Equal(t, uint64(1), b.countRangeKeys)
	require.True(t, len(b.data) > 0)
	require.True(t, b.SeqNum() > 0)
	require.True(t, b.memTableSize > 0)
	require.NotEqual(t, b.deferredOp, DeferredBatchOp{})
	// At this point b.data has not been modified since the db.NewBatch() and is
	// either nil or contains a byte slice of length batchHeaderLen, with a 0
	// seqnum encoded in data[0:8] and an arbitrary count encoded in data[8:12].
	// The following commented code will often fail.
	// 	count := binary.LittleEndian.Uint32(b.countData())
	//  if count != 0 && count != 3 {
	//  	t.Fatalf("count: %d", count)
	//  }
	// If we simply called b.Reset now and later used b.data to initialize
	// expected, the count in expected will also be arbitrary. So we fix the
	// count in b.data now by calling b.Repr(). This call isn't essential, since
	// we will call b.Repr() again, and just shows that it fixes the count in
	// b.data.
	_ = b.Repr()
	require.Equal(t, uint32(3), binary.LittleEndian.Uint32(b.countData()))

	b.Reset()
	require.Equal(t, db, b.db)
	require.Equal(t, false, b.applied.Load())
	require.Nil(t, b.commitErr)
	require.Equal(t, uint32(0), b.Count())
	require.Equal(t, uint64(0), b.countRangeDels)
	require.Equal(t, uint64(0), b.countRangeKeys)
	require.Equal(t, batchHeaderLen, len(b.data))
	require.Equal(t, uint64(0), b.SeqNum())
	require.Equal(t, uint64(0), b.memTableSize)
	require.Equal(t, FormatMajorVersion(0x00), b.minimumFormatMajorVersion)
	require.Equal(t, b.deferredOp, DeferredBatchOp{})
	_ = b.Repr()

	var expected Batch
	require.NoError(t, expected.SetRepr(b.data))
	expected.db = db
	require.Equal(t, &expected, b)

	// Reset batch can be used to write and commit a new record.
	b.Set([]byte(key), []byte(value), nil)
	require.NoError(t, db.Apply(b, nil))
	v, closer, err := db.Get([]byte(key))
	require.NoError(t, err)
	defer closer.Close()
	require.Equal(t, v, []byte(value))
}

func TestIndexedBatchReset(t *testing.T) {
	indexCount := func(sl *batchskl.Skiplist) int {
		count := 0
		iter := sl.NewIter(nil, nil)
		defer iter.Close()
		for iter.First(); iter.Valid(); iter.Next() {
			count++
		}
		return count
	}
	db, err := Open("", &Options{
		FS: vfs.NewMem(),
	})
	require.NoError(t, err)
	defer db.Close()
	b := newIndexedBatch(db, DefaultComparer)
	start := "start-key"
	end := "end-key"
	key := "test-key"
	value := "test-value"
	b.DeleteRange([]byte(start), []byte(end), nil)
	b.Set([]byte(key), []byte(value), nil)
	require.NoError(t, b.
		RangeKeySet([]byte(start), []byte(end), []byte("suffix"), []byte(value), nil))
	require.NotNil(t, b.rangeKeyIndex)
	require.NotNil(t, b.rangeDelIndex)
	require.NotNil(t, b.index)
	require.Equal(t, 1, indexCount(b.index))

	b.Reset()
	require.NotNil(t, b.cmp)
	require.NotNil(t, b.formatKey)
	require.NotNil(t, b.abbreviatedKey)
	require.NotNil(t, b.index)
	require.Nil(t, b.rangeDelIndex)
	require.Nil(t, b.rangeKeyIndex)

	count := func(ib *Batch) int {
		iter, _ := ib.NewIter(nil)
		defer iter.Close()
		iter2, err := iter.Clone(CloneOptions{})
		require.NoError(t, err)
		defer iter2.Close()
		var count [2]int
		for i, it := range []*Iterator{iter, iter2} {
			for it.First(); it.Valid(); it.Next() {
				count[i]++
			}
		}
		require.Equal(t, count[0], count[1])
		return count[0]
	}
	contains := func(ib *Batch, key, value string) bool {
		iter, _ := ib.NewIter(nil)
		defer iter.Close()
		iter2, err := iter.Clone(CloneOptions{})
		require.NoError(t, err)
		defer iter2.Close()
		var found [2]bool
		for i, it := range []*Iterator{iter, iter2} {
			for it.First(); it.Valid(); it.Next() {
				if string(it.Key()) == key &&
					string(it.Value()) == value {
					found[i] = true
				}
			}
		}
		require.Equal(t, found[0], found[1])
		return found[0]
	}
	// Set a key and check whether the key-value pair is visible.
	b.Set([]byte(key), []byte(value), nil)
	require.Equal(t, 1, indexCount(b.index))
	require.Equal(t, 1, count(b))
	require.True(t, contains(b, key, value))

	// Use range delete to delete the above inserted key-value pair.
	b.DeleteRange([]byte(key), []byte(value), nil)
	require.NotNil(t, b.rangeDelIndex)
	require.Equal(t, 1, indexCount(b.rangeDelIndex))
	require.Equal(t, 0, count(b))
	require.False(t, contains(b, key, value))
}

// TestIndexedBatchMutation tests mutating an indexed batch with an open
// iterator.
func TestIndexedBatchMutation(t *testing.T) {
	opts := &Options{
		Comparer:           testkeys.Comparer,
		FS:                 vfs.NewMem(),
		FormatMajorVersion: internalFormatNewest,
	}
	d, err := Open("", opts)
	require.NoError(t, err)
	defer func() { d.Close() }()

	b := newIndexedBatch(d, DefaultComparer)
	iters := map[string]*Iterator{}
	defer func() {
		for _, iter := range iters {
			require.NoError(t, iter.Close())
		}
	}()

	datadriven.RunTest(t, "testdata/indexed_batch_mutation", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "batch":
			writeBatch := newBatch(d)
			if err := runBatchDefineCmd(td, writeBatch); err != nil {
				return err.Error()
			}
			if err := writeBatch.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "new-batch-iter":
			name := td.CmdArgs[0].String()
			iters[name], _ = b.NewIter(&IterOptions{
				KeyTypes: IterKeyTypePointsAndRanges,
			})
			return ""
		case "new-db-iter":
			name := td.CmdArgs[0].String()
			iters[name], _ = d.NewIter(&IterOptions{
				KeyTypes: IterKeyTypePointsAndRanges,
			})
			return ""
		case "new-batch":
			if b != nil {
				require.NoError(t, b.Close())
			}
			b = newIndexedBatch(d, opts.Comparer)
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			return ""
		case "flush":
			require.NoError(t, d.Flush())
			return ""
		case "iter":
			var iter string
			td.ScanArgs(t, "iter", &iter)
			return runIterCmd(td, iters[iter], false /* closeIter */)
		case "mutate":
			mut := newBatch(d)
			if err := runBatchDefineCmd(td, mut); err != nil {
				return err.Error()
			}
			if err := b.Apply(mut, nil); err != nil {
				return err.Error()
			}
			return ""
		case "clone":
			var from, to string
			var refreshBatchView bool
			td.ScanArgs(t, "from", &from)
			td.ScanArgs(t, "to", &to)
			td.ScanArgs(t, "refresh-batch", &refreshBatchView)
			var err error
			iters[to], err = iters[from].Clone(CloneOptions{RefreshBatchView: refreshBatchView})
			if err != nil {
				return err.Error()
			}
			return ""
		case "reset":
			for key, iter := range iters {
				if err := iter.Close(); err != nil {
					return err.Error()
				}
				delete(iters, key)
			}
			if d != nil {
				if err := d.Close(); err != nil {
					return err.Error()
				}
			}
			opts.FS = vfs.NewMem()
			d, err = Open("", opts)
			require.NoError(t, err)
			return ""
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func TestIndexedBatch_GlobalVisibility(t *testing.T) {
	opts := &Options{
		FS:                 vfs.NewMem(),
		FormatMajorVersion: internalFormatNewest,
		Comparer:           testkeys.Comparer,
	}
	d, err := Open("", opts)
	require.NoError(t, err)
	defer d.Close()

	require.NoError(t, d.Set([]byte("foo"), []byte("foo"), nil))

	// Create an iterator over an empty indexed batch.
	b := newIndexedBatch(d, DefaultComparer)
	iterOpts := IterOptions{KeyTypes: IterKeyTypePointsAndRanges}
	iter, _ := b.NewIter(&iterOpts)
	defer iter.Close()

	// Mutate the database's committed state.
	mut := newBatch(d)
	require.NoError(t, mut.Set([]byte("bar"), []byte("bar"), nil))
	require.NoError(t, mut.DeleteRange([]byte("e"), []byte("g"), nil))
	require.NoError(t, mut.RangeKeySet([]byte("a"), []byte("c"), []byte("@1"), []byte("v"), nil))
	require.NoError(t, mut.Commit(nil))

	scanIter := func() string {
		var buf bytes.Buffer
		for valid := iter.First(); valid; valid = iter.Next() {
			fmt.Fprintf(&buf, "%s: (", iter.Key())
			hasPoint, hasRange := iter.HasPointAndRange()
			if hasPoint {
				fmt.Fprintf(&buf, "%s,", iter.Value())
			} else {
				fmt.Fprintf(&buf, ".,")
			}
			if hasRange {
				start, end := iter.RangeBounds()
				fmt.Fprintf(&buf, "[%s-%s)", start, end)
				writeRangeKeys(&buf, iter)
			} else {
				fmt.Fprintf(&buf, ".")
			}
			fmt.Fprintln(&buf, ")")
		}
		return strings.TrimSpace(buf.String())
	}
	// Scanning the iterator should only see the point key written before the
	// iterator was constructed.
	require.Equal(t, `foo: (foo,.)`, scanIter())

	// After calling SetOptions, the iterator should still only see the point
	// key written before the iterator was constructed. SetOptions refreshes the
	// iterator's view of its own indexed batch, but not committed state.
	iter.SetOptions(&iterOpts)
	require.Equal(t, `foo: (foo,.)`, scanIter())
}

func TestFlushableBatchReset(t *testing.T) {
	var b Batch
	var err error
	b.flushable, err = newFlushableBatch(&b, DefaultComparer)
	require.NoError(t, err)

	b.Reset()
	require.Nil(t, b.flushable)
}

func TestBatchIncrement(t *testing.T) {
	testCases := []uint32{
		0x00000000,
		0x00000001,
		0x00000002,
		0x0000007f,
		0x00000080,
		0x000000fe,
		0x000000ff,
		0x00000100,
		0x00000101,
		0x000001ff,
		0x00000200,
		0x00000fff,
		0x00001234,
		0x0000fffe,
		0x0000ffff,
		0x00010000,
		0x00010001,
		0x000100fe,
		0x000100ff,
		0x00020100,
		0x03fffffe,
		0x03ffffff,
		0x04000000,
		0x04000001,
		0x7fffffff,
		0xfffffffe,
	}
	for _, tc := range testCases {
		var buf [batchHeaderLen]byte
		binary.LittleEndian.PutUint32(buf[8:12], tc)
		var b Batch
		b.SetRepr(buf[:])
		b.count++
		got := binary.LittleEndian.Uint32(b.Repr()[8:12])
		want := tc + 1
		if got != want {
			t.Errorf("input=%d: got %d, want %d", tc, got, want)
		}
		_, count := ReadBatch(b.Repr())
		if got != want {
			t.Errorf("input=%d: got %d, want %d", tc, count, want)
		}
	}

	err := func() (err error) {
		defer func() {
			if v := recover(); v != nil {
				if verr, ok := v.(error); ok {
					err = verr
				}
			}
		}()
		var buf [batchHeaderLen]byte
		binary.LittleEndian.PutUint32(buf[8:12], 0xffffffff)
		var b Batch
		b.SetRepr(buf[:])
		b.count++
		b.Repr()
		return nil
	}()
	if err != ErrInvalidBatch {
		t.Fatalf("expected %v, but found %v", ErrInvalidBatch, err)
	}
}

func TestBatchOpDoesIncrement(t *testing.T) {
	var b Batch
	key := []byte("foo")
	value := []byte("bar")

	if b.Count() != 0 {
		t.Fatalf("new batch has a nonzero count: %d", b.Count())
	}

	// Should increment count by 1
	_ = b.Set(key, value, nil)
	if b.Count() != 1 {
		t.Fatalf("expected count: %d, got %d", 1, b.Count())
	}

	var b2 Batch
	// Should increment count by 1 each
	_ = b2.Set(key, value, nil)
	_ = b2.Delete(key, nil)
	if b2.Count() != 2 {
		t.Fatalf("expected count: %d, got %d", 2, b2.Count())
	}

	// Should increment count by b2.count()
	_ = b.Apply(&b2, nil)
	if b.Count() != 3 {
		t.Fatalf("expected count: %d, got %d", 3, b.Count())
	}

	// Should increment count by 1
	_ = b.Merge(key, value, nil)
	if b.Count() != 4 {
		t.Fatalf("expected count: %d, got %d", 4, b.Count())
	}

	// Should NOT increment count.
	_ = b.LogData([]byte("foobarbaz"), nil)
	if b.Count() != 4 {
		t.Fatalf("expected count: %d, got %d", 4, b.Count())
	}
}

func TestBatchGet(t *testing.T) {
	testCases := []struct {
		method       string
		memTableSize uint64
	}{
		{"build", 64 << 20},
		{"build", 2 << 10},
		{"apply", 64 << 20},
	}

	for _, c := range testCases {
		t.Run(fmt.Sprintf("%s,mem=%d", c.method, c.memTableSize), func(t *testing.T) {
			d, err := Open("", &Options{
				FS:           vfs.NewMem(),
				MemTableSize: c.memTableSize,
			})
			if err != nil {
				t.Fatalf("Open: %v", err)
			}
			defer d.Close()
			var b *Batch

			datadriven.RunTest(t, "testdata/batch_get", func(t *testing.T, td *datadriven.TestData) string {
				switch td.Cmd {
				case "define":
					switch c.method {
					case "build":
						b = d.NewIndexedBatch()
					case "apply":
						b = d.NewBatch()
					}

					if err := runBatchDefineCmd(td, b); err != nil {
						return err.Error()
					}

					switch c.method {
					case "apply":
						tmp := d.NewIndexedBatch()
						tmp.Apply(b, nil)
						b = tmp
					}
					return ""

				case "commit":
					if err := b.Commit(nil); err != nil {
						return err.Error()
					}
					b = nil
					return ""

				case "get":
					if len(td.CmdArgs) != 1 {
						return fmt.Sprintf("%s expects 1 argument", td.Cmd)
					}
					v, closer, err := b.Get([]byte(td.CmdArgs[0].String()))
					if err != nil {
						return err.Error()
					}
					defer closer.Close()
					return string(v)

				default:
					return fmt.Sprintf("unknown command: %s", td.Cmd)
				}
			})
		})
	}
}

func TestBatchIter(t *testing.T) {
	var b *Batch

	for _, method := range []string{"build", "apply"} {
		for _, testdata := range []string{
			"testdata/internal_iter_next", "testdata/internal_iter_bounds"} {
			t.Run(method, func(t *testing.T) {
				datadriven.RunTest(t, testdata, func(t *testing.T, d *datadriven.TestData) string {
					switch d.Cmd {
					case "define":
						switch method {
						case "build":
							b = newIndexedBatch(nil, DefaultComparer)
						case "apply":
							b = newBatch(nil)
						}

						for _, key := range strings.Split(d.Input, "\n") {
							j := strings.Index(key, ":")
							ikey := base.ParseInternalKey(key[:j])
							value := []byte(key[j+1:])
							b.Set(ikey.UserKey, value, nil)
						}

						switch method {
						case "apply":
							tmp := newIndexedBatch(nil, DefaultComparer)
							tmp.Apply(b, nil)
							b = tmp
						}
						return ""

					case "iter":
						var options IterOptions
						for _, arg := range d.CmdArgs {
							switch arg.Key {
							case "lower":
								if len(arg.Vals) != 1 {
									return fmt.Sprintf(
										"%s expects at most 1 value for lower", d.Cmd)
								}
								options.LowerBound = []byte(arg.Vals[0])
							case "upper":
								if len(arg.Vals) != 1 {
									return fmt.Sprintf(
										"%s expects at most 1 value for upper", d.Cmd)
								}
								options.UpperBound = []byte(arg.Vals[0])
							default:
								return fmt.Sprintf("unknown arg: %s", arg.Key)
							}
						}
						iter := b.newInternalIter(&options)
						defer iter.Close()
						return runInternalIterCmd(t, d, iter)

					default:
						return fmt.Sprintf("unknown command: %s", d.Cmd)
					}
				})
			})
		}
	}
}

func TestBatchRangeOps(t *testing.T) {
	var b *Batch

	datadriven.RunTest(t, "testdata/batch_range_ops", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "clear":
			b = nil
			return ""

		case "apply":
			if b == nil {
				b = newIndexedBatch(nil, DefaultComparer)
			}
			t := newBatch(nil)
			if err := runBatchDefineCmd(td, t); err != nil {
				return err.Error()
			}
			if err := b.Apply(t, nil); err != nil {
				return err.Error()
			}
			return ""

		case "define":
			if b == nil {
				b = newIndexedBatch(nil, DefaultComparer)
			}
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			return ""

		case "scan":
			if len(td.CmdArgs) > 1 {
				return fmt.Sprintf("%s expects at most 1 argument", td.Cmd)
			}
			var fragmentIter keyspan.FragmentIterator
			var internalIter base.InternalIterator
			switch {
			case td.HasArg("range-del"):
				fragmentIter = b.newRangeDelIter(nil, math.MaxUint64)
				defer fragmentIter.Close()
			case td.HasArg("range-key"):
				fragmentIter = b.newRangeKeyIter(nil, math.MaxUint64)
				defer fragmentIter.Close()
			default:
				internalIter = b.newInternalIter(nil)
				defer internalIter.Close()
			}

			var buf bytes.Buffer
			if fragmentIter != nil {
				for s := fragmentIter.First(); s != nil; s = fragmentIter.Next() {
					for i := range s.Keys {
						s.Keys[i].Trailer = base.MakeTrailer(
							s.Keys[i].SeqNum()&^base.InternalKeySeqNumBatch,
							s.Keys[i].Kind(),
						)
					}
					fmt.Fprintln(&buf, s)
				}
			} else {
				for k, v := internalIter.First(); k != nil; k, v = internalIter.Next() {
					k.SetSeqNum(k.SeqNum() &^ InternalKeySeqNumBatch)
					fmt.Fprintf(&buf, "%s:%s\n", k, v.InPlaceValue())
				}
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestBatchTooLarge(t *testing.T) {
	var b Batch
	var result interface{}
	func() {
		defer func() {
			if r := recover(); r != nil {
				result = r
			}
		}()
		b.grow(maxBatchSize)
	}()
	require.EqualValues(t, ErrBatchTooLarge, result)
}

func TestFlushableBatchIter(t *testing.T) {
	var b *flushableBatch
	datadriven.RunTest(t, "testdata/internal_iter_next", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			batch := newBatch(nil)
			for _, key := range strings.Split(d.Input, "\n") {
				j := strings.Index(key, ":")
				ikey := base.ParseInternalKey(key[:j])
				value := []byte(fmt.Sprint(ikey.SeqNum()))
				batch.Set(ikey.UserKey, value, nil)
			}
			var err error
			b, err = newFlushableBatch(batch, DefaultComparer)
			require.NoError(t, err)
			return ""

		case "iter":
			iter := b.newIter(nil)
			defer iter.Close()
			return runInternalIterCmd(t, d, iter)

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestFlushableBatch(t *testing.T) {
	var b *flushableBatch
	datadriven.RunTest(t, "testdata/flushable_batch", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			batch := newBatch(nil)
			for _, key := range strings.Split(d.Input, "\n") {
				j := strings.Index(key, ":")
				ikey := base.ParseInternalKey(key[:j])
				value := []byte(fmt.Sprint(ikey.SeqNum()))
				switch ikey.Kind() {
				case InternalKeyKindDelete:
					require.NoError(t, batch.Delete(ikey.UserKey, nil))
				case InternalKeyKindSet:
					require.NoError(t, batch.Set(ikey.UserKey, value, nil))
				case InternalKeyKindMerge:
					require.NoError(t, batch.Merge(ikey.UserKey, value, nil))
				case InternalKeyKindRangeDelete:
					require.NoError(t, batch.DeleteRange(ikey.UserKey, value, nil))
				case InternalKeyKindRangeKeyDelete:
					require.NoError(t, batch.RangeKeyDelete(ikey.UserKey, value, nil))
				case InternalKeyKindRangeKeySet:
					require.NoError(t, batch.RangeKeySet(ikey.UserKey, value, value, value, nil))
				case InternalKeyKindRangeKeyUnset:
					require.NoError(t, batch.RangeKeyUnset(ikey.UserKey, value, value, nil))
				}
			}
			var err error
			b, err = newFlushableBatch(batch, DefaultComparer)
			require.NoError(t, err)
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

			iter := b.newIter(&opts)
			defer iter.Close()
			return runInternalIterCmd(t, d, iter)

		case "dump":
			if len(d.CmdArgs) != 1 || len(d.CmdArgs[0].Vals) != 1 || d.CmdArgs[0].Key != "seq" {
				return "dump seq=<value>\n"
			}
			seqNum, err := strconv.Atoi(d.CmdArgs[0].Vals[0])
			if err != nil {
				return err.Error()
			}
			b.setSeqNum(uint64(seqNum))

			var buf bytes.Buffer

			iter := newInternalIterAdapter(b.newIter(nil))
			for valid := iter.First(); valid; valid = iter.Next() {
				fmt.Fprintf(&buf, "%s:%s\n", iter.Key(), iter.Value())
			}
			iter.Close()

			if rangeDelIter := b.newRangeDelIter(nil); rangeDelIter != nil {
				scanKeyspanIterator(&buf, rangeDelIter)
				rangeDelIter.Close()
			}
			if rangeKeyIter := b.newRangeKeyIter(nil); rangeKeyIter != nil {
				scanKeyspanIterator(&buf, rangeKeyIter)
				rangeKeyIter.Close()
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestFlushableBatchDeleteRange(t *testing.T) {
	var fb *flushableBatch
	var input string

	datadriven.RunTest(t, "testdata/delete_range", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "clear":
			input = ""
			return ""

		case "define":
			b := newBatch(nil)
			// NB: We can't actually add to the flushable batch as we can to a
			// memtable (which shares the "testdata/delete_range" data), so we fake
			// it by concatenating the input and rebuilding the flushable batch from
			// scratch.
			input += "\n" + td.Input
			td.Input = input
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			var err error
			fb, err = newFlushableBatch(b, DefaultComparer)
			require.NoError(t, err)
			return ""

		case "scan":
			var buf bytes.Buffer
			if td.HasArg("range-del") {
				fi := fb.newRangeDelIter(nil)
				defer fi.Close()
				scanKeyspanIterator(&buf, fi)
			} else {
				ii := fb.newIter(nil)
				defer ii.Close()
				scanInternalIter(&buf, ii)
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func scanInternalIter(w io.Writer, ii internalIterator) {
	for k, v := ii.First(); k != nil; k, v = ii.Next() {
		fmt.Fprintf(w, "%s:%s\n", k, v.InPlaceValue())
	}
}

func scanKeyspanIterator(w io.Writer, ki keyspan.FragmentIterator) {
	for s := ki.First(); s != nil; s = ki.Next() {
		fmt.Fprintln(w, s)
	}
}

func TestFlushableBatchBytesIterated(t *testing.T) {
	batch := newBatch(nil)
	for j := 0; j < 1000; j++ {
		key := make([]byte, 8+j%3)
		value := make([]byte, 7+j%5)
		batch.Set(key, value, nil)

		fb, err := newFlushableBatch(batch, DefaultComparer)
		require.NoError(t, err)

		var bytesIterated uint64
		it := fb.newFlushIter(nil, &bytesIterated)

		var prevIterated uint64
		for key, _ := it.First(); key != nil; key, _ = it.Next() {
			if bytesIterated < prevIterated {
				t.Fatalf("bytesIterated moved backward: %d < %d", bytesIterated, prevIterated)
			}
			prevIterated = bytesIterated
		}

		expected := fb.inuseBytes()
		if bytesIterated != expected {
			t.Fatalf("bytesIterated: got %d, want %d", bytesIterated, expected)
		}
	}
}

func TestEmptyFlushableBatch(t *testing.T) {
	// Verify that we can create a flushable batch on an empty batch.
	fb, err := newFlushableBatch(newBatch(nil), DefaultComparer)
	require.NoError(t, err)
	it := newInternalIterAdapter(fb.newIter(nil))
	require.False(t, it.First())
}

func TestBatchCommitStats(t *testing.T) {
	testFunc := func() error {
		db, err := Open("", &Options{
			FS: vfs.NewMem(),
		})
		require.NoError(t, err)
		defer db.Close()
		b := db.NewBatch()
		defer b.Close()
		stats := b.CommitStats()
		require.Equal(t, BatchCommitStats{}, stats)

		// The stall code peers into the internals, instead of adding general
		// purpose hooks, to avoid changing production code. We can revisit this
		// choice if it becomes hard to maintain.

		// Commit semaphore stall funcs.
		var unstallCommitSemaphore func()
		stallCommitSemaphore := func() {
			commitPipeline := db.commit
			commitSemaphoreReserved := 0
			done := false
			for !done {
				select {
				case commitPipeline.commitQueueSem <- struct{}{}:
					commitSemaphoreReserved++
				default:
					done = true
				}
				if done {
					break
				}
			}
			unstallCommitSemaphore = func() {
				for i := 0; i < commitSemaphoreReserved; i++ {
					<-commitPipeline.commitQueueSem
				}
			}
		}

		// Memstable stall funcs.
		var unstallMemtable func()
		stallMemtable := func() {
			db.mu.Lock()
			defer db.mu.Unlock()
			prev := db.opts.MemTableStopWritesThreshold
			db.opts.MemTableStopWritesThreshold = 0
			unstallMemtable = func() {
				db.mu.Lock()
				defer db.mu.Unlock()
				db.opts.MemTableStopWritesThreshold = prev
				db.mu.compact.cond.Broadcast()
			}
		}

		// L0 read-amp stall funcs.
		var unstallL0ReadAmp func()
		stallL0ReadAmp := func() {
			db.mu.Lock()
			defer db.mu.Unlock()
			prev := db.opts.L0StopWritesThreshold
			db.opts.L0StopWritesThreshold = 0
			unstallL0ReadAmp = func() {
				db.mu.Lock()
				defer db.mu.Unlock()
				db.opts.L0StopWritesThreshold = prev
				db.mu.compact.cond.Broadcast()
			}
		}

		// Commit wait stall funcs.
		var unstallCommitWait func()
		stallCommitWait := func() {
			b.commit.Add(1)
			unstallCommitWait = func() {
				b.commit.Done()
			}
		}

		// Stall everything.
		stallCommitSemaphore()
		stallMemtable()
		stallL0ReadAmp()
		stallCommitWait()

		// Exceed initialMemTableSize -- this is needed to make stallMemtable work.
		require.NoError(t, b.Set(make([]byte, initialMemTableSize), nil, nil))

		var commitWG sync.WaitGroup
		commitWG.Add(1)
		go func() {
			require.NoError(t, db.Apply(b, &WriteOptions{Sync: true}))
			commitWG.Done()
		}()
		// Unstall things in the order that the stalls will happen.
		sleepDuration := 10 * time.Millisecond
		time.Sleep(sleepDuration)
		unstallCommitSemaphore()
		time.Sleep(sleepDuration)
		unstallMemtable()
		time.Sleep(sleepDuration)
		unstallL0ReadAmp()
		time.Sleep(sleepDuration)
		unstallCommitWait()

		// Wait for Apply to return.
		commitWG.Wait()
		stats = b.CommitStats()
		expectedDuration := (2 * sleepDuration) / 3
		if expectedDuration > stats.SemaphoreWaitDuration {
			return errors.Errorf("SemaphoreWaitDuration %s is too low",
				stats.SemaphoreWaitDuration.String())
		}
		if expectedDuration > stats.MemTableWriteStallDuration {
			return errors.Errorf("MemTableWriteStallDuration %s is too low",
				stats.MemTableWriteStallDuration.String())
		}
		if expectedDuration > stats.L0ReadAmpWriteStallDuration {
			return errors.Errorf("L0ReadAmpWriteStallDuration %s is too low",
				stats.L0ReadAmpWriteStallDuration)
		}
		if expectedDuration > stats.CommitWaitDuration {
			return errors.Errorf("CommitWaitDuration %s is too low",
				stats.CommitWaitDuration)
		}
		if 5*expectedDuration > stats.TotalDuration {
			return errors.Errorf("TotalDuration %s is too low",
				stats.TotalDuration)
		}
		return nil
	}
	// Try a few times, and succeed if one of them succeeds.
	var err error
	for i := 0; i < 5; i++ {
		err = testFunc()
		if err == nil {
			break
		}
	}
	require.NoError(t, err)
}

func TestBatchReader(t *testing.T) {
	datadriven.RunTest(t, "testdata/batch_reader", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "scan":
			var repr bytes.Buffer
			for i, l := range strings.Split(td.Input, "\n") {
				// Remove any trailing comments behind #.
				if i := strings.IndexRune(l, '#'); i >= 0 {
					l = l[:i]
				}
				// Strip all whitespace from the line.
				l = strings.Map(func(r rune) rune {
					if unicode.IsSpace(r) {
						return -1
					}
					return r
				}, l)
				b, err := hex.DecodeString(l)
				if err != nil {
					return fmt.Sprintf("failed to decode hex; line %d", i)
				}
				repr.Write(b)
			}
			r, count := ReadBatch(repr.Bytes())
			var out strings.Builder
			fmt.Fprintf(&out, "Count: %d\n", count)
			for {
				kind, ukey, value, ok, err := r.Next()
				if !ok {
					if err != nil {
						fmt.Fprintf(&out, "err: %s\n", err)
					} else {
						fmt.Fprint(&out, "eof")
					}
					break
				}
				fmt.Fprintf(&out, "%s: %q: %q\n", kind, ukey, value)
			}
			return out.String()

		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func BenchmarkBatchSet(b *testing.B) {
	value := make([]byte, 10)
	for i := range value {
		value[i] = byte(i)
	}
	key := make([]byte, 8)
	batch := newBatch(nil)

	b.ResetTimer()

	const batchSize = 1000
	for i := 0; i < b.N; i += batchSize {
		end := i + batchSize
		if end > b.N {
			end = b.N
		}

		for j := i; j < end; j++ {
			binary.BigEndian.PutUint64(key, uint64(j))
			batch.Set(key, value, nil)
		}
		batch.Reset()
	}

	b.StopTimer()
}

func BenchmarkIndexedBatchSet(b *testing.B) {
	value := make([]byte, 10)
	for i := range value {
		value[i] = byte(i)
	}
	key := make([]byte, 8)
	batch := newIndexedBatch(nil, DefaultComparer)

	b.ResetTimer()

	const batchSize = 1000
	for i := 0; i < b.N; i += batchSize {
		end := i + batchSize
		if end > b.N {
			end = b.N
		}

		for j := i; j < end; j++ {
			binary.BigEndian.PutUint64(key, uint64(j))
			batch.Set(key, value, nil)
		}
		batch.Reset()
	}

	b.StopTimer()
}

func BenchmarkBatchSetDeferred(b *testing.B) {
	value := make([]byte, 10)
	for i := range value {
		value[i] = byte(i)
	}
	key := make([]byte, 8)
	batch := newBatch(nil)

	b.ResetTimer()

	const batchSize = 1000
	for i := 0; i < b.N; i += batchSize {
		end := i + batchSize
		if end > b.N {
			end = b.N
		}

		for j := i; j < end; j++ {
			binary.BigEndian.PutUint64(key, uint64(j))
			deferredOp := batch.SetDeferred(len(key), len(value))

			copy(deferredOp.Key, key)
			copy(deferredOp.Value, value)

			deferredOp.Finish()
		}
		batch.Reset()
	}

	b.StopTimer()
}

func BenchmarkIndexedBatchSetDeferred(b *testing.B) {
	value := make([]byte, 10)
	for i := range value {
		value[i] = byte(i)
	}
	key := make([]byte, 8)
	batch := newIndexedBatch(nil, DefaultComparer)

	b.ResetTimer()

	const batchSize = 1000
	for i := 0; i < b.N; i += batchSize {
		end := i + batchSize
		if end > b.N {
			end = b.N
		}

		for j := i; j < end; j++ {
			binary.BigEndian.PutUint64(key, uint64(j))
			deferredOp := batch.SetDeferred(len(key), len(value))

			copy(deferredOp.Key, key)
			copy(deferredOp.Value, value)

			deferredOp.Finish()
		}
		batch.Reset()
	}

	b.StopTimer()
}

func TestBatchMemTableSizeOverflow(t *testing.T) {
	opts := &Options{
		FS: vfs.NewMem(),
	}
	opts.EnsureDefaults()
	d, err := Open("", opts)
	require.NoError(t, err)

	bigValue := make([]byte, 1000)
	b := d.NewBatch()

	// memTableSize can overflow as a uint32.
	b.memTableSize = math.MaxUint32 - 50
	for i := 0; i < 10; i++ {
		k := fmt.Sprintf("key-%05d", i)
		require.NoError(t, b.Set([]byte(k), bigValue, nil))
	}
	require.Greater(t, b.memTableSize, uint64(math.MaxUint32))
	require.NoError(t, b.Close())
	require.NoError(t, d.Close())
}

// TestBatchSpanCaching stress tests the caching of keyspan.Spans for range
// tombstones and range keys.
func TestBatchSpanCaching(t *testing.T) {
	opts := &Options{
		Comparer:           testkeys.Comparer,
		FS:                 vfs.NewMem(),
		FormatMajorVersion: internalFormatNewest,
	}
	d, err := Open("", opts)
	require.NoError(t, err)
	defer d.Close()

	ks := testkeys.Alpha(1)
	b := d.NewIndexedBatch()
	for i := int64(0); i < ks.Count(); i++ {
		k := testkeys.Key(ks, i)
		require.NoError(t, b.Set(k, k, nil))
	}

	seed := int64(time.Now().UnixNano())
	t.Logf("seed = %d", seed)
	rng := rand.New(rand.NewSource(seed))
	iters := make([][]*Iterator, ks.Count())
	defer func() {
		for _, keyIters := range iters {
			for _, iter := range keyIters {
				_ = iter.Close()
			}
		}
	}()

	// This test begins with one point key for every letter of the alphabet.
	// Over the course of the test, point keys are 'replaced' with range keys
	// with narrow bounds from left to right. Iterators are created at random,
	// sometimes from the batch and sometimes by cloning existing iterators.

	checkIter := func(iter *Iterator, nextKey int64) {
		var i int64
		for valid := iter.First(); valid; valid = iter.Next() {
			hasPoint, hasRange := iter.HasPointAndRange()
			require.Equal(t, testkeys.Key(ks, i), iter.Key())
			if i < nextKey {
				// This key should not exist as a point key, just a range key.
				require.False(t, hasPoint)
				require.True(t, hasRange)
			} else {
				require.True(t, hasPoint)
				require.False(t, hasRange)
			}
			i++
		}
		require.Equal(t, ks.Count(), i)
	}

	// Each iteration of the below loop either reads or writes.
	//
	// A write iteration writes a new RANGEDEL and RANGEKEYSET into the batch,
	// covering a single point key seeded above. Writing these two span keys
	// together 'replaces' the point key with a range key. Each write iteration
	// ratchets nextWriteKey so the next write iteration will write the next
	// key.
	//
	// A read iteration creates a new iterator and ensures its state is
	// expected: some prefix of only point keys, followed by a suffix of only
	// range keys. Iterators created through Clone should observe the point keys
	// that existed when the cloned iterator was created.
	for nextWriteKey := int64(0); nextWriteKey < ks.Count(); {
		p := rng.Float64()
		switch {
		case p < .10: /* 10 % */
			// Write a new range deletion and range key.
			start := testkeys.Key(ks, nextWriteKey)
			end := append(start, 0x00)
			require.NoError(t, b.DeleteRange(start, end, nil))
			require.NoError(t, b.RangeKeySet(start, end, nil, []byte("foo"), nil))
			nextWriteKey++
		case p < .55: /* 45 % */
			// Create a new iterator directly from the batch and check that it
			// observes the correct state.
			iter, _ := b.NewIter(&IterOptions{KeyTypes: IterKeyTypePointsAndRanges})
			checkIter(iter, nextWriteKey)
			iters[nextWriteKey] = append(iters[nextWriteKey], iter)
		default: /* 45 % */
			// Create a new iterator through cloning a random existing iterator
			// and check that it observes the right state.
			readKey := rng.Int63n(nextWriteKey + 1)
			itersForReadKey := iters[readKey]
			if len(itersForReadKey) == 0 {
				continue
			}
			iter, err := itersForReadKey[rng.Intn(len(itersForReadKey))].Clone(CloneOptions{})
			require.NoError(t, err)
			checkIter(iter, readKey)
			iters[readKey] = append(iters[readKey], iter)
		}
	}
}
