package decoding

import (
	"encoding/binary"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) isPositiveFixNum(v byte) bool {
	return def.PositiveFixIntMin <= v && v <= def.PositiveFixIntMax
}

func (d *decoder) isNegativeFixNum(v byte) bool {
	return def.NegativeFixintMin <= int8(v) && int8(v) <= def.NegativeFixintMax
}

func (d *decoder) asInt(k reflect.Kind) (int64, error) {
	code, err := d.readSize1()
	if err != nil {
		return 0, err
	}
	return d.asIntWithCode(code, k)
}

func (d *decoder) asIntWithCode(code byte, k reflect.Kind) (int64, error) {
	switch {
	case d.isPositiveFixNum(code):
		return int64(code), nil

	case d.isNegativeFixNum(code):
		return int64(int8(code)), nil

	case code == def.Uint8:
		b, err := d.readSize1()
		if err != nil {
			return 0, err
		}
		return int64(b), nil

	case code == def.Int8:
		b, err := d.readSize1()
		if err != nil {
			return 0, err
		}
		return int64(int8(b)), nil

	case code == def.Uint16:
		bs, err := d.readSize2()
		if err != nil {
			return 0, err
		}
		v := binary.BigEndian.Uint16(bs)
		return int64(v), nil

	case code == def.Int16:
		bs, err := d.readSize2()
		if err != nil {
			return 0, err
		}
		v := int16(binary.BigEndian.Uint16(bs))
		return int64(v), nil

	case code == def.Uint32:
		bs, err := d.readSize4()
		if err != nil {
			return 0, err
		}
		v := binary.BigEndian.Uint32(bs)
		return int64(v), nil

	case code == def.Int32:
		bs, err := d.readSize4()
		if err != nil {
			return 0, err
		}
		v := int32(binary.BigEndian.Uint32(bs))
		return int64(v), nil

	case code == def.Uint64:
		bs, err := d.readSize8()
		if err != nil {
			return 0, err
		}
		return int64(binary.BigEndian.Uint64(bs)), nil

	case code == def.Int64:
		bs, err := d.readSize8()
		if err != nil {
			return 0, err
		}
		return int64(binary.BigEndian.Uint64(bs)), nil

	case code == def.Float32:
		v, err := d.asFloat32WithCode(code, k)
		if err != nil {
			return 0, err
		}
		return int64(v), nil

	case code == def.Float64:
		v, err := d.asFloat64WithCode(code, k)
		if err != nil {
			return 0, err
		}
		return int64(v), nil

	case code == def.Nil:
		return 0, nil
	}

	return 0, d.errorTemplate(code, k)
}
