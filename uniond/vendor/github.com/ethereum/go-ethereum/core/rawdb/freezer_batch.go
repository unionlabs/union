// Copyright 2021 The go-ethereum Authors
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
	"fmt"

	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/golang/snappy"
)

// This is the maximum amount of data that will be buffered in memory
// for a single freezer table batch.
const freezerBatchBufferLimit = 2 * 1024 * 1024

// freezerBatch is a write operation of multiple items on a freezer.
type freezerBatch struct {
	tables map[string]*freezerTableBatch
}

func newFreezerBatch(f *Freezer) *freezerBatch {
	batch := &freezerBatch{tables: make(map[string]*freezerTableBatch, len(f.tables))}
	for kind, table := range f.tables {
		batch.tables[kind] = table.newBatch()
	}
	return batch
}

// Append adds an RLP-encoded item of the given kind.
func (batch *freezerBatch) Append(kind string, num uint64, item interface{}) error {
	return batch.tables[kind].Append(num, item)
}

// AppendRaw adds an item of the given kind.
func (batch *freezerBatch) AppendRaw(kind string, num uint64, item []byte) error {
	return batch.tables[kind].AppendRaw(num, item)
}

// reset initializes the batch.
func (batch *freezerBatch) reset() {
	for _, tb := range batch.tables {
		tb.reset()
	}
}

// commit is called at the end of a write operation and
// writes all remaining data to tables.
func (batch *freezerBatch) commit() (item uint64, writeSize int64, err error) {
	// Check that count agrees on all batches.
	item = uint64(math.MaxUint64)
	for name, tb := range batch.tables {
		if item < math.MaxUint64 && tb.curItem != item {
			return 0, 0, fmt.Errorf("table %s is at item %d, want %d", name, tb.curItem, item)
		}
		item = tb.curItem
	}

	// Commit all table batches.
	for _, tb := range batch.tables {
		if err := tb.commit(); err != nil {
			return 0, 0, err
		}
		writeSize += tb.totalBytes
	}
	return item, writeSize, nil
}

// freezerTableBatch is a batch for a freezer table.
type freezerTableBatch struct {
	t *freezerTable

	sb          *snappyBuffer
	encBuffer   writeBuffer
	dataBuffer  []byte
	indexBuffer []byte
	curItem     uint64 // expected index of next append
	totalBytes  int64  // counts written bytes since reset
}

// newBatch creates a new batch for the freezer table.
func (t *freezerTable) newBatch() *freezerTableBatch {
	batch := &freezerTableBatch{t: t}
	if !t.noCompression {
		batch.sb = new(snappyBuffer)
	}
	batch.reset()
	return batch
}

// reset clears the batch for reuse.
func (batch *freezerTableBatch) reset() {
	batch.dataBuffer = batch.dataBuffer[:0]
	batch.indexBuffer = batch.indexBuffer[:0]
	batch.curItem = batch.t.items.Load()
	batch.totalBytes = 0
}

// Append rlp-encodes and adds data at the end of the freezer table. The item number is a
// precautionary parameter to ensure data correctness, but the table will reject already
// existing data.
func (batch *freezerTableBatch) Append(item uint64, data interface{}) error {
	if item != batch.curItem {
		return fmt.Errorf("%w: have %d want %d", errOutOrderInsertion, item, batch.curItem)
	}

	// Encode the item.
	batch.encBuffer.Reset()
	if err := rlp.Encode(&batch.encBuffer, data); err != nil {
		return err
	}
	encItem := batch.encBuffer.data
	if batch.sb != nil {
		encItem = batch.sb.compress(encItem)
	}
	return batch.appendItem(encItem)
}

// AppendRaw injects a binary blob at the end of the freezer table. The item number is a
// precautionary parameter to ensure data correctness, but the table will reject already
// existing data.
func (batch *freezerTableBatch) AppendRaw(item uint64, blob []byte) error {
	if item != batch.curItem {
		return fmt.Errorf("%w: have %d want %d", errOutOrderInsertion, item, batch.curItem)
	}

	encItem := blob
	if batch.sb != nil {
		encItem = batch.sb.compress(blob)
	}
	return batch.appendItem(encItem)
}

func (batch *freezerTableBatch) appendItem(data []byte) error {
	// Check if item fits into current data file.
	itemSize := int64(len(data))
	itemOffset := batch.t.headBytes + int64(len(batch.dataBuffer))
	if itemOffset+itemSize > int64(batch.t.maxFileSize) {
		// It doesn't fit, go to next file first.
		if err := batch.commit(); err != nil {
			return err
		}
		if err := batch.t.advanceHead(); err != nil {
			return err
		}
		itemOffset = 0
	}

	// Put data to buffer.
	batch.dataBuffer = append(batch.dataBuffer, data...)
	batch.totalBytes += itemSize

	// Put index entry to buffer.
	entry := indexEntry{filenum: batch.t.headId, offset: uint32(itemOffset + itemSize)}
	batch.indexBuffer = entry.append(batch.indexBuffer)
	batch.curItem++

	return batch.maybeCommit()
}

// maybeCommit writes the buffered data if the buffer is full enough.
func (batch *freezerTableBatch) maybeCommit() error {
	if len(batch.dataBuffer) > freezerBatchBufferLimit {
		return batch.commit()
	}
	return nil
}

// commit writes the batched items to the backing freezerTable.
func (batch *freezerTableBatch) commit() error {
	// Write data.
	_, err := batch.t.head.Write(batch.dataBuffer)
	if err != nil {
		return err
	}
	dataSize := int64(len(batch.dataBuffer))
	batch.dataBuffer = batch.dataBuffer[:0]

	// Write indices.
	_, err = batch.t.index.Write(batch.indexBuffer)
	if err != nil {
		return err
	}
	indexSize := int64(len(batch.indexBuffer))
	batch.indexBuffer = batch.indexBuffer[:0]

	// Update headBytes of table.
	batch.t.headBytes += dataSize
	batch.t.items.Store(batch.curItem)

	// Update metrics.
	batch.t.sizeGauge.Inc(dataSize + indexSize)
	batch.t.writeMeter.Mark(dataSize + indexSize)
	return nil
}

// snappyBuffer writes snappy in block format, and can be reused. It is
// reset when WriteTo is called.
type snappyBuffer struct {
	dst []byte
}

// compress snappy-compresses the data.
func (s *snappyBuffer) compress(data []byte) []byte {
	// The snappy library does not care what the capacity of the buffer is,
	// but only checks the length. If the length is too small, it will
	// allocate a brand new buffer.
	// To avoid that, we check the required size here, and grow the size of the
	// buffer to utilize the full capacity.
	if n := snappy.MaxEncodedLen(len(data)); len(s.dst) < n {
		if cap(s.dst) < n {
			s.dst = make([]byte, n)
		}
		s.dst = s.dst[:n]
	}

	s.dst = snappy.Encode(s.dst, data)
	return s.dst
}

// writeBuffer implements io.Writer for a byte slice.
type writeBuffer struct {
	data []byte
}

func (wb *writeBuffer) Write(data []byte) (int, error) {
	wb.data = append(wb.data, data...)
	return len(data), nil
}

func (wb *writeBuffer) Reset() {
	wb.data = wb.data[:0]
}
