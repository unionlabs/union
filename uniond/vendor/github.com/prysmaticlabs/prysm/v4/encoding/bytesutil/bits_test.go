package bytesutil_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSetBit(t *testing.T) {
	tests := []struct {
		a []byte
		b int
		c []byte
	}{
		{[]byte{0b00000000}, 1, []byte{0b00000010}},
		{[]byte{0b00000010}, 7, []byte{0b10000010}},
		{[]byte{0b10000010}, 9, []byte{0b10000010, 0b00000010}},
		{[]byte{0b10000010}, 27, []byte{0b10000010, 0b00000000, 0b00000000, 0b00001000}},
		{[]byte{0b10000010, 0b00000000}, 8, []byte{0b10000010, 0b00000001}},
		{[]byte{0b10000010, 0b00000000}, 31, []byte{0b10000010, 0b00000000, 0b00000000, 0b10000000}},
	}
	for _, tt := range tests {
		assert.DeepEqual(t, tt.c, bytesutil.SetBit(tt.a, tt.b))
	}
}

func TestClearBit(t *testing.T) {
	tests := []struct {
		a []byte
		b int
		c []byte
	}{
		{[]byte{0b00000000}, 1, []byte{0b00000000}},
		{[]byte{0b00000010}, 1, []byte{0b00000000}},
		{[]byte{0b10000010}, 1, []byte{0b10000000}},
		{[]byte{0b10000010}, 8, []byte{0b10000010}},
		{[]byte{0b10000010, 0b00001111}, 7, []byte{0b00000010, 0b00001111}},
		{[]byte{0b10000010, 0b00001111}, 10, []byte{0b10000010, 0b00001011}},
	}
	for _, tt := range tests {
		assert.DeepEqual(t, tt.c, bytesutil.ClearBit(tt.a, tt.b))
	}
}

func TestMakeEmptyBitlists(t *testing.T) {
	tests := []struct {
		a int
		b int
	}{
		{0, 1},
		{1, 1},
		{2, 1},
		{7, 1},
		{8, 2},
		{15, 2},
		{16, 3},
		{100, 13},
		{104, 14},
	}
	for _, tt := range tests {
		assert.DeepEqual(t, tt.b, len(bytesutil.MakeEmptyBitlists(tt.a)))
	}
}

func TestHighestBitIndex(t *testing.T) {
	tests := []struct {
		a     []byte
		b     int
		error bool
	}{
		{nil, 0, true},
		{[]byte{}, 0, true},
		{[]byte{0b00000001}, 1, false},
		{[]byte{0b10100101}, 8, false},
		{[]byte{0x00, 0x00}, 0, false},
		{[]byte{0xff, 0xa0}, 16, false},
		{[]byte{12, 34, 56, 78}, 31, false},
		{[]byte{255, 255, 255, 255}, 32, false},
	}
	for _, tt := range tests {
		i, err := bytesutil.HighestBitIndex(tt.a)
		if !tt.error {
			require.NoError(t, err)
			assert.DeepEqual(t, tt.b, i)
		} else {
			assert.ErrorContains(t, "input list can't be empty or nil", err)
		}
	}
}

func TestHighestBitIndexBelow(t *testing.T) {
	tests := []struct {
		a     []byte
		b     int
		c     int
		error bool
	}{
		{nil, 0, 0, true},
		{[]byte{}, 0, 0, true},
		{[]byte{0b00010001}, 0, 0, false},
		{[]byte{0b00010001}, 1, 1, false},
		{[]byte{0b00010001}, 2, 1, false},
		{[]byte{0b00010001}, 4, 1, false},
		{[]byte{0b00010001}, 5, 5, false},
		{[]byte{0b00010001}, 8, 5, false},
		{[]byte{0b00010001, 0b00000000}, 0, 0, false},
		{[]byte{0b00010001, 0b00000000}, 1, 1, false},
		{[]byte{0b00010001, 0b00000000}, 2, 1, false},
		{[]byte{0b00010001, 0b00000000}, 4, 1, false},
		{[]byte{0b00010001, 0b00000000}, 5, 5, false},
		{[]byte{0b00010001, 0b00000000}, 8, 5, false},
		{[]byte{0b00010001, 0b00000000}, 15, 5, false},
		{[]byte{0b00010001, 0b00000000}, 16, 5, false},
		{[]byte{0b00010001, 0b00100010}, 8, 5, false},
		{[]byte{0b00010001, 0b00100010}, 9, 5, false},
		{[]byte{0b00010001, 0b00100010}, 10, 10, false},
		{[]byte{0b00010001, 0b00100010}, 11, 10, false},
		{[]byte{0b00010001, 0b00100010}, 14, 14, false},
		{[]byte{0b00010001, 0b00100010}, 15, 14, false},
		{[]byte{0b00010001, 0b00100010}, 24, 14, false},
		{[]byte{0b00010001, 0b00100010, 0b10000000}, 23, 14, false},
		{[]byte{0b00010001, 0b00100010, 0b10000000}, 24, 24, false},
		{[]byte{0b00000000, 0b00000001, 0b00000011}, 17, 17, false},
		{[]byte{0b00000000, 0b00000001, 0b00000011}, 18, 18, false},
		{[]byte{12, 34, 56, 78}, 1000, 31, false},
		{[]byte{255, 255, 255, 255}, 1000, 32, false},
	}
	for _, tt := range tests {
		i, err := bytesutil.HighestBitIndexAt(tt.a, tt.b)
		if !tt.error {
			require.NoError(t, err)
			assert.DeepEqual(t, tt.c, i)
		} else {
			assert.ErrorContains(t, "input list can't be empty or nil", err)
		}
	}
}
