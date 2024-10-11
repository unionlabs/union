package decoding

import (
	"encoding/binary"
	"errors"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

var (
	typeMapStringInt   = reflect.TypeOf(map[string]int{})
	typeMapStringInt8  = reflect.TypeOf(map[string]int8{})
	typeMapStringInt16 = reflect.TypeOf(map[string]int16{})
	typeMapStringInt32 = reflect.TypeOf(map[string]int32{})
	typeMapStringInt64 = reflect.TypeOf(map[string]int64{})

	typeMapStringUint   = reflect.TypeOf(map[string]uint{})
	typeMapStringUint8  = reflect.TypeOf(map[string]uint8{})
	typeMapStringUint16 = reflect.TypeOf(map[string]uint16{})
	typeMapStringUint32 = reflect.TypeOf(map[string]uint32{})
	typeMapStringUint64 = reflect.TypeOf(map[string]uint64{})

	typeMapStringFloat32 = reflect.TypeOf(map[string]float32{})
	typeMapStringFloat64 = reflect.TypeOf(map[string]float64{})

	typeMapStringBool   = reflect.TypeOf(map[string]bool{})
	typeMapStringString = reflect.TypeOf(map[string]string{})

	typeMapIntString   = reflect.TypeOf(map[int]string{})
	typeMapInt8String  = reflect.TypeOf(map[int8]string{})
	typeMapInt16String = reflect.TypeOf(map[int16]string{})
	typeMapInt32String = reflect.TypeOf(map[int32]string{})
	typeMapInt64String = reflect.TypeOf(map[int64]string{})
	typeMapIntBool     = reflect.TypeOf(map[int]bool{})
	typeMapInt8Bool    = reflect.TypeOf(map[int8]bool{})
	typeMapInt16Bool   = reflect.TypeOf(map[int16]bool{})
	typeMapInt32Bool   = reflect.TypeOf(map[int32]bool{})
	typeMapInt64Bool   = reflect.TypeOf(map[int64]bool{})

	typeMapUintString   = reflect.TypeOf(map[uint]string{})
	typeMapUint8String  = reflect.TypeOf(map[uint8]string{})
	typeMapUint16String = reflect.TypeOf(map[uint16]string{})
	typeMapUint32String = reflect.TypeOf(map[uint32]string{})
	typeMapUint64String = reflect.TypeOf(map[uint64]string{})
	typeMapUintBool     = reflect.TypeOf(map[uint]bool{})
	typeMapUint8Bool    = reflect.TypeOf(map[uint8]bool{})
	typeMapUint16Bool   = reflect.TypeOf(map[uint16]bool{})
	typeMapUint32Bool   = reflect.TypeOf(map[uint32]bool{})
	typeMapUint64Bool   = reflect.TypeOf(map[uint64]bool{})

	typeMapFloat32String = reflect.TypeOf(map[float32]string{})
	typeMapFloat64String = reflect.TypeOf(map[float64]string{})
	typeMapFloat32Bool   = reflect.TypeOf(map[float32]bool{})
	typeMapFloat64Bool   = reflect.TypeOf(map[float64]bool{})
)

func (d *decoder) isFixMap(v byte) bool {
	return def.FixMap <= v && v <= def.FixMap+0x0f
}

func (d *decoder) mapLength(offset int, k reflect.Kind) (int, int, error) {
	code, offset, err := d.readSize1(offset)
	if err != nil {
		return 0, 0, err
	}

	switch {
	case d.isFixMap(code):
		return int(code - def.FixMap), offset, nil
	case code == def.Map16:
		bs, offset, err := d.readSize2(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(binary.BigEndian.Uint16(bs)), offset, nil
	case code == def.Map32:
		bs, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(binary.BigEndian.Uint32(bs)), offset, nil
	}

	return 0, 0, d.errorTemplate(code, k)
}

func (d *decoder) hasRequiredLeastMapSize(offset, length int) error {
	// minimum check (byte length)
	if len(d.data[offset:]) < length*2 {
		return errors.New("data length lacks to create map")
	}
	return nil
}

func (d *decoder) asFixedMap(rv reflect.Value, offset int, l int) (int, bool, error) {
	t := rv.Type()

	keyKind := t.Key().Kind()
	valueKind := t.Elem().Kind()

	switch t {
	case typeMapStringInt:
		m := make(map[string]int, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asInt(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = int(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringUint:
		m := make(map[string]uint, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asUint(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = uint(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringFloat32:
		m := make(map[string]float32, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asFloat32(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringFloat64:
		m := make(map[string]float64, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asFloat64(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringBool:
		m := make(map[string]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringString:
		m := make(map[string]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringInt8:
		m := make(map[string]int8, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asInt(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = int8(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringInt16:
		m := make(map[string]int16, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asInt(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = int16(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringInt32:
		m := make(map[string]int32, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asInt(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = int32(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringInt64:
		m := make(map[string]int64, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asInt(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringUint8:
		m := make(map[string]uint8, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asUint(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = uint8(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil
	case typeMapStringUint16:
		m := make(map[string]uint16, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asUint(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = uint16(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringUint32:
		m := make(map[string]uint32, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asUint(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = uint32(v)
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapStringUint64:
		m := make(map[string]uint64, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asString(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asUint(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapIntString:
		m := make(map[int]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt8String:
		m := make(map[int8]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int8(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt16String:
		m := make(map[int16]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int16(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt32String:
		m := make(map[int32]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int32(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt64String:
		m := make(map[int64]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapIntBool:
		m := make(map[int]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt8Bool:
		m := make(map[int8]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int8(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt16Bool:
		m := make(map[int16]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int16(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt32Bool:
		m := make(map[int32]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[int32(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapInt64Bool:
		m := make(map[int64]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asInt(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUintString:
		m := make(map[uint]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint8String:
		m := make(map[uint8]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint8(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint16String:
		m := make(map[uint16]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint16(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint32String:
		m := make(map[uint32]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint32(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint64String:
		m := make(map[uint64]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUintBool:
		m := make(map[uint]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint8Bool:
		m := make(map[uint8]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint8(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint16Bool:
		m := make(map[uint16]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint16(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint32Bool:
		m := make(map[uint32]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[uint32(k)] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapUint64Bool:
		m := make(map[uint64]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asUint(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapFloat32String:
		m := make(map[float32]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asFloat32(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapFloat64String:
		m := make(map[float64]string, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asFloat64(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asString(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapFloat32Bool:
		m := make(map[float32]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asFloat32(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil

	case typeMapFloat64Bool:
		m := make(map[float64]bool, l)
		for i := 0; i < l; i++ {
			k, o, err := d.asFloat64(offset, keyKind)
			if err != nil {
				return 0, false, err
			}
			v, o, err := d.asBool(o, valueKind)
			if err != nil {
				return 0, false, err
			}
			m[k] = v
			offset = o
		}
		rv.Set(reflect.ValueOf(m))
		return offset, true, nil
	}

	return offset, false, nil
}
