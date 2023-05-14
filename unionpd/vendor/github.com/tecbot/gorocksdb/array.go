package gorocksdb

// #include "stdlib.h"
// #include "rocksdb/c.h"
import "C"
import (
	"reflect"
	"unsafe"
)

type charsSlice []*C.char
type sizeTSlice []C.size_t
type columnFamilySlice []*C.rocksdb_column_family_handle_t

func (s charsSlice) c() **C.char {
	sH := (*reflect.SliceHeader)(unsafe.Pointer(&s))
	return (**C.char)(unsafe.Pointer(sH.Data))
}

func (s sizeTSlice) c() *C.size_t {
	sH := (*reflect.SliceHeader)(unsafe.Pointer(&s))
	return (*C.size_t)(unsafe.Pointer(sH.Data))
}

func (s columnFamilySlice) c() **C.rocksdb_column_family_handle_t {
	sH := (*reflect.SliceHeader)(unsafe.Pointer(&s))
	return (**C.rocksdb_column_family_handle_t)(unsafe.Pointer(sH.Data))
}

// bytesSliceToCSlices converts a slice of byte slices to two slices with C
// datatypes. One containing pointers to copies of the byte slices and one
// containing their sizes.
// IMPORTANT: All the contents of the charsSlice array are malloced and
// should be freed using the Destroy method of charsSlice.
func byteSlicesToCSlices(vals [][]byte) (charsSlice, sizeTSlice) {
	if len(vals) == 0 {
		return nil, nil
	}

	chars := make(charsSlice, len(vals))
	sizes := make(sizeTSlice, len(vals))
	for i, val := range vals {
		chars[i] = (*C.char)(C.CBytes(val))
		sizes[i] = C.size_t(len(val))
	}

	return chars, sizes
}

func (s charsSlice) Destroy() {
	for _, chars := range s {
		C.free(unsafe.Pointer(chars))
	}
}
