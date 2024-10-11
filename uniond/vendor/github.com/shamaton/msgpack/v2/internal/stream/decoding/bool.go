package decoding

import (
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) asBool(k reflect.Kind) (bool, error) {
	code, err := d.readSize1()
	if err != nil {
		return false, err
	}
	return d.asBoolWithCode(code, k)
}

func (d *decoder) asBoolWithCode(code byte, k reflect.Kind) (bool, error) {
	switch code {
	case def.True:
		return true, nil
	case def.False:
		return false, nil
	}
	return false, d.errorTemplate(code, k)
}
