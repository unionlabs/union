package cbor

import (
	"fmt"
	"math"
	"net"
	"reflect"
)

// AppendNil inserts a 'Nil' object into the dst byte array.
func (Encoder) AppendNil(dst []byte) []byte {
	return append(dst, majorTypeSimpleAndFloat|additionalTypeNull)
}

// AppendBeginMarker inserts a map start into the dst byte array.
func (Encoder) AppendBeginMarker(dst []byte) []byte {
	return append(dst, majorTypeMap|additionalTypeInfiniteCount)
}

// AppendEndMarker inserts a map end into the dst byte array.
func (Encoder) AppendEndMarker(dst []byte) []byte {
	return append(dst, majorTypeSimpleAndFloat|additionalTypeBreak)
}

// AppendObjectData takes an object in form of a byte array and appends to dst.
func (Encoder) AppendObjectData(dst []byte, o []byte) []byte {
	// BeginMarker is present in the dst, which
	// should not be copied when appending to existing data.
	return append(dst, o[1:]...)
}

// AppendArrayStart adds markers to indicate the start of an array.
func (Encoder) AppendArrayStart(dst []byte) []byte {
	return append(dst, majorTypeArray|additionalTypeInfiniteCount)
}

// AppendArrayEnd adds markers to indicate the end of an array.
func (Encoder) AppendArrayEnd(dst []byte) []byte {
	return append(dst, majorTypeSimpleAndFloat|additionalTypeBreak)
}

// AppendArrayDelim adds markers to indicate end of a particular array element.
func (Encoder) AppendArrayDelim(dst []byte) []byte {
	//No delimiters needed in cbor
	return dst
}

// AppendLineBreak is a noop that keep API compat with json encoder.
func (Encoder) AppendLineBreak(dst []byte) []byte {
	// No line breaks needed in binary format.
	return dst
}

// AppendBool encodes and inserts a boolean value into the dst byte array.
func (Encoder) AppendBool(dst []byte, val bool) []byte {
	b := additionalTypeBoolFalse
	if val {
		b = additionalTypeBoolTrue
	}
	return append(dst, majorTypeSimpleAndFloat|b)
}

// AppendBools encodes and inserts an array of boolean values into the dst byte array.
func (e Encoder) AppendBools(dst []byte, vals []bool) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendBool(dst, v)
	}
	return dst
}

// AppendInt encodes and inserts an integer value into the dst byte array.
func (Encoder) AppendInt(dst []byte, val int) []byte {
	major := majorTypeUnsignedInt
	contentVal := val
	if val < 0 {
		major = majorTypeNegativeInt
		contentVal = -val - 1
	}
	if contentVal <= additionalMax {
		lb := byte(contentVal)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(contentVal))
	}
	return dst
}

// AppendInts encodes and inserts an array of integer values into the dst byte array.
func (e Encoder) AppendInts(dst []byte, vals []int) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendInt(dst, v)
	}
	return dst
}

// AppendInt8 encodes and inserts an int8 value into the dst byte array.
func (e Encoder) AppendInt8(dst []byte, val int8) []byte {
	return e.AppendInt(dst, int(val))
}

// AppendInts8 encodes and inserts an array of integer values into the dst byte array.
func (e Encoder) AppendInts8(dst []byte, vals []int8) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendInt(dst, int(v))
	}
	return dst
}

// AppendInt16 encodes and inserts a int16 value into the dst byte array.
func (e Encoder) AppendInt16(dst []byte, val int16) []byte {
	return e.AppendInt(dst, int(val))
}

// AppendInts16 encodes and inserts an array of int16 values into the dst byte array.
func (e Encoder) AppendInts16(dst []byte, vals []int16) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendInt(dst, int(v))
	}
	return dst
}

// AppendInt32 encodes and inserts a int32 value into the dst byte array.
func (e Encoder) AppendInt32(dst []byte, val int32) []byte {
	return e.AppendInt(dst, int(val))
}

// AppendInts32 encodes and inserts an array of int32 values into the dst byte array.
func (e Encoder) AppendInts32(dst []byte, vals []int32) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendInt(dst, int(v))
	}
	return dst
}

// AppendInt64 encodes and inserts a int64 value into the dst byte array.
func (Encoder) AppendInt64(dst []byte, val int64) []byte {
	major := majorTypeUnsignedInt
	contentVal := val
	if val < 0 {
		major = majorTypeNegativeInt
		contentVal = -val - 1
	}
	if contentVal <= additionalMax {
		lb := byte(contentVal)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(contentVal))
	}
	return dst
}

// AppendInts64 encodes and inserts an array of int64 values into the dst byte array.
func (e Encoder) AppendInts64(dst []byte, vals []int64) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendInt64(dst, v)
	}
	return dst
}

// AppendUint encodes and inserts an unsigned integer value into the dst byte array.
func (e Encoder) AppendUint(dst []byte, val uint) []byte {
	return e.AppendInt64(dst, int64(val))
}

// AppendUints encodes and inserts an array of unsigned integer values into the dst byte array.
func (e Encoder) AppendUints(dst []byte, vals []uint) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendUint(dst, v)
	}
	return dst
}

// AppendUint8 encodes and inserts a unsigned int8 value into the dst byte array.
func (e Encoder) AppendUint8(dst []byte, val uint8) []byte {
	return e.AppendUint(dst, uint(val))
}

// AppendUints8 encodes and inserts an array of uint8 values into the dst byte array.
func (e Encoder) AppendUints8(dst []byte, vals []uint8) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendUint8(dst, v)
	}
	return dst
}

// AppendUint16 encodes and inserts a uint16 value into the dst byte array.
func (e Encoder) AppendUint16(dst []byte, val uint16) []byte {
	return e.AppendUint(dst, uint(val))
}

// AppendUints16 encodes and inserts an array of uint16 values into the dst byte array.
func (e Encoder) AppendUints16(dst []byte, vals []uint16) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendUint16(dst, v)
	}
	return dst
}

// AppendUint32 encodes and inserts a uint32 value into the dst byte array.
func (e Encoder) AppendUint32(dst []byte, val uint32) []byte {
	return e.AppendUint(dst, uint(val))
}

// AppendUints32 encodes and inserts an array of uint32 values into the dst byte array.
func (e Encoder) AppendUints32(dst []byte, vals []uint32) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendUint32(dst, v)
	}
	return dst
}

// AppendUint64 encodes and inserts a uint64 value into the dst byte array.
func (Encoder) AppendUint64(dst []byte, val uint64) []byte {
	major := majorTypeUnsignedInt
	contentVal := val
	if contentVal <= additionalMax {
		lb := byte(contentVal)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, contentVal)
	}
	return dst
}

// AppendUints64 encodes and inserts an array of uint64 values into the dst byte array.
func (e Encoder) AppendUints64(dst []byte, vals []uint64) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendUint64(dst, v)
	}
	return dst
}

// AppendFloat32 encodes and inserts a single precision float value into the dst byte array.
func (Encoder) AppendFloat32(dst []byte, val float32) []byte {
	switch {
	case math.IsNaN(float64(val)):
		return append(dst, "\xfa\x7f\xc0\x00\x00"...)
	case math.IsInf(float64(val), 1):
		return append(dst, "\xfa\x7f\x80\x00\x00"...)
	case math.IsInf(float64(val), -1):
		return append(dst, "\xfa\xff\x80\x00\x00"...)
	}
	major := majorTypeSimpleAndFloat
	subType := additionalTypeFloat32
	n := math.Float32bits(val)
	var buf [4]byte
	for i := uint(0); i < 4; i++ {
		buf[i] = byte(n >> ((3 - i) * 8))
	}
	return append(append(dst, major|subType), buf[0], buf[1], buf[2], buf[3])
}

// AppendFloats32 encodes and inserts an array of single precision float value into the dst byte array.
func (e Encoder) AppendFloats32(dst []byte, vals []float32) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendFloat32(dst, v)
	}
	return dst
}

// AppendFloat64 encodes and inserts a double precision float value into the dst byte array.
func (Encoder) AppendFloat64(dst []byte, val float64) []byte {
	switch {
	case math.IsNaN(val):
		return append(dst, "\xfb\x7f\xf8\x00\x00\x00\x00\x00\x00"...)
	case math.IsInf(val, 1):
		return append(dst, "\xfb\x7f\xf0\x00\x00\x00\x00\x00\x00"...)
	case math.IsInf(val, -1):
		return append(dst, "\xfb\xff\xf0\x00\x00\x00\x00\x00\x00"...)
	}
	major := majorTypeSimpleAndFloat
	subType := additionalTypeFloat64
	n := math.Float64bits(val)
	dst = append(dst, major|subType)
	for i := uint(1); i <= 8; i++ {
		b := byte(n >> ((8 - i) * 8))
		dst = append(dst, b)
	}
	return dst
}

// AppendFloats64 encodes and inserts an array of double precision float values into the dst byte array.
func (e Encoder) AppendFloats64(dst []byte, vals []float64) []byte {
	major := majorTypeArray
	l := len(vals)
	if l == 0 {
		return e.AppendArrayEnd(e.AppendArrayStart(dst))
	}
	if l <= additionalMax {
		lb := byte(l)
		dst = append(dst, major|lb)
	} else {
		dst = appendCborTypePrefix(dst, major, uint64(l))
	}
	for _, v := range vals {
		dst = e.AppendFloat64(dst, v)
	}
	return dst
}

// AppendInterface takes an arbitrary object and converts it to JSON and embeds it dst.
func (e Encoder) AppendInterface(dst []byte, i interface{}) []byte {
	marshaled, err := JSONMarshalFunc(i)
	if err != nil {
		return e.AppendString(dst, fmt.Sprintf("marshaling error: %v", err))
	}
	return AppendEmbeddedJSON(dst, marshaled)
}

// AppendType appends the parameter type (as a string) to the input byte slice.
func (e Encoder) AppendType(dst []byte, i interface{}) []byte {
	if i == nil {
		return e.AppendString(dst, "<nil>")
	}
	return e.AppendString(dst, reflect.TypeOf(i).String())
}

// AppendIPAddr encodes and inserts an IP Address (IPv4 or IPv6).
func (e Encoder) AppendIPAddr(dst []byte, ip net.IP) []byte {
	dst = append(dst, majorTypeTags|additionalTypeIntUint16)
	dst = append(dst, byte(additionalTypeTagNetworkAddr>>8))
	dst = append(dst, byte(additionalTypeTagNetworkAddr&0xff))
	return e.AppendBytes(dst, ip)
}

// AppendIPPrefix encodes and inserts an IP Address Prefix (Address + Mask Length).
func (e Encoder) AppendIPPrefix(dst []byte, pfx net.IPNet) []byte {
	dst = append(dst, majorTypeTags|additionalTypeIntUint16)
	dst = append(dst, byte(additionalTypeTagNetworkPrefix>>8))
	dst = append(dst, byte(additionalTypeTagNetworkPrefix&0xff))

	// Prefix is a tuple (aka MAP of 1 pair of elements) -
	// first element is prefix, second is mask length.
	dst = append(dst, majorTypeMap|0x1)
	dst = e.AppendBytes(dst, pfx.IP)
	maskLen, _ := pfx.Mask.Size()
	return e.AppendUint8(dst, uint8(maskLen))
}

// AppendMACAddr encodes and inserts a Hardware (MAC) address.
func (e Encoder) AppendMACAddr(dst []byte, ha net.HardwareAddr) []byte {
	dst = append(dst, majorTypeTags|additionalTypeIntUint16)
	dst = append(dst, byte(additionalTypeTagNetworkAddr>>8))
	dst = append(dst, byte(additionalTypeTagNetworkAddr&0xff))
	return e.AppendBytes(dst, ha)
}

// AppendHex adds a TAG and inserts a hex bytes as a string.
func (e Encoder) AppendHex(dst []byte, val []byte) []byte {
	dst = append(dst, majorTypeTags|additionalTypeIntUint16)
	dst = append(dst, byte(additionalTypeTagHexString>>8))
	dst = append(dst, byte(additionalTypeTagHexString&0xff))
	return e.AppendBytes(dst, val)
}
