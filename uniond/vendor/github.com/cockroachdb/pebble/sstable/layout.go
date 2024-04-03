// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"sort"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/base"
)

// Layout describes the block organization of an sstable.
type Layout struct {
	// NOTE: changes to fields in this struct should also be reflected in
	// ValidateBlockChecksums, which validates a static list of BlockHandles
	// referenced in this struct.

	Data       []BlockHandleWithProperties
	Index      []BlockHandle
	TopIndex   BlockHandle
	Filter     BlockHandle
	RangeDel   BlockHandle
	RangeKey   BlockHandle
	ValueBlock []BlockHandle
	ValueIndex BlockHandle
	Properties BlockHandle
	MetaIndex  BlockHandle
	Footer     BlockHandle
	Format     TableFormat
}

// Describe returns a description of the layout. If the verbose parameter is
// true, details of the structure of each block are returned as well.
func (l *Layout) Describe(
	w io.Writer, verbose bool, r *Reader, fmtRecord func(key *base.InternalKey, value []byte),
) {
	ctx := context.TODO()
	type block struct {
		BlockHandle
		name string
	}
	var blocks []block

	for i := range l.Data {
		blocks = append(blocks, block{l.Data[i].BlockHandle, "data"})
	}
	for i := range l.Index {
		blocks = append(blocks, block{l.Index[i], "index"})
	}
	if l.TopIndex.Length != 0 {
		blocks = append(blocks, block{l.TopIndex, "top-index"})
	}
	if l.Filter.Length != 0 {
		blocks = append(blocks, block{l.Filter, "filter"})
	}
	if l.RangeDel.Length != 0 {
		blocks = append(blocks, block{l.RangeDel, "range-del"})
	}
	if l.RangeKey.Length != 0 {
		blocks = append(blocks, block{l.RangeKey, "range-key"})
	}
	for i := range l.ValueBlock {
		blocks = append(blocks, block{l.ValueBlock[i], "value-block"})
	}
	if l.ValueIndex.Length != 0 {
		blocks = append(blocks, block{l.ValueIndex, "value-index"})
	}
	if l.Properties.Length != 0 {
		blocks = append(blocks, block{l.Properties, "properties"})
	}
	if l.MetaIndex.Length != 0 {
		blocks = append(blocks, block{l.MetaIndex, "meta-index"})
	}
	if l.Footer.Length != 0 {
		if l.Footer.Length == levelDBFooterLen {
			blocks = append(blocks, block{l.Footer, "leveldb-footer"})
		} else {
			blocks = append(blocks, block{l.Footer, "footer"})
		}
	}

	sort.Slice(blocks, func(i, j int) bool {
		return blocks[i].Offset < blocks[j].Offset
	})

	for i := range blocks {
		b := &blocks[i]
		fmt.Fprintf(w, "%10d  %s (%d)\n", b.Offset, b.name, b.Length)

		if !verbose {
			continue
		}
		if b.name == "filter" {
			continue
		}

		if b.name == "footer" || b.name == "leveldb-footer" {
			trailer, offset := make([]byte, b.Length), b.Offset
			_ = r.readable.ReadAt(ctx, trailer, int64(offset))

			if b.name == "footer" {
				checksumType := ChecksumType(trailer[0])
				fmt.Fprintf(w, "%10d    checksum type: %s\n", offset, checksumType)
				trailer, offset = trailer[1:], offset+1
			}

			metaHandle, n := binary.Uvarint(trailer)
			metaLen, m := binary.Uvarint(trailer[n:])
			fmt.Fprintf(w, "%10d    meta: offset=%d, length=%d\n", offset, metaHandle, metaLen)
			trailer, offset = trailer[n+m:], offset+uint64(n+m)

			indexHandle, n := binary.Uvarint(trailer)
			indexLen, m := binary.Uvarint(trailer[n:])
			fmt.Fprintf(w, "%10d    index: offset=%d, length=%d\n", offset, indexHandle, indexLen)
			trailer, offset = trailer[n+m:], offset+uint64(n+m)

			fmt.Fprintf(w, "%10d    [padding]\n", offset)

			trailing := 12
			if b.name == "leveldb-footer" {
				trailing = 8
			}

			offset += uint64(len(trailer) - trailing)
			trailer = trailer[len(trailer)-trailing:]

			if b.name == "footer" {
				version := trailer[:4]
				fmt.Fprintf(w, "%10d    version: %d\n", offset, binary.LittleEndian.Uint32(version))
				trailer, offset = trailer[4:], offset+4
			}

			magicNumber := trailer
			fmt.Fprintf(w, "%10d    magic number: 0x%x\n", offset, magicNumber)

			continue
		}

		h, err := r.readBlock(
			context.Background(), b.BlockHandle, nil /* transform */, nil /* readHandle */, nil /* stats */, nil /* buffer pool */)
		if err != nil {
			fmt.Fprintf(w, "  [err: %s]\n", err)
			continue
		}

		getRestart := func(data []byte, restarts, i int32) int32 {
			return decodeRestart(data[restarts+4*i:])
		}

		formatIsRestart := func(data []byte, restarts, numRestarts, offset int32) {
			i := sort.Search(int(numRestarts), func(i int) bool {
				return getRestart(data, restarts, int32(i)) >= offset
			})
			if i < int(numRestarts) && getRestart(data, restarts, int32(i)) == offset {
				fmt.Fprintf(w, " [restart]\n")
			} else {
				fmt.Fprintf(w, "\n")
			}
		}

		formatRestarts := func(data []byte, restarts, numRestarts int32) {
			for i := int32(0); i < numRestarts; i++ {
				offset := getRestart(data, restarts, i)
				fmt.Fprintf(w, "%10d    [restart %d]\n",
					b.Offset+uint64(restarts+4*i), b.Offset+uint64(offset))
			}
		}

		formatTrailer := func() {
			trailer := make([]byte, blockTrailerLen)
			offset := int64(b.Offset + b.Length)
			_ = r.readable.ReadAt(ctx, trailer, offset)
			bt := blockType(trailer[0])
			checksum := binary.LittleEndian.Uint32(trailer[1:])
			fmt.Fprintf(w, "%10d    [trailer compression=%s checksum=0x%04x]\n", offset, bt, checksum)
		}

		var lastKey InternalKey
		switch b.name {
		case "data", "range-del", "range-key":
			iter, _ := newBlockIter(r.Compare, h.Get())
			for key, value := iter.First(); key != nil; key, value = iter.Next() {
				ptr := unsafe.Pointer(uintptr(iter.ptr) + uintptr(iter.offset))
				shared, ptr := decodeVarint(ptr)
				unshared, ptr := decodeVarint(ptr)
				value2, _ := decodeVarint(ptr)

				total := iter.nextOffset - iter.offset
				// The format of the numbers in the record line is:
				//
				//   (<total> = <length> [<shared>] + <unshared> + <value>)
				//
				// <total>    is the total number of bytes for the record.
				// <length>   is the size of the 3 varint encoded integers for <shared>,
				//            <unshared>, and <value>.
				// <shared>   is the number of key bytes shared with the previous key.
				// <unshared> is the number of unshared key bytes.
				// <value>    is the number of value bytes.
				fmt.Fprintf(w, "%10d    record (%d = %d [%d] + %d + %d)",
					b.Offset+uint64(iter.offset), total,
					total-int32(unshared+value2), shared, unshared, value2)
				formatIsRestart(iter.data, iter.restarts, iter.numRestarts, iter.offset)
				if fmtRecord != nil {
					fmt.Fprintf(w, "              ")
					if l.Format < TableFormatPebblev3 {
						fmtRecord(key, value.InPlaceValue())
					} else {
						// InPlaceValue() will succeed even for data blocks where the
						// actual value is in a different location, since this value was
						// fetched from a blockIter which does not know about value
						// blocks.
						v := value.InPlaceValue()
						if base.TrailerKind(key.Trailer) != InternalKeyKindSet {
							fmtRecord(key, v)
						} else if !isValueHandle(valuePrefix(v[0])) {
							fmtRecord(key, v[1:])
						} else {
							vh := decodeValueHandle(v[1:])
							fmtRecord(key, []byte(fmt.Sprintf("value handle %+v", vh)))
						}
					}
				}

				if base.InternalCompare(r.Compare, lastKey, *key) >= 0 {
					fmt.Fprintf(w, "              WARNING: OUT OF ORDER KEYS!\n")
				}
				lastKey.Trailer = key.Trailer
				lastKey.UserKey = append(lastKey.UserKey[:0], key.UserKey...)
			}
			formatRestarts(iter.data, iter.restarts, iter.numRestarts)
			formatTrailer()
		case "index", "top-index":
			iter, _ := newBlockIter(r.Compare, h.Get())
			for key, value := iter.First(); key != nil; key, value = iter.Next() {
				bh, err := decodeBlockHandleWithProperties(value.InPlaceValue())
				if err != nil {
					fmt.Fprintf(w, "%10d    [err: %s]\n", b.Offset+uint64(iter.offset), err)
					continue
				}
				fmt.Fprintf(w, "%10d    block:%d/%d",
					b.Offset+uint64(iter.offset), bh.Offset, bh.Length)
				formatIsRestart(iter.data, iter.restarts, iter.numRestarts, iter.offset)
			}
			formatRestarts(iter.data, iter.restarts, iter.numRestarts)
			formatTrailer()
		case "properties":
			iter, _ := newRawBlockIter(r.Compare, h.Get())
			for valid := iter.First(); valid; valid = iter.Next() {
				fmt.Fprintf(w, "%10d    %s (%d)",
					b.Offset+uint64(iter.offset), iter.Key().UserKey, iter.nextOffset-iter.offset)
				formatIsRestart(iter.data, iter.restarts, iter.numRestarts, iter.offset)
			}
			formatRestarts(iter.data, iter.restarts, iter.numRestarts)
			formatTrailer()
		case "meta-index":
			iter, _ := newRawBlockIter(r.Compare, h.Get())
			for valid := iter.First(); valid; valid = iter.Next() {
				value := iter.Value()
				var bh BlockHandle
				var n int
				var vbih valueBlocksIndexHandle
				isValueBlocksIndexHandle := false
				if bytes.Equal(iter.Key().UserKey, []byte(metaValueIndexName)) {
					vbih, n, err = decodeValueBlocksIndexHandle(value)
					bh = vbih.h
					isValueBlocksIndexHandle = true
				} else {
					bh, n = decodeBlockHandle(value)
				}
				if n == 0 || n != len(value) {
					fmt.Fprintf(w, "%10d    [err: %s]\n", b.Offset+uint64(iter.offset), err)
					continue
				}
				var vbihStr string
				if isValueBlocksIndexHandle {
					vbihStr = fmt.Sprintf(" value-blocks-index-lengths: %d(num), %d(offset), %d(length)",
						vbih.blockNumByteLength, vbih.blockOffsetByteLength, vbih.blockLengthByteLength)
				}
				fmt.Fprintf(w, "%10d    %s block:%d/%d%s",
					b.Offset+uint64(iter.offset), iter.Key().UserKey,
					bh.Offset, bh.Length, vbihStr)
				formatIsRestart(iter.data, iter.restarts, iter.numRestarts, iter.offset)
			}
			formatRestarts(iter.data, iter.restarts, iter.numRestarts)
			formatTrailer()
		case "value-block":
			// We don't peer into the value-block since it can't be interpreted
			// without the valueHandles.
		case "value-index":
			// We have already read the value-index to construct the list of
			// value-blocks, so no need to do it again.
		}

		h.Release()
	}

	last := blocks[len(blocks)-1]
	fmt.Fprintf(w, "%10d  EOF\n", last.Offset+last.Length)
}
