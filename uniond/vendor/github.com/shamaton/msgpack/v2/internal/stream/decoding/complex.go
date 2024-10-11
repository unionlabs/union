package decoding

import (
	"encoding/binary"
	"fmt"
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) asComplex64(code byte, k reflect.Kind) (complex64, error) {
	switch code {
	case def.Fixext8:
		t, err := d.readSize1()
		if err != nil {
			return complex(0, 0), err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), fmt.Errorf("fixext8. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, err := d.readSize4()
		if err != nil {
			return complex(0, 0), err
		}
		r := math.Float32frombits(binary.BigEndian.Uint32(rb))

		ib, err := d.readSize4()
		if err != nil {
			return complex(0, 0), err
		}
		i := math.Float32frombits(binary.BigEndian.Uint32(ib))
		return complex(r, i), nil

	case def.Fixext16:
		t, err := d.readSize1()
		if err != nil {
			return complex(0, 0), err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), fmt.Errorf("fixext16. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, err := d.readSize8()
		if err != nil {
			return complex(0, 0), err
		}
		r := math.Float64frombits(binary.BigEndian.Uint64(rb))

		ib, err := d.readSize8()
		if err != nil {
			return complex(0, 0), err
		}
		i := math.Float64frombits(binary.BigEndian.Uint64(ib))
		return complex64(complex(r, i)), nil

	}

	return complex(0, 0), fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}

func (d *decoder) asComplex128(code byte, k reflect.Kind) (complex128, error) {
	switch code {
	case def.Fixext8:
		t, err := d.readSize1()
		if err != nil {
			return complex(0, 0), err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), fmt.Errorf("fixext8. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, err := d.readSize4()
		if err != nil {
			return complex(0, 0), err
		}
		r := math.Float32frombits(binary.BigEndian.Uint32(rb))

		ib, err := d.readSize4()
		if err != nil {
			return complex(0, 0), err
		}
		i := math.Float32frombits(binary.BigEndian.Uint32(ib))
		return complex128(complex(r, i)), nil

	case def.Fixext16:
		t, err := d.readSize1()
		if err != nil {
			return complex(0, 0), err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), fmt.Errorf("fixext16. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, err := d.readSize8()
		if err != nil {
			return complex(0, 0), err
		}
		r := math.Float64frombits(binary.BigEndian.Uint64(rb))
		
		ib, err := d.readSize8()
		if err != nil {
			return complex(0, 0), err
		}
		i := math.Float64frombits(binary.BigEndian.Uint64(ib))
		return complex(r, i), nil

	}

	return complex(0, 0), fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}
