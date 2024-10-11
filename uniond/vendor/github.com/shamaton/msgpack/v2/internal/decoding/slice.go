package decoding

import (
	"encoding/binary"
	"errors"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

var (
	typeIntSlice   = reflect.TypeOf([]int{})
	typeInt8Slice  = reflect.TypeOf([]int8{})
	typeInt16Slice = reflect.TypeOf([]int16{})
	typeInt32Slice = reflect.TypeOf([]int32{})
	typeInt64Slice = reflect.TypeOf([]int64{})

	typeUintSlice   = reflect.TypeOf([]uint{})
	typeUint8Slice  = reflect.TypeOf([]uint8{})
	typeUint16Slice = reflect.TypeOf([]uint16{})
	typeUint32Slice = reflect.TypeOf([]uint32{})
	typeUint64Slice = reflect.TypeOf([]uint64{})

	typeFloat32Slice = reflect.TypeOf([]float32{})
	typeFloat64Slice = reflect.TypeOf([]float64{})

	typeStringSlice = reflect.TypeOf([]string{})

	typeBoolSlice = reflect.TypeOf([]bool{})
)

func (d *decoder) isFixSlice(v byte) bool {
	return def.FixArray <= v && v <= def.FixArray+0x0f
}

func (d *decoder) sliceLength(offset int, k reflect.Kind) (int, int, error) {
	code, offset, err := d.readSize1(offset)
	if err != nil {
		return 0, 0, err
	}

	switch {
	case d.isFixSlice(code):
		return int(code - def.FixArray), offset, nil
	case code == def.Array16:
		bs, offset, err := d.readSize2(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(binary.BigEndian.Uint16(bs)), offset, nil
	case code == def.Array32:
		bs, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(binary.BigEndian.Uint32(bs)), offset, nil
	}
	return 0, 0, d.errorTemplate(code, k)
}

func (d *decoder) hasRequiredLeastSliceSize(offset, length int) error {
	// minimum check (byte length)
	if len(d.data[offset:]) < length {
		return errors.New("data length lacks to create map")
	}
	return nil
}

func (d *decoder) asFixedSlice(rv reflect.Value, offset int, l int) (int, bool, error) {
	t := rv.Type()
	k := t.Elem().Kind()

	switch t {
	case typeIntSlice:
		sli := make([]int, l)
		for i := range sli {
			v, o, err := d.asInt(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = int(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeUintSlice:
		sli := make([]uint, l)
		for i := range sli {
			v, o, err := d.asUint(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = uint(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeStringSlice:
		sli := make([]string, l)
		for i := range sli {
			v, o, err := d.asString(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeBoolSlice:
		sli := make([]bool, l)
		for i := range sli {
			v, o, err := d.asBool(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeFloat32Slice:
		sli := make([]float32, l)
		for i := range sli {
			v, o, err := d.asFloat32(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeFloat64Slice:
		sli := make([]float64, l)
		for i := range sli {
			v, o, err := d.asFloat64(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeInt8Slice:
		sli := make([]int8, l)
		for i := range sli {
			v, o, err := d.asInt(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = int8(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeInt16Slice:
		sli := make([]int16, l)
		for i := range sli {
			v, o, err := d.asInt(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = int16(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeInt32Slice:
		sli := make([]int32, l)
		for i := range sli {
			v, o, err := d.asInt(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = int32(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeInt64Slice:
		sli := make([]int64, l)
		for i := range sli {
			v, o, err := d.asInt(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeUint8Slice:
		sli := make([]uint8, l)
		for i := range sli {
			v, o, err := d.asUint(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = uint8(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeUint16Slice:
		sli := make([]uint16, l)
		for i := range sli {
			v, o, err := d.asUint(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = uint16(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeUint32Slice:
		sli := make([]uint32, l)
		for i := range sli {
			v, o, err := d.asUint(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = uint32(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil

	case typeUint64Slice:
		sli := make([]uint64, l)
		for i := range sli {
			v, o, err := d.asUint(offset, k)
			if err != nil {
				return 0, false, err
			}
			sli[i] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(sli))
		return offset, true, nil
	}

	return offset, false, nil
}
