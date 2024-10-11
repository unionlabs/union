package decoding

import (
	"encoding/binary"
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) asFloat32(offset int, k reflect.Kind) (float32, int, error) {
	code, _, err := d.readSize1(offset)
	if err != nil {
		return 0, 0, err
	}

	switch {
	case code == def.Float32:
		offset++
		bs, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		v := math.Float32frombits(binary.BigEndian.Uint32(bs))
		return v, offset, nil

	case d.isPositiveFixNum(code), code == def.Uint8, code == def.Uint16, code == def.Uint32, code == def.Uint64:
		v, offset, err := d.asUint(offset, k)
		if err != nil {
			break
		}
		return float32(v), offset, nil

	case d.isNegativeFixNum(code), code == def.Int8, code == def.Int16, code == def.Int32, code == def.Int64:
		v, offset, err := d.asInt(offset, k)
		if err != nil {
			break
		}
		return float32(v), offset, nil

	case code == def.Nil:
		offset++
		return 0, offset, nil
	}
	return 0, 0, d.errorTemplate(code, k)
}

func (d *decoder) asFloat64(offset int, k reflect.Kind) (float64, int, error) {
	code, _, err := d.readSize1(offset)
	if err != nil {
		return 0, 0, err
	}

	switch {
	case code == def.Float64:
		offset++
		bs, offset, err := d.readSize8(offset)
		if err != nil {
			return 0, 0, err
		}
		v := math.Float64frombits(binary.BigEndian.Uint64(bs))
		return v, offset, nil

	case code == def.Float32:
		offset++
		bs, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		v := math.Float32frombits(binary.BigEndian.Uint32(bs))
		return float64(v), offset, nil

	case d.isPositiveFixNum(code), code == def.Uint8, code == def.Uint16, code == def.Uint32, code == def.Uint64:
		v, offset, err := d.asUint(offset, k)
		if err != nil {
			break
		}
		return float64(v), offset, nil

	case d.isNegativeFixNum(code), code == def.Int8, code == def.Int16, code == def.Int32, code == def.Int64:
		v, offset, err := d.asInt(offset, k)
		if err != nil {
			break
		}
		return float64(v), offset, nil

	case code == def.Nil:
		offset++
		return 0, offset, nil
	}
	return 0, 0, d.errorTemplate(code, k)
}
