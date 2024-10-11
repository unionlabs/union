package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) isNegativeFixInt64(v int64) bool {
	return def.NegativeFixintMin <= v && v <= def.NegativeFixintMax
}

func (e *encoder) calcInt(v int64) int {
	if v >= 0 {
		return e.calcUint(uint64(v))
	} else if e.isNegativeFixInt64(v) {
		// format code only
		return 0
	} else if v >= math.MinInt8 {
		return def.Byte1
	} else if v >= math.MinInt16 {
		return def.Byte2
	} else if v >= math.MinInt32 {
		return def.Byte4
	}
	return def.Byte8
}

func (e *encoder) writeInt(v int64, offset int) int {
	if v >= 0 {
		offset = e.writeUint(uint64(v), offset)
	} else if e.isNegativeFixInt64(v) {
		offset = e.setByte1Int64(v, offset)
	} else if v >= math.MinInt8 {
		offset = e.setByte1Int(def.Int8, offset)
		offset = e.setByte1Int64(v, offset)
	} else if v >= math.MinInt16 {
		offset = e.setByte1Int(def.Int16, offset)
		offset = e.setByte2Int64(v, offset)
	} else if v >= math.MinInt32 {
		offset = e.setByte1Int(def.Int32, offset)
		offset = e.setByte4Int64(v, offset)
	} else {
		offset = e.setByte1Int(def.Int64, offset)
		offset = e.setByte8Int64(v, offset)
	}
	return offset
}
