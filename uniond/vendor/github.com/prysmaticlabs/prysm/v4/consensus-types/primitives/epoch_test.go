package primitives_test

import (
	"fmt"
	"math"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	mathprysm "github.com/prysmaticlabs/prysm/v4/math"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestMaxEpoch(t *testing.T) {
	require.Equal(t, primitives.Epoch(0), primitives.MaxEpoch(0, 0))
	require.Equal(t, primitives.Epoch(1), primitives.MaxEpoch(1, 0))
	require.Equal(t, primitives.Epoch(1), primitives.MaxEpoch(0, 1))
	require.Equal(t, primitives.Epoch(1000), primitives.MaxEpoch(100, 1000))
}

func TestEpoch_Mul(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Epoch
		panicMsg string
	}{
		{a: 0, b: 1, res: 0},
		{a: 1 << 32, b: 1, res: 1 << 32},
		{a: 1 << 32, b: 100, res: 429496729600},
		{a: 1 << 32, b: 1 << 31, res: 9223372036854775808},
		{a: 1 << 32, b: 1 << 32, res: 0, panicMsg: mathprysm.ErrMulOverflow.Error()},
		{a: 1 << 62, b: 2, res: 9223372036854775808},
		{a: 1 << 62, b: 4, res: 0, panicMsg: mathprysm.ErrMulOverflow.Error()},
		{a: 1 << 63, b: 1, res: 9223372036854775808},
		{a: 1 << 63, b: 2, res: 0, panicMsg: mathprysm.ErrMulOverflow.Error()},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("Epoch(%v).Mul(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Epoch
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Epoch(tt.a).Mul(tt.b)
				})
			} else {
				res = primitives.Epoch(tt.a).Mul(tt.b)
			}
			if tt.res != res {
				t.Errorf("Epoch.Mul() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestEpoch_Div(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Epoch
		panicMsg string
	}{
		{a: 0, b: 1, res: 0},
		{a: 1, b: 0, res: 0, panicMsg: mathprysm.ErrDivByZero.Error()},
		{a: 1 << 32, b: 1 << 32, res: 1},
		{a: 429496729600, b: 1 << 32, res: 100},
		{a: 9223372036854775808, b: 1 << 32, res: 1 << 31},
		{a: 1 << 32, b: 1 << 32, res: 1},
		{a: 9223372036854775808, b: 1 << 62, res: 2},
		{a: 9223372036854775808, b: 1 << 63, res: 1},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("Epoch(%v).Div(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Epoch
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Epoch(tt.a).Div(tt.b)
				})
			} else {
				res = primitives.Epoch(tt.a).Div(tt.b)
			}
			if tt.res != res {
				t.Errorf("Epoch.Div() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestEpoch_Add(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Epoch
		panicMsg string
	}{
		{a: 0, b: 1, res: 1},
		{a: 1 << 32, b: 1, res: 4294967297},
		{a: 1 << 32, b: 100, res: 4294967396},
		{a: 1 << 31, b: 1 << 31, res: 4294967296},
		{a: 1 << 63, b: 1 << 63, res: 0, panicMsg: mathprysm.ErrAddOverflow.Error()},
		{a: 1 << 63, b: 1, res: 9223372036854775809},
		{a: math.MaxUint64, b: 1, res: 0, panicMsg: mathprysm.ErrAddOverflow.Error()},
		{a: math.MaxUint64, b: 0, res: math.MaxUint64},
		{a: 1 << 63, b: 2, res: 9223372036854775810},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("Epoch(%v).Add(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Epoch
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Epoch(tt.a).Add(tt.b)
				})
			} else {
				res = primitives.Epoch(tt.a).Add(tt.b)
			}
			if tt.res != res {
				t.Errorf("Epoch.Add() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Epoch(%v).AddEpoch(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Epoch
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Epoch(tt.a).AddEpoch(primitives.Epoch(tt.b))
				})
			} else {
				res = primitives.Epoch(tt.a).AddEpoch(primitives.Epoch(tt.b))
			}
			if tt.res != res {
				t.Errorf("Epoch.AddEpoch() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestEpoch_Sub(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Epoch
		panicMsg string
	}{
		{a: 1, b: 0, res: 1},
		{a: 0, b: 1, res: 0, panicMsg: mathprysm.ErrSubUnderflow.Error()},
		{a: 1 << 32, b: 1, res: 4294967295},
		{a: 1 << 32, b: 100, res: 4294967196},
		{a: 1 << 31, b: 1 << 31, res: 0},
		{a: 1 << 63, b: 1 << 63, res: 0},
		{a: 1 << 63, b: 1, res: 9223372036854775807},
		{a: math.MaxUint64, b: math.MaxUint64, res: 0},
		{a: math.MaxUint64 - 1, b: math.MaxUint64, res: 0, panicMsg: mathprysm.ErrSubUnderflow.Error()},
		{a: math.MaxUint64, b: 0, res: math.MaxUint64},
		{a: 1 << 63, b: 2, res: 9223372036854775806},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("Epoch(%v).Sub(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Epoch
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Epoch(tt.a).Sub(tt.b)
				})
			} else {
				res = primitives.Epoch(tt.a).Sub(tt.b)
			}
			if tt.res != res {
				t.Errorf("Epoch.Sub() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestEpoch_Mod(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Epoch
		panicMsg string
	}{
		{a: 1, b: 0, res: 0, panicMsg: mathprysm.ErrDivByZero.Error()},
		{a: 0, b: 1, res: 0},
		{a: 1 << 32, b: 1 << 32, res: 0},
		{a: 429496729600, b: 1 << 32, res: 0},
		{a: 9223372036854775808, b: 1 << 32, res: 0},
		{a: 1 << 32, b: 1 << 32, res: 0},
		{a: 9223372036854775808, b: 1 << 62, res: 0},
		{a: 9223372036854775808, b: 1 << 63, res: 0},
		{a: 1 << 32, b: 17, res: 1},
		{a: 1 << 32, b: 19, res: (1 << 32) % 19},
		{a: math.MaxUint64, b: math.MaxUint64, res: 0},
		{a: 1 << 63, b: 2, res: 0},
		{a: 1<<63 + 1, b: 2, res: 1},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("Epoch(%v).Mod(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Epoch
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Epoch(tt.a).Mod(tt.b)
				})
			} else {
				res = primitives.Epoch(tt.a).Mod(tt.b)
			}
			if tt.res != res {
				t.Errorf("Epoch.Mod() = %v, want %v", res, tt.res)
			}
		})
	}
}

func assertPanic(t *testing.T, panicMessage string, f func()) {
	defer func() {
		if r := recover(); r == nil {
			t.Errorf("Expected panic not thrown")
		} else if r != panicMessage {
			t.Errorf("Unexpected panic thrown, want: %#v, got: %#v", panicMessage, r)
		}
	}()
	f()
}
