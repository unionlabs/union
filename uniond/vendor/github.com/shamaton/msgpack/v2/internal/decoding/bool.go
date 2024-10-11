package decoding

import (
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) asBool(offset int, k reflect.Kind) (bool, int, error) {
	code := d.data[offset]
	offset++

	switch code {
	case def.True:
		return true, offset, nil
	case def.False:
		return false, offset, nil
	}
	return false, 0, d.errorTemplate(code, k)
}
