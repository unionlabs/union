package encoding

import (
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) calcFixedSlice(rv reflect.Value) (int, bool) {
	size := 0

	switch sli := rv.Interface().(type) {
	case []int:
		for _, v := range sli {
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true

	case []uint:
		for _, v := range sli {
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true

	case []string:
		for _, v := range sli {
			size += def.Byte1 + e.calcString(v)
		}
		return size, true

	case []float32:
		for _, v := range sli {
			size += def.Byte1 + e.calcFloat32(float64(v))
		}
		return size, true

	case []float64:
		for _, v := range sli {
			size += def.Byte1 + e.calcFloat64(v)
		}
		return size, true

	case []bool:
		size += def.Byte1 * len(sli)
		return size, true

	case []int8:
		for _, v := range sli {
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true

	case []int16:
		for _, v := range sli {
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true

	case []int32:
		for _, v := range sli {
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true

	case []int64:
		for _, v := range sli {
			size += def.Byte1 + e.calcInt(v)
		}
		return size, true

	case []uint8:
		for _, v := range sli {
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true

	case []uint16:
		for _, v := range sli {
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true

	case []uint32:
		for _, v := range sli {
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true

	case []uint64:
		for _, v := range sli {
			size += def.Byte1 + e.calcUint(v)
		}
		return size, true
	}

	return size, false
}

func (e *encoder) writeSliceLength(l int, offset int) int {
	// format size
	if l <= 0x0f {
		offset = e.setByte1Int(def.FixArray+l, offset)
	} else if l <= math.MaxUint16 {
		offset = e.setByte1Int(def.Array16, offset)
		offset = e.setByte2Int(l, offset)
	} else if uint(l) <= math.MaxUint32 {
		offset = e.setByte1Int(def.Array32, offset)
		offset = e.setByte4Int(l, offset)
	}
	return offset
}

func (e *encoder) writeFixedSlice(rv reflect.Value, offset int) (int, bool) {

	switch sli := rv.Interface().(type) {
	case []int:
		for _, v := range sli {
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true

	case []uint:
		for _, v := range sli {
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true

	case []string:
		for _, v := range sli {
			offset = e.writeString(v, offset)
		}
		return offset, true

	case []float32:
		for _, v := range sli {
			offset = e.writeFloat32(float64(v), offset)
		}
		return offset, true

	case []float64:
		for _, v := range sli {
			offset = e.writeFloat64(float64(v), offset)
		}
		return offset, true

	case []bool:
		for _, v := range sli {
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case []int8:
		for _, v := range sli {
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true

	case []int16:
		for _, v := range sli {
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true

	case []int32:
		for _, v := range sli {
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true

	case []int64:
		for _, v := range sli {
			offset = e.writeInt(v, offset)
		}
		return offset, true

	case []uint8:
		for _, v := range sli {
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true

	case []uint16:
		for _, v := range sli {
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true

	case []uint32:
		for _, v := range sli {
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true

	case []uint64:
		for _, v := range sli {
			offset = e.writeUint(v, offset)
		}
		return offset, true
	}

	return offset, false
}
