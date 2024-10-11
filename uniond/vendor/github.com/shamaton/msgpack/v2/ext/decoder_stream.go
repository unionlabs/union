package ext

import (
	"github.com/shamaton/msgpack/v2/internal/common"
	"io"
	"reflect"
)

var emptyBytes []byte

type StreamDecoder interface {
	Code() int8
	IsType(code byte, innerType int8, dataLength int) bool
	ToValue(code byte, data []byte, k reflect.Kind) (any, error)
}

type DecoderStreamCommon struct {
}

func (d *DecoderStreamCommon) ReadSize1(r io.Reader, buf *common.Buffer) (byte, error) {
	if _, err := r.Read(buf.B1); err != nil {
		return 0, err
	}
	return buf.B1[0], nil
}

func (d *DecoderStreamCommon) ReadSize2(r io.Reader, buf *common.Buffer) ([]byte, error) {
	if _, err := r.Read(buf.B2); err != nil {
		return emptyBytes, err
	}
	return buf.B2, nil
}

func (d *DecoderStreamCommon) ReadSize4(r io.Reader, buf *common.Buffer) ([]byte, error) {
	if _, err := r.Read(buf.B4); err != nil {
		return emptyBytes, err
	}
	return buf.B4, nil
}

func (d *DecoderStreamCommon) ReadSize8(r io.Reader, buf *common.Buffer) ([]byte, error) {
	if _, err := r.Read(buf.B8); err != nil {
		return emptyBytes, err
	}
	return buf.B8, nil
}

func (d *DecoderStreamCommon) ReadSizeN(r io.Reader, buf *common.Buffer, n int) ([]byte, error) {
	var b []byte
	if len(buf.Data) <= n {
		b = buf.Data[:n]
	} else {
		buf.Data = append(buf.Data, make([]byte, n-len(buf.Data))...)
		b = buf.Data
	}
	if _, err := r.Read(b); err != nil {
		return emptyBytes, err
	}
	return b, nil
}
