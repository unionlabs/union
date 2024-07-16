package decoding

import (
	"github.com/shamaton/msgpack/v2/ext"
	"github.com/shamaton/msgpack/v2/time"
)

var extCoderMap = map[int8]ext.Decoder{time.Decoder.Code(): time.Decoder}
var extCoders = []ext.Decoder{time.Decoder}

// AddExtDecoder adds decoders for extension types.
func AddExtDecoder(f ext.Decoder) {
	// ignore time
	if f.Code() == time.Decoder.Code() {
		return
	}

	_, ok := extCoderMap[f.Code()]
	if !ok {
		extCoderMap[f.Code()] = f
		updateExtCoders()
	}
}

// RemoveExtDecoder removes decoders for extension types.
func RemoveExtDecoder(f ext.Decoder) {
	// ignore time
	if f.Code() == time.Decoder.Code() {
		return
	}

	_, ok := extCoderMap[f.Code()]
	if ok {
		delete(extCoderMap, f.Code())
		updateExtCoders()
	}
}

func updateExtCoders() {
	extCoders = make([]ext.Decoder, len(extCoderMap))
	i := 0
	for k := range extCoderMap {
		extCoders[i] = extCoderMap[k]
		i++
	}
}

/*
var zero = time.Unix(0,0)

func (d *decoder) isDateTime(offset int) bool {
	code, offset := d.readSize1(offset)

	if code == def.Fixext4 {
		t, _ := d.readSize1(offset)
		return int8(t) == def.TimeStamp
	} else if code == def.Fixext8 {
		t, _ := d.readSize1(offset)
		return int8(t) == def.TimeStamp
	} else if code == def.Ext8 {
		l, offset := d.readSize1(offset)
		t, _ := d.readSize1(offset)
		return l == 12 && int8(t) == def.TimeStamp
	}
	return false
}

func (d *decoder) asDateTime(offset int, k reflect.Kind) (time.Time, int, error) {
	code, offset := d.readSize1(offset)

	switch code {
	case def.Fixext4:
		_, offset = d.readSize1(offset)
		bs, offset := d.readSize4(offset)
		return time.Unix(int64(binary.BigEndian.Uint32(bs)), 0), offset, nil

	case def.Fixext8:
		_, offset = d.readSize1(offset)
		bs, offset := d.readSize8(offset)
		data64 := binary.BigEndian.Uint64(bs)
		nano := int64(data64 >> 34)
		if nano > 999999999 {
			return zero, 0, fmt.Errorf("In timestamp 64 formats, nanoseconds must not be larger than 999999999 : %d", nano)
		}
		return time.Unix(int64(data64&0x00000003ffffffff), nano), offset, nil

	case def.Ext8:
		_, offset = d.readSize1(offset)
		_, offset = d.readSize1(offset)
		nanobs, offset := d.readSize4(offset)
		secbs, offset := d.readSize8(offset)
		nano := binary.BigEndian.Uint32(nanobs)
		if nano > 999999999 {
			return zero, 0, fmt.Errorf("In timestamp 96 formats, nanoseconds must not be larger than 999999999 : %d", nano)
		}
		sec := binary.BigEndian.Uint64(secbs)
		return time.Unix(int64(sec), int64(nano)), offset, nil
	}

	return zero, 0, d.errorTemplate(code, k)
}
*/
