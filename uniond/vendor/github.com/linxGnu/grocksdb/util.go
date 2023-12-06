package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import (
	"errors"
	"unsafe"
)

// boolToChar converts a bool value to C.uchar.
func boolToChar(b bool) C.uchar {
	if b {
		return 1
	}
	return 0
}

// charToBool converts C.uchar to bool value
func charToBool(c C.uchar) bool {
	return c != 0
}

// refCBytes referencing *C.char.
func refCBytes(data *C.char, len C.size_t) []byte {
	return unsafe.Slice((*byte)(unsafe.Pointer(data)), int(len))
}

// refGoBytes returns *C.char from byte slice.
func refGoBytes(b []byte) *C.char {
	var c *C.char
	if len(b) > 0 {
		c = (*C.char)(unsafe.Pointer(&b[0]))
	}
	return c
}

// Go []byte to C string
// The C string is allocated in the C heap using malloc.
func cByteSlice(b []byte) *C.char {
	var c *C.char
	if len(b) > 0 {
		c = (*C.char)(C.CBytes(b))
	}
	return c
}

// charSlice converts a C array of *char to a []*C.char.
func charSlice(data **C.char, len C.int) []*C.char {
	return unsafe.Slice(data, int(len))
}

// charSliceIntoStringSlice converts a C array of *char to a []string.
func charSliceIntoStringSlice(data **C.char, len C.int) []string {
	s := charSlice(data, len)

	result := make([]string, int(len))
	for i := range s {
		result[i] = C.GoString(s[i])
	}

	return result
}

// sizeSlice converts a C array of size_t to a []C.size_t.
func sizeSlice(data *C.size_t, len C.int) []C.size_t {
	return unsafe.Slice(data, int(len))
}

// fromCError returns go error and free c_err if need.
func fromCError(cErr *C.char) (err error) {
	if cErr != nil {
		err = errors.New(C.GoString(cErr))
		C.rocksdb_free(unsafe.Pointer(cErr))
	}
	return
}

func toString(cVal *C.char, len C.int) string {
	s := C.GoStringN(cVal, C.int(len))
	C.rocksdb_free(unsafe.Pointer(cVal))
	return s
}
