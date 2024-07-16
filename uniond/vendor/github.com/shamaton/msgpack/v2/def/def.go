package def

// IntSize : 32 or 64
const IntSize = 32 << (^uint(0) >> 63)

var IsIntSize32 = IntSize == 32

// message pack format
const (
	PositiveFixIntMin = 0x00
	PositiveFixIntMax = 0x7f

	FixMap   = 0x80
	FixArray = 0x90
	FixStr   = 0xa0

	Nil = 0xc0

	False = 0xc2
	True  = 0xc3

	Bin8  = 0xc4
	Bin16 = 0xc5
	Bin32 = 0xc6

	Ext8  = 0xc7
	Ext16 = 0xc8
	Ext32 = 0xc9

	Float32 = 0xca
	Float64 = 0xcb

	Uint8  = 0xcc
	Uint16 = 0xcd
	Uint32 = 0xce
	Uint64 = 0xcf

	Int8  = 0xd0
	Int16 = 0xd1
	Int32 = 0xd2
	Int64 = 0xd3

	Fixext1  = 0xd4
	Fixext2  = 0xd5
	Fixext4  = 0xd6
	Fixext8  = 0xd7
	Fixext16 = 0xd8

	Str8  = 0xd9
	Str16 = 0xda
	Str32 = 0xdb

	Array16 = 0xdc
	Array32 = 0xdd

	Map16 = 0xde
	Map32 = 0xdf

	NegativeFixintMin = -32 // 0xe0
	NegativeFixintMax = -1  // 0xff
)

// byte
const (
	Byte1 = 1 << iota
	Byte2
	Byte4
	Byte8
	Byte16
	Byte32
)

// ext type
const (
	TimeStamp = -1
)

// ext type complex
var complexTypeCode = int8(-128)

// ComplexTypeCode gets complexTypeCode
func ComplexTypeCode() int8 { return complexTypeCode }

// SetComplexTypeCode sets complexTypeCode
func SetComplexTypeCode(code int8) {
	complexTypeCode = code
}
