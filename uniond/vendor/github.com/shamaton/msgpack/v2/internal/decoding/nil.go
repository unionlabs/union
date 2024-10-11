package decoding

import "github.com/shamaton/msgpack/v2/def"

func (d *decoder) isCodeNil(v byte) bool {
	return def.Nil == v
}
