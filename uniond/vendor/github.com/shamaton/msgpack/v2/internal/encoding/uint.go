package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) calcUint(v uint64) int {
	if v <= math.MaxInt8 {
		// format code only
		return 0
	} else if v <= math.MaxUint8 {
		return def.Byte1
	} else if v <= math.MaxUint16 {
		return def.Byte2
	} else if v <= math.MaxUint32 {
		return def.Byte4
	}
	return def.Byte8
}

func (e *encoder) writeUint(v uint64, offset int) int {
	if v <= math.MaxInt8 {
		offset = e.setByte1Uint64(v, offset)
	} else if v <= math.MaxUint8 {
		offset = e.setByte1Int(def.Uint8, offset)
		offset = e.setByte1Uint64(v, offset)
	} else if v <= math.MaxUint16 {
		offset = e.setByte1Int(def.Uint16, offset)
		offset = e.setByte2Uint64(v, offset)
	} else if v <= math.MaxUint32 {
		offset = e.setByte1Int(def.Uint32, offset)
		offset = e.setByte4Uint64(v, offset)
	} else {
		offset = e.setByte1Int(def.Uint64, offset)
		offset = e.setByte8Uint64(v, offset)
	}
	return offset
}
