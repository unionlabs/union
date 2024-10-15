package decoding

import (
	"encoding/binary"
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

var emptyString = ""
var emptyBytes = []byte{}

func (d *decoder) isCodeString(code byte) bool {
	return d.isFixString(code) || code == def.Str8 || code == def.Str16 || code == def.Str32
}

func (d *decoder) isFixString(v byte) bool {
	return def.FixStr <= v && v <= def.FixStr+0x1f
}

func (d *decoder) stringByteLength(offset int, k reflect.Kind) (int, int, error) {
	code, offset, err := d.readSize1(offset)
	if err != nil {
		return 0, 0, err
	}

	if def.FixStr <= code && code <= def.FixStr+0x1f {
		l := int(code - def.FixStr)
		return l, offset, nil
	} else if code == def.Str8 {
		b, offset, err := d.readSize1(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(b), offset, nil
	} else if code == def.Str16 {
		b, offset, err := d.readSize2(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(binary.BigEndian.Uint16(b)), offset, nil
	} else if code == def.Str32 {
		b, offset, err := d.readSize4(offset)
		if err != nil {
			return 0, 0, err
		}
		return int(binary.BigEndian.Uint32(b)), offset, nil
	} else if code == def.Nil {
		return 0, offset, nil
	}
	return 0, 0, d.errorTemplate(code, k)
}

func (d *decoder) asString(offset int, k reflect.Kind) (string, int, error) {
	bs, offset, err := d.asStringByte(offset, k)
	if err != nil {
		return emptyString, 0, err
	}
	return string(bs), offset, nil
}

func (d *decoder) asStringByte(offset int, k reflect.Kind) ([]byte, int, error) {
	l, offset, err := d.stringByteLength(offset, k)
	if err != nil {
		return emptyBytes, 0, err
	}

	return d.asStringByteByLength(offset, l, k)
}

func (d *decoder) asStringByteByLength(offset int, l int, k reflect.Kind) ([]byte, int, error) {
	if l < 1 {
		return emptyBytes, offset, nil
	}

	return d.readSizeN(offset, l)
}
