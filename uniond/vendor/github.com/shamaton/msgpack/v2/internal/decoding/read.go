package decoding

import (
	"errors"

	"github.com/shamaton/msgpack/v2/def"
)

func (d *decoder) readSize1(index int) (byte, int, error) {
	rb := def.Byte1
	if len(d.data) < index+rb {
		return 0, 0, errors.New("too short bytes")
	}
	return d.data[index], index + rb, nil
}

func (d *decoder) readSize2(index int) ([]byte, int, error) {
	return d.readSizeN(index, def.Byte2)
}

func (d *decoder) readSize4(index int) ([]byte, int, error) {
	return d.readSizeN(index, def.Byte4)
}

func (d *decoder) readSize8(index int) ([]byte, int, error) {
	return d.readSizeN(index, def.Byte8)
}

func (d *decoder) readSizeN(index, n int) ([]byte, int, error) {
	if len(d.data) < index+n {
		return emptyBytes, 0, errors.New("too short bytes")
	}
	return d.data[index : index+n], index + n, nil
}
