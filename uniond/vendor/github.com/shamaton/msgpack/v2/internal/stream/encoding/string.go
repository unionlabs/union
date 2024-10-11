package encoding

import (
	"math"
	"unsafe"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) writeString(str string) error {
	// NOTE : unsafe
	strBytes := *(*[]byte)(unsafe.Pointer(&str))
	l := len(strBytes)
	if l < 32 {
		if err := e.setByte1Int(def.FixStr + l); err != nil {
			return err
		}
	} else if l <= math.MaxUint8 {
		if err := e.setByte1Int(def.Str8); err != nil {
			return err
		}
		if err := e.setByte1Int(l); err != nil {
			return err
		}
	} else if l <= math.MaxUint16 {
		if err := e.setByte1Int(def.Str16); err != nil {
			return err
		}
		if err := e.setByte2Int(l); err != nil {
			return err
		}
	} else {
		if err := e.setByte1Int(def.Str32); err != nil {
			return err
		}
		if err := e.setByte4Int(l); err != nil {
			return err
		}
	}
	return e.setBytes(strBytes)
}
