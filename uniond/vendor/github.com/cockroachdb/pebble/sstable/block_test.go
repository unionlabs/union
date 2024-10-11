// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
	"testing"
	"time"
	"unsafe"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func ikey(s string) InternalKey {
	return InternalKey{UserKey: []byte(s)}
}

func TestBlockWriter(t *testing.T) {
	w := &rawBlockWriter{
		blockWriter: blockWriter{restartInterval: 16},
	}
	w.add(ikey("apple"), nil)
	w.add(ikey("apricot"), nil)
	w.add(ikey("banana"), nil)
	block := w.finish()

	expected := []byte(
		"\x00\x05\x00apple" +
			"\x02\x05\x00ricot" +
			"\x00\x06\x00banana" +
			"\x00\x00\x00\x00\x01\x00\x00\x00")
	if !bytes.Equal(expected, block) {
		t.Fatalf("expected\n%q\nfound\n%q", expected, block)
	}
}

func TestBlockWriterWithPrefix(t *testing.T) {
	w := &rawBlockWriter{
		blockWriter: blockWriter{restartInterval: 2},
	}
	curKey := func() string {
		return string(base.DecodeInternalKey(w.curKey).UserKey)
	}
	addAdapter := func(
		key InternalKey,
		value []byte,
		addValuePrefix bool,
		valuePrefix valuePrefix,
		setHasSameKeyPrefix bool) {
		w.addWithOptionalValuePrefix(
			key, false, value, len(key.UserKey), addValuePrefix, valuePrefix, setHasSameKeyPrefix)
	}
	addAdapter(
		ikey("apple"), []byte("red"), false, 0, true)
	require.Equal(t, "apple", curKey())
	require.Equal(t, "red", string(w.curValue))
	addAdapter(
		ikey("apricot"), []byte("orange"), true, '\xff', false)
	require.Equal(t, "apricot", curKey())
	require.Equal(t, "orange", string(w.curValue))
	// Even though this call has setHasSameKeyPrefix=true, the previous call,
	// which was after the last restart set it to false. So the restart encoded
	// with banana has this cumulative bit set to false.
	addAdapter(
		ikey("banana"), []byte("yellow"), true, '\x00', true)
	require.Equal(t, "banana", curKey())
	require.Equal(t, "yellow", string(w.curValue))
	addAdapter(
		ikey("cherry"), []byte("red"), false, 0, true)
	require.Equal(t, "cherry", curKey())
	require.Equal(t, "red", string(w.curValue))
	// All intervening calls has setHasSameKeyPrefix=true, so the cumulative bit
	// will be set to true in this restart.
	addAdapter(
		ikey("mango"), []byte("juicy"), false, 0, true)
	require.Equal(t, "mango", curKey())
	require.Equal(t, "juicy", string(w.curValue))

	block := w.finish()

	expected := []byte(
		"\x00\x0d\x03apple\x00\x00\x00\x00\x00\x00\x00\x00red" +
			"\x02\x0d\x07ricot\x00\x00\x00\x00\x00\x00\x00\x00\xfforange" +
			"\x00\x0e\x07banana\x00\x00\x00\x00\x00\x00\x00\x00\x00yellow" +
			"\x00\x0e\x03cherry\x00\x00\x00\x00\x00\x00\x00\x00red" +
			"\x00\x0d\x05mango\x00\x00\x00\x00\x00\x00\x00\x00juicy" +
			// Restarts are:
			// 00000000 (restart at apple), 2a000000 (restart at banana), 56000080 (restart at mango)
			// 03000000 (number of restart, i.e., 3). The restart at mango has 1 in the most significant
			// bit of the uint32, so the last byte in the little endian encoding is \x80.
			"\x00\x00\x00\x00\x2a\x00\x00\x00\x56\x00\x00\x80\x03\x00\x00\x00")
	if !bytes.Equal(expected, block) {
		t.Fatalf("expected\n%x\nfound\n%x", expected, block)
	}
}

func testBlockCleared(t *testing.T, w, b *blockWriter) {
	require.Equal(t, w.restartInterval, b.restartInterval)
	require.Equal(t, w.nEntries, b.nEntries)
	require.Equal(t, w.nextRestart, b.nextRestart)
	require.Equal(t, len(w.buf), len(b.buf))
	require.Equal(t, len(w.restarts), len(b.restarts))
	require.Equal(t, len(w.curKey), len(b.curKey))
	require.Equal(t, len(w.prevKey), len(b.prevKey))
	require.Equal(t, len(w.curValue), len(b.curValue))
	require.Equal(t, w.tmp, b.tmp)

	// Make sure that we didn't lose the allocated byte slices.
	require.True(t, cap(w.buf) > 0 && cap(b.buf) == 0)
	require.True(t, cap(w.restarts) > 0 && cap(b.restarts) == 0)
	require.True(t, cap(w.curKey) > 0 && cap(b.curKey) == 0)
	require.True(t, cap(w.prevKey) > 0 && cap(b.prevKey) == 0)
	require.True(t, cap(w.curValue) > 0 && cap(b.curValue) == 0)
}

func TestBlockClear(t *testing.T) {
	w := blockWriter{restartInterval: 16}
	w.add(ikey("apple"), nil)
	w.add(ikey("apricot"), nil)
	w.add(ikey("banana"), nil)

	w.clear()

	// Once a block is cleared, we expect its fields to be cleared, but we expect
	// it to keep its allocated byte slices.
	b := blockWriter{}
	testBlockCleared(t, &w, &b)
}

func TestInvalidInternalKeyDecoding(t *testing.T) {
	// Invalid keys since they don't have an 8 byte trailer.
	testCases := []string{
		"",
		"\x01\x02\x03\x04\x05\x06\x07",
		"foo",
	}
	for _, tc := range testCases {
		i := blockIter{}
		i.decodeInternalKey([]byte(tc))
		require.Nil(t, i.ikey.UserKey)
		require.Equal(t, uint64(InternalKeyKindInvalid), i.ikey.Trailer)
	}
}

func TestBlockIter(t *testing.T) {
	// k is a block that maps three keys "apple", "apricot", "banana" to empty strings.
	k := block([]byte(
		"\x00\x05\x00apple" +
			"\x02\x05\x00ricot" +
			"\x00\x06\x00banana" +
			"\x00\x00\x00\x00\x01\x00\x00\x00"))
	var testcases = []struct {
		index int
		key   string
	}{
		{0, ""},
		{0, "a"},
		{0, "aaaaaaaaaaaaaaa"},
		{0, "app"},
		{0, "apple"},
		{1, "appliance"},
		{1, "apricos"},
		{1, "apricot"},
		{2, "azzzzzzzzzzzzzz"},
		{2, "b"},
		{2, "banan"},
		{2, "banana"},
		{3, "banana\x00"},
		{3, "c"},
	}
	for _, tc := range testcases {
		i, err := newRawBlockIter(bytes.Compare, k)
		require.NoError(t, err)
		i.SeekGE([]byte(tc.key))
		for j, keyWant := range []string{"apple", "apricot", "banana"}[tc.index:] {
			if !i.Valid() {
				t.Fatalf("key=%q, index=%d, j=%d: Valid got false, keyWant true", tc.key, tc.index, j)
			}
			if keyGot := string(i.Key().UserKey); keyGot != keyWant {
				t.Fatalf("key=%q, index=%d, j=%d: got %q, keyWant %q", tc.key, tc.index, j, keyGot, keyWant)
			}
			i.Next()
		}
		if i.Valid() {
			t.Fatalf("key=%q, index=%d: Valid got true, keyWant false", tc.key, tc.index)
		}
		if err := i.Close(); err != nil {
			t.Fatalf("key=%q, index=%d: got err=%v", tc.key, tc.index, err)
		}
	}

	{
		i, err := newRawBlockIter(bytes.Compare, k)
		require.NoError(t, err)
		i.Last()
		for j, keyWant := range []string{"banana", "apricot", "apple"} {
			if !i.Valid() {
				t.Fatalf("j=%d: Valid got false, want true", j)
			}
			if keyGot := string(i.Key().UserKey); keyGot != keyWant {
				t.Fatalf("j=%d: got %q, want %q", j, keyGot, keyWant)
			}
			i.Prev()
		}
		if i.Valid() {
			t.Fatalf("Valid got true, want false")
		}
		if err := i.Close(); err != nil {
			t.Fatalf("got err=%v", err)
		}
	}
}

func TestBlockIter2(t *testing.T) {
	makeIkey := func(s string) InternalKey {
		j := strings.Index(s, ":")
		seqNum, err := strconv.Atoi(s[j+1:])
		if err != nil {
			panic(err)
		}
		return base.MakeInternalKey([]byte(s[:j]), uint64(seqNum), InternalKeyKindSet)
	}

	var block []byte

	for _, r := range []int{1, 2, 3, 4} {
		t.Run(fmt.Sprintf("restart=%d", r), func(t *testing.T) {
			datadriven.RunTest(t, "testdata/block", func(t *testing.T, d *datadriven.TestData) string {
				switch d.Cmd {
				case "build":
					w := &blockWriter{restartInterval: r}
					for _, e := range strings.Split(strings.TrimSpace(d.Input), ",") {
						w.add(makeIkey(e), nil)
					}
					block = w.finish()
					return ""

				case "iter":
					iter, err := newBlockIter(bytes.Compare, block)
					if err != nil {
						return err.Error()
					}

					iter.globalSeqNum, err = scanGlobalSeqNum(d)
					if err != nil {
						return err.Error()
					}

					var b bytes.Buffer
					for _, line := range strings.Split(d.Input, "\n") {
						parts := strings.Fields(line)
						if len(parts) == 0 {
							continue
						}
						switch parts[0] {
						case "seek-ge":
							if len(parts) != 2 {
								return "seek-ge <key>\n"
							}
							iter.SeekGE([]byte(strings.TrimSpace(parts[1])), base.SeekGEFlagsNone)
						case "seek-lt":
							if len(parts) != 2 {
								return "seek-lt <key>\n"
							}
							iter.SeekLT([]byte(strings.TrimSpace(parts[1])), base.SeekLTFlagsNone)
						case "first":
							iter.First()
						case "last":
							iter.Last()
						case "next":
							iter.Next()
						case "prev":
							iter.Prev()
						}
						if iter.valid() {
							fmt.Fprintf(&b, "<%s:%d>", iter.Key().UserKey, iter.Key().SeqNum())
						} else if err := iter.Error(); err != nil {
							fmt.Fprintf(&b, "<err=%v>", err)
						} else {
							fmt.Fprintf(&b, ".")
						}
					}
					b.WriteString("\n")
					return b.String()

				default:
					return fmt.Sprintf("unknown command: %s", d.Cmd)
				}
			})
		})
	}
}

func TestBlockIterKeyStability(t *testing.T) {
	w := &blockWriter{restartInterval: 1}
	expected := [][]byte{
		[]byte("apple"),
		[]byte("apricot"),
		[]byte("banana"),
	}
	for i := range expected {
		w.add(InternalKey{UserKey: expected[i]}, nil)
	}
	block := w.finish()

	i, err := newBlockIter(bytes.Compare, block)
	require.NoError(t, err)

	// Check that the supplied slice resides within the bounds of the block.
	check := func(v []byte) {
		t.Helper()
		begin := unsafe.Pointer(&v[0])
		end := unsafe.Pointer(uintptr(begin) + uintptr(len(v)))
		blockBegin := unsafe.Pointer(&block[0])
		blockEnd := unsafe.Pointer(uintptr(blockBegin) + uintptr(len(block)))
		if uintptr(begin) < uintptr(blockBegin) || uintptr(end) > uintptr(blockEnd) {
			t.Fatalf("key %p-%p resides outside of block %p-%p", begin, end, blockBegin, blockEnd)
		}
	}

	// Check that various means of iterating over the data match our expected
	// values. Note that this is only guaranteed because of the usage of a
	// restart-interval of 1 so that prefix compression was not performed.
	for j := range expected {
		keys := [][]byte{}
		for key, _ := i.SeekGE(expected[j], base.SeekGEFlagsNone); key != nil; key, _ = i.Next() {
			check(key.UserKey)
			keys = append(keys, key.UserKey)
		}
		require.EqualValues(t, expected[j:], keys)
	}

	for j := range expected {
		keys := [][]byte{}
		for key, _ := i.SeekLT(expected[j], base.SeekLTFlagsNone); key != nil; key, _ = i.Prev() {
			check(key.UserKey)
			keys = append(keys, key.UserKey)
		}
		for i, j := 0, len(keys)-1; i < j; i, j = i+1, j-1 {
			keys[i], keys[j] = keys[j], keys[i]
		}
		require.EqualValues(t, expected[:j], keys)
	}
}

// Regression test for a bug in blockIter.Next where it was failing to handle
// the case where it is switching from reverse to forward iteration. When that
// switch occurs we need to populate blockIter.fullKey so that prefix
// decompression works properly.
func TestBlockIterReverseDirections(t *testing.T) {
	w := &blockWriter{restartInterval: 4}
	keys := [][]byte{
		[]byte("apple0"),
		[]byte("apple1"),
		[]byte("apple2"),
		[]byte("banana"),
		[]byte("carrot"),
	}
	for i := range keys {
		w.add(InternalKey{UserKey: keys[i]}, nil)
	}
	block := w.finish()

	for targetPos := 0; targetPos < w.restartInterval; targetPos++ {
		t.Run("", func(t *testing.T) {
			i, err := newBlockIter(bytes.Compare, block)
			require.NoError(t, err)

			pos := 3
			if key, _ := i.SeekLT([]byte("carrot"), base.SeekLTFlagsNone); !bytes.Equal(keys[pos], key.UserKey) {
				t.Fatalf("expected %s, but found %s", keys[pos], key.UserKey)
			}
			for pos > targetPos {
				pos--
				if key, _ := i.Prev(); !bytes.Equal(keys[pos], key.UserKey) {
					t.Fatalf("expected %s, but found %s", keys[pos], key.UserKey)
				}
			}
			pos++
			if key, _ := i.Next(); !bytes.Equal(keys[pos], key.UserKey) {
				t.Fatalf("expected %s, but found %s", keys[pos], key.UserKey)
			}
		})
	}
}

func BenchmarkBlockIterSeekGE(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				w := &blockWriter{
					restartInterval: restartInterval,
				}

				var ikey InternalKey
				var keys [][]byte
				for i := 0; w.estimatedSize() < blockSize; i++ {
					key := []byte(fmt.Sprintf("%05d", i))
					keys = append(keys, key)
					ikey.UserKey = key
					w.add(ikey, nil)
				}

				it, err := newBlockIter(bytes.Compare, w.finish())
				if err != nil {
					b.Fatal(err)
				}
				rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					k := keys[rng.Intn(len(keys))]
					it.SeekGE(k, base.SeekGEFlagsNone)
					if testing.Verbose() {
						if !it.valid() {
							b.Fatal("expected to find key")
						}
						if !bytes.Equal(k, it.Key().UserKey) {
							b.Fatalf("expected %s, but found %s", k, it.Key().UserKey)
						}
					}
				}
			})
	}
}

func BenchmarkBlockIterSeekLT(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				w := &blockWriter{
					restartInterval: restartInterval,
				}

				var ikey InternalKey
				var keys [][]byte
				for i := 0; w.estimatedSize() < blockSize; i++ {
					key := []byte(fmt.Sprintf("%05d", i))
					keys = append(keys, key)
					ikey.UserKey = key
					w.add(ikey, nil)
				}

				it, err := newBlockIter(bytes.Compare, w.finish())
				if err != nil {
					b.Fatal(err)
				}
				rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					j := rng.Intn(len(keys))
					it.SeekLT(keys[j], base.SeekLTFlagsNone)
					if testing.Verbose() {
						if j == 0 {
							if it.valid() {
								b.Fatal("unexpected key")
							}
						} else {
							if !it.valid() {
								b.Fatal("expected to find key")
							}
							k := keys[j-1]
							if !bytes.Equal(k, it.Key().UserKey) {
								b.Fatalf("expected %s, but found %s", k, it.Key().UserKey)
							}
						}
					}
				}
			})
	}
}

func BenchmarkBlockIterNext(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				w := &blockWriter{
					restartInterval: restartInterval,
				}

				var ikey InternalKey
				for i := 0; w.estimatedSize() < blockSize; i++ {
					ikey.UserKey = []byte(fmt.Sprintf("%05d", i))
					w.add(ikey, nil)
				}

				it, err := newBlockIter(bytes.Compare, w.finish())
				if err != nil {
					b.Fatal(err)
				}

				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					if !it.valid() {
						it.First()
					}
					it.Next()
				}
			})
	}
}

func BenchmarkBlockIterPrev(b *testing.B) {
	const blockSize = 32 << 10

	for _, restartInterval := range []int{16} {
		b.Run(fmt.Sprintf("restart=%d", restartInterval),
			func(b *testing.B) {
				w := &blockWriter{
					restartInterval: restartInterval,
				}

				var ikey InternalKey
				for i := 0; w.estimatedSize() < blockSize; i++ {
					ikey.UserKey = []byte(fmt.Sprintf("%05d", i))
					w.add(ikey, nil)
				}

				it, err := newBlockIter(bytes.Compare, w.finish())
				if err != nil {
					b.Fatal(err)
				}

				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					if !it.valid() {
						it.Last()
					}
					it.Prev()
				}
			})
	}
}
