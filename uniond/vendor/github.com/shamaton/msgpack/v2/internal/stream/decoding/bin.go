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

func (d *decoder) asBinWithCode(code byte, k reflect.Kind) ([]byte, error) {

	switch code {
	case def.Bin8:
		l, err := d.readSize1()
		if err != nil {
			return emptyBytes, err
		}
		return d.readSizeN(int(l))

	case def.Bin16:
		bs, err := d.readSize2()
		if err != nil {
			return emptyBytes, err
		}
		return d.readSizeN(int(binary.BigEndian.Uint16(bs)))

	case def.Bin32:
		bs, err := d.readSize4()
		if err != nil {
			return emptyBytes, err
		}
		return d.readSizeN(int(binary.BigEndian.Uint32(bs)))
	}

	return emptyBytes, d.errorTemplate(code, k)
}

func (d *decoder) asBinStringWithCode(code byte, k reflect.Kind) (string, error) {
	bs, err := d.asBinWithCode(code, k)
	return *(*string)(unsafe.Pointer(&bs)), err
}
