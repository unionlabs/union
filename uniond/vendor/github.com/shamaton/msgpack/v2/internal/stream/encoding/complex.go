package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) writeComplex64(v complex64) error {
	if err := e.setByte1Int(def.Fixext8); err != nil {
		return err
	}
	if err := e.setByte1Int(int(def.ComplexTypeCode())); err != nil {
		return err
	}
	if err := e.setByte4Uint64(uint64(math.Float32bits(real(v)))); err != nil {
		return err
	}
	if err := e.setByte4Uint64(uint64(math.Float32bits(imag(v)))); err != nil {
		return err
	}
	return nil
}

func (e *encoder) writeComplex128(v complex128) error {
	if err := e.setByte1Int(def.Fixext16); err != nil {
		return err
	}
	if err := e.setByte1Int(int(def.ComplexTypeCode())); err != nil {
		return err
	}
	if err := e.setByte8Uint64(math.Float64bits(real(v))); err != nil {
		return err
	}
	if err := e.setByte8Uint64(math.Float64bits(imag(v))); err != nil {
		return err
	}
	return nil
}
