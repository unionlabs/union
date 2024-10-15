package decoding

import (
	"encoding/binary"
	"github.com/shamaton/msgpack/v2/def"
	"github.com/shamaton/msgpack/v2/ext"
	"github.com/shamaton/msgpack/v2/time"
)

var extCoderMap = map[int8]ext.StreamDecoder{time.StreamDecoder.Code(): time.StreamDecoder}
var extCoders = []ext.StreamDecoder{time.StreamDecoder}

// AddExtDecoder adds decoders for extension types.
func AddExtDecoder(f ext.StreamDecoder) {
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
func RemoveExtDecoder(f ext.StreamDecoder) {
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
	extCoders = make([]ext.StreamDecoder, len(extCoderMap))
	i := 0
	for k := range extCoderMap {
		extCoders[i] = extCoderMap[k]
		i++
	}
}

func (d *decoder) readIfExtType(code byte) (innerType int8, data []byte, err error) {
	switch code {
	case def.Fixext1:
		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		v, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), []byte{v}, nil

	case def.Fixext2:
		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSize2()
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil

	case def.Fixext4:
		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSize4()
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil

	case def.Fixext8:
		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSize8()
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil

	case def.Fixext16:
		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSize16()
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil

	case def.Ext8:
		bs, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		size := int(bs)

		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSizeN(size)
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil

	case def.Ext16:
		bs, err := d.readSize2()
		if err != nil {
			return 0, nil, err
		}
		size := int(binary.BigEndian.Uint16(bs))

		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSizeN(size)
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil

	case def.Ext32:
		bs, err := d.readSize4()
		if err != nil {
			return 0, nil, err
		}
		size := int(binary.BigEndian.Uint32(bs))

		typ, err := d.readSize1()
		if err != nil {
			return 0, nil, err
		}
		data, err = d.readSizeN(size)
		if err != nil {
			return 0, nil, err
		}
		return int8(typ), data, nil
	}

	return 0, nil, nil
}
