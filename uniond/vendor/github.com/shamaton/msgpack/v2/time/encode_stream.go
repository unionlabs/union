package time

import (
	"github.com/shamaton/msgpack/v2/internal/common"
	"io"
	"reflect"
	"time"

	"github.com/shamaton/msgpack/v2/def"
	"github.com/shamaton/msgpack/v2/ext"
)

var StreamEncoder = new(timeStreamEncoder)

type timeStreamEncoder struct {
	ext.StreamEncoderCommon
}

var _ ext.StreamEncoder = (*timeStreamEncoder)(nil)

func (timeStreamEncoder) Code() int8 {
	return def.TimeStamp
}

func (timeStreamEncoder) Type() reflect.Type {
	return typeOf
}

func (e timeStreamEncoder) Write(w io.Writer, value reflect.Value, buf *common.Buffer) error {
	t := value.Interface().(time.Time)

	secs := uint64(t.Unix())
	if secs>>34 == 0 {
		data := uint64(t.Nanosecond())<<34 | secs
		if data&0xffffffff00000000 == 0 {
			if err := e.WriteByte1Int(w, def.Fixext4, buf); err != nil {
				return err
			}
			if err := e.WriteByte1Int(w, def.TimeStamp, buf); err != nil {
				return err
			}
			if err := e.WriteByte4Uint64(w, data, buf); err != nil {
				return err
			}
			return nil
		}

		if err := e.WriteByte1Int(w, def.Fixext8, buf); err != nil {
			return err
		}
		if err := e.WriteByte1Int(w, def.TimeStamp, buf); err != nil {
			return err
		}
		if err := e.WriteByte8Uint64(w, data, buf); err != nil {
			return err
		}
		return nil
	}

	if err := e.WriteByte1Int(w, def.Ext8, buf); err != nil {
		return err
	}
	if err := e.WriteByte1Int(w, 12, buf); err != nil {
		return err
	}
	if err := e.WriteByte1Int(w, def.TimeStamp, buf); err != nil {
		return err
	}
	if err := e.WriteByte4Int(w, t.Nanosecond(), buf); err != nil {
		return err
	}
	if err := e.WriteByte8Uint64(w, secs, buf); err != nil {
		return err
	}
	return nil
}
