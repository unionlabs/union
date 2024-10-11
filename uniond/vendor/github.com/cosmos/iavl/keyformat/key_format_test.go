package keyformat

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestKeyFormatBytes(t *testing.T) {
	type keyPairs struct {
		key      [][]byte
		expected []byte
	}
	emptyTestVector := keyPairs{key: [][]byte{}, expected: []byte{'e'}}
	threeByteTestVector := keyPairs{
		key:      [][]byte{{1, 2, 3}},
		expected: []byte{'e', 0, 0, 0, 0, 0, 1, 2, 3},
	}
	eightByteTestVector := keyPairs{
		key:      [][]byte{{1, 2, 3, 4, 5, 6, 7, 8}},
		expected: []byte{'e', 1, 2, 3, 4, 5, 6, 7, 8},
	}

	tests := []struct {
		name        string
		kf          *KeyFormat
		testVectors []keyPairs
	}{{
		name: "simple 3 int key format",
		kf:   NewKeyFormat(byte('e'), 8, 8, 8),
		testVectors: []keyPairs{
			emptyTestVector,
			threeByteTestVector,
			eightByteTestVector,
			{
				key:      [][]byte{{1, 2, 3, 4, 5, 6, 7, 8}, {1, 2, 3, 4, 5, 6, 7, 8}, {1, 1, 2, 2, 3, 3}},
				expected: []byte{'e', 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 1, 1, 2, 2, 3, 3},
			},
		},
	}, {
		name: "zero suffix key format",
		kf:   NewKeyFormat(byte('e'), 8, 0),
		testVectors: []keyPairs{
			emptyTestVector,
			threeByteTestVector,
			eightByteTestVector,
			{
				key:      [][]byte{{1, 2, 3, 4, 5, 6, 7, 8}, {1, 2, 3, 4, 5, 6, 7, 8, 9}},
				expected: []byte{'e', 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 9},
			},
			{
				key:      [][]byte{{1, 2, 3, 4, 5, 6, 7, 8}, []byte("hellohello")},
				expected: []byte{'e', 1, 2, 3, 4, 5, 6, 7, 8, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x68, 0x65, 0x6c, 0x6c, 0x6f},
			},
		},
	}}
	for _, tc := range tests {
		kf := tc.kf
		for i, v := range tc.testVectors {
			assert.Equal(t, v.expected, kf.KeyBytes(v.key...), "key format %s, test case %d", tc.name, i)
		}
	}
}

func TestKeyFormat(t *testing.T) {
	kf := NewKeyFormat(byte('e'), 8, 8, 8)
	key := []byte{'e', 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 1, 144}
	var a, b, c int64 = 100, 200, 400
	assert.Equal(t, key, kf.Key(a, b, c))

	ao, bo, co := new(int64), new(int64), new(int64)
	kf.Scan(key, ao, bo, co)
	assert.Equal(t, a, *ao)
	assert.Equal(t, b, *bo)
	assert.Equal(t, c, *co)

	bs := new([]byte)
	kf.Scan(key, ao, bo, bs)
	assert.Equal(t, a, *ao)
	assert.Equal(t, b, *bo)
	assert.Equal(t, []byte{0, 0, 0, 0, 0, 0, 1, 144}, *bs)

	assert.Equal(t, []byte{'e', 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 200}, kf.Key(a, b))
}

func TestNegativeKeys(t *testing.T) {
	kf := NewKeyFormat(byte('e'), 8, 8)

	var a, b int64 = -100, -200
	// One's complement plus one
	key := []byte{
		'e',
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, byte(0xff + a + 1),
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, byte(0xff + b + 1),
	}
	assert.Equal(t, key, kf.Key(a, b))

	ao, bo := new(int64), new(int64)
	kf.Scan(key, ao, bo)
	assert.Equal(t, a, *ao)
	assert.Equal(t, b, *bo)
}

func TestOverflow(t *testing.T) {
	kf := NewKeyFormat(byte('o'), 8, 8)

	var a int64 = 1 << 62
	var b uint64 = 1 << 63
	key := []byte{
		'o',
		0x40, 0, 0, 0, 0, 0, 0, 0,
		0x80, 0, 0, 0, 0, 0, 0, 0,
	}
	assert.Equal(t, key, kf.Key(a, b))

	ao, bo := new(int64), new(int64)
	kf.Scan(key, ao, bo)
	assert.Equal(t, a, *ao)
	assert.Equal(t, int64(b), *bo)
}

func benchmarkKeyFormatBytes(b *testing.B, kf *KeyFormat, segments ...[]byte) {
	for i := 0; i < b.N; i++ {
		kf.KeyBytes(segments...)
	}
}

func BenchmarkKeyFormat_KeyBytesOneSegment(b *testing.B) {
	benchmarkKeyFormatBytes(b, NewKeyFormat('e', 8, 8, 8), nil)
}

func BenchmarkKeyFormat_KeyBytesThreeSegment(b *testing.B) {
	segments := [][]byte{
		{1, 2, 3, 4, 5, 6, 7, 8},
		{1, 2, 3, 4, 5, 6, 7, 8},
		{1, 1, 2, 2, 3, 3},
	}
	benchmarkKeyFormatBytes(b, NewKeyFormat('e', 8, 8, 8), segments...)
}

func BenchmarkKeyFormat_KeyBytesOneSegmentWithVariousLayouts(b *testing.B) {
	benchmarkKeyFormatBytes(b, NewKeyFormat('e', 8, 16, 32), nil)
}

func BenchmarkKeyFormat_KeyBytesThreeSegmentWithVariousLayouts(b *testing.B) {
	segments := [][]byte{
		{1, 2, 3, 4, 5, 6, 7, 8},
		{1, 2, 3, 4, 5, 6, 7, 8},
		{1, 1, 2, 2, 3, 3},
	}
	benchmarkKeyFormatBytes(b, NewKeyFormat('e', 8, 16, 32), segments...)
}
