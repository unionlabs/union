package ext

import (
	"reflect"
)

type Encoder interface {
	Code() int8
	Type() reflect.Type
	CalcByteSize(value reflect.Value) (int, error)
	WriteToBytes(value reflect.Value, offset int, bytes *[]byte) int
}

type EncoderCommon struct {
}

func (c *EncoderCommon) SetByte1Int64(value int64, offset int, d *[]byte) int {
	(*d)[offset] = byte(value)
	return offset + 1
}

func (c *EncoderCommon) SetByte2Int64(value int64, offset int, d *[]byte) int {
	(*d)[offset+0] = byte(value >> 8)
	(*d)[offset+1] = byte(value)
	return offset + 2
}

func (c *EncoderCommon) SetByte4Int64(value int64, offset int, d *[]byte) int {
	(*d)[offset+0] = byte(value >> 24)
	(*d)[offset+1] = byte(value >> 16)
	(*d)[offset+2] = byte(value >> 8)
	(*d)[offset+3] = byte(value)
	return offset + 4
}

func (c *EncoderCommon) SetByte8Int64(value int64, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 56)
	(*d)[offset+1] = byte(value >> 48)
	(*d)[offset+2] = byte(value >> 40)
	(*d)[offset+3] = byte(value >> 32)
	(*d)[offset+4] = byte(value >> 24)
	(*d)[offset+5] = byte(value >> 16)
	(*d)[offset+6] = byte(value >> 8)
	(*d)[offset+7] = byte(value)
	return offset + 8
}

func (c *EncoderCommon) SetByte1Uint64(value uint64, offset int, d *[]byte) int {
	(*d)[offset] = byte(value)
	return offset + 1
}

func (c *EncoderCommon) SetByte2Uint64(value uint64, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 8)
	(*d)[offset+1] = byte(value)
	return offset + 2
}

func (c *EncoderCommon) SetByte4Uint64(value uint64, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 24)
	(*d)[offset+1] = byte(value >> 16)
	(*d)[offset+2] = byte(value >> 8)
	(*d)[offset+3] = byte(value)
	return offset + 4
}

func (c *EncoderCommon) SetByte8Uint64(value uint64, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 56)
	(*d)[offset+1] = byte(value >> 48)
	(*d)[offset+2] = byte(value >> 40)
	(*d)[offset+3] = byte(value >> 32)
	(*d)[offset+4] = byte(value >> 24)
	(*d)[offset+5] = byte(value >> 16)
	(*d)[offset+6] = byte(value >> 8)
	(*d)[offset+7] = byte(value)
	return offset + 8
}

func (c *EncoderCommon) SetByte1Int(code, offset int, d *[]byte) int {
	(*d)[offset] = byte(code)
	return offset + 1
}

func (c *EncoderCommon) SetByte2Int(value int, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 8)
	(*d)[offset+1] = byte(value)
	return offset + 2
}

func (c *EncoderCommon) SetByte4Int(value int, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 24)
	(*d)[offset+1] = byte(value >> 16)
	(*d)[offset+2] = byte(value >> 8)
	(*d)[offset+3] = byte(value)
	return offset + 4
}

func (c *EncoderCommon) SetByte4Uint32(value uint32, offset int, d *[]byte) int {
	(*d)[offset] = byte(value >> 24)
	(*d)[offset+1] = byte(value >> 16)
	(*d)[offset+2] = byte(value >> 8)
	(*d)[offset+3] = byte(value)
	return offset + 4
}

func (c *EncoderCommon) SetBytes(bs []byte, offset int, d *[]byte) int {
	for i := range bs {
		(*d)[offset+i] = bs[i]
	}
	return offset + len(bs)
}
