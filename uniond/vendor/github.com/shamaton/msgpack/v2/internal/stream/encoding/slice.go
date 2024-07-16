package encoding

import (
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) writeSliceLength(l int) error {
	// format size
	if l <= 0x0f {
		if err := e.setByte1Int(def.FixArray + l); err != nil {
			return err
		}
	} else if l <= math.MaxUint16 {
		if err := e.setByte1Int(def.Array16); err != nil {
			return err
		}
		if err := e.setByte2Int(l); err != nil {
			return err
		}
	} else if uint(l) <= math.MaxUint32 {
		if err := e.setByte1Int(def.Array32); err != nil {
			return err
		}
		if err := e.setByte4Int(l); err != nil {
			return err
		}
	}
	return nil
}

func (e *encoder) writeFixedSlice(rv reflect.Value) (bool, error) {

	switch sli := rv.Interface().(type) {
	case []int:
		for _, v := range sli {
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []uint:
		for _, v := range sli {
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []string:
		for _, v := range sli {
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case []float32:
		for _, v := range sli {
			if err := e.writeFloat32(float64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []float64:
		for _, v := range sli {
			if err := e.writeFloat64(float64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []bool:
		for _, v := range sli {
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case []int8:
		for _, v := range sli {
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []int16:
		for _, v := range sli {
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []int32:
		for _, v := range sli {
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []int64:
		for _, v := range sli {
			if err := e.writeInt(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case []uint8:
		for _, v := range sli {
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []uint16:
		for _, v := range sli {
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []uint32:
		for _, v := range sli {
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case []uint64:
		for _, v := range sli {
			if err := e.writeUint(v); err != nil {
				return false, err
			}
		}
		return true, nil
	}

	return false, nil
}
