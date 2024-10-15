package encoding

import (
	"math"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) writeMapLength(l int) error {

	// format
	if l <= 0x0f {
		if err := e.setByte1Int(def.FixMap + l); err != nil {
			return err
		}
	} else if l <= math.MaxUint16 {
		if err := e.setByte1Int(def.Map16); err != nil {
			return err
		}
		if err := e.setByte2Int(l); err != nil {
			return err
		}
	} else if uint(l) <= math.MaxUint32 {
		if err := e.setByte1Int(def.Map32); err != nil {
			return err
		}
		if err := e.setByte4Int(l); err != nil {
			return err
		}
	}
	return nil
}

func (e *encoder) writeFixedMap(rv reflect.Value) (bool, error) {
	switch m := rv.Interface().(type) {
	case map[string]int:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]uint:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]float32:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeFloat32(float64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]float64:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeFloat64(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]bool:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]string:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]int8:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[string]int16:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[string]int32:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[string]int64:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeInt(int64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[string]uint8:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[string]uint16:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[string]uint32:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[string]uint64:
		for k, v := range m {
			if err := e.writeString(k); err != nil {
				return false, err
			}
			if err := e.writeUint(uint64(v)); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[int]string:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int]bool:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[uint]string:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint]bool:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[float32]string:
		for k, v := range m {
			if err := e.writeFloat32(float64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[float32]bool:
		for k, v := range m {
			if err := e.writeFloat32(float64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[float64]string:
		for k, v := range m {
			if err := e.writeFloat64(k); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[float64]bool:
		for k, v := range m {
			if err := e.writeFloat64(k); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[int8]string:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int8]bool:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int16]string:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int16]bool:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int32]string:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int32]bool:
		for k, v := range m {
			if err := e.writeInt(int64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int64]string:
		for k, v := range m {
			if err := e.writeInt(k); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[int64]bool:
		for k, v := range m {
			if err := e.writeInt(k); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	case map[uint8]string:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint8]bool:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint16]string:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint16]bool:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint32]string:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint32]bool:
		for k, v := range m {
			if err := e.writeUint(uint64(k)); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint64]string:
		for k, v := range m {
			if err := e.writeUint(k); err != nil {
				return false, err
			}
			if err := e.writeString(v); err != nil {
				return false, err
			}
		}
		return true, nil
	case map[uint64]bool:
		for k, v := range m {
			if err := e.writeUint(k); err != nil {
				return false, err
			}
			if err := e.writeBool(v); err != nil {
				return false, err
			}
		}
		return true, nil

	}
	return false, nil
}
