package encoding

import "github.com/shamaton/msgpack/v2/def"

//func (e *encoder) calcBool() int {
//	return 0
//}

func (e *encoder) writeBool(v bool, offset int) int {
	if v {
		offset = e.setByte1Int(def.True, offset)
	} else {
		offset = e.setByte1Int(def.False, offset)
	}
	return offset
}
