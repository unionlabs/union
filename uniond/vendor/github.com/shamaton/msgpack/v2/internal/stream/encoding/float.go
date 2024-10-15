package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) writeFloat32(v float64) error {
	if err := e.setByte1Int(def.Float32); err != nil {
		return err
	}
	if err := e.setByte4Uint64(uint64(math.Float32bits(float32(v)))); err != nil {
		return err
	}
	return nil
}

func (e *encoder) writeFloat64(v float64) error {
	if err := e.setByte1Int(def.Float64); err != nil {
		return err
	}
	if err := e.setByte8Uint64(math.Float64bits(v)); err != nil {
		return err
	}
	return nil
}
