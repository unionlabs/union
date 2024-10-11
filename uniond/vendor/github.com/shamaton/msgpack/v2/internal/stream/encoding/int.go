package encoding

import (
	"math"

	"github.com/shamaton/msgpack/v2/def"
)

func (e *encoder) isNegativeFixInt64(v int64) bool {
	return def.NegativeFixintMin <= v && v <= def.NegativeFixintMax
}

func (e *encoder) writeInt(v int64) error {
	if v >= 0 {
		if err := e.writeUint(uint64(v)); err != nil {
			return err
		}
	} else if e.isNegativeFixInt64(v) {
		if err := e.setByte1Int64(v); err != nil {
			return err
		}
	} else if v >= math.MinInt8 {
		if err := e.setByte1Int(def.Int8); err != nil {
			return err
		}
		if err := e.setByte1Int64(v); err != nil {
			return err
		}
	} else if v >= math.MinInt16 {
		if err := e.setByte1Int(def.Int16); err != nil {
			return err
		}
		if err := e.setByte2Int64(v); err != nil {
			return err
		}
	} else if v >= math.MinInt32 {
		if err := e.setByte1Int(def.Int32); err != nil {
			return err
		}
		if err := e.setByte4Int64(v); err != nil {
			return err
		}
	} else {
		if err := e.setByte1Int(def.Int64); err != nil {
			return err
		}
		if err := e.setByte8Int64(v); err != nil {
			return err
		}
	}
	return nil
}
