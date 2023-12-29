package zstd

/*
// support decoding of "legacy" zstd payloads from versions [0.4, 0.8], matching the
// default configuration of the zstd command line tool:
// https://github.com/facebook/zstd/blob/dev/programs/README.md
#cgo CFLAGS: -DZSTD_LEGACY_SUPPORT=4 -DZSTD_MULTITHREAD=1

#include "zstd.h"
*/
import "C"
import (
	"bytes"
	"errors"
	"io/ioutil"
	"unsafe"
)

// Defines best and standard values for zstd cli
const (
	BestSpeed          = 1
	BestCompression    = 20
	DefaultCompression = 5
)

var (
	// ErrEmptySlice is returned when there is nothing to compress
	ErrEmptySlice = errors.New("Bytes slice is empty")
)

const (
	// decompressSizeBufferLimit is the limit we set on creating a decompression buffer for the Decompress API
	// This is made to prevent DOS from maliciously-created payloads (aka zipbomb).
	// For large payloads with a compression ratio > 10, you can do your own allocation and pass it to the method:
	// dst := make([]byte, 1GB)
	// decompressed, err := zstd.Decompress(dst, src)
	decompressSizeBufferLimit = 1000 * 1000

	zstdFrameHeaderSizeMin = 2 // From zstd.h. Since it's experimental API, hardcoding it
)

// CompressBound returns the worst case size needed for a destination buffer,
// which can be used to preallocate a destination buffer or select a previously
// allocated buffer from a pool.
// See zstd.h to mirror implementation of ZSTD_COMPRESSBOUND
func CompressBound(srcSize int) int {
	lowLimit := 128 << 10 // 128 kB
	var margin int
	if srcSize < lowLimit {
		margin = (lowLimit - srcSize) >> 11
	}
	return srcSize + (srcSize >> 8) + margin
}

// cCompressBound is a cgo call to check the go implementation above against the c code.
func cCompressBound(srcSize int) int {
	return int(C.ZSTD_compressBound(C.size_t(srcSize)))
}

// decompressSizeHint tries to give a hint on how much of the output buffer size we should have
// based on zstd frame descriptors. To prevent DOS from maliciously-created payloads, limit the size
func decompressSizeHint(src []byte) int {
	// 1 MB or 10x input size
	upperBound := 10 * len(src)
	if upperBound < decompressSizeBufferLimit {
		upperBound = decompressSizeBufferLimit
	}

	hint := upperBound
	if len(src) >= zstdFrameHeaderSizeMin {
		hint = int(C.ZSTD_getFrameContentSize(unsafe.Pointer(&src[0]), C.size_t(len(src))))
		if hint < 0 { // On error, just use upperBound
			hint = upperBound
		}
		if hint == 0 { // When compressing the empty slice, we need an output of at least 1 to pass down to the C lib
			hint = 1
		}
	}

	// Take the minimum of both
	if hint > upperBound {
		return upperBound
	}
	return hint
}

// Compress src into dst.  If you have a buffer to use, you can pass it to
// prevent allocation.  If it is too small, or if nil is passed, a new buffer
// will be allocated and returned.
func Compress(dst, src []byte) ([]byte, error) {
	return CompressLevel(dst, src, DefaultCompression)
}

// CompressLevel is the same as Compress but you can pass a compression level
func CompressLevel(dst, src []byte, level int) ([]byte, error) {
	bound := CompressBound(len(src))
	if cap(dst) >= bound {
		dst = dst[0:bound] // Reuse dst buffer
	} else {
		dst = make([]byte, bound)
	}

	// We need unsafe.Pointer(&src[0]) in the Cgo call to avoid "Go pointer to Go pointer" panics.
	// This means we need to special case empty input. See:
	// https://github.com/golang/go/issues/14210#issuecomment-346402945
	var cWritten C.size_t
	if len(src) == 0 {
		cWritten = C.ZSTD_compress(
			unsafe.Pointer(&dst[0]),
			C.size_t(len(dst)),
			unsafe.Pointer(nil),
			C.size_t(0),
			C.int(level))
	} else {
		cWritten = C.ZSTD_compress(
			unsafe.Pointer(&dst[0]),
			C.size_t(len(dst)),
			unsafe.Pointer(&src[0]),
			C.size_t(len(src)),
			C.int(level))
	}

	written := int(cWritten)
	// Check if the return is an Error code
	if err := getError(written); err != nil {
		return nil, err
	}
	return dst[:written], nil
}

// Decompress src into dst.  If you have a buffer to use, you can pass it to
// prevent allocation.  If it is too small, or if nil is passed, a new buffer
// will be allocated and returned.
func Decompress(dst, src []byte) ([]byte, error) {
	if len(src) == 0 {
		return []byte{}, ErrEmptySlice
	}

	bound := decompressSizeHint(src)
	if cap(dst) >= bound {
		dst = dst[0:cap(dst)]
	} else {
		dst = make([]byte, bound)
	}

	written := int(C.ZSTD_decompress(
		unsafe.Pointer(&dst[0]),
		C.size_t(len(dst)),
		unsafe.Pointer(&src[0]),
		C.size_t(len(src))))
	err := getError(written)
	if err == nil {
		return dst[:written], nil
	}
	if !IsDstSizeTooSmallError(err) {
		return nil, err
	}

	// We failed getting a dst buffer of correct size, use stream API
	r := NewReader(bytes.NewReader(src))
	defer r.Close()
	return ioutil.ReadAll(r)
}
