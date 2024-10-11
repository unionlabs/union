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

func (d *decoder) asInt(offset int, k reflect.Kind) (int64, int, error) {

	code, _, err := d.readSize1(offset)
	if err != nil {
		return 0, 0, err
	}

	switch {
	case d.isPositiveFixNum(code):
		b, offset, err := d.readSize1(offset)
		if err != nil {
			return 0, 0, err
		}
		return int64(b), offset, nil

	case d.isNegativeFixNum(code):
		b, offset, err := d.readSize1(offset)
		if err != nil {
			return 0, 0, err
		}
		return int64(int8(b)), offset, nil

	case code == def.Uint8:
		offset++
		b, offset, err := d.readSize1(offset)
		if err != nil {
			return 0, 0, err
		}
		return int64(uint8(b)), offset, nil

	case code == def.Int8:
		offset++
		b, offset, err := d.readSize1(offset)
		if err != nil {
			return 0, 0, err
		}
		return int64(int8(b)), offset, nil

	case code == def.Uint16:
		offset++
		bs, offset, err := d.readSize2(offset)
		if err != nil {
			return 0, 0, err
		}
		v := binary.BigEndian.Uint16(bs)
		return int64(v), offset, nil

	case code == def.Int16:
		offset++
		bs, offset, err := d.readSize2(offset)
		if err != nil {
			return 0, 0, err
		}
		v := int16(binary.BigEndian.Uint16(bs))
		return int64(v), offset, nil

	case code == def.Uint32:
		offset++
		bs, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		v := binary.BigEndian.Uint32(bs)
		return int64(v), offset, nil

	case code == def.Int32:
		offset++
		bs, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		v := int32(binary.BigEndian.Uint32(bs))
		return int64(v), offset, nil

	case code == def.Uint64:
		offset++
		bs, offset, err := d.readSize8(offset)
		if err != nil {
			return 0, 0, err
		}
		return int64(binary.BigEndian.Uint64(bs)), offset, nil

	case code == def.Int64:
		offset++
		bs, offset, err := d.readSize8(offset)
		if err != nil {
			return 0, 0, err
		}
		return int64(binary.BigEndian.Uint64(bs)), offset, nil

	case code == def.Float32:
		v, offset, err := d.asFloat32(offset, k)
		if err != nil {
			return 0, 0, err
		}
		return int64(v), offset, nil

	case code == def.Float64:
		v, offset, err := d.asFloat64(offset, k)
		if err != nil {
			return 0, 0, err
		}
		return int64(v), offset, nil

	case code == def.Nil:
		offset++
		return 0, offset, nil
	}

	return 0, 0, d.errorTemplate(code, k)
}
