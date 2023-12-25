package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"
import "unsafe"

// Slice is used as a wrapper for non-copy values
type Slice struct {
	data  *C.char
	size  C.size_t
	freed bool
}

// Slices is collection of Slice.
type Slices []*Slice

// Destroy free slices.
func (slices Slices) Destroy() {
	for _, s := range slices {
		s.Free()
	}
}

// NewSlice returns a slice with the given data.
func NewSlice(data *C.char, size C.size_t) *Slice {
	return &Slice{data, size, false}
}

// Exists returns if underlying data exists.
func (s *Slice) Exists() bool {
	return s.data != nil
}

// Data returns the data of the slice. If the key doesn't exist this will be a
// nil slice.
func (s *Slice) Data() []byte {
	if s.Exists() {
		return refCBytes(s.data, s.size)
	}

	return nil
}

// Size returns the size of the data.
func (s *Slice) Size() int {
	return int(s.size)
}

// Free frees the slice data.
func (s *Slice) Free() {
	if !s.freed {
		C.rocksdb_free(unsafe.Pointer(s.data))
		s.data = nil
		s.freed = true
	}
}

// PinnableSliceHandle represents a handle to a PinnableSlice.
type PinnableSliceHandle struct {
	c *C.rocksdb_pinnableslice_t
}

// NewNativePinnableSliceHandle creates a PinnableSliceHandle object.
func newNativePinnableSliceHandle(c *C.rocksdb_pinnableslice_t) *PinnableSliceHandle {
	return &PinnableSliceHandle{c: c}
}

// Exists returns if underlying data exists.
func (h *PinnableSliceHandle) Exists() bool {
	return h.c != nil
}

// Data returns the data of the slice.
func (h *PinnableSliceHandle) Data() []byte {
	if h.Exists() {
		var cValLen C.size_t
		cValue := C.rocksdb_pinnableslice_value(h.c, &cValLen)
		return refCBytes(cValue, cValLen)
	}

	return nil
}

// Destroy calls the destructor of the underlying pinnable slice handle.
func (h *PinnableSliceHandle) Destroy() {
	C.rocksdb_pinnableslice_destroy(h.c)
	h.c = nil
}
