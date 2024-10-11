package detect

import (
	"encoding/binary"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestTypeMismatch(t *testing.T) {
	wrong := fieldSpec{
		offset: 52,
		t:      typeBytes4,
	}
	_, err := wrong.uint64([]byte{})
	require.ErrorIs(t, err, errWrongMethodForType)

	wrong = fieldSpec{
		offset: 100,
		t:      typeUint64,
	}
	_, err = wrong.bytes4([]byte{})
	require.ErrorIs(t, err, errWrongMethodForType)
}

func TestFieldSpecUint(t *testing.T) {
	var expectedUint uint64 = 23
	buf := make([]byte, binary.MaxVarintLen64)
	uv := binary.PutUvarint(buf, expectedUint)
	require.Equal(t, 1, uv)
	padded := make([]byte, 100)
	uintOffset := 10
	copy(padded[uintOffset:], buf)
	fs := fieldSpec{offset: uintOffset, t: typeUint64}
	u, err := fs.uint64(padded)
	require.NoError(t, err)
	require.Equal(t, expectedUint, u)
}

func TestFieldSpecBytes4(t *testing.T) {
	expectedBytes := []byte("cafe")
	padded := make([]byte, 100)
	byteOffset := 42
	copy(padded[byteOffset:], expectedBytes)
	fs := fieldSpec{offset: byteOffset, t: typeBytes4}
	b, err := fs.bytes4(padded)
	require.NoError(t, err)
	require.DeepEqual(t, expectedBytes, b[:])
}

func TestFieldSpecSlice(t *testing.T) {
	cases := []struct {
		offset    int
		fieldType fieldType
		slice     []byte
		err       error
		name      string
		expected  []byte
	}{
		{
			offset:    0,
			fieldType: typeBytes4,
			slice:     []byte{},
			err:       errIndexOutOfRange,
			name:      "zero length, out of range",
		},
		{
			offset:    1,
			fieldType: typeBytes4,
			slice:     []byte("1234"),
			err:       errIndexOutOfRange,
			name:      "non-zero length, out of range",
		},
		{
			offset:    1,
			fieldType: typeBytes4,
			slice:     []byte("12345"),
			expected:  []byte("2345"),
			name:      "success",
		},
		{
			offset:    1,
			fieldType: typeUint64,
			slice:     []byte("123456789"),
			expected:  []byte("23456789"),
			name:      "uint success",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			s := fieldSpec{
				offset: c.offset,
				t:      c.fieldType,
			}
			b, err := s.slice(c.slice)
			if c.err == nil {
				require.NoError(t, err)
				require.DeepEqual(t, c.expected, b)
			} else {
				require.ErrorIs(t, err, c.err)
			}
		})
	}
}
