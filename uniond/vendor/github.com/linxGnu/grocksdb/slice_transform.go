package grocksdb

// #include "rocksdb/c.h"
import "C"
import "unsafe"

// A SliceTransform can be used as a prefix extractor.
type SliceTransform interface {
	// Transform a src in domain to a dst in the range.
	Transform(src []byte) []byte

	// Determine whether this is a valid src upon the function applies.
	InDomain(src []byte) bool

	// Determine whether dst=Transform(src) for some src.
	InRange(src []byte) bool

	// Return the name of this transformation.
	Name() string

	// Destroy underlying pointer/data
	Destroy()
}

// NewFixedPrefixTransform creates a new fixed prefix transform.
func NewFixedPrefixTransform(prefixLen int) SliceTransform {
	return NewNativeSliceTransform(unsafe.Pointer(C.rocksdb_slicetransform_create_fixed_prefix(C.size_t(prefixLen))))
}

// NewNoopPrefixTransform creates a new no-op prefix transform.
func NewNoopPrefixTransform() SliceTransform {
	return NewNativeSliceTransform(unsafe.Pointer(C.rocksdb_slicetransform_create_noop()))
}

// NewNativeSliceTransform creates a SliceTransform object.
func NewNativeSliceTransform(c unsafe.Pointer) SliceTransform {
	return &nativeSliceTransform{c: (*C.rocksdb_slicetransform_t)(c)}
}

type nativeSliceTransform struct {
	c *C.rocksdb_slicetransform_t
}

func (st *nativeSliceTransform) Transform(src []byte) []byte { return nil }
func (st *nativeSliceTransform) InDomain(src []byte) bool    { return false }
func (st *nativeSliceTransform) InRange(src []byte) bool     { return false }
func (st *nativeSliceTransform) Name() string                { return "" }
func (st *nativeSliceTransform) Destroy() {
	C.rocksdb_slicetransform_destroy(st.c)
	st.c = nil
}

// Hold references to slice transforms.
var sliceTransforms = NewCOWList()

type sliceTransformWrapper struct {
	name           *C.char
	sliceTransform SliceTransform
}

func registerSliceTransform(st SliceTransform) int {
	return sliceTransforms.Append(sliceTransformWrapper{C.CString(st.Name()), st})
}

//export gorocksdb_slicetransform_transform
func gorocksdb_slicetransform_transform(idx int, cKey *C.char, cKeyLen C.size_t, cDstLen *C.size_t) *C.char {
	key := refCBytes(cKey, cKeyLen)
	dst := sliceTransforms.Get(idx).(sliceTransformWrapper).sliceTransform.Transform(key)
	*cDstLen = C.size_t(len(dst))
	return cByteSlice(dst)
}

//export gorocksdb_slicetransform_in_domain
func gorocksdb_slicetransform_in_domain(idx int, cKey *C.char, cKeyLen C.size_t) C.uchar {
	key := refCBytes(cKey, cKeyLen)
	inDomain := sliceTransforms.Get(idx).(sliceTransformWrapper).sliceTransform.InDomain(key)
	return boolToChar(inDomain)
}

//export gorocksdb_slicetransform_in_range
func gorocksdb_slicetransform_in_range(idx int, cKey *C.char, cKeyLen C.size_t) C.uchar {
	key := refCBytes(cKey, cKeyLen)
	inRange := sliceTransforms.Get(idx).(sliceTransformWrapper).sliceTransform.InRange(key)
	return boolToChar(inRange)
}

//export gorocksdb_slicetransform_name
func gorocksdb_slicetransform_name(idx int) *C.char {
	return sliceTransforms.Get(idx).(sliceTransformWrapper).name
}
