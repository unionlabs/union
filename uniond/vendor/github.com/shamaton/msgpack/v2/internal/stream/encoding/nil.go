package encoding

import "github.com/shamaton/msgpack/v2/def"

func (e *encoder) writeNil() error {
	return e.setByte1Int(def.Nil)
}
