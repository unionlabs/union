package encoding

import (
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) calcFixedMap(rv reflect.Value) (int, bool) {
	size := 0

	switch m := rv.Interface().(type) {
	case map[string]int:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true

	case map[string]uint:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true

	case map[string]string:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcString(v)
		}
		return size, true

	case map[string]float32:
		for k := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcFloat32(0)
		}
		return size, true

	case map[string]float64:
		for k := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcFloat64(0)
		}
		return size, true

	case map[string]bool:
		for k := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 /*+ e.calcBool()*/
		}
		return size, true

	case map[string]int8:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true
	case map[string]int16:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true
	case map[string]int32:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcInt(int64(v))
		}
		return size, true
	case map[string]int64:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcInt(v)
		}
		return size, true
	case map[string]uint8:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true
	case map[string]uint16:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true
	case map[string]uint32:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcUint(uint64(v))
		}
		return size, true
	case map[string]uint64:
		for k, v := range m {
			size += def.Byte1 + e.calcString(k)
			size += def.Byte1 + e.calcUint(v)
		}
		return size, true

	case map[int]string:
		for k, v := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[int]bool:
		for k := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true

	case map[uint]string:
		for k, v := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[uint]bool:
		for k := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true

	case map[float32]string:
		for k, v := range m {
			size += def.Byte1 + e.calcFloat32(float64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[float32]bool:
		for k := range m {
			size += def.Byte1 + e.calcFloat32(float64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true

	case map[float64]string:
		for k, v := range m {
			size += def.Byte1 + e.calcFloat64(k)
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[float64]bool:
		for k := range m {
			size += def.Byte1 + e.calcFloat64(k)
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true

	case map[int8]string:
		for k, v := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[int8]bool:
		for k := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true
	case map[int16]string:
		for k, v := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[int16]bool:
		for k := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true
	case map[int32]string:
		for k, v := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[int32]bool:
		for k := range m {
			size += def.Byte1 + e.calcInt(int64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true
	case map[int64]string:
		for k, v := range m {
			size += def.Byte1 + e.calcInt(k)
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[int64]bool:
		for k := range m {
			size += def.Byte1 + e.calcInt(k)
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true

	case map[uint8]string:
		for k, v := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[uint8]bool:
		for k := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true
	case map[uint16]string:
		for k, v := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[uint16]bool:
		for k := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true
	case map[uint32]string:
		for k, v := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[uint32]bool:
		for k := range m {
			size += def.Byte1 + e.calcUint(uint64(k))
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true
	case map[uint64]string:
		for k, v := range m {
			size += def.Byte1 + e.calcUint(k)
			size += def.Byte1 + e.calcString(v)
		}
		return size, true
	case map[uint64]bool:
		for k := range m {
			size += def.Byte1 + e.calcUint(k)
			size += def.Byte1 /* + e.calcBool()*/
		}
		return size, true

	}
	return size, false
}

func (e *encoder) writeMapLength(l int, offset int) int {

	// format
	if l <= 0x0f {
		offset = e.setByte1Int(def.FixMap+l, offset)
	} else if l <= math.MaxUint16 {
		offset = e.setByte1Int(def.Map16, offset)
		offset = e.setByte2Int(l, offset)
	} else if uint(l) <= math.MaxUint32 {
		offset = e.setByte1Int(def.Map32, offset)
		offset = e.setByte4Int(l, offset)
	}
	return offset
}

func (e *encoder) writeFixedMap(rv reflect.Value, offset int) (int, bool) {
	switch m := rv.Interface().(type) {
	case map[string]int:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true

	case map[string]uint:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true

	case map[string]float32:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeFloat32(float64(v), offset)
		}
		return offset, true

	case map[string]float64:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeFloat64(v, offset)
		}
		return offset, true

	case map[string]bool:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case map[string]string:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeString(v, offset)
		}
		return offset, true

	case map[string]int8:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true
	case map[string]int16:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true
	case map[string]int32:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true
	case map[string]int64:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeInt(int64(v), offset)
		}
		return offset, true

	case map[string]uint8:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true
	case map[string]uint16:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true
	case map[string]uint32:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true
	case map[string]uint64:
		for k, v := range m {
			offset = e.writeString(k, offset)
			offset = e.writeUint(uint64(v), offset)
		}
		return offset, true

	case map[int]string:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[int]bool:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case map[uint]string:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[uint]bool:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case map[float32]string:
		for k, v := range m {
			offset = e.writeFloat32(float64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[float32]bool:
		for k, v := range m {
			offset = e.writeFloat32(float64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case map[float64]string:
		for k, v := range m {
			offset = e.writeFloat64(k, offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[float64]bool:
		for k, v := range m {
			offset = e.writeFloat64(k, offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case map[int8]string:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[int8]bool:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true
	case map[int16]string:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[int16]bool:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true
	case map[int32]string:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[int32]bool:
		for k, v := range m {
			offset = e.writeInt(int64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true
	case map[int64]string:
		for k, v := range m {
			offset = e.writeInt(k, offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[int64]bool:
		for k, v := range m {
			offset = e.writeInt(k, offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	case map[uint8]string:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[uint8]bool:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true
	case map[uint16]string:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[uint16]bool:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true
	case map[uint32]string:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[uint32]bool:
		for k, v := range m {
			offset = e.writeUint(uint64(k), offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true
	case map[uint64]string:
		for k, v := range m {
			offset = e.writeUint(k, offset)
			offset = e.writeString(v, offset)
		}
		return offset, true
	case map[uint64]bool:
		for k, v := range m {
			offset = e.writeUint(k, offset)
			offset = e.writeBool(v, offset)
		}
		return offset, true

	}
	return offset, false
}
