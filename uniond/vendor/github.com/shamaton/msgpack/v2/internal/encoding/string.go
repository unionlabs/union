package encoding

import (
	"math"
	"unsafe"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) calcString(v string) int {
	// NOTE : unsafe
	strBytes := *(*[]byte)(unsafe.Pointer(&v))
	l := len(strBytes)
	if l < 32 {
		return l
	} else if l <= math.MaxUint8 {
		return def.Byte1 + l
	} else if l <= math.MaxUint16 {
		return def.Byte2 + l
	}
	return def.Byte4 + l
	// NOTE : length over uint32
}

func (e *encoder) writeString(str string, offset int) int {
	// NOTE : unsafe
	strBytes := *(*[]byte)(unsafe.Pointer(&str))
	l := len(strBytes)
	if l < 32 {
		offset = e.setByte1Int(def.FixStr+l, offset)
	} else if l <= math.MaxUint8 {
		offset = e.setByte1Int(def.Str8, offset)
		offset = e.setByte1Int(l, offset)
	} else if l <= math.MaxUint16 {
		offset = e.setByte1Int(def.Str16, offset)
		offset = e.setByte2Int(l, offset)
	} else {
		offset = e.setByte1Int(def.Str32, offset)
		offset = e.setByte4Int(l, offset)
	}
	offset += copy(e.d[offset:], str)
	return offset
}
