package bytesutil

import (
	"encoding/binary"
	"errors"
	"fmt"
	"math/big"

	"github.com/prysmaticlabs/prysm/v4/math"
)

// ToBytes returns integer x to bytes in little-endian format at the specified length.
// Spec defines similar method uint_to_bytes(n: uint) -> bytes, which is equivalent to ToBytes(n, 8).
func ToBytes(x uint64, length int) []byte {
	if length < 0 {
		length = 0
	}
	makeLength := length
	if length < 8 {
		makeLength = 8
	}
	bytes := make([]byte, makeLength)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes[:length]
}

// Bytes1 returns integer x to bytes in little-endian format, x.to_bytes(1, 'little').
func Bytes1(x uint64) []byte {
	bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes[:1]
}

// Bytes2 returns integer x to bytes in little-endian format, x.to_bytes(2, 'little').
func Bytes2(x uint64) []byte {
	bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes[:2]
}

// Bytes3 returns integer x to bytes in little-endian format, x.to_bytes(3, 'little').
func Bytes3(x uint64) []byte {
	bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes[:3]
}

// Bytes4 returns integer x to bytes in little-endian format, x.to_bytes(4, 'little').
func Bytes4(x uint64) []byte {
	bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes[:4]
}

// Bytes8 returns integer x to bytes in little-endian format, x.to_bytes(8, 'little').
func Bytes8(x uint64) []byte {
	bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes
}

// Bytes32 returns integer x to bytes in little-endian format, x.to_bytes(32, 'little').
func Bytes32(x uint64) []byte {
	bytes := make([]byte, 32)
	binary.LittleEndian.PutUint64(bytes, x)
	return bytes
}

// FromBytes2 returns an integer which is stored in the little-endian format(2, 'little')
// from a byte array.
func FromBytes2(x []byte) uint16 {
	if len(x) < 2 {
		return 0
	}
	return binary.LittleEndian.Uint16(x[:2])
}

// FromBytes4 returns an integer which is stored in the little-endian format(4, 'little')
// from a byte array.
func FromBytes4(x []byte) uint64 {
	if len(x) < 4 {
		return 0
	}
	empty4bytes := make([]byte, 4)
	return binary.LittleEndian.Uint64(append(x[:4], empty4bytes...))
}

// FromBytes8 returns an integer which is stored in the little-endian format(8, 'little')
// from a byte array.
func FromBytes8(x []byte) uint64 {
	if len(x) < 8 {
		return 0
	}
	return binary.LittleEndian.Uint64(x)
}

// ToLowInt64 returns the lowest 8 bytes interpreted as little endian.
func ToLowInt64(x []byte) int64 {
	if len(x) < 8 {
		return 0
	}
	// Use the first 8 bytes.
	x = x[:8]
	return int64(binary.LittleEndian.Uint64(x)) // lint:ignore uintcast -- A negative number might be the expected result.
}

// Uint32ToBytes4 is a convenience method for converting uint32 to a fix
// sized 4 byte array in big endian order. Returns 4 byte array.
func Uint32ToBytes4(i uint32) [4]byte {
	buf := make([]byte, 4)
	binary.BigEndian.PutUint32(buf, i)
	return ToBytes4(buf)
}

// Uint64ToBytesLittleEndian conversion.
func Uint64ToBytesLittleEndian(i uint64) []byte {
	buf := make([]byte, 8)
	binary.LittleEndian.PutUint64(buf, i)
	return buf
}

// Uint64ToBytesLittleEndian32 conversion of a uint64 to a fix
// sized 32 byte array in little endian order. Returns 32 byte array.
func Uint64ToBytesLittleEndian32(i uint64) []byte {
	buf := make([]byte, 32)
	binary.LittleEndian.PutUint64(buf, i)
	return buf
}

// Uint64ToBytesBigEndian conversion.
func Uint64ToBytesBigEndian(i uint64) []byte {
	buf := make([]byte, 8)
	binary.BigEndian.PutUint64(buf, i)
	return buf
}

// BytesToUint64BigEndian conversion. Returns 0 if empty bytes or byte slice with length less
// than 8.
func BytesToUint64BigEndian(b []byte) uint64 {
	if len(b) < 8 { // This will panic otherwise.
		return 0
	}
	return binary.BigEndian.Uint64(b)
}

// LittleEndianBytesToBigInt takes bytes of a number stored as little-endian and returns a big integer
func LittleEndianBytesToBigInt(bytes []byte) *big.Int {
	// Integers are stored as little-endian, but big.Int expects big-endian. So we need to reverse the byte order before decoding.
	return new(big.Int).SetBytes(ReverseByteOrder(bytes))
}

// BigIntToLittleEndianBytes takes a big integer and returns its bytes stored as little-endian
func BigIntToLittleEndianBytes(bigInt *big.Int) []byte {
	// big.Int.Bytes() returns bytes in big-endian order, so we need to reverse the byte order
	return ReverseByteOrder(bigInt.Bytes())
}

// Uint256ToSSZBytes takes a string representation of uint256 and returns its bytes stored as little-endian
func Uint256ToSSZBytes(num string) ([]byte, error) {
	uint256, ok := new(big.Int).SetString(num, 10)
	if !ok {
		return nil, errors.New("could not parse Uint256")
	}
	if !math.IsValidUint256(uint256) {
		return nil, fmt.Errorf("%s is not a valid Uint256", num)
	}
	return PadTo(ReverseByteOrder(uint256.Bytes()), 32), nil
}
