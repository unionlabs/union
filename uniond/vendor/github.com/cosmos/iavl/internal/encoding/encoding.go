package encoding

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math/bits"
	"sync"
)

var bufPool = &sync.Pool{
	New: func() interface{} {
		return new(bytes.Buffer)
	},
}

var varintPool = &sync.Pool{
	New: func() interface{} {
		return &[binary.MaxVarintLen64]byte{}
	},
}

var uvarintPool = &sync.Pool{
	New: func() interface{} {
		return &[binary.MaxVarintLen64]byte{}
	},
}

// decodeBytes decodes a varint length-prefixed byte slice, returning it along with the number
// of input bytes read.
// Assumes bz will not be mutated.
func DecodeBytes(bz []byte) ([]byte, int, error) {
	s, n, err := DecodeUvarint(bz)
	if err != nil {
		return nil, n, err
	}
	// Make sure size doesn't overflow. ^uint(0) >> 1 will help determine the
	// max int value variably on 32-bit and 64-bit machines. We also doublecheck
	// that size is positive.
	size := int(s)
	if s >= uint64(^uint(0)>>1) || size < 0 {
		return nil, n, fmt.Errorf("invalid out of range length %v decoding []byte", s)
	}
	// Make sure end index doesn't overflow. We know n>0 from decodeUvarint().
	end := n + size
	if end < n {
		return nil, n, fmt.Errorf("invalid out of range length %v decoding []byte", size)
	}
	// Make sure the end index is within bounds.
	if len(bz) < end {
		return nil, n, fmt.Errorf("insufficient bytes decoding []byte of length %v", size)
	}
	return bz[n:end], end, nil
}

// decodeUvarint decodes a varint-encoded unsigned integer from a byte slice, returning it and the
// number of bytes decoded.
func DecodeUvarint(bz []byte) (uint64, int, error) {
	u, n := binary.Uvarint(bz)
	if n == 0 {
		// buf too small
		return u, n, errors.New("buffer too small")
	} else if n < 0 {
		// value larger than 64 bits (overflow)
		// and -n is the number of bytes read
		n = -n
		return u, n, errors.New("EOF decoding uvarint")
	}
	return u, n, nil
}

// decodeVarint decodes a varint-encoded integer from a byte slice, returning it and the number of
// bytes decoded.
func DecodeVarint(bz []byte) (int64, int, error) {
	i, n := binary.Varint(bz)
	if n == 0 {
		return i, n, errors.New("buffer too small")
	} else if n < 0 {
		// value larger than 64 bits (overflow)
		// and -n is the number of bytes read
		n = -n
		return i, n, errors.New("EOF decoding varint")
	}
	return i, n, nil
}

// EncodeBytes writes a varint length-prefixed byte slice to the writer.
func EncodeBytes(w io.Writer, bz []byte) error {
	err := EncodeUvarint(w, uint64(len(bz)))
	if err != nil {
		return err
	}
	_, err = w.Write(bz)
	return err
}

var hashLenBz []byte

func init() {
	hashLenBz = make([]byte, 1)
	binary.PutUvarint(hashLenBz, 32)
}

// Encode 32 byte long hash
func Encode32BytesHash(w io.Writer, bz []byte) error {
	_, err := w.Write(hashLenBz)
	if err != nil {
		return err
	}
	_, err = w.Write(bz)
	return err
}

// encodeBytesSlice length-prefixes the byte slice and returns it.
func EncodeBytesSlice(bz []byte) ([]byte, error) {
	buf := bufPool.Get().(*bytes.Buffer)
	buf.Reset()
	defer bufPool.Put(buf)

	err := EncodeBytes(buf, bz)

	bytesCopy := make([]byte, buf.Len())
	copy(bytesCopy, buf.Bytes())

	return bytesCopy, err
}

// encodeBytesSize returns the byte size of the given slice including length-prefixing.
func EncodeBytesSize(bz []byte) int {
	return EncodeUvarintSize(uint64(len(bz))) + len(bz)
}

// EncodeUvarint writes a varint-encoded unsigned integer to an io.Writer.
func EncodeUvarint(w io.Writer, u uint64) error {
	// See comment in encodeVarint
	buf := uvarintPool.Get().(*[binary.MaxVarintLen64]byte)

	n := binary.PutUvarint(buf[:], u)
	_, err := w.Write(buf[0:n])

	uvarintPool.Put(buf)

	return err
}

// EncodeUvarintSize returns the byte size of the given integer as a varint.
func EncodeUvarintSize(u uint64) int {
	if u == 0 {
		return 1
	}
	return (bits.Len64(u) + 6) / 7
}

// EncodeVarint writes a varint-encoded integer to an io.Writer.
func EncodeVarint(w io.Writer, i int64) error {
	if bw, ok := w.(io.ByteWriter); ok {
		return fVarintEncode(bw, i)
	}

	// Use a pool here to reduce allocations.
	//
	// Though this allocates just 10 bytes on the stack, doing allocation for every calls
	// cost us a huge memory. The profiling show that using pool save us ~30% memory.
	//
	// Since when we don't have concurrent access to the pool, the speed will nearly identical.
	// If we need to support concurrent access, we can accept a *[binary.MaxVarintLen64]byte as
	// input, so the caller can allocate just one and pass the same array pointer to each call.
	buf := varintPool.Get().(*[binary.MaxVarintLen64]byte)

	n := binary.PutVarint(buf[:], i)
	_, err := w.Write(buf[0:n])

	varintPool.Put(buf)

	return err
}

func fVarintEncode(bw io.ByteWriter, x int64) error {
	// Firstly convert it into a uvarint
	ux := uint64(x) << 1
	if x < 0 {
		ux = ^ux
	}
	for ux >= 0x80 { // While there are 7 or more bits in the value, keep going
		// Convert it into a byte then toggle the
		// 7th bit to indicate that more bytes coming.
		// byte(x & 0x7f) is redundant but useful for illustrative
		// purposes when translating to other languages
		if err := bw.WriteByte(byte(ux&0x7f) | 0x80); err != nil {
			return err
		}
		ux >>= 7
	}

	return bw.WriteByte(byte(ux & 0x7f))
}

// EncodeVarintSize returns the byte size of the given integer as a varint.
func EncodeVarintSize(i int64) int {
	ux := uint64(i) << 1
	if i < 0 {
		ux = ^ux
	}
	return EncodeUvarintSize(ux)
}
