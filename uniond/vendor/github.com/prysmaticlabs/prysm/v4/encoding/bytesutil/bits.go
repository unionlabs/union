package bytesutil

import (
	"math/bits"

	"github.com/pkg/errors"
)

// SetBit sets the index `i` of bitlist `b` to 1.
// It grows and returns a longer bitlist with 1 set
// if index `i` is out of range.
func SetBit(b []byte, i int) []byte {
	if i >= len(b)*8 {
		h := (i + (8 - i%8)) / 8
		b = append(b, make([]byte, h-len(b))...)
	}

	bit := uint8(1 << (i % 8))
	b[i/8] |= bit
	return b
}

// ClearBit clears the index `i` of bitlist `b`.
// Returns the original bitlist if the index `i`
// is out of range.
func ClearBit(b []byte, i int) []byte {
	if i >= len(b)*8 || i < 0 {
		return b
	}

	bit := uint8(1 << (i % 8))
	b[i/8] &^= bit
	return b
}

// MakeEmptyBitlists returns an empty bitlist with
// input size `i`.
func MakeEmptyBitlists(i int) []byte {
	return make([]byte, (i+(8-i%8))/8)
}

// HighestBitIndex returns the index of the highest
// bit set from bitlist `b`.
func HighestBitIndex(b []byte) (int, error) {
	if len(b) == 0 {
		return 0, errors.New("input list can't be empty or nil")
	}

	for i := len(b) - 1; i >= 0; i-- {
		if b[i] == 0 {
			continue
		}
		return bits.Len8(b[i]) + (i * 8), nil
	}

	return 0, nil
}

// HighestBitIndexAt returns the index of the highest
// bit set from bitlist `b` that is at `index` (inclusive).
func HighestBitIndexAt(b []byte, index int) (int, error) {
	bLength := len(b)
	if b == nil || bLength == 0 {
		return 0, errors.New("input list can't be empty or nil")
	}
	if index < 0 {
		return 0, errors.Errorf("index is negative: %d", index)
	}

	start := index / 8
	if start >= bLength {
		start = bLength - 1
	}

	mask := byte(1<<(index%8) - 1)
	for i := start; i >= 0; i-- {
		if index/8 > i {
			mask = 0xff
		}
		masked := b[i] & mask
		minBitsMasked := bits.Len8(masked)
		if b[i] == 0 || (minBitsMasked == 0 && index/8 <= i) {
			continue
		}

		return minBitsMasked + (i * 8), nil
	}

	return 0, nil
}
