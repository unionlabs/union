// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package record

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func short(s string) string {
	if len(s) < 64 {
		return s
	}
	return fmt.Sprintf("%s...(skipping %d bytes)...%s", s[:20], len(s)-40, s[len(s)-20:])
}

// big returns a string of length n, composed of repetitions of partial.
func big(partial string, n int) string {
	return strings.Repeat(partial, n/len(partial)+1)[:n]
}

type recordWriter interface {
	WriteRecord([]byte) (int64, error)
	Close() error
}

func testGeneratorWriter(
	t *testing.T, reset func(), gen func() (string, bool), newWriter func(io.Writer) recordWriter,
) {
	buf := new(bytes.Buffer)

	reset()
	w := newWriter(buf)
	for {
		s, ok := gen()
		if !ok {
			break
		}
		if _, err := w.WriteRecord([]byte(s)); err != nil {
			t.Fatalf("Write: %v", err)
		}
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}
	reset()
	r := NewReader(buf, 0 /* logNum */)
	for {
		s, ok := gen()
		if !ok {
			break
		}
		rr, err := r.Next()
		if err != nil {
			t.Fatalf("reader.Next: %v", err)
		}
		x, err := io.ReadAll(rr)
		if err != nil {
			t.Fatalf("ReadAll: %v", err)
		}
		if string(x) != s {
			t.Fatalf("got %q, want %q", short(string(x)), short(s))
		}
	}
	if _, err := r.Next(); err != io.EOF {
		t.Fatalf("got %v, want %v", err, io.EOF)
	}
}

func testGenerator(t *testing.T, reset func(), gen func() (string, bool)) {
	t.Run("Writer", func(t *testing.T) {
		testGeneratorWriter(t, reset, gen, func(w io.Writer) recordWriter {
			return NewWriter(w)
		})
	})

	t.Run("LogWriter", func(t *testing.T) {
		testGeneratorWriter(t, reset, gen, func(w io.Writer) recordWriter {
			return NewLogWriter(w, 0 /* logNum */, LogWriterConfig{
				WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
		})
	})
}

func testLiterals(t *testing.T, s []string) {
	var i int
	reset := func() {
		i = 0
	}
	gen := func() (string, bool) {
		if i == len(s) {
			return "", false
		}
		i++
		return s[i-1], true
	}
	testGenerator(t, reset, gen)
}

func TestMany(t *testing.T) {
	const n = 1e5
	var i int
	reset := func() {
		i = 0
	}
	gen := func() (string, bool) {
		if i == n {
			return "", false
		}
		i++
		return fmt.Sprintf("%d.", i-1), true
	}
	testGenerator(t, reset, gen)
}

func TestRandom(t *testing.T) {
	const n = 1e2
	var (
		i int
		r *rand.Rand
	)
	reset := func() {
		i, r = 0, rand.New(rand.NewSource(0))
	}
	gen := func() (string, bool) {
		if i == n {
			return "", false
		}
		i++
		return strings.Repeat(string(uint8(i)), r.Intn(2*blockSize+16)), true
	}
	testGenerator(t, reset, gen)
}

func TestBasic(t *testing.T) {
	testLiterals(t, []string{
		strings.Repeat("a", 1000),
		strings.Repeat("b", 97270),
		strings.Repeat("c", 8000),
	})
}

func TestBoundary(t *testing.T) {
	for i := blockSize - 16; i < blockSize+16; i++ {
		s0 := big("abcd", i)
		for j := blockSize - 16; j < blockSize+16; j++ {
			s1 := big("ABCDE", j)
			testLiterals(t, []string{s0, s1})
			testLiterals(t, []string{s0, "", s1})
			testLiterals(t, []string{s0, "x", s1})
		}
	}
}

func TestFlush(t *testing.T) {
	buf := new(bytes.Buffer)
	w := NewWriter(buf)
	// Write a couple of records. Everything should still be held
	// in the record.Writer buffer, so that buf.Len should be 0.
	w0, _ := w.Next()
	w0.Write([]byte("0"))
	w1, _ := w.Next()
	w1.Write([]byte("11"))
	if got, want := buf.Len(), 0; got != want {
		t.Fatalf("buffer length #0: got %d want %d", got, want)
	}
	// Flush the record.Writer buffer, which should yield 17 bytes.
	// 17 = 2*7 + 1 + 2, which is two headers and 1 + 2 payload bytes.
	require.NoError(t, w.Flush())
	if got, want := buf.Len(), 17; got != want {
		t.Fatalf("buffer length #1: got %d want %d", got, want)
	}
	// Do another write, one that isn't large enough to complete the block.
	// The write should not have flowed through to buf.
	w2, _ := w.Next()
	w2.Write(bytes.Repeat([]byte("2"), 10000))
	if got, want := buf.Len(), 17; got != want {
		t.Fatalf("buffer length #2: got %d want %d", got, want)
	}
	// Flushing should get us up to 10024 bytes written.
	// 10024 = 17 + 7 + 10000.
	require.NoError(t, w.Flush())
	if got, want := buf.Len(), 10024; got != want {
		t.Fatalf("buffer length #3: got %d want %d", got, want)
	}
	// Do a bigger write, one that completes the current block.
	// We should now have 32768 bytes (a complete block), without
	// an explicit flush.
	w3, _ := w.Next()
	w3.Write(bytes.Repeat([]byte("3"), 40000))
	if got, want := buf.Len(), 32768; got != want {
		t.Fatalf("buffer length #4: got %d want %d", got, want)
	}
	// Flushing should get us up to 50038 bytes written.
	// 50038 = 10024 + 2*7 + 40000. There are two headers because
	// the one record was split into two chunks.
	require.NoError(t, w.Flush())
	if got, want := buf.Len(), 50038; got != want {
		t.Fatalf("buffer length #5: got %d want %d", got, want)
	}
	// Check that reading those records give the right lengths.
	r := NewReader(buf, 0 /* logNum */)
	wants := []int64{1, 2, 10000, 40000}
	for i, want := range wants {
		rr, _ := r.Next()
		n, err := io.Copy(io.Discard, rr)
		if err != nil {
			t.Fatalf("read #%d: %v", i, err)
		}
		if n != want {
			t.Fatalf("read #%d: got %d bytes want %d", i, n, want)
		}
	}
}

func TestNonExhaustiveRead(t *testing.T) {
	const n = 100
	buf := new(bytes.Buffer)
	p := make([]byte, 10)
	rnd := rand.New(rand.NewSource(1))

	w := NewWriter(buf)
	for i := 0; i < n; i++ {
		length := len(p) + rnd.Intn(3*blockSize)
		s := string(uint8(i)) + "123456789abcdefgh"
		_, _ = w.WriteRecord([]byte(big(s, length)))
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	r := NewReader(buf, 0 /* logNum */)
	for i := 0; i < n; i++ {
		rr, _ := r.Next()
		_, err := io.ReadFull(rr, p)
		if err != nil {
			t.Fatalf("ReadFull: %v", err)
		}
		want := string(uint8(i)) + "123456789"
		if got := string(p); got != want {
			t.Fatalf("read #%d: got %q want %q", i, got, want)
		}
	}
}

func TestStaleReader(t *testing.T) {
	buf := new(bytes.Buffer)

	w := NewWriter(buf)
	_, err := w.WriteRecord([]byte("0"))
	require.NoError(t, err)

	_, err = w.WriteRecord([]byte("11"))
	require.NoError(t, err)

	require.NoError(t, w.Close())

	r := NewReader(buf, 0 /* logNum */)
	r0, err := r.Next()
	require.NoError(t, err)

	r1, err := r.Next()
	require.NoError(t, err)

	p := make([]byte, 1)
	if _, err := r0.Read(p); err == nil || !strings.Contains(err.Error(), "stale") {
		t.Fatalf("stale read #0: unexpected error: %v", err)
	}
	if _, err := r1.Read(p); err != nil {
		t.Fatalf("fresh read #1: got %v want nil error", err)
	}
	if p[0] != '1' {
		t.Fatalf("fresh read #1: byte contents: got '%c' want '1'", p[0])
	}
}

type testRecords struct {
	records [][]byte // The raw value of each record.
	offsets []int64  // The offset of each record within buf, derived from writer.LastRecordOffset.
	buf     []byte   // The serialized records form of all records.
}

// makeTestRecords generates test records of specified lengths.
// The first record will consist of repeating 0x00 bytes, the next record of
// 0x01 bytes, and so forth. The values will loop back to 0x00 after 0xff.
func makeTestRecords(recordLengths ...int) (*testRecords, error) {
	ret := &testRecords{}
	ret.records = make([][]byte, len(recordLengths))
	ret.offsets = make([]int64, len(recordLengths))
	for i, n := range recordLengths {
		ret.records[i] = bytes.Repeat([]byte{byte(i)}, n)
	}

	buf := new(bytes.Buffer)
	w := NewWriter(buf)
	for i, rec := range ret.records {
		wRec, err := w.Next()
		if err != nil {
			return nil, err
		}

		// Alternate between one big write and many small writes.
		cSize := 8
		if i&1 == 0 {
			cSize = len(rec)
		}
		for ; len(rec) > cSize; rec = rec[cSize:] {
			if _, err := wRec.Write(rec[:cSize]); err != nil {
				return nil, err
			}
		}
		if _, err := wRec.Write(rec); err != nil {
			return nil, err
		}

		ret.offsets[i], err = w.LastRecordOffset()
		if err != nil {
			return nil, err
		}
	}

	if err := w.Close(); err != nil {
		return nil, err
	}

	ret.buf = buf.Bytes()
	return ret, nil
}

// corruptBlock corrupts the checksum of the record that starts at the
// specified block offset. The number of the block offset is 0 based.
func corruptBlock(buf []byte, blockNum int) {
	// Ensure we always permute at least 1 byte of the checksum.
	if buf[blockSize*blockNum] == 0x00 {
		buf[blockSize*blockNum] = 0xff
	} else {
		buf[blockSize*blockNum] = 0x00
	}

	buf[blockSize*blockNum+1] = 0x00
	buf[blockSize*blockNum+2] = 0x00
	buf[blockSize*blockNum+3] = 0x00
}

func TestRecoverNoOp(t *testing.T) {
	recs, err := makeTestRecords(
		blockSize-legacyHeaderSize,
		blockSize-legacyHeaderSize,
		blockSize-legacyHeaderSize,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	r := NewReader(bytes.NewReader(recs.buf), 0 /* logNum */)
	_, err = r.Next()
	if err != nil || r.err != nil {
		t.Fatalf("reader.Next: %v reader.err: %v", err, r.err)
	}

	seq, begin, end, n := r.seq, r.begin, r.end, r.n

	// Should be a no-op since r.err == nil.
	r.recover()

	// r.err was nil, nothing should have changed.
	if seq != r.seq || begin != r.begin || end != r.end || n != r.n {
		t.Fatal("reader.Recover when no error existed, was not a no-op")
	}
}

func TestBasicRecover(t *testing.T) {
	recs, err := makeTestRecords(
		blockSize-legacyHeaderSize,
		blockSize-legacyHeaderSize,
		blockSize-legacyHeaderSize,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	// Corrupt the checksum of the second record r1 in our file.
	corruptBlock(recs.buf, 1)

	underlyingReader := bytes.NewReader(recs.buf)
	r := NewReader(underlyingReader, 0 /* logNum */)

	// The first record r0 should be read just fine.
	r0, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}
	r0Data, err := io.ReadAll(r0)
	if err != nil {
		t.Fatalf("ReadAll: %v", err)
	}
	if !bytes.Equal(r0Data, recs.records[0]) {
		t.Fatal("Unexpected output in r0's data")
	}

	// The next record should have a checksum mismatch.
	_, err = r.Next()
	if err == nil {
		t.Fatal("Expected an error while reading a corrupted record")
	}
	if err != ErrInvalidChunk {
		t.Fatalf("Unexpected error returned: %v", err)
	}

	// Recover from that checksum mismatch.
	r.recover()
	currentOffset, err := underlyingReader.Seek(0, io.SeekCurrent)
	if err != nil {
		t.Fatalf("current offset: %v", err)
	}
	if currentOffset != blockSize*2 {
		t.Fatalf("current offset: got %d, want %d", currentOffset, blockSize*2)
	}

	// The third record r2 should be read just fine.
	r2, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}
	r2Data, err := io.ReadAll(r2)
	if err != nil {
		t.Fatalf("ReadAll: %v", err)
	}
	if !bytes.Equal(r2Data, recs.records[2]) {
		t.Fatal("Unexpected output in r2's data")
	}
}

func TestRecoverSingleBlock(t *testing.T) {
	// The first record will be blockSize * 3 bytes long. Since each block has
	// a 7 byte header, the first record will roll over into 4 blocks.
	recs, err := makeTestRecords(
		blockSize*3,
		blockSize-legacyHeaderSize,
		blockSize/2,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	// Corrupt the checksum for the portion of the first record that exists in
	// the 4th block.
	corruptBlock(recs.buf, 3)

	// The first record should fail, but only when we read deeper beyond the
	// first block.
	r := NewReader(bytes.NewReader(recs.buf), 0 /* logNum */)
	r0, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}

	// Reading deeper should yield a checksum mismatch.
	_, err = io.ReadAll(r0)
	if err == nil {
		t.Fatal("Expected a checksum mismatch error, got nil")
	}
	if err != ErrInvalidChunk {
		t.Fatalf("Unexpected error returned: %v", err)
	}

	// Recover from that checksum mismatch.
	r.recover()

	// All of the data in the second record r1 is lost because the first record
	// r0 shared a partial block with it. The second record also overlapped
	// into the block with the third record r2. Recovery should jump to that
	// block, skipping over the end of the second record and start parsing the
	// third record.
	r2, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}
	r2Data, _ := io.ReadAll(r2)
	if !bytes.Equal(r2Data, recs.records[2]) {
		t.Fatal("Unexpected output in r2's data")
	}
}

func TestRecoverMultipleBlocks(t *testing.T) {
	recs, err := makeTestRecords(
		// The first record will consume 3 entire blocks but a fraction of the 4th.
		blockSize*3,
		// The second record will completely fill the remainder of the 4th block.
		3*(blockSize-legacyHeaderSize)-2*blockSize-2*legacyHeaderSize,
		// Consume the entirety of the 5th block.
		blockSize-legacyHeaderSize,
		// Consume the entirety of the 6th block.
		blockSize-legacyHeaderSize,
		// Consume roughly half of the 7th block.
		blockSize/2,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	// Corrupt the checksum for the portion of the first record that exists in the 4th block.
	corruptBlock(recs.buf, 3)

	// Now corrupt the two blocks in a row that correspond to recs.records[2:4].
	corruptBlock(recs.buf, 4)
	corruptBlock(recs.buf, 5)

	// The first record should fail, but only when we read deeper beyond the first block.
	r := NewReader(bytes.NewReader(recs.buf), 0 /* logNum */)
	r0, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}

	// Reading deeper should yield a checksum mismatch.
	_, err = io.ReadAll(r0)
	if err == nil {
		t.Fatal("Exptected a checksum mismatch error, got nil")
	}
	if err != ErrInvalidChunk {
		t.Fatalf("Unexpected error returned: %v", err)
	}

	// Recover from that checksum mismatch.
	r.recover()

	// All of the data in the second record is lost because the first
	// record shared a partial block with it. The following two records
	// have corrupted checksums as well, so the call above to r.Recover
	// should result in r.Next() being a reader to the 5th record.
	r4, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}

	r4Data, _ := io.ReadAll(r4)
	if !bytes.Equal(r4Data, recs.records[4]) {
		t.Fatal("Unexpected output in r4's data")
	}
}

// verifyLastBlockRecover reads each record from recs expecting that the
// last record will be corrupted. It will then try Recover and verify that EOF
// is returned.
func verifyLastBlockRecover(recs *testRecords) error {
	r := NewReader(bytes.NewReader(recs.buf), 0 /* logNum */)
	// Loop to one element larger than the number of records to verify EOF.
	for i := 0; i < len(recs.records)+1; i++ {
		_, err := r.Next()
		switch i {
		case len(recs.records) - 1:
			if err == nil {
				return errors.New("Expected a checksum mismatch error, got nil")
			}
			r.recover()
		case len(recs.records):
			if err != io.EOF {
				return errors.Errorf("Expected io.EOF, got %v", err)
			}
		default:
			if err != nil {
				return errors.Errorf("Next: %v", err)
			}
		}
	}
	return nil
}

func TestRecoverLastPartialBlock(t *testing.T) {
	recs, err := makeTestRecords(
		// The first record will consume 3 entire blocks but a fraction of the 4th.
		blockSize*3,
		// The second record will completely fill the remainder of the 4th block.
		3*(blockSize-legacyHeaderSize)-2*blockSize-2*legacyHeaderSize,
		// Consume roughly half of the 5th block.
		blockSize/2,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	// Corrupt the 5th block.
	corruptBlock(recs.buf, 4)

	// Verify Recover works when the last block is corrupted.
	if err := verifyLastBlockRecover(recs); err != nil {
		t.Fatalf("verifyLastBlockRecover: %v", err)
	}
}

func TestRecoverLastCompleteBlock(t *testing.T) {
	recs, err := makeTestRecords(
		// The first record will consume 3 entire blocks but a fraction of the 4th.
		blockSize*3,
		// The second record will completely fill the remainder of the 4th block.
		3*(blockSize-legacyHeaderSize)-2*blockSize-2*legacyHeaderSize,
		// Consume the entire 5th block.
		blockSize-legacyHeaderSize,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	// Corrupt the 5th block.
	corruptBlock(recs.buf, 4)

	// Verify Recover works when the last block is corrupted.
	if err := verifyLastBlockRecover(recs); err != nil {
		t.Fatalf("verifyLastBlockRecover: %v", err)
	}
}

func TestReaderOffset(t *testing.T) {
	recs, err := makeTestRecords(
		blockSize*2,
		400,
		500,
		600,
		700,
		800,
		9000,
		1000,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	// The first record should fail, but only when we read deeper beyond the first block.
	r := NewReader(bytes.NewReader(recs.buf), 0 /* logNum */)
	for i, offset := range recs.offsets {
		if offset != r.Offset() {
			t.Fatalf("%d: expected offset %d, but found %d", i, offset, r.Offset())
		}
		rec, err := r.Next()
		if err != nil {
			t.Fatalf("Next: %v", err)
		}
		if _, err = io.ReadAll(rec); err != nil {
			t.Fatalf("ReadAll: %v", err)
		}
	}
}

func TestSeekRecord(t *testing.T) {
	recs, err := makeTestRecords(
		// The first record will consume 3 entire blocks but a fraction of the 4th.
		blockSize*3,
		// The second record will completely fill the remainder of the 4th block.
		3*(blockSize-legacyHeaderSize)-2*blockSize-2*legacyHeaderSize,
		// Consume the entirety of the 5th block.
		blockSize-legacyHeaderSize,
		// Consume the entirety of the 6th block.
		blockSize-legacyHeaderSize,
		// Consume roughly half of the 7th block.
		blockSize/2,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	r := NewReader(bytes.NewReader(recs.buf), 0 /* logNum */)
	// Seek to a valid block offset, but within a multiblock record. This should cause the next call to
	// Next after SeekRecord to return the next valid FIRST/FULL chunk of the subsequent record.
	err = r.seekRecord(blockSize)
	if err != nil {
		t.Fatalf("SeekRecord: %v", err)
	}
	rec, err := r.Next()
	if err != nil {
		t.Fatalf("Next: %v", err)
	}
	rData, _ := io.ReadAll(rec)
	if !bytes.Equal(rData, recs.records[1]) {
		t.Fatalf("Unexpected output in record 1's data, got %v want %v", rData, recs.records[1])
	}

	// Seek 3 bytes into the second block, which is still in the middle of the first record, but not
	// at a valid chunk boundary. Should result in an error upon calling r.Next.
	err = r.seekRecord(blockSize + 3)
	if err != nil {
		t.Fatalf("SeekRecord: %v", err)
	}
	if _, err = r.Next(); err == nil {
		t.Fatalf("Expected an error seeking to an invalid chunk boundary")
	}
	r.recover()

	// Seek to the fifth block and verify all records can be read as appropriate.
	err = r.seekRecord(blockSize * 4)
	if err != nil {
		t.Fatalf("SeekRecord: %v", err)
	}

	check := func(i int) {
		for ; i < len(recs.records); i++ {
			rec, err := r.Next()
			if err != nil {
				t.Fatalf("Next: %v", err)
			}

			rData, _ := io.ReadAll(rec)
			if !bytes.Equal(rData, recs.records[i]) {
				t.Fatalf("Unexpected output in record #%d's data, got %v want %v", i, rData, recs.records[i])
			}
		}
	}
	check(2)

	// Seek back to the fourth block, and read all subsequent records and verify them.
	err = r.seekRecord(blockSize * 3)
	if err != nil {
		t.Fatalf("SeekRecord: %v", err)
	}
	check(1)

	// Now seek past the end of the file and verify it causes an error.
	err = r.seekRecord(1 << 20)
	if err == nil {
		t.Fatalf("Seek past the end of a file didn't cause an error")
	}
	if err != io.ErrUnexpectedEOF {
		t.Fatalf("Seeking past EOF raised unexpected error: %v", err)
	}
	r.recover() // Verify recovery works.

	// Validate the current records are returned after seeking to a valid offset.
	err = r.seekRecord(blockSize * 4)
	if err != nil {
		t.Fatalf("SeekRecord: %v", err)
	}
	check(2)
}

func TestLastRecordOffset(t *testing.T) {
	recs, err := makeTestRecords(
		// The first record will consume 3 entire blocks but a fraction of the 4th.
		blockSize*3,
		// The second record will completely fill the remainder of the 4th block.
		3*(blockSize-legacyHeaderSize)-2*blockSize-2*legacyHeaderSize,
		// Consume the entirety of the 5th block.
		blockSize-legacyHeaderSize,
		// Consume the entirety of the 6th block.
		blockSize-legacyHeaderSize,
		// Consume roughly half of the 7th block.
		blockSize/2,
	)
	if err != nil {
		t.Fatalf("makeTestRecords: %v", err)
	}

	wants := []int64{0, 98332, 131072, 163840, 196608}
	for i, got := range recs.offsets {
		if want := wants[i]; got != want {
			t.Errorf("record #%d: got %d, want %d", i, got, want)
		}
	}
}

func TestNoLastRecordOffset(t *testing.T) {
	buf := new(bytes.Buffer)
	w := NewWriter(buf)
	defer w.Close()

	if _, err := w.LastRecordOffset(); err != ErrNoLastRecord {
		t.Fatalf("Expected ErrNoLastRecord, got: %v", err)
	}

	require.NoError(t, w.Flush())

	if _, err := w.LastRecordOffset(); err != ErrNoLastRecord {
		t.Fatalf("LastRecordOffset: got: %v, want ErrNoLastRecord", err)
	}

	_, err := w.WriteRecord([]byte("testrecord"))
	require.NoError(t, err)

	if off, err := w.LastRecordOffset(); err != nil {
		t.Fatalf("LastRecordOffset: %v", err)
	} else if off != 0 {
		t.Fatalf("LastRecordOffset: got %d, want 0", off)
	}
}

func TestInvalidLogNum(t *testing.T) {
	var buf bytes.Buffer
	w := NewLogWriter(&buf, 1, LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	for i := 0; i < 10; i++ {
		s := fmt.Sprintf("%04d\n", i)
		_, err := w.WriteRecord([]byte(s))
		require.NoError(t, err)
	}
	require.NoError(t, w.Close())

	{
		r := NewReader(bytes.NewReader(buf.Bytes()), 1)
		for i := 0; i < 10; i++ {
			rr, err := r.Next()
			require.NoError(t, err)

			x, err := io.ReadAll(rr)
			require.NoError(t, err)

			s := fmt.Sprintf("%04d\n", i)
			if s != string(x) {
				t.Fatalf("expected %s, but found %s", s, x)
			}
		}
		if _, err := r.Next(); err != io.EOF {
			t.Fatalf("expected EOF, but found %s", err)
		}
	}

	{
		r := NewReader(bytes.NewReader(buf.Bytes()), 2)
		if _, err := r.Next(); err != io.EOF {
			t.Fatalf("expected %s, but found %s\n", io.EOF, err)
		}
	}
}

func TestSize(t *testing.T) {
	var buf bytes.Buffer
	zeroes := make([]byte, 8<<10)
	w := NewWriter(&buf)
	for i := 0; i < 100; i++ {
		n := rand.Intn(len(zeroes))
		_, err := w.WriteRecord(zeroes[:n])
		require.NoError(t, err)
		require.NoError(t, w.Flush())
		if buf.Len() != int(w.Size()) {
			t.Fatalf("expected %d, but found %d", buf.Len(), w.Size())
		}
	}
	require.NoError(t, w.Close())
}

type limitedWriter struct {
	io.Writer
	limit int
}

func (w *limitedWriter) Write(p []byte) (n int, err error) {
	w.limit--
	if w.limit < 0 {
		return len(p), nil
	}
	return w.Writer.Write(p)
}

func TestRecycleLog(t *testing.T) {
	const min = 16
	const max = 4096

	rnd := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
	randBlock := func() []byte {
		data := make([]byte, rand.Intn(max-min)+min)
		tmp := data
		for len(tmp) >= 8 {
			binary.LittleEndian.PutUint64(tmp, rand.Uint64())
			tmp = tmp[8:]
		}
		r := rand.Uint64()
		for i := 0; i < len(tmp); i++ {
			tmp[i] = byte(r)
			r >>= 8
		}
		return data
	}

	// Recycle a log file 100 times, writing a random number of records filled
	// with random data.
	backing := make([]byte, 1<<20)
	for i := 1; i <= 100; i++ {
		blocks := rnd.Intn(100)
		limitedBuf := &limitedWriter{
			Writer: bytes.NewBuffer(backing[:0]),
			limit:  blocks,
		}

		w := NewLogWriter(limitedBuf, base.FileNum(i), LogWriterConfig{
			WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
		sizes := make([]int, 10+rnd.Intn(100))
		for j := range sizes {
			data := randBlock()
			if _, err := w.WriteRecord(data); err != nil {
				t.Fatalf("%d/%d: %v", i, j, err)
			}
			sizes[j] = len(data)
		}
		if err := w.Close(); err != nil {
			t.Fatalf("%d: %v", i, err)
		}

		r := NewReader(bytes.NewReader(backing), base.FileNum(i))
		for j := range sizes {
			rr, err := r.Next()
			if err != nil {
				// If we limited output then an EOF, zeroed, or invalid chunk is expected.
				if limitedBuf.limit < 0 && (err == io.EOF || err == ErrZeroedChunk || err == ErrInvalidChunk) {
					break
				}
				t.Fatalf("%d/%d: %v", i, j, err)
			}
			x, err := io.ReadAll(rr)
			if err != nil {
				// If we limited output then an EOF, zeroed, or invalid chunk is expected.
				if limitedBuf.limit < 0 && (err == io.EOF || err == ErrZeroedChunk || err == ErrInvalidChunk) {
					break
				}
				t.Fatalf("%d/%d: %v", i, j, err)
			}
			if sizes[j] != len(x) {
				t.Fatalf("%d/%d: expected record %d, but found %d", i, j, sizes[j], len(x))
			}
		}
		if _, err := r.Next(); err != io.EOF && err != ErrZeroedChunk && err != ErrInvalidChunk {
			t.Fatalf("%d: expected EOF, but found %v", i, err)
		}
	}
}

func TestTruncatedLog(t *testing.T) {
	backing := make([]byte, 2*blockSize)
	w := NewLogWriter(bytes.NewBuffer(backing[:0]), base.FileNum(1), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	// Write a record that spans 2 blocks.
	_, err := w.WriteRecord(bytes.Repeat([]byte("s"), blockSize+100))
	require.NoError(t, err)
	require.NoError(t, w.Close())
	// Create a reader only for the first block.
	r := NewReader(bytes.NewReader(backing[:blockSize]), base.FileNum(1))
	rr, err := r.Next()
	require.NoError(t, err)
	_, err = io.ReadAll(rr)
	require.EqualValues(t, err, io.ErrUnexpectedEOF)
}

func TestRecycleLogWithPartialBlock(t *testing.T) {
	backing := make([]byte, 27)
	w := NewLogWriter(bytes.NewBuffer(backing[:0]), base.FileNum(1), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	// Will write a chunk with 11 byte header + 5 byte payload.
	_, err := w.WriteRecord([]byte("aaaaa"))
	require.NoError(t, err)
	// Close will write a 11-byte EOF chunk.
	require.NoError(t, w.Close())

	w = NewLogWriter(bytes.NewBuffer(backing[:0]), base.FileNum(2), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	// Will write a chunk with 11 byte header + 1 byte payload.
	_, err = w.WriteRecord([]byte("a"))
	require.NoError(t, err)
	// Close will write a 11-byte EOF chunk.
	require.NoError(t, w.Close())

	r := NewReader(bytes.NewReader(backing), base.FileNum(2))
	_, err = r.Next()
	require.NoError(t, err)
	// 4 bytes left, which are not enough for even the legacy header.
	if _, err = r.Next(); err != io.EOF {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRecycleLogNumberOverflow(t *testing.T) {
	// We truncate log numbers to 32-bits when writing to the WAL. Test log
	// recycling at the wraparound point, ensuring that EOF chunks are
	// interpreted correctly.

	backing := make([]byte, 27)
	w := NewLogWriter(bytes.NewBuffer(backing[:0]), base.FileNum(math.MaxUint32), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	// Will write a chunk with 11 byte header + 5 byte payload.
	_, err := w.WriteRecord([]byte("aaaaa"))
	require.NoError(t, err)
	// Close will write a 11-byte EOF chunk.
	require.NoError(t, w.Close())

	w = NewLogWriter(bytes.NewBuffer(backing[:0]), base.FileNum(math.MaxUint32+1), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	// Will write a chunk with 11 byte header + 1 byte payload.
	_, err = w.WriteRecord([]byte("a"))
	require.NoError(t, err)
	// Close will write a 11-byte EOF chunk.
	require.NoError(t, w.Close())

	r := NewReader(bytes.NewReader(backing), base.FileNum(math.MaxUint32+1))
	_, err = r.Next()
	require.NoError(t, err)
	// 4 bytes left, which are not enough for even the legacy header.
	if _, err = r.Next(); err != io.EOF {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRecycleLogWithPartialRecord(t *testing.T) {
	const recordSize = (blockSize * 3) / 2

	// Write a record that is larger than the log block size.
	backing1 := make([]byte, 2*blockSize)
	w := NewLogWriter(bytes.NewBuffer(backing1[:0]), base.FileNum(1), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	_, err := w.WriteRecord(bytes.Repeat([]byte("a"), recordSize))
	require.NoError(t, err)
	require.NoError(t, w.Close())

	// Write another record to a new incarnation of the WAL that is larger than
	// the block size.
	backing2 := make([]byte, 2*blockSize)
	w = NewLogWriter(bytes.NewBuffer(backing2[:0]), base.FileNum(2), LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
	_, err = w.WriteRecord(bytes.Repeat([]byte("b"), recordSize))
	require.NoError(t, err)
	require.NoError(t, w.Close())

	// Copy the second block from the first WAL to the second block of the second
	// WAL. This produces a scenario where it appears we crashed after writing
	// the first block of the second WAL, but before writing the second block.
	copy(backing2[blockSize:], backing1[blockSize:])

	// Verify that we can't read a partial record from the second WAL.
	r := NewReader(bytes.NewReader(backing2), base.FileNum(2))
	rr, err := r.Next()
	require.NoError(t, err)

	_, err = io.ReadAll(rr)
	require.Equal(t, err, ErrInvalidChunk)
}

func BenchmarkRecordWrite(b *testing.B) {
	for _, size := range []int{8, 16, 32, 64, 256, 1028, 4096, 65_536} {
		b.Run(fmt.Sprintf("size=%d", size), func(b *testing.B) {
			w := NewLogWriter(io.Discard, 0 /* logNum */, LogWriterConfig{
				WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{})})
			defer w.Close()
			buf := make([]byte, size)

			b.SetBytes(int64(len(buf)))
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				if _, err := w.WriteRecord(buf); err != nil {
					b.Fatal(err)
				}
			}
			b.StopTimer()
		})
	}
}
