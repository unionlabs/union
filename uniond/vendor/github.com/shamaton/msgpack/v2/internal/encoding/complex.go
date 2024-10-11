package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) calcComplex64() int {
	return def.Byte1 + def.Byte8
}

func (e *encoder) calcComplex128() int {
	return def.Byte1 + def.Byte16
}

func (e *encoder) writeComplex64(v complex64, offset int) int {
	offset = e.setByte1Int(def.Fixext8, offset)
	offset = e.setByte1Int(int(def.ComplexTypeCode()), offset)
	offset = e.setByte4Uint64(uint64(math.Float32bits(real(v))), offset)
	offset = e.setByte4Uint64(uint64(math.Float32bits(imag(v))), offset)
	return offset
}

func (e *encoder) writeComplex128(v complex128, offset int) int {
	offset = e.setByte1Int(def.Fixext16, offset)
	offset = e.setByte1Int(int(def.ComplexTypeCode()), offset)
	offset = e.setByte8Uint64(math.Float64bits(real(v)), offset)
	offset = e.setByte8Uint64(math.Float64bits(imag(v)), offset)
	return offset
}
