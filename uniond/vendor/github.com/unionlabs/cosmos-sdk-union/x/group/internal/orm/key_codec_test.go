package orm

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestAddLengthPrefix(t *testing.T) {
	tcs := []struct {
		name     string
		in       []byte
		expected []byte
	}{
		{"empty", []byte{}, []byte{0}},
		{"nil", nil, []byte{0}},
		{"some data", []byte{0, 1, 100, 200}, []byte{4, 0, 1, 100, 200}},
	}
	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			out := AddLengthPrefix(tc.in)
			require.Equal(t, tc.expected, out)
		})
	}

	require.Panics(t, func() {
		AddLengthPrefix(make([]byte, 256))
	})
}

func TestNullTerminatedBytes(t *testing.T) {
	tcs := []struct {
		name     string
		in       string
		expected []byte
	}{
		{"empty", "", []byte{0}},
		{"some data", "abc", []byte{0x61, 0x62, 0x63, 0}},
	}
	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			out := NullTerminatedBytes(tc.in)
			require.Equal(t, tc.expected, out)
		})
	}
}
