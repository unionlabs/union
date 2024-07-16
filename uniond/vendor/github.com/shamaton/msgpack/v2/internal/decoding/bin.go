package decoding

import (
	"encoding/binary"
	"reflect"
	"unsafe"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) isCodeBin(v byte) bool {
	switch v {
	case def.Bin8, def.Bin16, def.Bin32:
		return true
	}
	return false
}

func (d *decoder) asBin(offset int, k reflect.Kind) ([]byte, int, error) {
	code, offset, err := d.readSize1(offset)
	if err != nil {
		return emptyBytes, 0, err
	}

	switch code {
	case def.Bin8:
		l, offset, err := d.readSize1(offset)
		if err != nil {
			return emptyBytes, 0, err
		}
		o := offset + int(uint8(l))
		return d.data[offset:o], o, nil
	case def.Bin16:
		bs, offset, err := d.readSize2(offset)
		o := offset + int(binary.BigEndian.Uint16(bs))
		if err != nil {
			return emptyBytes, 0, err
		}
		return d.data[offset:o], o, nil
	case def.Bin32:
		bs, offset, err := d.readSize4(offset)
		o := offset + int(binary.BigEndian.Uint32(bs))
		if err != nil {
			return emptyBytes, 0, err
		}
		return d.data[offset:o], o, nil
	}

	return emptyBytes, 0, d.errorTemplate(code, k)
}

func (d *decoder) asBinString(offset int, k reflect.Kind) (string, int, error) {
	bs, offset, err := d.asBin(offset, k)
	return *(*string)(unsafe.Pointer(&bs)), offset, err
}
