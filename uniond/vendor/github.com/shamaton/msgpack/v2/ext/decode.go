package ext

import (
	"reflect"

	"github.com/shamaton/msgpack/v2/def"
)

type Decoder interface {
	Code() int8
	IsType(offset int, d *[]byte) bool
	AsValue(offset int, k reflect.Kind, d *[]byte) (interface{}, int, error)
}

type DecoderCommon struct {
}

func (cd *DecoderCommon) ReadSize1(index int, d *[]byte) (byte, int) {
	rb := def.Byte1
	return (*d)[index], index + rb
}

func (cd *DecoderCommon) ReadSize2(index int, d *[]byte) ([]byte, int) {
	rb := def.Byte2
	return (*d)[index : index+rb], index + rb
}

func (cd *DecoderCommon) ReadSize4(index int, d *[]byte) ([]byte, int) {
	rb := def.Byte4
	return (*d)[index : index+rb], index + rb
}

func (cd *DecoderCommon) ReadSize8(index int, d *[]byte) ([]byte, int) {
	rb := def.Byte8
	return (*d)[index : index+rb], index + rb
}

func (cd *DecoderCommon) ReadSizeN(index, n int, d *[]byte) ([]byte, int) {
	return (*d)[index : index+n], index + n
}
