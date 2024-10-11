package bytesutil_test

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math/big"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestToBytes(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0, []byte{0}},
		{1, []byte{1}},
		{2, []byte{2}},
		{253, []byte{253}},
		{254, []byte{254}},
		{255, []byte{255}},
		{0, []byte{0, 0}},
		{1, []byte{1, 0}},
		{255, []byte{255, 0}},
		{256, []byte{0, 1}},
		{65534, []byte{254, 255}},
		{65535, []byte{255, 255}},
		{0, []byte{0, 0, 0}},
		{255, []byte{255, 0, 0}},
		{256, []byte{0, 1, 0}},
		{65535, []byte{255, 255, 0}},
		{65536, []byte{0, 0, 1}},
		{16777215, []byte{255, 255, 255}},
		{0, []byte{0, 0, 0, 0}},
		{256, []byte{0, 1, 0, 0}},
		{65536, []byte{0, 0, 1, 0}},
		{16777216, []byte{0, 0, 0, 1}},
		{16777217, []byte{1, 0, 0, 1}},
		{4294967295, []byte{255, 255, 255, 255}},
		{0, []byte{0, 0, 0, 0, 0, 0, 0, 0}},
		{16777216, []byte{0, 0, 0, 1, 0, 0, 0, 0}},
		{4294967296, []byte{0, 0, 0, 0, 1, 0, 0, 0}},
		{4294967297, []byte{1, 0, 0, 0, 1, 0, 0, 0}},
		{9223372036854775806, []byte{254, 255, 255, 255, 255, 255, 255, 127}},
		{9223372036854775807, []byte{255, 255, 255, 255, 255, 255, 255, 127}},
	}
	for _, tt := range tests {
		b := bytesutil.ToBytes(tt.a, len(tt.b))
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestBytes1(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0, []byte{0}},
		{1, []byte{1}},
		{2, []byte{2}},
		{253, []byte{253}},
		{254, []byte{254}},
		{255, []byte{255}},
	}
	for _, tt := range tests {
		b := bytesutil.Bytes1(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestBytes2(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0, []byte{0, 0}},
		{1, []byte{1, 0}},
		{255, []byte{255, 0}},
		{256, []byte{0, 1}},
		{65534, []byte{254, 255}},
		{65535, []byte{255, 255}},
	}
	for _, tt := range tests {
		b := bytesutil.Bytes2(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestBytes3(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0, []byte{0, 0, 0}},
		{255, []byte{255, 0, 0}},
		{256, []byte{0, 1, 0}},
		{65535, []byte{255, 255, 0}},
		{65536, []byte{0, 0, 1}},
		{16777215, []byte{255, 255, 255}},
	}
	for _, tt := range tests {
		b := bytesutil.Bytes3(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestBytes4(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0, []byte{0, 0, 0, 0}},
		{256, []byte{0, 1, 0, 0}},
		{65536, []byte{0, 0, 1, 0}},
		{16777216, []byte{0, 0, 0, 1}},
		{16777217, []byte{1, 0, 0, 1}},
		{4294967295, []byte{255, 255, 255, 255}},
	}
	for _, tt := range tests {
		b := bytesutil.Bytes4(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestBytes8(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0, []byte{0, 0, 0, 0, 0, 0, 0, 0}},
		{16777216, []byte{0, 0, 0, 1, 0, 0, 0, 0}},
		{4294967296, []byte{0, 0, 0, 0, 1, 0, 0, 0}},
		{4294967297, []byte{1, 0, 0, 0, 1, 0, 0, 0}},
		{9223372036854775806, []byte{254, 255, 255, 255, 255, 255, 255, 127}},
		{9223372036854775807, []byte{255, 255, 255, 255, 255, 255, 255, 127}},
	}
	for _, tt := range tests {
		b := bytesutil.Bytes8(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestBytes32(t *testing.T) {
	tests := []struct {
		a uint64
		b []byte
	}{
		{0,
			[]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{16777216,
			[]byte{0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{4294967296,
			[]byte{0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{4294967297,
			[]byte{1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{9223372036854775806,
			[]byte{254, 255, 255, 255, 255, 255, 255, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{9223372036854775807,
			[]byte{255, 255, 255, 255, 255, 255, 255, 127, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
	}
	for _, tt := range tests {
		b := bytesutil.Bytes32(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestFromBool(t *testing.T) {
	tests := []byte{
		0,
		1,
	}
	for _, tt := range tests {
		b := bytesutil.ToBool(tt)
		c := bytesutil.FromBool(b)
		assert.Equal(t, tt, c)
	}
}

func TestFromBytes2(t *testing.T) {
	tests := []uint64{
		0,
		1776,
		96726,
		(1 << 16) - 1,
	}
	for _, tt := range tests {
		b := bytesutil.ToBytes(tt, 2)
		c := bytesutil.FromBytes2(b)
		assert.Equal(t, uint16(tt), c)
	}
}

func TestFromBytes4(t *testing.T) {
	tests := []uint64{
		0,
		1776,
		96726,
		4290997,
		4294967295, // 2^32 - 1
		4294967200,
		3894948296,
	}
	for _, tt := range tests {
		b := bytesutil.ToBytes(tt, 4)
		c := bytesutil.FromBytes4(b)
		if c != tt {
			t.Errorf("Wanted %d but got %d", tt, c)
		}
		assert.Equal(t, tt, c)
	}
}

func TestFromBytes8(t *testing.T) {
	tests := []uint64{
		0,
		1776,
		96726,
		4290997,
		922376854775806,
		42893720984775807,
		18446744073709551615,
	}
	for _, tt := range tests {
		b := bytesutil.ToBytes(tt, 8)
		c := bytesutil.FromBytes8(b)
		assert.Equal(t, tt, c)
	}
}

func TestUint32ToBytes4(t *testing.T) {
	tests := []struct {
		value uint32
		want  [4]byte
	}{
		{
			value: 0x01000000,
			want:  [4]byte{1, 0, 0, 0},
		},
		{
			value: 0x00000001,
			want:  [4]byte{0, 0, 0, 1},
		},
	}
	for _, tt := range tests {
		t.Run(fmt.Sprintf("0x%08x", tt.value), func(t *testing.T) {
			if got := bytesutil.Uint32ToBytes4(tt.value); !bytes.Equal(got[:], tt.want[:]) {
				t.Errorf("Uint32ToBytes4() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestUint64ToBytes_RoundTrip(t *testing.T) {
	for i := uint64(0); i < 10000; i++ {
		b := bytesutil.Uint64ToBytesBigEndian(i)
		if got := bytesutil.BytesToUint64BigEndian(b); got != i {
			t.Error("Round trip did not match original value")
		}
	}
}

func TestLittleEndianBytesToBigInt(t *testing.T) {
	bytes := make([]byte, 8)
	binary.LittleEndian.PutUint64(bytes, 1234567890)
	converted := bytesutil.LittleEndianBytesToBigInt(bytes)
	expected := new(big.Int).SetInt64(1234567890)
	assert.DeepEqual(t, expected, converted)
}

func TestBigIntToLittleEndianBytes(t *testing.T) {
	expected := make([]byte, 4)
	binary.LittleEndian.PutUint32(expected, 1234567890)
	bigInt := new(big.Int).SetUint64(1234567890)
	converted := bytesutil.BigIntToLittleEndianBytes(bigInt)
	assert.DeepEqual(t, expected, converted)
}

func TestUint64ToBytesLittleEndian(t *testing.T) {
	tests := []struct {
		value uint64
		want  [8]byte
	}{
		{
			value: 0x01000000,
			want:  [8]byte{0, 0, 0, 1, 0, 0, 0, 0},
		},
		{
			value: 0x00000001,
			want:  [8]byte{1, 0, 0, 0, 0, 0, 0, 0},
		},
	}
	for _, tt := range tests {
		t.Run(fmt.Sprintf("0x%08x", tt.value), func(t *testing.T) {
			if got := bytesutil.Uint64ToBytesLittleEndian(tt.value); !bytes.Equal(got, tt.want[:]) {
				t.Errorf("Uint64ToBytesLittleEndian() = got %v, want %v", got, tt.want)
			}
		})
	}
}

func TestUint64ToBytesLittleEndian32(t *testing.T) {
	tests := []struct {
		value uint64
		want  [32]byte
	}{
		{
			value: 0x01000000,
			want:  [32]byte{0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		{
			value: 0x00000001,
			want:  [32]byte{1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
		},
	}
	for _, tt := range tests {
		t.Run(fmt.Sprintf("0x%08x", tt.value), func(t *testing.T) {
			if got := bytesutil.Uint64ToBytesLittleEndian32(tt.value); !bytes.Equal(got, tt.want[:]) {
				t.Errorf("Uint64ToBytesLittleEndian32() = got %v, want %v", got, tt.want)
			}
		})
	}
}
