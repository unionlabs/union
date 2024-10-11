package decoding

import (
	"encoding/binary"
	"reflect"
	"sync"

	"github.com/shamaton/msgpack/v2/def"
)

type structCacheTypeMap struct {
	keys    [][]byte
	indexes []int
}

type structCacheTypeArray struct {
	m []int
}

// struct cache map
var mapSCTM = sync.Map{}
var mapSCTA = sync.Map{}

func (d *decoder) setStruct(code byte, rv reflect.Value, k reflect.Kind) error {
	if len(extCoders) > 0 {
		innerType, data, err := d.readIfExtType(code)
		if err != nil {
			return err
		}
		if data != nil {
			for i := range extCoders {
				if extCoders[i].IsType(code, innerType, len(data)) {
					v, err := extCoders[i].ToValue(code, data, k)
					if err != nil {
						return err
					}

					// Validate that the receptacle is of the right value type.
					if rv.Type() == reflect.TypeOf(v) {
						rv.Set(reflect.ValueOf(v))
						return nil
					}
				}
			}
		}
	}

	if d.asArray {
		return d.setStructFromArray(code, rv, k)
	}
	return d.setStructFromMap(code, rv, k)
}

func (d *decoder) setStructFromArray(code byte, rv reflect.Value, k reflect.Kind) error {
	// get length
	l, err := d.sliceLength(code, k)
	if err != nil {
		return err
	}

	// find or create reference
	var scta *structCacheTypeArray
	cache, findCache := mapSCTA.Load(rv.Type())
	if !findCache {
		scta = &structCacheTypeArray{}
		for i := 0; i < rv.NumField(); i++ {
			if ok, _ := d.CheckField(rv.Type().Field(i)); ok {
				scta.m = append(scta.m, i)
			}
		}
		mapSCTA.Store(rv.Type(), scta)
	} else {
		scta = cache.(*structCacheTypeArray)
	}
	// set value
	for i := 0; i < l; i++ {
		if i < len(scta.m) {
			err = d.decode(rv.Field(scta.m[i]))
			if err != nil {
				return err
			}
		} else {
			err = d.jumpOffset()
			if err != nil {
				return err
			}
		}
	}
	return nil
}

func (d *decoder) setStructFromMap(code byte, rv reflect.Value, k reflect.Kind) error {
	// get length
	l, err := d.mapLength(code, k)
	if err != nil {
		return err
	}

	var sctm *structCacheTypeMap
	cache, cacheFind := mapSCTM.Load(rv.Type())
	if !cacheFind {
		sctm = &structCacheTypeMap{}
		for i := 0; i < rv.NumField(); i++ {
			if ok, name := d.CheckField(rv.Type().Field(i)); ok {
				sctm.keys = append(sctm.keys, []byte(name))
				sctm.indexes = append(sctm.indexes, i)
			}
		}
		mapSCTM.Store(rv.Type(), sctm)
	} else {
		sctm = cache.(*structCacheTypeMap)
	}

	for i := 0; i < l; i++ {
		dataKey, err := d.asStringByte(k)
		if err != nil {
			return err
		}

		fieldIndex := -1
		for keyIndex, keyBytes := range sctm.keys {
			if len(keyBytes) != len(dataKey) {
				continue
			}

			fieldIndex = sctm.indexes[keyIndex]
			for dataIndex := range dataKey {
				if dataKey[dataIndex] != keyBytes[dataIndex] {
					fieldIndex = -1
					break
				}
			}
			if fieldIndex >= 0 {
				break
			}
		}

		if fieldIndex >= 0 {
			err = d.decode(rv.Field(fieldIndex))
			if err != nil {
				return err
			}
		} else {
			err = d.jumpOffset()
			if err != nil {
				return err
			}
		}
	}
	return nil
}

func (d *decoder) jumpOffset() error {
	code, err := d.readSize1()
	if err != nil {
		return err
	}

	switch {
	case code == def.True, code == def.False, code == def.Nil:
		// do nothing

	case d.isPositiveFixNum(code) || d.isNegativeFixNum(code):
		// do nothing
	case code == def.Uint8, code == def.Int8:
		_, err = d.readSize1()
		return err
	case code == def.Uint16, code == def.Int16:
		_, err = d.readSize2()
		return err
	case code == def.Uint32, code == def.Int32, code == def.Float32:
		_, err = d.readSize4()
		return err
	case code == def.Uint64, code == def.Int64, code == def.Float64:
		_, err = d.readSize8()
		return err

	case d.isFixString(code):
		_, err = d.readSizeN(int(code - def.FixStr))
		return err
	case code == def.Str8, code == def.Bin8:
		b, err := d.readSize1()
		if err != nil {
			return err
		}
		_, err = d.readSizeN(int(b))
		return err
	case code == def.Str16, code == def.Bin16:
		bs, err := d.readSize2()
		if err != nil {
			return err
		}
		_, err = d.readSizeN(int(binary.BigEndian.Uint16(bs)))
		return err
	case code == def.Str32, code == def.Bin32:
		bs, err := d.readSize4()
		if err != nil {
			return err
		}
		_, err = d.readSizeN(int(binary.BigEndian.Uint32(bs)))
		return err

	case d.isFixSlice(code):
		l := int(code - def.FixArray)
		for i := 0; i < l; i++ {
			if err = d.jumpOffset(); err != nil {
				return err
			}
		}
	case code == def.Array16:
		bs, err := d.readSize2()
		if err != nil {
			return err
		}
		l := int(binary.BigEndian.Uint16(bs))
		for i := 0; i < l; i++ {
			if err = d.jumpOffset(); err != nil {
				return err
			}
		}
	case code == def.Array32:
		bs, err := d.readSize4()
		if err != nil {
			return err
		}
		l := int(binary.BigEndian.Uint32(bs))
		for i := 0; i < l; i++ {
			if err = d.jumpOffset(); err != nil {
				return err
			}
		}

	case d.isFixMap(code):
		l := int(code - def.FixMap)
		for i := 0; i < l*2; i++ {
			if err = d.jumpOffset(); err != nil {
				return err
			}
		}
	case code == def.Map16:
		bs, err := d.readSize2()
		if err != nil {
			return err
		}
		l := int(binary.BigEndian.Uint16(bs))
		for i := 0; i < l*2; i++ {
			if err = d.jumpOffset(); err != nil {
				return err
			}
		}
	case code == def.Map32:
		bs, err := d.readSize4()
		if err != nil {
			return err
		}
		l := int(binary.BigEndian.Uint32(bs))
		for i := 0; i < l*2; i++ {
			if err = d.jumpOffset(); err != nil {
				return err
			}
		}

	case code == def.Fixext1:
		_, err = d.readSizeN(def.Byte1 + def.Byte1)
		return err
	case code == def.Fixext2:
		_, err = d.readSizeN(def.Byte1 + def.Byte2)
		return err
	case code == def.Fixext4:
		_, err = d.readSizeN(def.Byte1 + def.Byte4)
		return err
	case code == def.Fixext8:
		_, err = d.readSizeN(def.Byte1 + def.Byte8)
		return err
	case code == def.Fixext16:
		_, err = d.readSizeN(def.Byte1 + def.Byte16)
		return err
		
	case code == def.Ext8:
		b, err := d.readSize1()
		if err != nil {
			return err
		}
		_, err = d.readSizeN(def.Byte1 + int(b))
		return err
	case code == def.Ext16:
		bs, err := d.readSize2()
		if err != nil {
			return err
		}
		_, err = d.readSizeN(def.Byte1 + int(binary.BigEndian.Uint16(bs)))
		return err
	case code == def.Ext32:
		bs, err := d.readSize4()
		if err != nil {
			return err
		}
		_, err = d.readSizeN(def.Byte1 + int(binary.BigEndian.Uint32(bs)))
		return err
	}
	return nil
}
