package time

import (
	"encoding/binary"
	"fmt"
	"reflect"
	"time"

	"github.com/shamaton/msgpack/v2/def"
	"github.com/shamaton/msgpack/v2/ext"
)

var zero = time.Unix(0, 0)

var Decoder = new(timeDecoder)

type timeDecoder struct {
	ext.DecoderCommon
}

var _ ext.Decoder = (*timeDecoder)(nil)

func (td *timeDecoder) Code() int8 {
	return def.TimeStamp
}

func (td *timeDecoder) IsType(offset int, d *[]byte) bool {
	code, offset := td.ReadSize1(offset, d)

	if code == def.Fixext4 {
		t, _ := td.ReadSize1(offset, d)
		return int8(t) == td.Code()
	} else if code == def.Fixext8 {
		t, _ := td.ReadSize1(offset, d)
		return int8(t) == td.Code()
	} else if code == def.Ext8 {
		l, offset := td.ReadSize1(offset, d)
		t, _ := td.ReadSize1(offset, d)
		return l == 12 && int8(t) == td.Code()
	}
	return false
}

func (td *timeDecoder) AsValue(offset int, k reflect.Kind, d *[]byte) (interface{}, int, error) {
	code, offset := td.ReadSize1(offset, d)

	switch code {
	case def.Fixext4:
		_, offset = td.ReadSize1(offset, d)
		bs, offset := td.ReadSize4(offset, d)
		return time.Unix(int64(binary.BigEndian.Uint32(bs)), 0), offset, nil

	case def.Fixext8:
		_, offset = td.ReadSize1(offset, d)
		bs, offset := td.ReadSize8(offset, d)
		data64 := binary.BigEndian.Uint64(bs)
		nano := int64(data64 >> 34)
		if nano > 999999999 {
			return zero, 0, fmt.Errorf("In timestamp 64 formats, nanoseconds must not be larger than 999999999 : %d", nano)
		}
		return time.Unix(int64(data64&0x00000003ffffffff), nano), offset, nil

	case def.Ext8:
		_, offset = td.ReadSize1(offset, d)
		_, offset = td.ReadSize1(offset, d)
		nanobs, offset := td.ReadSize4(offset, d)
		secbs, offset := td.ReadSize8(offset, d)
		nano := binary.BigEndian.Uint32(nanobs)
		if nano > 999999999 {
			return zero, 0, fmt.Errorf("In timestamp 96 formats, nanoseconds must not be larger than 999999999 : %d", nano)
		}
		sec := binary.BigEndian.Uint64(secbs)
		return time.Unix(int64(sec), int64(nano)), offset, nil
	}

	return zero, 0, fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}
