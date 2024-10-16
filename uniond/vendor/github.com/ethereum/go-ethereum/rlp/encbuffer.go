// Copyright 2022 The go-ethereum Authors
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

package rlp

import (
	"encoding/binary"
	"io"
	"math/big"
	"reflect"
	"sync"

	"github.com/holiman/uint256"
)

type encBuffer struct {
	str     []byte     // string data, contains everything except list headers
	lheads  []listhead // all list headers
	lhsize  int        // sum of sizes of all encoded list headers
	sizebuf [9]byte    // auxiliary buffer for uint encoding
}

// The global encBuffer pool.
var encBufferPool = sync.Pool{
	New: func() interface{} { return new(encBuffer) },
}

func getEncBuffer() *encBuffer {
	buf := encBufferPool.Get().(*encBuffer)
	buf.reset()
	return buf
}

func (buf *encBuffer) reset() {
	buf.lhsize = 0
	buf.str = buf.str[:0]
	buf.lheads = buf.lheads[:0]
}

// size returns the length of the encoded data.
func (buf *encBuffer) size() int {
	return len(buf.str) + buf.lhsize
}

// makeBytes creates the encoder output.
func (buf *encBuffer) makeBytes() []byte {
	out := make([]byte, buf.size())
	buf.copyTo(out)
	return out
}

func (buf *encBuffer) copyTo(dst []byte) {
	strpos := 0
	pos := 0
	for _, head := range buf.lheads {
		// write string data before header
		n := copy(dst[pos:], buf.str[strpos:head.offset])
		pos += n
		strpos += n
		// write the header
		enc := head.encode(dst[pos:])
		pos += len(enc)
	}
	// copy string data after the last list header
	copy(dst[pos:], buf.str[strpos:])
}

// writeTo writes the encoder output to w.
func (buf *encBuffer) writeTo(w io.Writer) (err error) {
	strpos := 0
	for _, head := range buf.lheads {
		// write string data before header
		if head.offset-strpos > 0 {
			n, err := w.Write(buf.str[strpos:head.offset])
			strpos += n
			if err != nil {
				return err
			}
		}
		// write the header
		enc := head.encode(buf.sizebuf[:])
		if _, err = w.Write(enc); err != nil {
			return err
		}
	}
	if strpos < len(buf.str) {
		// write string data after the last list header
		_, err = w.Write(buf.str[strpos:])
	}
	return err
}

// Write implements io.Writer and appends b directly to the output.
func (buf *encBuffer) Write(b []byte) (int, error) {
	buf.str = append(buf.str, b...)
	return len(b), nil
}

// writeBool writes b as the integer 0 (false) or 1 (true).
func (buf *encBuffer) writeBool(b bool) {
	if b {
		buf.str = append(buf.str, 0x01)
	} else {
		buf.str = append(buf.str, 0x80)
	}
}

func (buf *encBuffer) writeUint64(i uint64) {
	if i == 0 {
		buf.str = append(buf.str, 0x80)
	} else if i < 128 {
		// fits single byte
		buf.str = append(buf.str, byte(i))
	} else {
		s := putint(buf.sizebuf[1:], i)
		buf.sizebuf[0] = 0x80 + byte(s)
		buf.str = append(buf.str, buf.sizebuf[:s+1]...)
	}
}

func (buf *encBuffer) writeBytes(b []byte) {
	if len(b) == 1 && b[0] <= 0x7F {
		// fits single byte, no string header
		buf.str = append(buf.str, b[0])
	} else {
		buf.encodeStringHeader(len(b))
		buf.str = append(buf.str, b...)
	}
}

func (buf *encBuffer) writeString(s string) {
	buf.writeBytes([]byte(s))
}

// wordBytes is the number of bytes in a big.Word
const wordBytes = (32 << (uint64(^big.Word(0)) >> 63)) / 8

// writeBigInt writes i as an integer.
func (buf *encBuffer) writeBigInt(i *big.Int) {
	bitlen := i.BitLen()
	if bitlen <= 64 {
		buf.writeUint64(i.Uint64())
		return
	}
	// Integer is larger than 64 bits, encode from i.Bits().
	// The minimal byte length is bitlen rounded up to the next
	// multiple of 8, divided by 8.
	length := ((bitlen + 7) & -8) >> 3
	buf.encodeStringHeader(length)
	buf.str = append(buf.str, make([]byte, length)...)
	index := length
	bytesBuf := buf.str[len(buf.str)-length:]
	for _, d := range i.Bits() {
		for j := 0; j < wordBytes && index > 0; j++ {
			index--
			bytesBuf[index] = byte(d)
			d >>= 8
		}
	}
}

// writeUint256 writes z as an integer.
func (buf *encBuffer) writeUint256(z *uint256.Int) {
	bitlen := z.BitLen()
	if bitlen <= 64 {
		buf.writeUint64(z.Uint64())
		return
	}
	nBytes := byte((bitlen + 7) / 8)
	var b [33]byte
	binary.BigEndian.PutUint64(b[1:9], z[3])
	binary.BigEndian.PutUint64(b[9:17], z[2])
	binary.BigEndian.PutUint64(b[17:25], z[1])
	binary.BigEndian.PutUint64(b[25:33], z[0])
	b[32-nBytes] = 0x80 + nBytes
	buf.str = append(buf.str, b[32-nBytes:]...)
}

// list adds a new list header to the header stack. It returns the index of the header.
// Call listEnd with this index after encoding the content of the list.
func (buf *encBuffer) list() int {
	buf.lheads = append(buf.lheads, listhead{offset: len(buf.str), size: buf.lhsize})
	return len(buf.lheads) - 1
}

func (buf *encBuffer) listEnd(index int) {
	lh := &buf.lheads[index]
	lh.size = buf.size() - lh.offset - lh.size
	if lh.size < 56 {
		buf.lhsize++ // length encoded into kind tag
	} else {
		buf.lhsize += 1 + intsize(uint64(lh.size))
	}
}

func (buf *encBuffer) encode(val interface{}) error {
	rval := reflect.ValueOf(val)
	writer, err := cachedWriter(rval.Type())
	if err != nil {
		return err
	}
	return writer(rval, buf)
}

func (buf *encBuffer) encodeStringHeader(size int) {
	if size < 56 {
		buf.str = append(buf.str, 0x80+byte(size))
	} else {
		sizesize := putint(buf.sizebuf[1:], uint64(size))
		buf.sizebuf[0] = 0xB7 + byte(sizesize)
		buf.str = append(buf.str, buf.sizebuf[:sizesize+1]...)
	}
}

// encReader is the io.Reader returned by EncodeToReader.
// It releases its encbuf at EOF.
type encReader struct {
	buf    *encBuffer // the buffer we're reading from. this is nil when we're at EOF.
	lhpos  int        // index of list header that we're reading
	strpos int        // current position in string buffer
	piece  []byte     // next piece to be read
}

func (r *encReader) Read(b []byte) (n int, err error) {
	for {
		if r.piece = r.next(); r.piece == nil {
			// Put the encode buffer back into the pool at EOF when it
			// is first encountered. Subsequent calls still return EOF
			// as the error but the buffer is no longer valid.
			if r.buf != nil {
				encBufferPool.Put(r.buf)
				r.buf = nil
			}
			return n, io.EOF
		}
		nn := copy(b[n:], r.piece)
		n += nn
		if nn < len(r.piece) {
			// piece didn't fit, see you next time.
			r.piece = r.piece[nn:]
			return n, nil
		}
		r.piece = nil
	}
}

// next returns the next piece of data to be read.
// it returns nil at EOF.
func (r *encReader) next() []byte {
	switch {
	case r.buf == nil:
		return nil

	case r.piece != nil:
		// There is still data available for reading.
		return r.piece

	case r.lhpos < len(r.buf.lheads):
		// We're before the last list header.
		head := r.buf.lheads[r.lhpos]
		sizebefore := head.offset - r.strpos
		if sizebefore > 0 {
			// String data before header.
			p := r.buf.str[r.strpos:head.offset]
			r.strpos += sizebefore
			return p
		}
		r.lhpos++
		return head.encode(r.buf.sizebuf[:])

	case r.strpos < len(r.buf.str):
		// String data at the end, after all list headers.
		p := r.buf.str[r.strpos:]
		r.strpos = len(r.buf.str)
		return p

	default:
		return nil
	}
}

func encBufferFromWriter(w io.Writer) *encBuffer {
	switch w := w.(type) {
	case EncoderBuffer:
		return w.buf
	case *EncoderBuffer:
		return w.buf
	case *encBuffer:
		return w
	default:
		return nil
	}
}

// EncoderBuffer is a buffer for incremental encoding.
//
// The zero value is NOT ready for use. To get a usable buffer,
// create it using NewEncoderBuffer or call Reset.
type EncoderBuffer struct {
	buf *encBuffer
	dst io.Writer

	ownBuffer bool
}

// NewEncoderBuffer creates an encoder buffer.
func NewEncoderBuffer(dst io.Writer) EncoderBuffer {
	var w EncoderBuffer
	w.Reset(dst)
	return w
}

// Reset truncates the buffer and sets the output destination.
func (w *EncoderBuffer) Reset(dst io.Writer) {
	if w.buf != nil && !w.ownBuffer {
		panic("can't Reset derived EncoderBuffer")
	}

	// If the destination writer has an *encBuffer, use it.
	// Note that w.ownBuffer is left false here.
	if dst != nil {
		if outer := encBufferFromWriter(dst); outer != nil {
			*w = EncoderBuffer{outer, nil, false}
			return
		}
	}

	// Get a fresh buffer.
	if w.buf == nil {
		w.buf = encBufferPool.Get().(*encBuffer)
		w.ownBuffer = true
	}
	w.buf.reset()
	w.dst = dst
}

// Flush writes encoded RLP data to the output writer. This can only be called once.
// If you want to re-use the buffer after Flush, you must call Reset.
func (w *EncoderBuffer) Flush() error {
	var err error
	if w.dst != nil {
		err = w.buf.writeTo(w.dst)
	}
	// Release the internal buffer.
	if w.ownBuffer {
		encBufferPool.Put(w.buf)
	}
	*w = EncoderBuffer{}
	return err
}

// ToBytes returns the encoded bytes.
func (w *EncoderBuffer) ToBytes() []byte {
	return w.buf.makeBytes()
}

// AppendToBytes appends the encoded bytes to dst.
func (w *EncoderBuffer) AppendToBytes(dst []byte) []byte {
	size := w.buf.size()
	out := append(dst, make([]byte, size)...)
	w.buf.copyTo(out[len(dst):])
	return out
}

// Write appends b directly to the encoder output.
func (w EncoderBuffer) Write(b []byte) (int, error) {
	return w.buf.Write(b)
}

// WriteBool writes b as the integer 0 (false) or 1 (true).
func (w EncoderBuffer) WriteBool(b bool) {
	w.buf.writeBool(b)
}

// WriteUint64 encodes an unsigned integer.
func (w EncoderBuffer) WriteUint64(i uint64) {
	w.buf.writeUint64(i)
}

// WriteBigInt encodes a big.Int as an RLP string.
// Note: Unlike with Encode, the sign of i is ignored.
func (w EncoderBuffer) WriteBigInt(i *big.Int) {
	w.buf.writeBigInt(i)
}

// WriteUint256 encodes uint256.Int as an RLP string.
func (w EncoderBuffer) WriteUint256(i *uint256.Int) {
	w.buf.writeUint256(i)
}

// WriteBytes encodes b as an RLP string.
func (w EncoderBuffer) WriteBytes(b []byte) {
	w.buf.writeBytes(b)
}

// WriteString encodes s as an RLP string.
func (w EncoderBuffer) WriteString(s string) {
	w.buf.writeString(s)
}

// List starts a list. It returns an internal index. Call EndList with
// this index after encoding the content to finish the list.
func (w EncoderBuffer) List() int {
	return w.buf.list()
}

// ListEnd finishes the given list.
func (w EncoderBuffer) ListEnd(index int) {
	w.buf.listEnd(index)
}
