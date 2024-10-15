package decoding

import (
	"fmt"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) asInterface(k reflect.Kind) (interface{}, error) {
	code, err := d.readSize1()
	if err != nil {
		return 0, err
	}
	return d.asInterfaceWithCode(code, k)
}

func (d *decoder) asInterfaceWithCode(code byte, k reflect.Kind) (interface{}, error) {
	switch {
	case code == def.Nil:
		return nil, nil

	case code == def.True, code == def.False:
		v, err := d.asBoolWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, nil

	case d.isPositiveFixNum(code), code == def.Uint8:
		v, err := d.asUintWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return uint8(v), err
	case code == def.Uint16:
		v, err := d.asUintWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return uint16(v), err
	case code == def.Uint32:
		v, err := d.asUintWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return uint32(v), err
	case code == def.Uint64:
		v, err := d.asUintWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, err

	case d.isNegativeFixNum(code), code == def.Int8:
		v, err := d.asIntWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return int8(v), err
	case code == def.Int16:
		v, err := d.asIntWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return int16(v), err
	case code == def.Int32:
		v, err := d.asIntWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return int32(v), err
	case code == def.Int64:
		v, err := d.asIntWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, err

	case code == def.Float32:
		v, err := d.asFloat32WithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, err
	case code == def.Float64:
		v, err := d.asFloat64WithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, err

	case d.isFixString(code), code == def.Str8, code == def.Str16, code == def.Str32:
		v, err := d.asStringWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, err

	case code == def.Bin8, code == def.Bin16, code == def.Bin32:
		v, err := d.asBinWithCode(code, k)
		if err != nil {
			return nil, err
		}
		return v, err

	case d.isFixSlice(code), code == def.Array16, code == def.Array32:
		l, err := d.sliceLength(code, k)
		if err != nil {
			return nil, err
		}

		v := make([]interface{}, l)
		for i := 0; i < l; i++ {
			vv, err := d.asInterface(k)
			if err != nil {
				return nil, err
			}
			v[i] = vv
		}
		return v, nil

	case d.isFixMap(code), code == def.Map16, code == def.Map32:
		l, err := d.mapLength(code, k)
		if err != nil {
			return nil, err
		}

		v := make(map[interface{}]interface{}, l)
		for i := 0; i < l; i++ {
			keyCode, err := d.readSize1()
			if err != nil {
				return 0, err
			}

			if d.canSetAsMapKey(keyCode) != nil {
				return nil, err
			}
			key, err := d.asInterfaceWithCode(keyCode, k)
			if err != nil {
				return nil, err
			}
			value, err := d.asInterface(k)
			if err != nil {
				return nil, err
			}
			v[key] = value
		}
		return v, nil
	}

	// ext
	extInnerType, extData, err := d.readIfExtType(code)
	if err != nil {
		return nil, err
	}
	for i := range extCoders {
		if extCoders[i].IsType(code, extInnerType, len(extData)) {
			v, err := extCoders[i].ToValue(code, extData, k)
			if err != nil {
				return nil, err
			}
			return v, nil
		}
	}
	return nil, d.errorTemplate(code, k)
}

func (d *decoder) canSetAsMapKey(code byte) error {
	switch {
	case d.isFixSlice(code), code == def.Array16, code == def.Array32:
		return fmt.Errorf("can not use slice code for map key/ code: %x", code)
	case d.isFixMap(code), code == def.Map16, code == def.Map32:
		return fmt.Errorf("can not use map code for map key/ code: %x", code)
	}
	return nil
}
