package time

import (
	"encoding/binary"
	"fmt"
	"github.com/shamaton/msgpack/v2/def"
	"github.com/shamaton/msgpack/v2/ext"
	"reflect"
	"time"
)

var StreamDecoder = new(timeStreamDecoder)

type timeStreamDecoder struct {
	ext.DecoderStreamCommon
}

var _ ext.StreamDecoder = (*timeStreamDecoder)(nil)

func (td *timeStreamDecoder) Code() int8 {
	return def.TimeStamp
}

func (td *timeStreamDecoder) IsType(code byte, innerType int8, dataLength int) bool {
	switch code {
	case def.Fixext4, def.Fixext8:
		return innerType == td.Code()
	case def.Ext8:
		return innerType == td.Code() && dataLength == 12
	}
	return false
}

func (td *timeStreamDecoder) ToValue(code byte, data []byte, k reflect.Kind) (interface{}, error) {
	switch code {
	case def.Fixext4:
		return time.Unix(int64(binary.BigEndian.Uint32(data)), 0), nil

	case def.Fixext8:
		data64 := binary.BigEndian.Uint64(data)
		nano := int64(data64 >> 34)
		if nano > 999999999 {
			return zero, fmt.Errorf("in timestamp 64 formats, nanoseconds must not be larger than 999999999 : %d", nano)
		}
		return time.Unix(int64(data64&0x00000003ffffffff), nano), nil

	case def.Ext8:
		nano := binary.BigEndian.Uint32(data[:4])
		if nano > 999999999 {
			return zero, fmt.Errorf("in timestamp 96 formats, nanoseconds must not be larger than 999999999 : %d", nano)
		}
		sec := binary.BigEndian.Uint64(data[4:12])
		return time.Unix(int64(sec), int64(nano)), nil
	}

	return zero, fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}
