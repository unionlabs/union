package zstd

/*
#include "zstd.h"
*/
import "C"
import (
	"bytes"
	"io/ioutil"
	"runtime"
	"unsafe"
)

type Ctx interface {
	// Compress src into dst.  If you have a buffer to use, you can pass it to
	// prevent allocation.  If it is too small, or if nil is passed, a new buffer
	// will be allocated and returned.
	Compress(dst, src []byte) ([]byte, error)

	// CompressLevel is the same as Compress but you can pass a compression level
	CompressLevel(dst, src []byte, level int) ([]byte, error)

	// Decompress src into dst.  If you have a buffer to use, you can pass it to
	// prevent allocation.  If it is too small, or if nil is passed, a new buffer
	// will be allocated and returned.
	Decompress(dst, src []byte) ([]byte, error)
}

type ctx struct {
	cctx *C.ZSTD_CCtx
	dctx *C.ZSTD_DCtx
}

// Create a new ZStd Context.
//  When compressing/decompressing many times, it is recommended to allocate a
//  context just once, and re-use it for each successive compression operation.
//  This will make workload friendlier for system's memory.
//  Note : re-using context is just a speed / resource optimization.
//         It doesn't change the compression ratio, which remains identical.
//  Note 2 : In multi-threaded environments,
//         use one different context per thread for parallel execution.
//
func NewCtx() Ctx {
	c := &ctx{
		cctx: C.ZSTD_createCCtx(),
		dctx: C.ZSTD_createDCtx(),
	}

	runtime.SetFinalizer(c, finalizeCtx)
	return c
}

func (c *ctx) Compress(dst, src []byte) ([]byte, error) {
	return c.CompressLevel(dst, src, DefaultCompression)
}

func (c *ctx) CompressLevel(dst, src []byte, level int) ([]byte, error) {
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
		cWritten = C.ZSTD_compressCCtx(
			c.cctx,
			unsafe.Pointer(&dst[0]),
			C.size_t(len(dst)),
			unsafe.Pointer(nil),
			C.size_t(0),
			C.int(level))
	} else {
		cWritten = C.ZSTD_compressCCtx(
			c.cctx,
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

func (c *ctx) Decompress(dst, src []byte) ([]byte, error) {
	if len(src) == 0 {
		return []byte{}, ErrEmptySlice
	}

	bound := decompressSizeHint(src)
	if cap(dst) >= bound {
		dst = dst[0:cap(dst)]
	} else {
		dst = make([]byte, bound)
	}

	written := int(C.ZSTD_decompressDCtx(
		c.dctx,
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

func finalizeCtx(c *ctx) {
	C.ZSTD_freeCCtx(c.cctx)
	C.ZSTD_freeDCtx(c.dctx)
}
