//go:build !jemalloc
// +build !jemalloc

package main

// #include <stdlib.h>
import "C"
import (
	"log"
	"reflect"
	"sync/atomic"
	"unsafe"
)

func Calloc(size int) []byte {
	if size == 0 {
		return make([]byte, 0)
	}
	ptr := C.calloc(C.size_t(size), 1)
	if ptr == nil {
		panic("OOM")
	}
	hdr := reflect.SliceHeader{Data: uintptr(ptr), Len: size, Cap: size}
	atomic.AddInt64(&numbytes, int64(size))
	//nolint:govet
	return *(*[]byte)(unsafe.Pointer(&hdr))
}

func Free(bs []byte) {
	if len(bs) == 0 {
		return
	}

	if sz := cap(bs); sz != 0 {
		bs = bs[:cap(bs)]
		C.free(unsafe.Pointer(&bs[0]))
		atomic.AddInt64(&numbytes, -int64(sz))
	}
}

func NumAllocBytes() int64 { return atomic.LoadInt64(&numbytes) }

func check() {}

func init() {
	log.Println("USING CALLOC")
}
