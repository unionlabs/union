package danger_test

import (
	"testing"
	"unsafe"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/pelletier/go-toml/v2/internal/danger"
)

func TestSubsliceOffsetValid(t *testing.T) {
	examples := []struct {
		desc   string
		test   func() ([]byte, []byte)
		offset int
	}{
		{
			desc: "simple",
			test: func() ([]byte, []byte) {
				data := []byte("hello")
				return data, data[1:]
			},
			offset: 1,
		},
	}

	for _, e := range examples {
		t.Run(e.desc, func(t *testing.T) {
			d, s := e.test()
			offset := danger.SubsliceOffset(d, s)
			assert.Equal(t, e.offset, offset)
		})
	}
}

func TestSubsliceOffsetInvalid(t *testing.T) {
	examples := []struct {
		desc string
		test func() ([]byte, []byte)
	}{
		{
			desc: "unrelated arrays",
			test: func() ([]byte, []byte) {
				return []byte("one"), []byte("two")
			},
		},
		{
			desc: "slice starts before data",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[5:], full[1:]
			},
		},
		{
			desc: "slice starts after data",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[:3], full[5:]
			},
		},
		{
			desc: "slice ends after data",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[:5], full[3:8]
			},
		},
	}

	for _, e := range examples {
		t.Run(e.desc, func(t *testing.T) {
			d, s := e.test()
			require.Panics(t, func() {
				danger.SubsliceOffset(d, s)
			})
		})
	}
}

func TestStride(t *testing.T) {
	a := []byte{1, 2, 3, 4}
	x := &a[1]
	n := (*byte)(danger.Stride(unsafe.Pointer(x), unsafe.Sizeof(byte(0)), 1))
	require.Equal(t, &a[2], n)
	n = (*byte)(danger.Stride(unsafe.Pointer(x), unsafe.Sizeof(byte(0)), -1))
	require.Equal(t, &a[0], n)
}

func TestBytesRange(t *testing.T) {
	type fn = func() ([]byte, []byte)
	examples := []struct {
		desc     string
		test     fn
		expected []byte
	}{
		{
			desc: "simple",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[1:3], full[6:8]
			},
			expected: []byte("ello wo"),
		},
		{
			desc: "full",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[0:1], full[len(full)-1:]
			},
			expected: []byte("hello world"),
		},
		{
			desc: "end before start",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[len(full)-1:], full[0:1]
			},
		},
		{
			desc: "nils",
			test: func() ([]byte, []byte) {
				return nil, nil
			},
		},
		{
			desc: "nils start",
			test: func() ([]byte, []byte) {
				return nil, []byte("foo")
			},
		},
		{
			desc: "nils end",
			test: func() ([]byte, []byte) {
				return []byte("foo"), nil
			},
		},
		{
			desc: "start is end",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[1:3], full[1:3]
			},
			expected: []byte("el"),
		},
		{
			desc: "end contained in start",
			test: func() ([]byte, []byte) {
				full := []byte("hello world")
				return full[1:7], full[2:4]
			},
			expected: []byte("ello w"),
		},
		{
			desc: "different backing arrays",
			test: func() ([]byte, []byte) {
				one := []byte("hello world")
				two := []byte("hello world")
				return one, two
			},
		},
	}

	for _, e := range examples {
		t.Run(e.desc, func(t *testing.T) {
			start, end := e.test()
			if e.expected == nil {
				require.Panics(t, func() {
					danger.BytesRange(start, end)
				})
			} else {
				res := danger.BytesRange(start, end)
				require.Equal(t, e.expected, res)
			}
		})
	}
}
