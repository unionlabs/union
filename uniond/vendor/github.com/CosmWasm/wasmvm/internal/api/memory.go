package api

/*
#include "bindings.h"
*/
import "C"

import "unsafe"

// makeView creates a view into the given byte slice what allows Rust code to read it.
// The byte slice is managed by Go and will be garbage collected. Use runtime.KeepAlive
// to ensure the byte slice lives long enough.
func makeView(s []byte) C.ByteSliceView {
	if s == nil {
		return C.ByteSliceView{is_nil: true, ptr: cu8_ptr(nil), len: cusize(0)}
	}

	// In Go, accessing the 0-th element of an empty array triggers a panic. That is why in the case
	// of an empty `[]byte` we can't get the internal heap pointer to the underlying array as we do
	// below with `&data[0]`. https://play.golang.org/p/xvDY3g9OqUk
	if len(s) == 0 {
		return C.ByteSliceView{is_nil: false, ptr: cu8_ptr(nil), len: cusize(0)}
	}

	return C.ByteSliceView{
		is_nil: false,
		ptr:    cu8_ptr(unsafe.Pointer(&s[0])),
		len:    cusize(len(s)),
	}
}

// Creates a C.UnmanagedVector, which cannot be done in test files directly
func constructUnmanagedVector(is_none cbool, ptr cu8_ptr, len cusize, cap cusize) C.UnmanagedVector {
	return C.UnmanagedVector{
		is_none: is_none,
		ptr:     ptr,
		len:     len,
		cap:     cap,
	}
}

// uninitializedUnmanagedVector returns an invalid C.UnmanagedVector
// instance. Only use then after someone wrote an instance to it.
func uninitializedUnmanagedVector() C.UnmanagedVector {
	return C.UnmanagedVector{}
}

func newUnmanagedVector(data []byte) C.UnmanagedVector {
	if data == nil {
		return C.new_unmanaged_vector(cbool(true), cu8_ptr(nil), cusize(0))
	} else if len(data) == 0 {
		// in Go, accessing the 0-th element of an empty array triggers a panic. That is why in the case
		// of an empty `[]byte` we can't get the internal heap pointer to the underlying array as we do
		// below with `&data[0]`.
		// https://play.golang.org/p/xvDY3g9OqUk
		return C.new_unmanaged_vector(cbool(false), cu8_ptr(nil), cusize(0))
	} else {
		// This will allocate a proper vector with content and return a description of it
		return C.new_unmanaged_vector(cbool(false), cu8_ptr(unsafe.Pointer(&data[0])), cusize(len(data)))
	}
}

func copyAndDestroyUnmanagedVector(v C.UnmanagedVector) []byte {
	var out []byte
	if v.is_none {
		out = nil
	} else if v.cap == cusize(0) {
		// There is no allocation we can copy
		out = []byte{}
	} else {
		// C.GoBytes create a copy (https://stackoverflow.com/a/40950744/2013738)
		out = C.GoBytes(unsafe.Pointer(v.ptr), cint(v.len))
	}
	C.destroy_unmanaged_vector(v)
	return out
}

// copyU8Slice copies the contents of an Option<&[u8]> that was allocated on the Rust side.
// Returns nil if and only if the source is None.
func copyU8Slice(view C.U8SliceView) []byte {
	if view.is_none {
		return nil
	}
	if view.len == 0 {
		// In this case, we don't want to look into the ptr
		return []byte{}
	}
	// C.GoBytes create a copy (https://stackoverflow.com/a/40950744/2013738)
	res := C.GoBytes(unsafe.Pointer(view.ptr), cint(view.len))
	return res
}
