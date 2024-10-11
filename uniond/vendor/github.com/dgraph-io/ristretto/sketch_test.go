package ristretto

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSketch(t *testing.T) {
	defer func() {
		require.NotNil(t, recover())
	}()

	s := newCmSketch(5)
	require.Equal(t, uint64(7), s.mask)
	newCmSketch(0)
}

func TestSketchIncrement(t *testing.T) {
	s := newCmSketch(16)
	s.Increment(1)
	s.Increment(5)
	s.Increment(9)
	for i := 0; i < cmDepth; i++ {
		if s.rows[i].string() != s.rows[0].string() {
			break
		}
		require.False(t, i == cmDepth-1, "identical rows, bad seeding")
	}
}

func TestSketchEstimate(t *testing.T) {
	s := newCmSketch(16)
	s.Increment(1)
	s.Increment(1)
	require.Equal(t, int64(2), s.Estimate(1))
	require.Equal(t, int64(0), s.Estimate(0))
}

func TestSketchReset(t *testing.T) {
	s := newCmSketch(16)
	s.Increment(1)
	s.Increment(1)
	s.Increment(1)
	s.Increment(1)
	s.Reset()
	require.Equal(t, int64(2), s.Estimate(1))
}

func TestSketchClear(t *testing.T) {
	s := newCmSketch(16)
	for i := 0; i < 16; i++ {
		s.Increment(uint64(i))
	}
	s.Clear()
	for i := 0; i < 16; i++ {
		require.Equal(t, int64(0), s.Estimate(uint64(i)))
	}
}

func TestNext2Power(t *testing.T) {
	sz := 12 << 30
	szf := float64(sz) * 0.01
	val := int64(szf)
	t.Logf("szf = %.2f val = %d\n", szf, val)
	pow := next2Power(val)
	t.Logf("pow = %d. mult 4 = %d\n", pow, pow*4)
}

func BenchmarkSketchIncrement(b *testing.B) {
	s := newCmSketch(16)
	b.SetBytes(1)
	for n := 0; n < b.N; n++ {
		s.Increment(1)
	}
}

func BenchmarkSketchEstimate(b *testing.B) {
	s := newCmSketch(16)
	s.Increment(1)
	b.SetBytes(1)
	for n := 0; n < b.N; n++ {
		s.Estimate(1)
	}
}
