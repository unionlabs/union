package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) writeUint(v uint64) error {
	if v <= math.MaxInt8 {
		if err := e.setByte1Uint64(v); err != nil {
			return err
		}
	} else if v <= math.MaxUint8 {
		if err := e.setByte1Int(def.Uint8); err != nil {
			return err
		}
		if err := e.setByte1Uint64(v); err != nil {
			return err
		}
	} else if v <= math.MaxUint16 {
		if err := e.setByte1Int(def.Uint16); err != nil {
			return err
		}
		if err := e.setByte2Uint64(v); err != nil {
			return err
		}
	} else if v <= math.MaxUint32 {
		if err := e.setByte1Int(def.Uint32); err != nil {
			return err
		}
		if err := e.setByte4Uint64(v); err != nil {
			return err
		}
	} else {
		if err := e.setByte1Int(def.Uint64); err != nil {
			return err
		}
		if err := e.setByte8Uint64(v); err != nil {
			return err
		}
	}
	return nil
}
