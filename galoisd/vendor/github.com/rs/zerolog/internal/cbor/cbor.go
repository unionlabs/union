// Package cbor provides primitives for storing different data
// in the CBOR (binary) format. CBOR is defined in RFC7049.
package cbor

import "time"

const (
	majorOffset   = 5
	additionalMax = 23

	// Non Values.
	additionalTypeBoolFalse byte = 20
	additionalTypeBoolTrue  byte = 21
	additionalTypeNull      byte = 22

	// Integer (+ve and -ve) Sub-types.
	additionalTypeIntUint8  byte = 24
	additionalTypeIntUint16 byte = 25
	additionalTypeIntUint32 byte = 26
	additionalTypeIntUint64 byte = 27

	// Float Sub-types.
	additionalTypeFloat16 byte = 25
	additionalTypeFloat32 byte = 26
	additionalTypeFloat64 byte = 27
	additionalTypeBreak   byte = 31

	// Tag Sub-types.
	additionalTypeTimestamp    byte = 01
	additionalTypeEmbeddedCBOR byte = 63

	// Extended Tags - from https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml
	additionalTypeTagNetworkAddr   uint16 = 260
	additionalTypeTagNetworkPrefix uint16 = 261
	additionalTypeEmbeddedJSON     uint16 = 262
	additionalTypeTagHexString     uint16 = 263

	// Unspecified number of elements.
	additionalTypeInfiniteCount byte = 31
)
const (
	majorTypeUnsignedInt    byte = iota << majorOffset // Major type 0
	majorTypeNegativeInt                               // Major type 1
	majorTypeByteString                                // Major type 2
	majorTypeUtf8String                                // Major type 3
	majorTypeArray                                     // Major type 4
	majorTypeMap                                       // Major type 5
	majorTypeTags                                      // Major type 6
	majorTypeSimpleAndFloat                            // Major type 7
)

const (
	maskOutAdditionalType byte = (7 << majorOffset)
	maskOutMajorType      byte = 31
)

const (
	float32Nan         = "\xfa\x7f\xc0\x00\x00"
	float32PosInfinity = "\xfa\x7f\x80\x00\x00"
	float32NegInfinity = "\xfa\xff\x80\x00\x00"
	float64Nan         = "\xfb\x7f\xf8\x00\x00\x00\x00\x00\x00"
	float64PosInfinity = "\xfb\x7f\xf0\x00\x00\x00\x00\x00\x00"
	float64NegInfinity = "\xfb\xff\xf0\x00\x00\x00\x00\x00\x00"
)

// IntegerTimeFieldFormat indicates the format of timestamp decoded
// from an integer (time in seconds).
var IntegerTimeFieldFormat = time.RFC3339

// NanoTimeFieldFormat indicates the format of timestamp decoded
// from a float value (time in seconds and nanoseconds).
var NanoTimeFieldFormat = time.RFC3339Nano

func appendCborTypePrefix(dst []byte, major byte, number uint64) []byte {
	byteCount := 8
	var minor byte
	switch {
	case number < 256:
		byteCount = 1
		minor = additionalTypeIntUint8

	case number < 65536:
		byteCount = 2
		minor = additionalTypeIntUint16

	case number < 4294967296:
		byteCount = 4
		minor = additionalTypeIntUint32

	default:
		byteCount = 8
		minor = additionalTypeIntUint64

	}

	dst = append(dst, major|minor)
	byteCount--
	for ; byteCount >= 0; byteCount-- {
		dst = append(dst, byte(number>>(uint(byteCount)*8)))
	}
	return dst
}
