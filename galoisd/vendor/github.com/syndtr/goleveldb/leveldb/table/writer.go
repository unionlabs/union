// Copyright (c) 2012, Suryandaru Triandana <syndtr@gmail.com>
// All rights reserved.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package table

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"

	"github.com/golang/snappy"

	"github.com/syndtr/goleveldb/leveldb/comparer"
	"github.com/syndtr/goleveldb/leveldb/filter"
	"github.com/syndtr/goleveldb/leveldb/opt"
	"github.com/syndtr/goleveldb/leveldb/util"
)

func sharedPrefixLen(a, b []byte) int {
	i, n := 0, len(a)
	if n > len(b) {
		n = len(b)
	}
	for i < n && a[i] == b[i] {
		i++
	}
	return i
}

type blockWriter struct {
	restartInterval int
	buf             util.Buffer
	nEntries        int
	prevKey         []byte
	restarts        []uint32
	scratch         []byte
}

func (w *blockWriter) append(key, value []byte) (err error) {
	nShared := 0
	if w.nEntries%w.restartInterval == 0 {
		w.restarts = append(w.restarts, uint32(w.buf.Len()))
	} else {
		nShared = sharedPrefixLen(w.prevKey, key)
	}
	n := binary.PutUvarint(w.scratch[0:], uint64(nShared))
	n += binary.PutUvarint(w.scratch[n:], uint64(len(key)-nShared))
	n += binary.PutUvarint(w.scratch[n:], uint64(len(value)))
	if _, err = w.buf.Write(w.scratch[:n]); err != nil {
		return err
	}
	if _, err = w.buf.Write(key[nShared:]); err != nil {
		return err
	}
	if _, err = w.buf.Write(value); err != nil {
		return err
	}
	w.prevKey = append(w.prevKey[:0], key...)
	w.nEntries++
	return nil
}

func (w *blockWriter) finish() error {
	// Write restarts entry.
	if w.nEntries == 0 {
		// Must have at least one restart entry.
		w.restarts = append(w.restarts, 0)
	}
	w.restarts = append(w.restarts, uint32(len(w.restarts)))
	for _, x := range w.restarts {
		buf4 := w.buf.Alloc(4)
		binary.LittleEndian.PutUint32(buf4, x)
	}
	return nil
}

func (w *blockWriter) reset() {
	w.buf.Reset()
	w.nEntries = 0
	w.restarts = w.restarts[:0]
}

func (w *blockWriter) bytesLen() int {
	restartsLen := len(w.restarts)
	if restartsLen == 0 {
		restartsLen = 1
	}
	return w.buf.Len() + 4*restartsLen + 4
}

type filterWriter struct {
	generator filter.FilterGenerator
	buf       util.Buffer
	nKeys     int
	offsets   []uint32
	baseLg    uint
}

func (w *filterWriter) add(key []byte) {
	if w.generator == nil {
		return
	}
	w.generator.Add(key)
	w.nKeys++
}

func (w *filterWriter) flush(offset uint64) {
	if w.generator == nil {
		return
	}
	for x := int(offset / uint64(1<<w.baseLg)); x > len(w.offsets); {
		w.generate()
	}
}

func (w *filterWriter) finish() error {
	if w.generator == nil {
		return nil
	}
	// Generate last keys.

	if w.nKeys > 0 {
		w.generate()
	}
	w.offsets = append(w.offsets, uint32(w.buf.Len()))
	for _, x := range w.offsets {
		buf4 := w.buf.Alloc(4)
		binary.LittleEndian.PutUint32(buf4, x)
	}
	return w.buf.WriteByte(byte(w.baseLg))
}

func (w *filterWriter) generate() {
	// Record offset.
	w.offsets = append(w.offsets, uint32(w.buf.Len()))
	// Generate filters.
	if w.nKeys > 0 {
		w.generator.Generate(&w.buf)
		w.nKeys = 0
	}
}

// Writer is a table writer.
type Writer struct {
	writer io.Writer
	err    error
	// Options
	cmp         comparer.Comparer
	filter      filter.Filter
	compression opt.Compression
	blockSize   int

	bpool       *util.BufferPool
	dataBlock   blockWriter
	indexBlock  blockWriter
	filterBlock filterWriter
	pendingBH   blockHandle
	offset      uint64
	nEntries    int
	// Scratch allocated enough for 5 uvarint. Block writer should not use
	// first 20-bytes since it will be used to encode block handle, which
	// then passed to the block writer itself.
	scratch            [50]byte
	comparerScratch    []byte
	compressionScratch []byte
}

func (w *Writer) writeBlock(buf *util.Buffer, compression opt.Compression) (bh blockHandle, err error) {
	// Compress the buffer if necessary.
	var b []byte
	if compression == opt.SnappyCompression {
		// Allocate scratch enough for compression and block trailer.
		if n := snappy.MaxEncodedLen(buf.Len()) + blockTrailerLen; len(w.compressionScratch) < n {
			w.compressionScratch = make([]byte, n)
		}
		compressed := snappy.Encode(w.compressionScratch, buf.Bytes())
		n := len(compressed)
		b = compressed[:n+blockTrailerLen]
		b[n] = blockTypeSnappyCompression
	} else {
		tmp := buf.Alloc(blockTrailerLen)
		tmp[0] = blockTypeNoCompression
		b = buf.Bytes()
	}

	// Calculate the checksum.
	n := len(b) - 4
	checksum := util.NewCRC(b[:n]).Value()
	binary.LittleEndian.PutUint32(b[n:], checksum)

	// Write the buffer to the file.
	_, err = w.writer.Write(b)
	if err != nil {
		return
	}
	bh = blockHandle{w.offset, uint64(len(b) - blockTrailerLen)}
	w.offset += uint64(len(b))
	return
}

func (w *Writer) flushPendingBH(key []byte) error {
	if w.pendingBH.length == 0 {
		return nil
	}
	var separator []byte
	if len(key) == 0 {
		separator = w.cmp.Successor(w.comparerScratch[:0], w.dataBlock.prevKey)
	} else {
		separator = w.cmp.Separator(w.comparerScratch[:0], w.dataBlock.prevKey, key)
	}
	if separator == nil {
		separator = w.dataBlock.prevKey
	} else {
		w.comparerScratch = separator
	}
	n := encodeBlockHandle(w.scratch[:20], w.pendingBH)
	// Append the block handle to the index block.
	if err := w.indexBlock.append(separator, w.scratch[:n]); err != nil {
		return err
	}
	// Reset prev key of the data block.
	w.dataBlock.prevKey = w.dataBlock.prevKey[:0]
	// Clear pending block handle.
	w.pendingBH = blockHandle{}
	return nil
}

func (w *Writer) finishBlock() error {
	if err := w.dataBlock.finish(); err != nil {
		return err
	}
	bh, err := w.writeBlock(&w.dataBlock.buf, w.compression)
	if err != nil {
		return err
	}
	w.pendingBH = bh
	// Reset the data block.
	w.dataBlock.reset()
	// Flush the filter block.
	w.filterBlock.flush(w.offset)
	return nil
}

// Append appends key/value pair to the table. The keys passed must
// be in increasing order.
//
// It is safe to modify the contents of the arguments after Append returns.
func (w *Writer) Append(key, value []byte) error {
	if w.err != nil {
		return w.err
	}
	if w.nEntries > 0 && w.cmp.Compare(w.dataBlock.prevKey, key) >= 0 {
		w.err = fmt.Errorf("leveldb/table: Writer: keys are not in increasing order: %q, %q", w.dataBlock.prevKey, key)
		return w.err
	}

	if err := w.flushPendingBH(key); err != nil {
		return err
	}
	// Append key/value pair to the data block.
	if err := w.dataBlock.append(key, value); err != nil {
		return err
	}
	// Add key to the filter block.
	w.filterBlock.add(key)

	// Finish the data block if block size target reached.
	if w.dataBlock.bytesLen() >= w.blockSize {
		if err := w.finishBlock(); err != nil {
			w.err = err
			return w.err
		}
	}
	w.nEntries++
	return nil
}

// BlocksLen returns number of blocks written so far.
func (w *Writer) BlocksLen() int {
	n := w.indexBlock.nEntries
	if w.pendingBH.length > 0 {
		// Includes the pending block.
		n++
	}
	return n
}

// EntriesLen returns number of entries added so far.
func (w *Writer) EntriesLen() int {
	return w.nEntries
}

// BytesLen returns number of bytes written so far.
func (w *Writer) BytesLen() int {
	return int(w.offset)
}

// Close will finalize the table. Calling Append is not possible
// after Close, but calling BlocksLen, EntriesLen and BytesLen
// is still possible.
func (w *Writer) Close() error {
	defer func() {
		if w.bpool != nil {
			// Buffer.Bytes() returns [offset:] of the buffer.
			// We need to Reset() so that the offset = 0, resulting
			// in buf.Bytes() returning the whole allocated bytes.
			w.dataBlock.buf.Reset()
			w.bpool.Put(w.dataBlock.buf.Bytes())
		}
	}()

	if w.err != nil {
		return w.err
	}

	// Write the last data block. Or empty data block if there
	// aren't any data blocks at all.
	if w.dataBlock.nEntries > 0 || w.nEntries == 0 {
		if err := w.finishBlock(); err != nil {
			w.err = err
			return w.err
		}
	}
	if err := w.flushPendingBH(nil); err != nil {
		return err
	}

	// Write the filter block.
	var filterBH blockHandle
	if err := w.filterBlock.finish(); err != nil {
		return err
	}
	if buf := &w.filterBlock.buf; buf.Len() > 0 {
		filterBH, w.err = w.writeBlock(buf, opt.NoCompression)
		if w.err != nil {
			return w.err
		}
	}

	// Write the metaindex block.
	if filterBH.length > 0 {
		key := []byte("filter." + w.filter.Name())
		n := encodeBlockHandle(w.scratch[:20], filterBH)
		if err := w.dataBlock.append(key, w.scratch[:n]); err != nil {
			return err
		}
	}
	if err := w.dataBlock.finish(); err != nil {
		return err
	}
	metaindexBH, err := w.writeBlock(&w.dataBlock.buf, w.compression)
	if err != nil {
		w.err = err
		return w.err
	}

	// Write the index block.
	if err := w.indexBlock.finish(); err != nil {
		return err
	}
	indexBH, err := w.writeBlock(&w.indexBlock.buf, w.compression)
	if err != nil {
		w.err = err
		return w.err
	}

	// Write the table footer.
	footer := w.scratch[:footerLen]
	for i := range footer {
		footer[i] = 0
	}
	n := encodeBlockHandle(footer, metaindexBH)
	encodeBlockHandle(footer[n:], indexBH)
	copy(footer[footerLen-len(magic):], magic)
	if _, err := w.writer.Write(footer); err != nil {
		w.err = err
		return w.err
	}
	w.offset += footerLen

	w.err = errors.New("leveldb/table: writer is closed")
	return nil
}

// NewWriter creates a new initialized table writer for the file.
//
// Table writer is not safe for concurrent use.
func NewWriter(f io.Writer, o *opt.Options, pool *util.BufferPool, size int) *Writer {
	var bufBytes []byte
	if pool == nil {
		bufBytes = make([]byte, size)
	} else {
		bufBytes = pool.Get(size)
	}
	bufBytes = bufBytes[:0]

	w := &Writer{
		writer:          f,
		cmp:             o.GetComparer(),
		filter:          o.GetFilter(),
		compression:     o.GetCompression(),
		blockSize:       o.GetBlockSize(),
		comparerScratch: make([]byte, 0),
		bpool:           pool,
		dataBlock:       blockWriter{buf: *util.NewBuffer(bufBytes)},
	}
	// data block
	w.dataBlock.restartInterval = o.GetBlockRestartInterval()
	// The first 20-bytes are used for encoding block handle.
	w.dataBlock.scratch = w.scratch[20:]
	// index block
	w.indexBlock.restartInterval = 1
	w.indexBlock.scratch = w.scratch[20:]
	// filter block
	if w.filter != nil {
		w.filterBlock.generator = w.filter.NewGenerator()
		w.filterBlock.baseLg = uint(o.GetFilterBaseLg())
		w.filterBlock.flush(0)
	}
	return w
}
