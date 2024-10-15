package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) calcFloat32(v float64) int {
	return def.Byte4
}

func (e *encoder) calcFloat64(v float64) int {
	return def.Byte8
}

func (e *encoder) writeFloat32(v float64, offset int) int {
	offset = e.setByte1Int(def.Float32, offset)
	offset = e.setByte4Uint64(uint64(math.Float32bits(float32(v))), offset)
	return offset
}

func (e *encoder) writeFloat64(v float64, offset int) int {
	offset = e.setByte1Int(def.Float64, offset)
	offset = e.setByte8Uint64(math.Float64bits(v), offset)
	return offset
}
