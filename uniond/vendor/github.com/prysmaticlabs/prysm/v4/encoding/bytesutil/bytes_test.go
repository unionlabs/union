package bytesutil_test

import (
	"bytes"
	"fmt"
	"reflect"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestTruncate(t *testing.T) {
	tests := []struct {
		a []byte
		b []byte
	}{
		{[]byte{'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O'},
			[]byte{'A', 'C', 'D', 'E', 'F', 'G'}},
		{[]byte{'A', 'C', 'D', 'E', 'F'},
			[]byte{'A', 'C', 'D', 'E', 'F'}},
		{[]byte{}, []byte{}},
	}
	for _, tt := range tests {
		b := bytesutil.Trunc(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestReverse(t *testing.T) {
	tests := []struct {
		input  [][32]byte
		output [][32]byte
	}{
		{[][32]byte{{'A'}, {'B'}, {'C'}, {'D'}, {'E'}, {'F'}, {'G'}, {'H'}},
			[][32]byte{{'H'}, {'G'}, {'F'}, {'E'}, {'D'}, {'C'}, {'B'}, {'A'}}},
		{[][32]byte{{1}, {2}, {3}, {4}},
			[][32]byte{{4}, {3}, {2}, {1}}},
		{[][32]byte{}, [][32]byte{}},
	}
	for _, tt := range tests {
		b := bytesutil.ReverseBytes32Slice(tt.input)
		assert.DeepEqual(t, tt.output, b)
	}
}

func TestSafeCopyRootAtIndex(t *testing.T) {
	tests := []struct {
		name    string
		input   [][]byte
		idx     uint64
		want    []byte
		wantErr bool
	}{
		{
			name:    "index out of range in non-empty slice",
			input:   [][]byte{{0x1}, {0x2}},
			idx:     2,
			wantErr: true,
		},
		{
			name:    "index out of range in empty slice",
			input:   [][]byte{},
			idx:     0,
			wantErr: true,
		},
		{
			name:  "nil input",
			input: nil,
			idx:   3,
			want:  nil,
		},
		{
			name:  "correct copy",
			input: [][]byte{{0x1}, {0x2}},
			idx:   1,
			want:  bytesutil.PadTo([]byte{0x2}, 32),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := bytesutil.SafeCopyRootAtIndex(tt.input, tt.idx)
			if (err != nil) != tt.wantErr {
				t.Errorf("SafeCopyRootAtIndex() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("SafeCopyRootAtIndex() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSafeCopy2dBytes(t *testing.T) {
	tests := []struct {
		name  string
		input [][]byte
	}{
		{
			name:  "nil input",
			input: nil,
		},
		{
			name:  "correct copy",
			input: [][]byte{{0x1}, {0x2}},
		},
		{
			name:  "empty",
			input: [][]byte{},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := bytesutil.SafeCopy2dBytes(tt.input); !reflect.DeepEqual(got, tt.input) {
				t.Errorf("SafeCopy2dBytes() = %v, want %v", got, tt.input)
			}
		})
	}
}

func TestBytesInvalidInputs(t *testing.T) {
	defer func() {
		if r := recover(); r != nil {
			t.Errorf("Test panicked: %v", r)
		}
	}()
	rawBytes := bytesutil.ToBytes(100, -10)
	assert.DeepEqual(t, []byte{}, rawBytes)

	_, err := bytesutil.HighestBitIndexAt([]byte{'A', 'B', 'C'}, -5)
	assert.ErrorContains(t, "index is negative", err)

	// There should be no panic
	_ = bytesutil.ClearBit([]byte{'C', 'D', 'E'}, -7)
	res := bytesutil.FromBytes4([]byte{})
	assert.Equal(t, res, uint64(0))
	newRes := bytesutil.FromBytes2([]byte{})
	assert.Equal(t, newRes, uint16(0))
	res = bytesutil.FromBytes8([]byte{})
	assert.Equal(t, res, uint64(0))

	intRes := bytesutil.ToLowInt64([]byte{})
	assert.Equal(t, intRes, int64(0))
}

func TestReverseByteOrder(t *testing.T) {
	input := []byte{0, 1, 2, 3, 4, 5}
	expectedResult := []byte{5, 4, 3, 2, 1, 0}
	output := bytesutil.ReverseByteOrder(input)

	// check that the input is not modified and the output is reversed
	assert.Equal(t, bytes.Equal(input, []byte{0, 1, 2, 3, 4, 5}), true)
	assert.Equal(t, bytes.Equal(expectedResult, output), true)
}

func TestSafeCopy2d32Bytes(t *testing.T) {
	input := make([][32]byte, 2)
	input[0] = bytesutil.ToBytes32([]byte{'a'})
	input[1] = bytesutil.ToBytes32([]byte{'b'})
	output := bytesutil.SafeCopy2d32Bytes(input)
	assert.Equal(t, false, &input == &output, "No copy was made")
	assert.DeepEqual(t, input, output)
}

func TestSafeCopy2dHexUtilBytes(t *testing.T) {
	input := make([]hexutil.Bytes, 2)
	input[0] = hexutil.Bytes{'a'}
	input[1] = hexutil.Bytes{'b'}
	output := bytesutil.SafeCopy2dHexUtilBytes(input)
	assert.DeepEqual(t, output, [][]byte{{'a'}, {'b'}})
}

func TestToBytes48Array(t *testing.T) {
	tests := []struct {
		a [][]byte
		b [][48]byte
	}{
		{[][]byte{{0}}, [][48]byte{{0}}},
		{[][]byte{{253}}, [][48]byte{{253}}},
		{[][]byte{{254, 255, 255, 255, 255, 255, 255, 127}},
			[][48]byte{{254, 255, 255, 255, 255, 255, 255, 127}}},
		{[][]byte{{255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255}},
			[][48]byte{{255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255}},
		},
	}
	for _, tt := range tests {
		b := bytesutil.ToBytes48Array(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func TestToBytes20(t *testing.T) {
	tests := []struct {
		a []byte
		b [20]byte
	}{
		{nil, [20]byte{}},
		{[]byte{}, [20]byte{}},
		{[]byte{1}, [20]byte{1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{[]byte{1, 2, 3}, [20]byte{1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{[]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20}, [20]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20}},
		{[]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21}, [20]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20}},
	}
	for _, tt := range tests {
		b := bytesutil.ToBytes20(tt.a)
		assert.DeepEqual(t, tt.b, b)
	}
}

func BenchmarkToBytes32(b *testing.B) {
	x := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31}
	for i := 0; i < b.N; i++ {
		bytesutil.ToBytes32(x)
	}
}

func TestFromBytes48Array(t *testing.T) {
	tests := []struct {
		a [][]byte
		b [][48]byte
	}{
		{[][]byte{{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
			[][48]byte{{0}}},
		{[][]byte{{253, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
			[][48]byte{{253}}},
		{[][]byte{{254, 255, 255, 255, 255, 255, 255, 127, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}},
			[][48]byte{{254, 255, 255, 255, 255, 255, 255, 127}}},
		{[][]byte{{255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
			255, 255, 255}},
			[][48]byte{{255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
				255, 255, 255}},
		},
	}
	for _, tt := range tests {
		a := bytesutil.FromBytes48Array(tt.b)
		assert.DeepEqual(t, tt.a, a)
	}
}

func TestSafeCopyBytes_Copy(t *testing.T) {
	slice := make([]byte, 32)
	slice[0] = 'A'

	copiedSlice := bytesutil.SafeCopyBytes(slice)

	assert.NotEqual(t, fmt.Sprintf("%p", slice), fmt.Sprintf("%p", copiedSlice))
	assert.Equal(t, slice[0], copiedSlice[0])
	slice[1] = 'B'

	assert.NotEqual(t, slice[1], copiedSlice[1])
}

func BenchmarkSafeCopyBytes(b *testing.B) {
	dSlice := make([][]byte, 900000)
	for i := 0; i < 900000; i++ {
		slice := make([]byte, 32)
		slice[0] = 'A'
		dSlice[i] = slice
	}

	b.ReportAllocs()
	b.ResetTimer()
	b.Run("Copy Bytes", func(b *testing.B) {
		cSlice := bytesutil.SafeCopy2dBytes(dSlice)
		a := cSlice
		_ = a
	})
}
