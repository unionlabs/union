package xerial

import (
	"bytes"
	"encoding/binary"
	"errors"

	"github.com/klauspost/compress/s2"
)

var (
	xerialHeader = []byte{130, 83, 78, 65, 80, 80, 89, 0}

	// This is xerial version 1 and minimally compatible with version 1
	xerialVersionInfo = []byte{0, 0, 0, 1, 0, 0, 0, 1}

	// ErrMalformed is returned by the decoder when the xerial framing
	// is malformed
	ErrMalformed = errors.New("malformed xerial framing")
)

// Encode *appends* to the specified 'dst' the compressed
// 'src' in xerial framing format. If 'dst' does not have enough
// capacity, then a new slice will be allocated. If 'dst' has
// non-zero length, then if *must* have been built using this function.
func Encode(dst, src []byte) []byte {
	if len(dst) == 0 {
		dst = append(dst, xerialHeader...)
		dst = append(dst, xerialVersionInfo...)
	}

	// Snappy encode in blocks of maximum 32KB
	var (
		max       = len(src)
		blockSize = 32 * 1024
		pos       = 0
		chunk     []byte
	)

	for pos < max {
		newPos := min(pos+blockSize, max)
		// Find maximum length we need
		needLen := s2.MaxEncodedLen(newPos-pos) + 4
		if cap(dst)-len(dst) >= needLen {
			// Encode directly into dst
			dstStart := len(dst) + 4             // Start offset in dst
			dstSizePos := dst[len(dst):dstStart] // Reserve space for compressed size
			dstEnd := len(dst) + needLen         // End offset in dst
			// Compress into dst and get actual size.
			actual := s2.EncodeSnappy(dst[dstStart:dstEnd], src[pos:newPos])
			// Update dst size
			dst = dst[:dstStart+len(actual)]
			// Store compressed size
			binary.BigEndian.PutUint32(dstSizePos, uint32(len(actual)))
		} else {
			chunk = s2.EncodeSnappy(chunk[:cap(chunk)], src[pos:newPos])
			origLen := len(dst)
			// First encode the compressed size (big-endian)
			// Put* panics if the buffer is too small, so pad 4 bytes first
			dst = append(dst, dst[0:4]...)
			binary.BigEndian.PutUint32(dst[origLen:], uint32(len(chunk)))
			// And now the compressed data
			dst = append(dst, chunk...)
		}
		pos = newPos
	}
	return dst
}

// EncodeBetter *appends* to the specified 'dst' the compressed
// 'src' in xerial framing format. If 'dst' does not have enough
// capacity, then a new slice will be allocated. If 'dst' has
// non-zero length, then if *must* have been built using this function.
func EncodeBetter(dst, src []byte) []byte {
	if len(dst) == 0 {
		dst = append(dst, xerialHeader...)
		dst = append(dst, xerialVersionInfo...)
	}

	// Snappy encode in blocks of maximum 32KB
	var (
		max       = len(src)
		blockSize = 32 * 1024
		pos       = 0
		chunk     []byte
	)

	for pos < max {
		newPos := min(pos+blockSize, max)
		// Find maximum length we need
		needLen := s2.MaxEncodedLen(newPos-pos) + 4
		if cap(dst)-len(dst) >= needLen {
			// Encode directly into dst
			dstStart := len(dst) + 4             // Start offset in dst
			dstSizePos := dst[len(dst):dstStart] // Reserve space for compressed size
			dstEnd := len(dst) + needLen         // End offset in dst
			// Compress into dst and get actual size.
			actual := s2.EncodeSnappyBetter(dst[dstStart:dstEnd], src[pos:newPos])
			// Update dst size
			dst = dst[:dstStart+len(actual)]
			// Store compressed size
			binary.BigEndian.PutUint32(dstSizePos, uint32(len(actual)))
		} else {
			chunk = s2.EncodeSnappyBetter(chunk[:cap(chunk)], src[pos:newPos])
			origLen := len(dst)
			// First encode the compressed size (big-endian)
			// Put* panics if the buffer is too small, so pad 4 bytes first
			dst = append(dst, dst[0:4]...)
			binary.BigEndian.PutUint32(dst[origLen:], uint32(len(chunk)))
			// And now the compressed data
			dst = append(dst, chunk...)
		}
		pos = newPos
	}
	return dst
}

func min(x, y int) int {
	if x < y {
		return x
	}
	return y
}

const (
	sizeOffset = 16
	sizeBytes  = 4
)

// Decode decodes snappy data whether it is traditional unframed
// or includes the xerial framing format.
func Decode(src []byte) ([]byte, error) {
	return DecodeInto(nil, src)
}

// DecodeInto decodes snappy data whether it is traditional unframed
// or includes the xerial framing format into the specified `dst`.
// It is assumed that the entirety of `dst` including all capacity is available
// for use by this function. If `dst` is nil *or* insufficiently large to hold
// the decoded `src`, new space will be allocated.
// To never allocate bigger destination, use DecodeCapped.
func DecodeInto(dst, src []byte) ([]byte, error) {
	var max = len(src)

	if max < len(xerialHeader) || !bytes.Equal(src[:8], xerialHeader) {
		dst, err := s2.Decode(dst[:cap(dst)], src)
		if err != nil {
			return dst, ErrMalformed
		}
		return dst, nil
	}
	if max == sizeOffset {
		return []byte{}, nil
	}
	if max < sizeOffset+sizeBytes {
		return nil, ErrMalformed
	}
	if len(dst) > 0 {
		dst = dst[:0]
	}
	var (
		pos   = sizeOffset
		chunk []byte
	)

	for pos+sizeBytes <= max {
		size := int(binary.BigEndian.Uint32(src[pos : pos+sizeBytes]))
		pos += sizeBytes

		nextPos := pos + size
		// On architectures where int is 32-bytes wide size + pos could
		// overflow so we need to check the low bound as well as the
		// high
		if nextPos < pos || nextPos > max {
			return nil, ErrMalformed
		}
		nextLen, err := s2.DecodedLen(src[pos:nextPos])
		if err != nil {
			return nil, err
		}
		if cap(dst)-len(dst) >= nextLen {
			// Decode directly into dst
			dstStart := len(dst)
			dstEnd := dstStart + nextLen
			_, err = s2.Decode(dst[dstStart:dstEnd], src[pos:nextPos])
			if err != nil {
				return nil, err
			}
			dst = dst[:dstEnd]
		} else {
			chunk, err = s2.Decode(chunk[:cap(chunk)], src[pos:nextPos])
			if err != nil {
				return nil, err
			}
			dst = append(dst, chunk...)
		}
		pos = nextPos
	}
	return dst, nil
}

var ErrDstTooSmall = errors.New("destination buffer too small")

// DecodeCapped decodes snappy data whether it is traditional unframed
// or includes the xerial framing format into the specified `dst`.
// It is assumed that the entirety of `dst` including all capacity is available
// for use by this function. If `dst` is nil *or* insufficiently large to hold
// the decoded `src`, ErrDstTooSmall is returned.
func DecodeCapped(dst, src []byte) ([]byte, error) {
	var max = len(src)
	if dst == nil {
		return nil, ErrDstTooSmall
	}
	if max < len(xerialHeader) || !bytes.Equal(src[:8], xerialHeader) {
		l, err := s2.DecodedLen(src)
		if err != nil {
			return nil, ErrMalformed
		}
		if l > cap(dst) {
			return nil, ErrDstTooSmall
		}
		return s2.Decode(dst[:cap(dst)], src)
	}
	dst = dst[:0]
	if max == sizeOffset {
		return dst, nil
	}
	if max < sizeOffset+sizeBytes {
		return nil, ErrMalformed
	}
	pos := sizeOffset

	for pos+sizeBytes <= max {
		size := int(binary.BigEndian.Uint32(src[pos : pos+sizeBytes]))
		pos += sizeBytes

		nextPos := pos + size
		// On architectures where int is 32-bytes wide size + pos could
		// overflow so we need to check the low bound as well as the
		// high
		if nextPos < pos || nextPos > max {
			return nil, ErrMalformed
		}
		nextLen, err := s2.DecodedLen(src[pos:nextPos])
		if err != nil {
			return nil, err
		}
		if cap(dst)-len(dst) < nextLen {
			return nil, ErrDstTooSmall
		}
		// Decode directly into dst
		dstStart := len(dst)
		dstEnd := dstStart + nextLen
		_, err = s2.Decode(dst[dstStart:dstEnd], src[pos:nextPos])
		if err != nil {
			return nil, err
		}
		dst = dst[:dstEnd]
		pos = nextPos
	}
	return dst, nil
}
