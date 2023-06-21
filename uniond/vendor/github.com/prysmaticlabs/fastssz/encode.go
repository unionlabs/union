package ssz

import (
	"encoding/binary"
	"fmt"
	"math/bits"
)

// MarshalSSZ marshals an object
func MarshalSSZ(m Marshaler) ([]byte, error) {
	buf := make([]byte, m.SizeSSZ())
	return m.MarshalSSZTo(buf[:0])
}

// Errors

var (
	ErrOffset                = fmt.Errorf("incorrect offset")
	ErrSize                  = fmt.Errorf("incorrect size")
	ErrBytesLength           = fmt.Errorf("bytes array does not have the correct length")
	ErrVectorLength          = fmt.Errorf("vector does not have the correct length")
	ErrListTooBig            = fmt.Errorf("list length is higher than max value")
	ErrEmptyBitlist          = fmt.Errorf("bitlist is empty")
	ErrInvalidVariableOffset = fmt.Errorf("invalid ssz encoding. first variable element offset indexes into fixed value data")
)

func ErrBytesLengthFn(name string, found, expected int) error {
	return fmt.Errorf("%s (%v): expected %d and %d found", name, ErrBytesLength, expected, found)
}

func ErrVectorLengthFn(name string, found, expected int) error {
	return fmt.Errorf("%s (%v): expected %d and %d found", name, ErrBytesLength, expected, found)
}

func ErrListTooBigFn(name string, found, max int) error {
	return fmt.Errorf("%s (%v): max expected %d and %d found", name, ErrListTooBig, max, found)
}

// ---- Unmarshal functions ----

// UnmarshallUint64 unmarshals a little endian uint64 from the src input
func UnmarshallUint64(src []byte) uint64 {
	return binary.LittleEndian.Uint64(src)
}

// UnmarshallUint32 unmarshals a little endian uint32 from the src input
func UnmarshallUint32(src []byte) uint32 {
	return binary.LittleEndian.Uint32(src[:4])
}

// UnmarshallUint16 unmarshals a little endian uint16 from the src input
func UnmarshallUint16(src []byte) uint16 {
	return binary.LittleEndian.Uint16(src[:2])
}

// UnmarshallUint8 unmarshals a little endian uint8 from the src input
func UnmarshallUint8(src []byte) uint8 {
	return uint8(src[0])
}

// UnmarshalBool unmarshals a boolean from the src input
func UnmarshalBool(src []byte) bool {
	if src[0] == 1 {
		return true
	}
	return false
}

// ---- Marshal functions ----

// MarshalUint64 marshals a little endian uint64 to dst
func MarshalUint64(dst []byte, i uint64) []byte {
	buf := make([]byte, 8)
	binary.LittleEndian.PutUint64(buf, i)
	dst = append(dst, buf...)
	return dst
}

// MarshalUint32 marshals a little endian uint32 to dst
func MarshalUint32(dst []byte, i uint32) []byte {
	buf := make([]byte, 4)
	binary.LittleEndian.PutUint32(buf, i)
	dst = append(dst, buf...)
	return dst
}

// MarshalUint16 marshals a little endian uint16 to dst
func MarshalUint16(dst []byte, i uint16) []byte {
	buf := make([]byte, 2)
	binary.LittleEndian.PutUint16(buf, i)
	dst = append(dst, buf...)
	return dst
}

// MarshalUint8 marshals a little endian uint8 to dst
func MarshalUint8(dst []byte, i uint8) []byte {
	dst = append(dst, byte(i))
	return dst
}

// MarshalBool marshals a boolean to dst
func MarshalBool(dst []byte, b bool) []byte {
	if b {
		dst = append(dst, 1)
	} else {
		dst = append(dst, 0)
	}
	return dst
}

// ---- offset functions ----

// WriteOffset writes an offset to dst
func WriteOffset(dst []byte, i int) []byte {
	return MarshalUint32(dst, uint32(i))
}

// ReadOffset reads an offset from buf
func ReadOffset(buf []byte) uint64 {
	return uint64(binary.LittleEndian.Uint32(buf))
}

func safeReadOffset(buf []byte) (uint64, []byte, error) {
	if len(buf) < 4 {
		return 0, nil, fmt.Errorf("")
	}
	offset := ReadOffset(buf)
	return offset, buf[4:], nil
}

// ---- extend functions ----

func extendByteSlice(b []byte, needLen int) []byte {
	b = b[:cap(b)]
	if n := needLen - cap(b); n > 0 {
		b = append(b, make([]byte, n)...)
	}
	return b[:needLen]
}

// ExtendUint64 extends a uint64 buffer to a given size
func ExtendUint64(b []uint64, needLen int) []uint64 {
	b = b[:cap(b)]
	if n := needLen - cap(b); n > 0 {
		b = append(b, make([]uint64, n)...)
	}
	return b[:needLen]
}

// ExtendUint16 extends a uint16 buffer to a given size
func ExtendUint16(b []uint16, needLen int) []uint16 {
	b = b[:cap(b)]
	if n := needLen - cap(b); n > 0 {
		b = append(b, make([]uint16, n)...)
	}
	return b[:needLen]
}

// ExtendUint16 extends a uint16 buffer to a given size
func ExtendUint8(b []uint8, needLen int) []uint8 {
	b = b[:cap(b)]
	if n := needLen - cap(b); n > 0 {
		b = append(b, make([]uint8, n)...)
	}
	return b[:needLen]
}

// ---- unmarshal dynamic content ----

const bytesPerLengthOffset = 4

// ValidateBitlist validates that the bitlist is correct
func ValidateBitlist(buf []byte, bitLimit uint64) error {
	byteLen := len(buf)
	if byteLen == 0 {
		return fmt.Errorf("bitlist empty, it does not have length bit")
	}
	// Maximum possible bytes in a bitlist with provided bitlimit.
	maxBytes := (bitLimit >> 3) + 1
	if byteLen > int(maxBytes) {
		return fmt.Errorf("unexpected number of bytes, got %d but found %d", byteLen, maxBytes)
	}

	// The most significant bit is present in the last byte in the array.
	last := buf[byteLen-1]
	if last == 0 {
		return fmt.Errorf("trailing byte is zero")
	}

	// Determine the position of the most significant bit.
	msb := bits.Len8(last)

	// The absolute position of the most significant bit will be the number of
	// bits in the preceding bytes plus the position of the most significant
	// bit. Subtract this value by 1 to determine the length of the bitlist.
	numOfBits := uint64(8*(byteLen-1) + msb - 1)

	if numOfBits > bitLimit {
		return fmt.Errorf("too many bits")
	}
	return nil
}

// DecodeDynamicLength decodes the length from the dynamic input
func DecodeDynamicLength(buf []byte, maxSize int) (int, error) {
	if len(buf) == 0 {
		return 0, nil
	}
	if len(buf) < 4 {
		return 0, fmt.Errorf("not enough data")
	}
	offset := binary.LittleEndian.Uint32(buf[:4])
	length, ok := DivideInt(int(offset), bytesPerLengthOffset)
	if !ok {
		return 0, fmt.Errorf("bad")
	}
	if length > maxSize {
		return 0, fmt.Errorf("too big for the list")
	}
	return length, nil
}

// UnmarshalDynamic unmarshals the dynamic items from the input
func UnmarshalDynamic(src []byte, length int, f func(indx int, b []byte) error) error {
	var err error
	if length == 0 {
		return nil
	}

	size := uint64(len(src))

	indx := 0
	dst := src

	var offset, endOffset uint64
	offset, dst = ReadOffset(src), dst[4:]

	for {
		if length != 1 {
			endOffset, dst, err = safeReadOffset(dst)
			if err != nil {
				return err
			}
		} else {
			endOffset = uint64(len(src))
		}
		if offset > endOffset {
			return fmt.Errorf("four")
		}
		if endOffset > size {
			return fmt.Errorf("five")
		}

		err := f(indx, src[offset:endOffset])
		if err != nil {
			return err
		}

		indx++

		offset = endOffset
		if length == 1 {
			break
		}
		length--
	}
	return nil
}

func DivideInt2(a, b, max int) (int, error) {
	num, ok := DivideInt(a, b)
	if !ok {
		return 0, fmt.Errorf("xx")
	}
	if num > max {
		return 0, fmt.Errorf("yy")
	}
	return num, nil
}

// DivideInt divides the int fully
func DivideInt(a, b int) (int, bool) {
	return a / b, a%b == 0
}
