// Package bytesutil defines helper methods for converting integers to byte slices.
package bytesutil

import (
	"fmt"

	"github.com/ethereum/go-ethereum/common/hexutil"
)

// ToBytes48Array is a convenience method for converting an array of
// byte slices to an array of fixed-sized byte arrays.
func ToBytes48Array(x [][]byte) [][48]byte {
	y := make([][48]byte, len(x))
	for i := range x {
		y[i] = ToBytes48(x[i])
	}
	return y
}

// ToBool is a convenience method for converting a byte to a bool.
// This method will use the first bit of the 0 byte to generate the returned value.
func ToBool(x byte) bool {
	return x&1 == 1
}

// FromBool is a convenience method for converting a bool to a byte.
// This method will use the first bit to generate the returned value.
func FromBool(x bool) byte {
	if x {
		return 1
	}
	return 0
}

// FromBytes48 is a convenience method for converting a fixed-size byte array
// to a byte slice.
func FromBytes48(x [48]byte) []byte {
	return x[:]
}

// FromBytes48Array is a convenience method for converting an array of
// fixed-size byte arrays to an array of byte slices.
func FromBytes48Array(x [][48]byte) [][]byte {
	y := make([][]byte, len(x))
	for i := range x {
		y[i] = x[i][:]
	}
	return y
}

// Trunc truncates the byte slices to 6 bytes.
func Trunc(x []byte) []byte {
	if len(x) > 6 {
		return x[:6]
	}
	return x
}

// SafeCopyRootAtIndex takes a copy of an 32-byte slice in a slice of byte slices. Returns error if index out of range.
func SafeCopyRootAtIndex(input [][]byte, idx uint64) ([]byte, error) {
	if input == nil {
		return nil, nil
	}

	if uint64(len(input)) <= idx {
		return nil, fmt.Errorf("index %d out of range", idx)
	}
	item := make([]byte, 32)
	copy(item, input[idx])
	return item, nil
}

// SafeCopyBytes will copy and return a non-nil byte slice, otherwise it returns nil.
func SafeCopyBytes(cp []byte) []byte {
	if cp != nil {
		if len(cp) == 32 {
			copied := [32]byte(cp)
			return copied[:]
		}
		copied := make([]byte, len(cp))
		copy(copied, cp)
		return copied
	}
	return nil
}

// SafeCopy2dBytes will copy and return a non-nil 2d byte slice, otherwise it returns nil.
func SafeCopy2dBytes(ary [][]byte) [][]byte {
	if ary != nil {
		copied := make([][]byte, len(ary))
		for i, a := range ary {
			copied[i] = SafeCopyBytes(a)
		}
		return copied
	}
	return nil
}

// SafeCopy2d32Bytes will copy and return a non-nil 2d byte slice, otherwise it returns nil.
func SafeCopy2d32Bytes(ary [][32]byte) [][32]byte {
	if ary != nil {
		copied := make([][32]byte, len(ary))
		copy(copied, ary)
		return copied
	}
	return nil
}

// SafeCopy2dHexUtilBytes will copy and return a non-nil 2d hex util byte slice, otherwise it returns nil.
func SafeCopy2dHexUtilBytes(ary []hexutil.Bytes) [][]byte {
	if ary != nil {
		copied := make([][]byte, len(ary))
		for i, a := range ary {
			copied[i] = SafeCopyBytes(a)
		}
		return copied
	}
	return nil
}

// ReverseBytes32Slice will reverse the provided slice's order.
func ReverseBytes32Slice(arr [][32]byte) [][32]byte {
	for i, j := 0, len(arr)-1; i < j; i, j = i+1, j-1 {
		arr[i], arr[j] = arr[j], arr[i]
	}
	return arr
}

// PadTo pads a byte slice to the given size. If the byte slice is larger than the given size, the
// original slice is returned.
func PadTo(b []byte, size int) []byte {
	if len(b) >= size {
		return b
	}
	return append(b, make([]byte, size-len(b))...)
}

// ReverseByteOrder Switch the endianness of a byte slice by reversing its order.
// This function does not modify the actual input bytes.
func ReverseByteOrder(input []byte) []byte {
	b := make([]byte, len(input))
	copy(b, input)
	for i := 0; i < len(b)/2; i++ {
		b[i], b[len(b)-i-1] = b[len(b)-i-1], b[i]
	}
	return b
}
