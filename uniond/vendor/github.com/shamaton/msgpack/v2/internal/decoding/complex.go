package decoding

import (
	"encoding/binary"
	"fmt"
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) asComplex64(offset int, k reflect.Kind) (complex64, int, error) {
	code, offset, err := d.readSize1(offset)
	if err != nil {
		return complex(0, 0), 0, err
	}

	switch code {
	case def.Fixext8:
		t, offset, err := d.readSize1(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), 0, fmt.Errorf("fixext8. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, offset, err := d.readSize4(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		ib, offset, err := d.readSize4(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		r := math.Float32frombits(binary.BigEndian.Uint32(rb))
		i := math.Float32frombits(binary.BigEndian.Uint32(ib))
		return complex(r, i), offset, nil

	case def.Fixext16:
		t, offset, err := d.readSize1(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), 0, fmt.Errorf("fixext16. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, offset, err := d.readSize8(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		ib, offset, err := d.readSize8(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		r := math.Float64frombits(binary.BigEndian.Uint64(rb))
		i := math.Float64frombits(binary.BigEndian.Uint64(ib))
		return complex64(complex(r, i)), offset, nil

	}

	return complex(0, 0), 0, fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}

func (d *decoder) asComplex128(offset int, k reflect.Kind) (complex128, int, error) {
	code, offset, err := d.readSize1(offset)
	if err != nil {
		return complex(0, 0), 0, err
	}

	switch code {
	case def.Fixext8:
		t, offset, err := d.readSize1(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), 0, fmt.Errorf("fixext8. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, offset, err := d.readSize4(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		ib, offset, err := d.readSize4(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		r := math.Float32frombits(binary.BigEndian.Uint32(rb))
		i := math.Float32frombits(binary.BigEndian.Uint32(ib))
		return complex128(complex(r, i)), offset, nil

	case def.Fixext16:
		t, offset, err := d.readSize1(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		if int8(t) != def.ComplexTypeCode() {
			return complex(0, 0), 0, fmt.Errorf("fixext16. complex type is diffrent %d, %d", t, def.ComplexTypeCode())
		}
		rb, offset, err := d.readSize8(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		ib, offset, err := d.readSize8(offset)
		if err != nil {
			return complex(0, 0), 0, err
		}
		r := math.Float64frombits(binary.BigEndian.Uint64(rb))
		i := math.Float64frombits(binary.BigEndian.Uint64(ib))
		return complex(r, i), offset, nil

	}

	return complex(0, 0), 0, fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}
