package zstd

/*
#include "zstd.h"
*/
import "C"
import (
	"errors"
	"runtime"
	"unsafe"
)

var (
	// ErrEmptyDictionary is returned when the given dictionary is empty
	ErrEmptyDictionary = errors.New("Dictionary is empty")
	// ErrBadDictionary is returned when cannot load the given dictionary
	ErrBadDictionary = errors.New("Cannot load dictionary")
)

// BulkProcessor implements Bulk processing dictionary API.
// When compressing multiple messages or blocks using the same dictionary,
// it's recommended to digest the dictionary only once, since it's a costly operation.
// NewBulkProcessor() will create a state from digesting a dictionary.
// The resulting state can be used for future compression/decompression operations with very limited startup cost.
// BulkProcessor can be created once and shared by multiple threads concurrently, since its usage is read-only.
// The state will be freed when gc cleans up BulkProcessor.
type BulkProcessor struct {
	cDict *C.struct_ZSTD_CDict_s
	dDict *C.struct_ZSTD_DDict_s
}

// NewBulkProcessor creates a new BulkProcessor with a pre-trained dictionary and compression level
func NewBulkProcessor(dictionary []byte, compressionLevel int) (*BulkProcessor, error) {
	if len(dictionary) < 1 {
		return nil, ErrEmptyDictionary
	}

	p := &BulkProcessor{}
	runtime.SetFinalizer(p, finalizeBulkProcessor)

	p.cDict = C.ZSTD_createCDict(
		unsafe.Pointer(&dictionary[0]),
		C.size_t(len(dictionary)),
		C.int(compressionLevel),
	)
	if p.cDict == nil {
		return nil, ErrBadDictionary
	}
	p.dDict = C.ZSTD_createDDict(
		unsafe.Pointer(&dictionary[0]),
		C.size_t(len(dictionary)),
	)
	if p.dDict == nil {
		return nil, ErrBadDictionary
	}

	return p, nil
}

// Compress compresses `src` into `dst` with the dictionary given when creating the BulkProcessor.
// If you have a buffer to use, you can pass it to prevent allocation.
// If it is too small, or if nil is passed, a new buffer will be allocated and returned.
func (p *BulkProcessor) Compress(dst, src []byte) ([]byte, error) {
	bound := CompressBound(len(src))
	if cap(dst) >= bound {
		dst = dst[0:bound]
	} else {
		dst = make([]byte, bound)
	}

	cctx := C.ZSTD_createCCtx()
	// We need unsafe.Pointer(&src[0]) in the Cgo call to avoid "Go pointer to Go pointer" panics.
	// This means we need to special case empty input. See:
	// https://github.com/golang/go/issues/14210#issuecomment-346402945
	var cWritten C.size_t
	if len(src) == 0 {
		cWritten = C.ZSTD_compress_usingCDict(
			cctx,
			unsafe.Pointer(&dst[0]),
			C.size_t(len(dst)),
			unsafe.Pointer(nil),
			C.size_t(len(src)),
			p.cDict,
		)
	} else {
		cWritten = C.ZSTD_compress_usingCDict(
			cctx,
			unsafe.Pointer(&dst[0]),
			C.size_t(len(dst)),
			unsafe.Pointer(&src[0]),
			C.size_t(len(src)),
			p.cDict,
		)
	}

	C.ZSTD_freeCCtx(cctx)

	written := int(cWritten)
	if err := getError(written); err != nil {
		return nil, err
	}
	return dst[:written], nil
}

// Decompress decompresses `src` into `dst` with the dictionary given when creating the BulkProcessor.
// If you have a buffer to use, you can pass it to prevent allocation.
// If it is too small, or if nil is passed, a new buffer will be allocated and returned.
func (p *BulkProcessor) Decompress(dst, src []byte) ([]byte, error) {
	if len(src) == 0 {
		return nil, ErrEmptySlice
	}

	contentSize := decompressSizeHint(src)
	if cap(dst) >= contentSize {
		dst = dst[0:contentSize]
	} else {
		dst = make([]byte, contentSize)
	}

	if contentSize == 0 {
		return dst, nil
	}

	dctx := C.ZSTD_createDCtx()
	cWritten := C.ZSTD_decompress_usingDDict(
		dctx,
		unsafe.Pointer(&dst[0]),
		C.size_t(contentSize),
		unsafe.Pointer(&src[0]),
		C.size_t(len(src)),
		p.dDict,
	)
	C.ZSTD_freeDCtx(dctx)

	written := int(cWritten)
	if err := getError(written); err != nil {
		return nil, err
	}

	return dst[:written], nil
}

// finalizeBulkProcessor frees compression and decompression dictionaries from memory
func finalizeBulkProcessor(p *BulkProcessor) {
	if p.cDict != nil {
		C.ZSTD_freeCDict(p.cDict)
	}
	if p.dDict != nil {
		C.ZSTD_freeDDict(p.dDict)
	}
}
