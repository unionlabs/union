package json

import (
	"fmt"
	"math"
	"net"
	"reflect"
	"strconv"
)

// AppendNil inserts a 'Nil' object into the dst byte array.
func (Encoder) AppendNil(dst []byte) []byte {
	return append(dst, "null"...)
}

// AppendBeginMarker inserts a map start into the dst byte array.
func (Encoder) AppendBeginMarker(dst []byte) []byte {
	return append(dst, '{')
}

// AppendEndMarker inserts a map end into the dst byte array.
func (Encoder) AppendEndMarker(dst []byte) []byte {
	return append(dst, '}')
}

// AppendLineBreak appends a line break.
func (Encoder) AppendLineBreak(dst []byte) []byte {
	return append(dst, '\n')
}

// AppendArrayStart adds markers to indicate the start of an array.
func (Encoder) AppendArrayStart(dst []byte) []byte {
	return append(dst, '[')
}

// AppendArrayEnd adds markers to indicate the end of an array.
func (Encoder) AppendArrayEnd(dst []byte) []byte {
	return append(dst, ']')
}

// AppendArrayDelim adds markers to indicate end of a particular array element.
func (Encoder) AppendArrayDelim(dst []byte) []byte {
	if len(dst) > 0 {
		return append(dst, ',')
	}
	return dst
}

// AppendBool converts the input bool to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendBool(dst []byte, val bool) []byte {
	return strconv.AppendBool(dst, val)
}

// AppendBools encodes the input bools to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendBools(dst []byte, vals []bool) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendBool(dst, vals[0])
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendBool(append(dst, ','), val)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendInt converts the input int to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendInt(dst []byte, val int) []byte {
	return strconv.AppendInt(dst, int64(val), 10)
}

// AppendInts encodes the input ints to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendInts(dst []byte, vals []int) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendInt(dst, int64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendInt(append(dst, ','), int64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendInt8 converts the input []int8 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendInt8(dst []byte, val int8) []byte {
	return strconv.AppendInt(dst, int64(val), 10)
}

// AppendInts8 encodes the input int8s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendInts8(dst []byte, vals []int8) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendInt(dst, int64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendInt(append(dst, ','), int64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendInt16 converts the input int16 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendInt16(dst []byte, val int16) []byte {
	return strconv.AppendInt(dst, int64(val), 10)
}

// AppendInts16 encodes the input int16s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendInts16(dst []byte, vals []int16) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendInt(dst, int64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendInt(append(dst, ','), int64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendInt32 converts the input int32 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendInt32(dst []byte, val int32) []byte {
	return strconv.AppendInt(dst, int64(val), 10)
}

// AppendInts32 encodes the input int32s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendInts32(dst []byte, vals []int32) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendInt(dst, int64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendInt(append(dst, ','), int64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendInt64 converts the input int64 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendInt64(dst []byte, val int64) []byte {
	return strconv.AppendInt(dst, val, 10)
}

// AppendInts64 encodes the input int64s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendInts64(dst []byte, vals []int64) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendInt(dst, vals[0], 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendInt(append(dst, ','), val, 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendUint converts the input uint to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendUint(dst []byte, val uint) []byte {
	return strconv.AppendUint(dst, uint64(val), 10)
}

// AppendUints encodes the input uints to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendUints(dst []byte, vals []uint) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendUint(dst, uint64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendUint(append(dst, ','), uint64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendUint8 converts the input uint8 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendUint8(dst []byte, val uint8) []byte {
	return strconv.AppendUint(dst, uint64(val), 10)
}

// AppendUints8 encodes the input uint8s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendUints8(dst []byte, vals []uint8) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendUint(dst, uint64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendUint(append(dst, ','), uint64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendUint16 converts the input uint16 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendUint16(dst []byte, val uint16) []byte {
	return strconv.AppendUint(dst, uint64(val), 10)
}

// AppendUints16 encodes the input uint16s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendUints16(dst []byte, vals []uint16) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendUint(dst, uint64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendUint(append(dst, ','), uint64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendUint32 converts the input uint32 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendUint32(dst []byte, val uint32) []byte {
	return strconv.AppendUint(dst, uint64(val), 10)
}

// AppendUints32 encodes the input uint32s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendUints32(dst []byte, vals []uint32) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendUint(dst, uint64(vals[0]), 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendUint(append(dst, ','), uint64(val), 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendUint64 converts the input uint64 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendUint64(dst []byte, val uint64) []byte {
	return strconv.AppendUint(dst, val, 10)
}

// AppendUints64 encodes the input uint64s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendUints64(dst []byte, vals []uint64) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = strconv.AppendUint(dst, vals[0], 10)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = strconv.AppendUint(append(dst, ','), val, 10)
		}
	}
	dst = append(dst, ']')
	return dst
}

func appendFloat(dst []byte, val float64, bitSize, precision int) []byte {
	// JSON does not permit NaN or Infinity. A typical JSON encoder would fail
	// with an error, but a logging library wants the data to get through so we
	// make a tradeoff and store those types as string.
	switch {
	case math.IsNaN(val):
		return append(dst, `"NaN"`...)
	case math.IsInf(val, 1):
		return append(dst, `"+Inf"`...)
	case math.IsInf(val, -1):
		return append(dst, `"-Inf"`...)
	}
	// convert as if by es6 number to string conversion
	// see also https://cs.opensource.google/go/go/+/refs/tags/go1.20.3:src/encoding/json/encode.go;l=573
	strFmt := byte('f')
	// If precision is set to a value other than -1, we always just format the float using that precision.
	if precision == -1 {
		// Use float32 comparisons for underlying float32 value to get precise cutoffs right.
		if abs := math.Abs(val); abs != 0 {
			if bitSize == 64 && (abs < 1e-6 || abs >= 1e21) || bitSize == 32 && (float32(abs) < 1e-6 || float32(abs) >= 1e21) {
				strFmt = 'e'
			}
		}
	}
	dst = strconv.AppendFloat(dst, val, strFmt, precision, bitSize)
	if strFmt == 'e' {
		// Clean up e-09 to e-9
		n := len(dst)
		if n >= 4 && dst[n-4] == 'e' && dst[n-3] == '-' && dst[n-2] == '0' {
			dst[n-2] = dst[n-1]
			dst = dst[:n-1]
		}
	}
	return dst
}

// AppendFloat32 converts the input float32 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendFloat32(dst []byte, val float32, precision int) []byte {
	return appendFloat(dst, float64(val), 32, precision)
}

// AppendFloats32 encodes the input float32s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendFloats32(dst []byte, vals []float32, precision int) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = appendFloat(dst, float64(vals[0]), 32, precision)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = appendFloat(append(dst, ','), float64(val), 32, precision)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendFloat64 converts the input float64 to a string and
// appends the encoded string to the input byte slice.
func (Encoder) AppendFloat64(dst []byte, val float64, precision int) []byte {
	return appendFloat(dst, val, 64, precision)
}

// AppendFloats64 encodes the input float64s to json and
// appends the encoded string list to the input byte slice.
func (Encoder) AppendFloats64(dst []byte, vals []float64, precision int) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = appendFloat(dst, vals[0], 64, precision)
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = appendFloat(append(dst, ','), val, 64, precision)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendInterface marshals the input interface to a string and
// appends the encoded string to the input byte slice.
func (e Encoder) AppendInterface(dst []byte, i interface{}) []byte {
	marshaled, err := JSONMarshalFunc(i)
	if err != nil {
		return e.AppendString(dst, fmt.Sprintf("marshaling error: %v", err))
	}
	return append(dst, marshaled...)
}

// AppendType appends the parameter type (as a string) to the input byte slice.
func (e Encoder) AppendType(dst []byte, i interface{}) []byte {
	if i == nil {
		return e.AppendString(dst, "<nil>")
	}
	return e.AppendString(dst, reflect.TypeOf(i).String())
}

// AppendObjectData takes in an object that is already in a byte array
// and adds it to the dst.
func (Encoder) AppendObjectData(dst []byte, o []byte) []byte {
	// Three conditions apply here:
	// 1. new content starts with '{' - which should be dropped   OR
	// 2. new content starts with '{' - which should be replaced with ','
	//    to separate with existing content OR
	// 3. existing content has already other fields
	if o[0] == '{' {
		if len(dst) > 1 {
			dst = append(dst, ',')
		}
		o = o[1:]
	} else if len(dst) > 1 {
		dst = append(dst, ',')
	}
	return append(dst, o...)
}

// AppendIPAddr adds IPv4 or IPv6 address to dst.
func (e Encoder) AppendIPAddr(dst []byte, ip net.IP) []byte {
	return e.AppendString(dst, ip.String())
}

// AppendIPPrefix adds IPv4 or IPv6 Prefix (address & mask) to dst.
func (e Encoder) AppendIPPrefix(dst []byte, pfx net.IPNet) []byte {
	return e.AppendString(dst, pfx.String())

}

// AppendMACAddr adds MAC address to dst.
func (e Encoder) AppendMACAddr(dst []byte, ha net.HardwareAddr) []byte {
	return e.AppendString(dst, ha.String())
}
