package math

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestMax(t *testing.T) {
	maxInt := Max(10, -10, 20, 1_000_000, 10, 8, -11_000_000, 20)
	require.Equal(t, 1_000_000, maxInt, "invalid max for int")
	minInt := Min(10, -10, 20, 1_000_000, 10, 8, -11_000_000, 20)
	require.Equal(t, -11_000_000, minInt, "invalid min for int")

	maxf64 := Max(10.1, -10.1, 20.8, 1_000_000.9, 10.5, 8.4, -11_000_000.9, 20.7)
	require.Equal(t, 1_000_000.9, maxf64, "invalid max for float64")
	minf64 := Min(10.1, -10.1, 20.8, 1_000_000.9, 10.5, 8.4, -11_000_000.9, 20.7)
	require.Equal(t, -11_000_000.9, minf64, "invalid min for float64")
}
