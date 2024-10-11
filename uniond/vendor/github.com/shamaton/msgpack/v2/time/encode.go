package time

import (
	"reflect"
	"time"

	"github.com/shamaton/msgpack/v2/def"
	"github.com/shamaton/msgpack/v2/ext"
)

var Encoder = new(timeEncoder)

type timeEncoder struct {
	ext.EncoderCommon
}

var typeOf = reflect.TypeOf(time.Time{})

func (td *timeEncoder) Code() int8 {
	return def.TimeStamp
}

func (s *timeEncoder) Type() reflect.Type {
	return typeOf
}

func (s *timeEncoder) CalcByteSize(value reflect.Value) (int, error) {
	t := value.Interface().(time.Time)
	secs := uint64(t.Unix())
	if secs>>34 == 0 {
		data := uint64(t.Nanosecond())<<34 | secs
		if data&0xffffffff00000000 == 0 {
			return def.Byte1 + def.Byte4, nil
		}
		return def.Byte1 + def.Byte8, nil
	}

	return def.Byte1 + def.Byte1 + def.Byte4 + def.Byte8, nil
}

func (s *timeEncoder) WriteToBytes(value reflect.Value, offset int, bytes *[]byte) int {
	t := value.Interface().(time.Time)

	secs := uint64(t.Unix())
	if secs>>34 == 0 {
		data := uint64(t.Nanosecond())<<34 | secs
		if data&0xffffffff00000000 == 0 {
			offset = s.SetByte1Int(def.Fixext4, offset, bytes)
			offset = s.SetByte1Int(def.TimeStamp, offset, bytes)
			offset = s.SetByte4Uint64(data, offset, bytes)
			return offset
		}

		offset = s.SetByte1Int(def.Fixext8, offset, bytes)
		offset = s.SetByte1Int(def.TimeStamp, offset, bytes)
		offset = s.SetByte8Uint64(data, offset, bytes)
		return offset
	}

	offset = s.SetByte1Int(def.Ext8, offset, bytes)
	offset = s.SetByte1Int(12, offset, bytes)
	offset = s.SetByte1Int(def.TimeStamp, offset, bytes)
	offset = s.SetByte4Int(t.Nanosecond(), offset, bytes)
	offset = s.SetByte8Uint64(secs, offset, bytes)
	return offset
}
