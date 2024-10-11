// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rawdb

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"reflect"
	"testing"
	"testing/quick"

	"github.com/davecgh/go-spew/spew"
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/stretchr/testify/require"
)

// TestFreezerBasics test initializing a freezertable from scratch, writing to the table,
// and reading it back.
func TestFreezerBasics(t *testing.T) {
	t.Parallel()
	// set cutoff at 50 bytes
	f, err := newTable(os.TempDir(),
		fmt.Sprintf("unittest-%d", rand.Uint64()),
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, false)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	// Write 15 bytes 255 times, results in 85 files
	writeChunks(t, f, 255, 15)

	//print(t, f, 0)
	//print(t, f, 1)
	//print(t, f, 2)
	//
	//db[0] =  000000000000000000000000000000
	//db[1] =  010101010101010101010101010101
	//db[2] =  020202020202020202020202020202

	for y := 0; y < 255; y++ {
		exp := getChunk(15, y)
		got, err := f.Retrieve(uint64(y))
		if err != nil {
			t.Fatalf("reading item %d: %v", y, err)
		}
		if !bytes.Equal(got, exp) {
			t.Fatalf("test %d, got \n%x != \n%x", y, got, exp)
		}
	}
	// Check that we cannot read too far
	_, err = f.Retrieve(uint64(255))
	if err != errOutOfBounds {
		t.Fatal(err)
	}
}

// TestFreezerBasicsClosing tests same as TestFreezerBasics, but also closes and reopens the freezer between
// every operation
func TestFreezerBasicsClosing(t *testing.T) {
	t.Parallel()
	// set cutoff at 50 bytes
	var (
		fname      = fmt.Sprintf("basics-close-%d", rand.Uint64())
		rm, wm, sg = metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
		f          *freezerTable
		err        error
	)
	f, err = newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
	if err != nil {
		t.Fatal(err)
	}

	// Write 15 bytes 255 times, results in 85 files.
	// In-between writes, the table is closed and re-opened.
	for x := 0; x < 255; x++ {
		data := getChunk(15, x)
		batch := f.newBatch()
		require.NoError(t, batch.AppendRaw(uint64(x), data))
		require.NoError(t, batch.commit())
		f.Close()

		f, err = newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
	}
	defer f.Close()

	for y := 0; y < 255; y++ {
		exp := getChunk(15, y)
		got, err := f.Retrieve(uint64(y))
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(got, exp) {
			t.Fatalf("test %d, got \n%x != \n%x", y, got, exp)
		}
		f.Close()
		f, err = newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
	}
}

// TestFreezerRepairDanglingHead tests that we can recover if index entries are removed
func TestFreezerRepairDanglingHead(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("dangling_headtest-%d", rand.Uint64())

	// Fill table
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 255 times
		writeChunks(t, f, 255, 15)

		// The last item should be there
		if _, err = f.Retrieve(0xfe); err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	// open the index
	idxFile, err := os.OpenFile(filepath.Join(os.TempDir(), fmt.Sprintf("%s.ridx", fname)), os.O_RDWR, 0644)
	if err != nil {
		t.Fatalf("Failed to open index file: %v", err)
	}
	// Remove 4 bytes
	stat, err := idxFile.Stat()
	if err != nil {
		t.Fatalf("Failed to stat index file: %v", err)
	}
	idxFile.Truncate(stat.Size() - 4)
	idxFile.Close()

	// Now open it again
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// The last item should be missing
		if _, err = f.Retrieve(0xff); err == nil {
			t.Errorf("Expected error for missing index entry")
		}
		// The one before should still be there
		if _, err = f.Retrieve(0xfd); err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
	}
}

// TestFreezerRepairDanglingHeadLarge tests that we can recover if very many index entries are removed
func TestFreezerRepairDanglingHeadLarge(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("dangling_headtest-%d", rand.Uint64())

	// Fill a table and close it
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 255 times
		writeChunks(t, f, 255, 15)

		// The last item should be there
		if _, err = f.Retrieve(f.items.Load() - 1); err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	// open the index
	idxFile, err := os.OpenFile(filepath.Join(os.TempDir(), fmt.Sprintf("%s.ridx", fname)), os.O_RDWR, 0644)
	if err != nil {
		t.Fatalf("Failed to open index file: %v", err)
	}
	// Remove everything but the first item, and leave data unaligned
	// 0-indexEntry, 1-indexEntry, corrupt-indexEntry
	idxFile.Truncate(2*indexEntrySize + indexEntrySize/2)
	idxFile.Close()

	// Now open it again
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// The first item should be there
		if _, err = f.Retrieve(0); err != nil {
			t.Fatal(err)
		}
		// The second item should be missing
		if _, err = f.Retrieve(1); err == nil {
			t.Errorf("Expected error for missing index entry")
		}
		// We should now be able to store items again, from item = 1
		batch := f.newBatch()
		for x := 1; x < 0xff; x++ {
			require.NoError(t, batch.AppendRaw(uint64(x), getChunk(15, ^x)))
		}
		require.NoError(t, batch.commit())
		f.Close()
	}

	// And if we open it, we should now be able to read all of them (new values)
	{
		f, _ := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		for y := 1; y < 255; y++ {
			exp := getChunk(15, ^y)
			got, err := f.Retrieve(uint64(y))
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(got, exp) {
				t.Fatalf("test %d, got \n%x != \n%x", y, got, exp)
			}
		}
	}
}

// TestSnappyDetection tests that we fail to open a snappy database and vice versa
func TestSnappyDetection(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("snappytest-%d", rand.Uint64())

	// Open with snappy
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 255 times
		writeChunks(t, f, 255, 15)
		f.Close()
	}

	// Open without snappy
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, false, false)
		if err != nil {
			t.Fatal(err)
		}
		if _, err = f.Retrieve(0); err == nil {
			f.Close()
			t.Fatalf("expected empty table")
		}
	}

	// Open with snappy
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// There should be 255 items
		if _, err = f.Retrieve(0xfe); err != nil {
			f.Close()
			t.Fatalf("expected no error, got %v", err)
		}
	}
}

func assertFileSize(f string, size int64) error {
	stat, err := os.Stat(f)
	if err != nil {
		return err
	}
	if stat.Size() != size {
		return fmt.Errorf("error, expected size %d, got %d", size, stat.Size())
	}
	return nil
}

// TestFreezerRepairDanglingIndex checks that if the index has more entries than there are data,
// the index is repaired
func TestFreezerRepairDanglingIndex(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("dangling_indextest-%d", rand.Uint64())

	// Fill a table and close it
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 9 times : 150 bytes
		writeChunks(t, f, 9, 15)

		// The last item should be there
		if _, err = f.Retrieve(f.items.Load() - 1); err != nil {
			f.Close()
			t.Fatal(err)
		}
		f.Close()
		// File sizes should be 45, 45, 45 : items[3, 3, 3)
	}

	// Crop third file
	fileToCrop := filepath.Join(os.TempDir(), fmt.Sprintf("%s.0002.rdat", fname))
	// Truncate third file: 45 ,45, 20
	{
		if err := assertFileSize(fileToCrop, 45); err != nil {
			t.Fatal(err)
		}
		file, err := os.OpenFile(fileToCrop, os.O_RDWR, 0644)
		if err != nil {
			t.Fatal(err)
		}
		file.Truncate(20)
		file.Close()
	}

	// Open db it again
	// It should restore the file(s) to
	// 45, 45, 15
	// with 3+3+1 items
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		defer f.Close()
		if f.items.Load() != 7 {
			t.Fatalf("expected %d items, got %d", 7, f.items.Load())
		}
		if err := assertFileSize(fileToCrop, 15); err != nil {
			t.Fatal(err)
		}
	}
}

func TestFreezerTruncate(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("truncation-%d", rand.Uint64())

	// Fill table
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 30 times
		writeChunks(t, f, 30, 15)

		// The last item should be there
		if _, err = f.Retrieve(f.items.Load() - 1); err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	// Reopen, truncate
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		defer f.Close()
		f.truncateHead(10) // 150 bytes
		if f.items.Load() != 10 {
			t.Fatalf("expected %d items, got %d", 10, f.items.Load())
		}
		// 45, 45, 45, 15 -- bytes should be 15
		if f.headBytes != 15 {
			t.Fatalf("expected %d bytes, got %d", 15, f.headBytes)
		}
	}
}

// TestFreezerRepairFirstFile tests a head file with the very first item only half-written.
// That will rewind the index, and _should_ truncate the head file
func TestFreezerRepairFirstFile(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("truncationfirst-%d", rand.Uint64())

	// Fill table
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 80 bytes, splitting out into two files
		batch := f.newBatch()
		require.NoError(t, batch.AppendRaw(0, getChunk(40, 0xFF)))
		require.NoError(t, batch.AppendRaw(1, getChunk(40, 0xEE)))
		require.NoError(t, batch.commit())

		// The last item should be there
		if _, err = f.Retrieve(1); err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	// Truncate the file in half
	fileToCrop := filepath.Join(os.TempDir(), fmt.Sprintf("%s.0001.rdat", fname))
	{
		if err := assertFileSize(fileToCrop, 40); err != nil {
			t.Fatal(err)
		}
		file, err := os.OpenFile(fileToCrop, os.O_RDWR, 0644)
		if err != nil {
			t.Fatal(err)
		}
		file.Truncate(20)
		file.Close()
	}

	// Reopen
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		if f.items.Load() != 1 {
			f.Close()
			t.Fatalf("expected %d items, got %d", 0, f.items.Load())
		}

		// Write 40 bytes
		batch := f.newBatch()
		require.NoError(t, batch.AppendRaw(1, getChunk(40, 0xDD)))
		require.NoError(t, batch.commit())

		f.Close()

		// Should have been truncated down to zero and then 40 written
		if err := assertFileSize(fileToCrop, 40); err != nil {
			t.Fatal(err)
		}
	}
}

// TestFreezerReadAndTruncate tests:
// - we have a table open
// - do some reads, so files are open in readonly
// - truncate so those files are 'removed'
// - check that we did not keep the rdonly file descriptors
func TestFreezerReadAndTruncate(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("read_truncate-%d", rand.Uint64())

	// Fill table
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 30 times
		writeChunks(t, f, 30, 15)

		// The last item should be there
		if _, err = f.Retrieve(f.items.Load() - 1); err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	// Reopen and read all files
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		if f.items.Load() != 30 {
			f.Close()
			t.Fatalf("expected %d items, got %d", 0, f.items.Load())
		}
		for y := byte(0); y < 30; y++ {
			f.Retrieve(uint64(y))
		}

		// Now, truncate back to zero
		f.truncateHead(0)

		// Write the data again
		batch := f.newBatch()
		for x := 0; x < 30; x++ {
			require.NoError(t, batch.AppendRaw(uint64(x), getChunk(15, ^x)))
		}
		require.NoError(t, batch.commit())
		f.Close()
	}
}

func TestFreezerOffset(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("offset-%d", rand.Uint64())

	// Fill table
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
		if err != nil {
			t.Fatal(err)
		}

		// Write 6 x 20 bytes, splitting out into three files
		batch := f.newBatch()
		require.NoError(t, batch.AppendRaw(0, getChunk(20, 0xFF)))
		require.NoError(t, batch.AppendRaw(1, getChunk(20, 0xEE)))

		require.NoError(t, batch.AppendRaw(2, getChunk(20, 0xdd)))
		require.NoError(t, batch.AppendRaw(3, getChunk(20, 0xcc)))

		require.NoError(t, batch.AppendRaw(4, getChunk(20, 0xbb)))
		require.NoError(t, batch.AppendRaw(5, getChunk(20, 0xaa)))
		require.NoError(t, batch.commit())

		t.Log(f.dumpIndexString(0, 100))
		f.Close()
	}

	// Now crop it.
	{
		// delete files 0 and 1
		for i := 0; i < 2; i++ {
			p := filepath.Join(os.TempDir(), fmt.Sprintf("%v.%04d.rdat", fname, i))
			if err := os.Remove(p); err != nil {
				t.Fatal(err)
			}
		}
		// Read the index file
		p := filepath.Join(os.TempDir(), fmt.Sprintf("%v.ridx", fname))
		indexFile, err := os.OpenFile(p, os.O_RDWR, 0644)
		if err != nil {
			t.Fatal(err)
		}
		indexBuf := make([]byte, 7*indexEntrySize)
		indexFile.Read(indexBuf)

		// Update the index file, so that we store
		// [ file = 2, offset = 4 ] at index zero

		zeroIndex := indexEntry{
			filenum: uint32(2), // First file is 2
			offset:  uint32(4), // We have removed four items
		}
		buf := zeroIndex.append(nil)

		// Overwrite index zero
		copy(indexBuf, buf)

		// Remove the four next indices by overwriting
		copy(indexBuf[indexEntrySize:], indexBuf[indexEntrySize*5:])
		indexFile.WriteAt(indexBuf, 0)

		// Need to truncate the moved index items
		indexFile.Truncate(indexEntrySize * (1 + 2))
		indexFile.Close()
	}

	// Now open again
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
		if err != nil {
			t.Fatal(err)
		}
		defer f.Close()
		t.Log(f.dumpIndexString(0, 100))

		// It should allow writing item 6.
		batch := f.newBatch()
		require.NoError(t, batch.AppendRaw(6, getChunk(20, 0x99)))
		require.NoError(t, batch.commit())

		checkRetrieveError(t, f, map[uint64]error{
			0: errOutOfBounds,
			1: errOutOfBounds,
			2: errOutOfBounds,
			3: errOutOfBounds,
		})
		checkRetrieve(t, f, map[uint64][]byte{
			4: getChunk(20, 0xbb),
			5: getChunk(20, 0xaa),
			6: getChunk(20, 0x99),
		})
	}

	// Edit the index again, with a much larger initial offset of 1M.
	{
		// Read the index file
		p := filepath.Join(os.TempDir(), fmt.Sprintf("%v.ridx", fname))
		indexFile, err := os.OpenFile(p, os.O_RDWR, 0644)
		if err != nil {
			t.Fatal(err)
		}
		indexBuf := make([]byte, 3*indexEntrySize)
		indexFile.Read(indexBuf)

		// Update the index file, so that we store
		// [ file = 2, offset = 1M ] at index zero

		zeroIndex := indexEntry{
			offset:  uint32(1000000), // We have removed 1M items
			filenum: uint32(2),       // First file is 2
		}
		buf := zeroIndex.append(nil)

		// Overwrite index zero
		copy(indexBuf, buf)
		indexFile.WriteAt(indexBuf, 0)
		indexFile.Close()
	}

	// Check that existing items have been moved to index 1M.
	{
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
		if err != nil {
			t.Fatal(err)
		}
		defer f.Close()
		t.Log(f.dumpIndexString(0, 100))

		checkRetrieveError(t, f, map[uint64]error{
			0:      errOutOfBounds,
			1:      errOutOfBounds,
			2:      errOutOfBounds,
			3:      errOutOfBounds,
			999999: errOutOfBounds,
		})
		checkRetrieve(t, f, map[uint64][]byte{
			1000000: getChunk(20, 0xbb),
			1000001: getChunk(20, 0xaa),
		})
	}
}

func TestTruncateTail(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("truncate-tail-%d", rand.Uint64())

	// Fill table
	f, err := newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
	if err != nil {
		t.Fatal(err)
	}

	// Write 7 x 20 bytes, splitting out into four files
	batch := f.newBatch()
	require.NoError(t, batch.AppendRaw(0, getChunk(20, 0xFF)))
	require.NoError(t, batch.AppendRaw(1, getChunk(20, 0xEE)))
	require.NoError(t, batch.AppendRaw(2, getChunk(20, 0xdd)))
	require.NoError(t, batch.AppendRaw(3, getChunk(20, 0xcc)))
	require.NoError(t, batch.AppendRaw(4, getChunk(20, 0xbb)))
	require.NoError(t, batch.AppendRaw(5, getChunk(20, 0xaa)))
	require.NoError(t, batch.AppendRaw(6, getChunk(20, 0x11)))
	require.NoError(t, batch.commit())

	// nothing to do, all the items should still be there.
	f.truncateTail(0)
	fmt.Println(f.dumpIndexString(0, 1000))
	checkRetrieve(t, f, map[uint64][]byte{
		0: getChunk(20, 0xFF),
		1: getChunk(20, 0xEE),
		2: getChunk(20, 0xdd),
		3: getChunk(20, 0xcc),
		4: getChunk(20, 0xbb),
		5: getChunk(20, 0xaa),
		6: getChunk(20, 0x11),
	})

	// truncate single element( item 0 ), deletion is only supported at file level
	f.truncateTail(1)
	fmt.Println(f.dumpIndexString(0, 1000))
	checkRetrieveError(t, f, map[uint64]error{
		0: errOutOfBounds,
	})
	checkRetrieve(t, f, map[uint64][]byte{
		1: getChunk(20, 0xEE),
		2: getChunk(20, 0xdd),
		3: getChunk(20, 0xcc),
		4: getChunk(20, 0xbb),
		5: getChunk(20, 0xaa),
		6: getChunk(20, 0x11),
	})

	// Reopen the table, the deletion information should be persisted as well
	f.Close()
	f, err = newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
	if err != nil {
		t.Fatal(err)
	}
	checkRetrieveError(t, f, map[uint64]error{
		0: errOutOfBounds,
	})
	checkRetrieve(t, f, map[uint64][]byte{
		1: getChunk(20, 0xEE),
		2: getChunk(20, 0xdd),
		3: getChunk(20, 0xcc),
		4: getChunk(20, 0xbb),
		5: getChunk(20, 0xaa),
		6: getChunk(20, 0x11),
	})

	// truncate two elements( item 0, item 1 ), the file 0 should be deleted
	f.truncateTail(2)
	checkRetrieveError(t, f, map[uint64]error{
		0: errOutOfBounds,
		1: errOutOfBounds,
	})
	checkRetrieve(t, f, map[uint64][]byte{
		2: getChunk(20, 0xdd),
		3: getChunk(20, 0xcc),
		4: getChunk(20, 0xbb),
		5: getChunk(20, 0xaa),
		6: getChunk(20, 0x11),
	})

	// Reopen the table, the above testing should still pass
	f.Close()
	f, err = newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	checkRetrieveError(t, f, map[uint64]error{
		0: errOutOfBounds,
		1: errOutOfBounds,
	})
	checkRetrieve(t, f, map[uint64][]byte{
		2: getChunk(20, 0xdd),
		3: getChunk(20, 0xcc),
		4: getChunk(20, 0xbb),
		5: getChunk(20, 0xaa),
		6: getChunk(20, 0x11),
	})

	// truncate all, the entire freezer should be deleted
	f.truncateTail(7)
	checkRetrieveError(t, f, map[uint64]error{
		0: errOutOfBounds,
		1: errOutOfBounds,
		2: errOutOfBounds,
		3: errOutOfBounds,
		4: errOutOfBounds,
		5: errOutOfBounds,
		6: errOutOfBounds,
	})
}

func TestTruncateHead(t *testing.T) {
	t.Parallel()
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("truncate-head-blow-tail-%d", rand.Uint64())

	// Fill table
	f, err := newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
	if err != nil {
		t.Fatal(err)
	}

	// Write 7 x 20 bytes, splitting out into four files
	batch := f.newBatch()
	require.NoError(t, batch.AppendRaw(0, getChunk(20, 0xFF)))
	require.NoError(t, batch.AppendRaw(1, getChunk(20, 0xEE)))
	require.NoError(t, batch.AppendRaw(2, getChunk(20, 0xdd)))
	require.NoError(t, batch.AppendRaw(3, getChunk(20, 0xcc)))
	require.NoError(t, batch.AppendRaw(4, getChunk(20, 0xbb)))
	require.NoError(t, batch.AppendRaw(5, getChunk(20, 0xaa)))
	require.NoError(t, batch.AppendRaw(6, getChunk(20, 0x11)))
	require.NoError(t, batch.commit())

	f.truncateTail(4) // Tail = 4

	// NewHead is required to be 3, the entire table should be truncated
	f.truncateHead(4)
	checkRetrieveError(t, f, map[uint64]error{
		0: errOutOfBounds, // Deleted by tail
		1: errOutOfBounds, // Deleted by tail
		2: errOutOfBounds, // Deleted by tail
		3: errOutOfBounds, // Deleted by tail
		4: errOutOfBounds, // Deleted by Head
		5: errOutOfBounds, // Deleted by Head
		6: errOutOfBounds, // Deleted by Head
	})

	// Append new items
	batch = f.newBatch()
	require.NoError(t, batch.AppendRaw(4, getChunk(20, 0xbb)))
	require.NoError(t, batch.AppendRaw(5, getChunk(20, 0xaa)))
	require.NoError(t, batch.AppendRaw(6, getChunk(20, 0x11)))
	require.NoError(t, batch.commit())

	checkRetrieve(t, f, map[uint64][]byte{
		4: getChunk(20, 0xbb),
		5: getChunk(20, 0xaa),
		6: getChunk(20, 0x11),
	})
}

func checkRetrieve(t *testing.T, f *freezerTable, items map[uint64][]byte) {
	t.Helper()

	for item, wantBytes := range items {
		value, err := f.Retrieve(item)
		if err != nil {
			t.Fatalf("can't get expected item %d: %v", item, err)
		}
		if !bytes.Equal(value, wantBytes) {
			t.Fatalf("item %d has wrong value %x (want %x)", item, value, wantBytes)
		}
	}
}

func checkRetrieveError(t *testing.T, f *freezerTable, items map[uint64]error) {
	t.Helper()

	for item, wantError := range items {
		value, err := f.Retrieve(item)
		if err == nil {
			t.Fatalf("unexpected value %x for item %d, want error %v", item, value, wantError)
		}
		if err != wantError {
			t.Fatalf("wrong error for item %d: %v", item, err)
		}
	}
}

// Gets a chunk of data, filled with 'b'
func getChunk(size int, b int) []byte {
	data := make([]byte, size)
	for i := range data {
		data[i] = byte(b)
	}
	return data
}

// TODO (?)
// - test that if we remove several head-files, aswell as data last data-file,
//   the index is truncated accordingly
// Right now, the freezer would fail on these conditions:
// 1. have data files d0, d1, d2, d3
// 2. remove d2,d3
//
// However, all 'normal' failure modes arising due to failing to sync() or save a file
// should be handled already, and the case described above can only (?) happen if an
// external process/user deletes files from the filesystem.

func writeChunks(t *testing.T, ft *freezerTable, n int, length int) {
	t.Helper()

	batch := ft.newBatch()
	for i := 0; i < n; i++ {
		if err := batch.AppendRaw(uint64(i), getChunk(length, i)); err != nil {
			t.Fatalf("AppendRaw(%d, ...) returned error: %v", i, err)
		}
	}
	if err := batch.commit(); err != nil {
		t.Fatalf("Commit returned error: %v", err)
	}
}

// TestSequentialRead does some basic tests on the RetrieveItems.
func TestSequentialRead(t *testing.T) {
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("batchread-%d", rand.Uint64())
	{ // Fill table
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 15 bytes 30 times
		writeChunks(t, f, 30, 15)
		f.dumpIndexStdout(0, 30)
		f.Close()
	}
	{ // Open it, iterate, verify iteration
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 50, true, false)
		if err != nil {
			t.Fatal(err)
		}
		items, err := f.RetrieveItems(0, 10000, 100000)
		if err != nil {
			t.Fatal(err)
		}
		if have, want := len(items), 30; have != want {
			t.Fatalf("want %d items, have %d ", want, have)
		}
		for i, have := range items {
			want := getChunk(15, i)
			if !bytes.Equal(want, have) {
				t.Fatalf("data corruption: have\n%x\n, want \n%x\n", have, want)
			}
		}
		f.Close()
	}
	{ // Open it, iterate, verify byte limit. The byte limit is less than item
		// size, so each lookup should only return one item
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 40, true, false)
		if err != nil {
			t.Fatal(err)
		}
		items, err := f.RetrieveItems(0, 10000, 10)
		if err != nil {
			t.Fatal(err)
		}
		if have, want := len(items), 1; have != want {
			t.Fatalf("want %d items, have %d ", want, have)
		}
		for i, have := range items {
			want := getChunk(15, i)
			if !bytes.Equal(want, have) {
				t.Fatalf("data corruption: have\n%x\n, want \n%x\n", have, want)
			}
		}
		f.Close()
	}
}

// TestSequentialReadByteLimit does some more advanced tests on batch reads.
// These tests check that when the byte limit hits, we correctly abort in time,
// but also properly do all the deferred reads for the previous data, regardless
// of whether the data crosses a file boundary or not.
func TestSequentialReadByteLimit(t *testing.T) {
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("batchread-2-%d", rand.Uint64())
	{ // Fill table
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 100, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 10 bytes 30 times,
		// Splitting it at every 100 bytes (10 items)
		writeChunks(t, f, 30, 10)
		f.Close()
	}
	for i, tc := range []struct {
		items uint64
		limit uint64
		want  int
	}{
		{9, 89, 8},
		{10, 99, 9},
		{11, 109, 10},
		{100, 89, 8},
		{100, 99, 9},
		{100, 109, 10},
	} {
		{
			f, err := newTable(os.TempDir(), fname, rm, wm, sg, 100, true, false)
			if err != nil {
				t.Fatal(err)
			}
			items, err := f.RetrieveItems(0, tc.items, tc.limit)
			if err != nil {
				t.Fatal(err)
			}
			if have, want := len(items), tc.want; have != want {
				t.Fatalf("test %d: want %d items, have %d ", i, want, have)
			}
			for ii, have := range items {
				want := getChunk(10, ii)
				if !bytes.Equal(want, have) {
					t.Fatalf("test %d: data corruption item %d: have\n%x\n, want \n%x\n", i, ii, have, want)
				}
			}
			f.Close()
		}
	}
}

// TestSequentialReadNoByteLimit tests the batch-read if maxBytes is not specified.
// Freezer should return the requested items regardless the size limitation.
func TestSequentialReadNoByteLimit(t *testing.T) {
	rm, wm, sg := metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge()
	fname := fmt.Sprintf("batchread-3-%d", rand.Uint64())
	{ // Fill table
		f, err := newTable(os.TempDir(), fname, rm, wm, sg, 100, true, false)
		if err != nil {
			t.Fatal(err)
		}
		// Write 10 bytes 30 times,
		// Splitting it at every 100 bytes (10 items)
		writeChunks(t, f, 30, 10)
		f.Close()
	}
	for i, tc := range []struct {
		items uint64
		want  int
	}{
		{1, 1},
		{30, 30},
		{31, 30},
	} {
		{
			f, err := newTable(os.TempDir(), fname, rm, wm, sg, 100, true, false)
			if err != nil {
				t.Fatal(err)
			}
			items, err := f.RetrieveItems(0, tc.items, 0)
			if err != nil {
				t.Fatal(err)
			}
			if have, want := len(items), tc.want; have != want {
				t.Fatalf("test %d: want %d items, have %d ", i, want, have)
			}
			for ii, have := range items {
				want := getChunk(10, ii)
				if !bytes.Equal(want, have) {
					t.Fatalf("test %d: data corruption item %d: have\n%x\n, want \n%x\n", i, ii, have, want)
				}
			}
			f.Close()
		}
	}
}

func TestFreezerReadonly(t *testing.T) {
	tmpdir := os.TempDir()
	// Case 1: Check it fails on non-existent file.
	_, err := newTable(tmpdir,
		fmt.Sprintf("readonlytest-%d", rand.Uint64()),
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, true)
	if err == nil {
		t.Fatal("readonly table instantiation should fail for non-existent table")
	}

	// Case 2: Check that it fails on invalid index length.
	fname := fmt.Sprintf("readonlytest-%d", rand.Uint64())
	idxFile, err := openFreezerFileForAppend(filepath.Join(tmpdir, fmt.Sprintf("%s.ridx", fname)))
	if err != nil {
		t.Errorf("Failed to open index file: %v\n", err)
	}
	// size should not be a multiple of indexEntrySize.
	idxFile.Write(make([]byte, 17))
	idxFile.Close()
	_, err = newTable(tmpdir, fname,
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, true)
	if err == nil {
		t.Errorf("readonly table instantiation should fail for invalid index size")
	}

	// Case 3: Open table non-readonly table to write some data.
	// Then corrupt the head file and make sure opening the table
	// again in readonly triggers an error.
	fname = fmt.Sprintf("readonlytest-%d", rand.Uint64())
	f, err := newTable(tmpdir, fname,
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, false)
	if err != nil {
		t.Fatalf("failed to instantiate table: %v", err)
	}
	writeChunks(t, f, 8, 32)
	// Corrupt table file
	if _, err := f.head.Write([]byte{1, 1}); err != nil {
		t.Fatal(err)
	}
	if err := f.Close(); err != nil {
		t.Fatal(err)
	}
	_, err = newTable(tmpdir, fname,
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, true)
	if err == nil {
		t.Errorf("readonly table instantiation should fail for corrupt table file")
	}

	// Case 4: Write some data to a table and later re-open it as readonly.
	// Should be successful.
	fname = fmt.Sprintf("readonlytest-%d", rand.Uint64())
	f, err = newTable(tmpdir, fname,
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, false)
	if err != nil {
		t.Fatalf("failed to instantiate table: %v\n", err)
	}
	writeChunks(t, f, 32, 128)
	if err := f.Close(); err != nil {
		t.Fatal(err)
	}
	f, err = newTable(tmpdir, fname,
		metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, true)
	if err != nil {
		t.Fatal(err)
	}
	v, err := f.Retrieve(10)
	if err != nil {
		t.Fatal(err)
	}
	exp := getChunk(128, 10)
	if !bytes.Equal(v, exp) {
		t.Errorf("retrieved value is incorrect")
	}

	// Case 5: Now write some data via a batch.
	// This should fail either during AppendRaw or Commit
	batch := f.newBatch()
	writeErr := batch.AppendRaw(32, make([]byte, 1))
	if writeErr == nil {
		writeErr = batch.commit()
	}
	if writeErr == nil {
		t.Fatalf("Writing to readonly table should fail")
	}
}

// randTest performs random freezer table operations.
// Instances of this test are created by Generate.
type randTest []randTestStep

type randTestStep struct {
	op     int
	items  []uint64 // for append and retrieve
	blobs  [][]byte // for append
	target uint64   // for truncate(head/tail)
	err    error    // for debugging
}

const (
	opReload = iota
	opAppend
	opRetrieve
	opTruncateHead
	opTruncateHeadAll
	opTruncateTail
	opTruncateTailAll
	opCheckAll
	opMax // boundary value, not an actual op
)

func getVals(first uint64, n int) [][]byte {
	var ret [][]byte
	for i := 0; i < n; i++ {
		val := make([]byte, 8)
		binary.BigEndian.PutUint64(val, first+uint64(i))
		ret = append(ret, val)
	}
	return ret
}

func (randTest) Generate(r *rand.Rand, size int) reflect.Value {
	var (
		deleted uint64   // The number of deleted items from tail
		items   []uint64 // The index of entries in table

		// getItems retrieves the indexes for items in table.
		getItems = func(n int) []uint64 {
			length := len(items)
			if length == 0 {
				return nil
			}
			var ret []uint64
			index := rand.Intn(length)
			for i := index; len(ret) < n && i < length; i++ {
				ret = append(ret, items[i])
			}
			return ret
		}

		// addItems appends the given length items into the table.
		addItems = func(n int) []uint64 {
			var first = deleted
			if len(items) != 0 {
				first = items[len(items)-1] + 1
			}
			var ret []uint64
			for i := 0; i < n; i++ {
				ret = append(ret, first+uint64(i))
			}
			items = append(items, ret...)
			return ret
		}
	)

	var steps randTest
	for i := 0; i < size; i++ {
		step := randTestStep{op: r.Intn(opMax)}
		switch step.op {
		case opReload, opCheckAll:
		case opAppend:
			num := r.Intn(3)
			step.items = addItems(num)
			if len(step.items) == 0 {
				step.blobs = nil
			} else {
				step.blobs = getVals(step.items[0], num)
			}
		case opRetrieve:
			step.items = getItems(r.Intn(3))
		case opTruncateHead:
			if len(items) == 0 {
				step.target = deleted
			} else {
				index := r.Intn(len(items))
				items = items[:index]
				step.target = deleted + uint64(index)
			}
		case opTruncateHeadAll:
			step.target = deleted
			items = items[:0]
		case opTruncateTail:
			if len(items) == 0 {
				step.target = deleted
			} else {
				index := r.Intn(len(items))
				items = items[index:]
				deleted += uint64(index)
				step.target = deleted
			}
		case opTruncateTailAll:
			step.target = deleted + uint64(len(items))
			items = items[:0]
			deleted = step.target
		}
		steps = append(steps, step)
	}
	return reflect.ValueOf(steps)
}

func runRandTest(rt randTest) bool {
	fname := fmt.Sprintf("randtest-%d", rand.Uint64())
	f, err := newTable(os.TempDir(), fname, metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, false)
	if err != nil {
		panic("failed to initialize table")
	}
	var values [][]byte
	for i, step := range rt {
		switch step.op {
		case opReload:
			f.Close()
			f, err = newTable(os.TempDir(), fname, metrics.NewMeter(), metrics.NewMeter(), metrics.NewGauge(), 50, true, false)
			if err != nil {
				rt[i].err = fmt.Errorf("failed to reload table %v", err)
			}
		case opCheckAll:
			tail := f.itemHidden.Load()
			head := f.items.Load()

			if tail == head {
				continue
			}
			got, err := f.RetrieveItems(f.itemHidden.Load(), head-tail, 100000)
			if err != nil {
				rt[i].err = err
			} else {
				if !reflect.DeepEqual(got, values) {
					rt[i].err = fmt.Errorf("mismatch on retrieved values %v %v", got, values)
				}
			}

		case opAppend:
			batch := f.newBatch()
			for i := 0; i < len(step.items); i++ {
				batch.AppendRaw(step.items[i], step.blobs[i])
			}
			batch.commit()
			values = append(values, step.blobs...)

		case opRetrieve:
			var blobs [][]byte
			if len(step.items) == 0 {
				continue
			}
			tail := f.itemHidden.Load()
			for i := 0; i < len(step.items); i++ {
				blobs = append(blobs, values[step.items[i]-tail])
			}
			got, err := f.RetrieveItems(step.items[0], uint64(len(step.items)), 100000)
			if err != nil {
				rt[i].err = err
			} else {
				if !reflect.DeepEqual(got, blobs) {
					rt[i].err = fmt.Errorf("mismatch on retrieved values %v %v %v", got, blobs, step.items)
				}
			}

		case opTruncateHead:
			f.truncateHead(step.target)

			length := f.items.Load() - f.itemHidden.Load()
			values = values[:length]

		case opTruncateHeadAll:
			f.truncateHead(step.target)
			values = nil

		case opTruncateTail:
			prev := f.itemHidden.Load()
			f.truncateTail(step.target)

			truncated := f.itemHidden.Load() - prev
			values = values[truncated:]

		case opTruncateTailAll:
			f.truncateTail(step.target)
			values = nil
		}
		// Abort the test on error.
		if rt[i].err != nil {
			return false
		}
	}
	f.Close()
	return true
}

func TestRandom(t *testing.T) {
	if err := quick.Check(runRandTest, nil); err != nil {
		if cerr, ok := err.(*quick.CheckError); ok {
			t.Fatalf("random test iteration %d failed: %s", cerr.Count, spew.Sdump(cerr.In))
		}
		t.Fatal(err)
	}
}
