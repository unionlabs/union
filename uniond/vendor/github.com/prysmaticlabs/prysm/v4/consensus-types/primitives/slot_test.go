package primitives_test

import (
	"fmt"
	"math"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	mathprysm "github.com/prysmaticlabs/prysm/v4/math"
)

func TestSlot_Casting(t *testing.T) {
	slot := primitives.Slot(42)

	t.Run("time.Duration", func(t *testing.T) {
		if uint64(time.Duration(slot)) != uint64(slot) {
			t.Error("Slot should produce the same result with time.Duration")
		}
	})

	t.Run("floats", func(t *testing.T) {
		var x1 float32 = 42.2
		if primitives.Slot(x1) != slot {
			t.Errorf("Unequal: %v = %v", primitives.Slot(x1), slot)
		}

		var x2 = 42.2
		if primitives.Slot(x2) != slot {
			t.Errorf("Unequal: %v = %v", primitives.Slot(x2), slot)
		}
	})

	t.Run("int", func(t *testing.T) {
		var x = 42
		if primitives.Slot(x) != slot {
			t.Errorf("Unequal: %v = %v", primitives.Slot(x), slot)
		}
	})
}

func TestSlot_Mul(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Slot
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
		t.Run(fmt.Sprintf("Slot(%v).Mul(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).Mul(tt.b)
				})
			} else {
				res = primitives.Slot(tt.a).Mul(tt.b)
			}
			if tt.res != res {
				t.Errorf("Slot.Mul() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).MulSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).MulSlot(primitives.Slot(tt.b))
				})
			} else {
				res = primitives.Slot(tt.a).MulSlot(primitives.Slot(tt.b))
			}
			if tt.res != res {
				t.Errorf("Slot.MulSlot() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).SafeMulSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			res, err := primitives.Slot(tt.a).SafeMulSlot(primitives.Slot(tt.b))
			if tt.panicMsg != "" && (err == nil || err.Error() != tt.panicMsg) {
				t.Errorf("Expected error not thrown, wanted: %v, got: %v", tt.panicMsg, err)
				return
			}
			if tt.res != res {
				t.Errorf("Slot.SafeMulSlot() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestSlot_Div(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Slot
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
		t.Run(fmt.Sprintf("Slot(%v).Div(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).Div(tt.b)
				})
			} else {
				res = primitives.Slot(tt.a).Div(tt.b)
			}
			if tt.res != res {
				t.Errorf("Slot.Div() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).DivSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).DivSlot(primitives.Slot(tt.b))
				})
			} else {
				res = primitives.Slot(tt.a).DivSlot(primitives.Slot(tt.b))
			}
			if tt.res != res {
				t.Errorf("Slot.DivSlot() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).SafeDivSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			res, err := primitives.Slot(tt.a).SafeDivSlot(primitives.Slot(tt.b))
			if tt.panicMsg != "" && (err == nil || err.Error() != tt.panicMsg) {
				t.Errorf("Expected error not thrown, wanted: %v, got: %v", tt.panicMsg, err)
				return
			}
			if tt.res != res {
				t.Errorf("Slot.SafeDivSlot() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestSlot_Add(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Slot
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
		t.Run(fmt.Sprintf("Slot(%v).Add(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).Add(tt.b)
				})
			} else {
				res = primitives.Slot(tt.a).Add(tt.b)
			}
			if tt.res != res {
				t.Errorf("Slot.Add() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).AddSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).AddSlot(primitives.Slot(tt.b))
				})
			} else {
				res = primitives.Slot(tt.a).AddSlot(primitives.Slot(tt.b))
			}
			if tt.res != res {
				t.Errorf("Slot.AddSlot() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).SafeAddSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			res, err := primitives.Slot(tt.a).SafeAddSlot(primitives.Slot(tt.b))
			if tt.panicMsg != "" && (err == nil || err.Error() != tt.panicMsg) {
				t.Errorf("Expected error not thrown, wanted: %v, got: %v", tt.panicMsg, err)
				return
			}
			if tt.res != res {
				t.Errorf("Slot.SafeAddSlot() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestSlot_Sub(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Slot
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
		t.Run(fmt.Sprintf("Slot(%v).Sub(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).Sub(tt.b)
				})
			} else {
				res = primitives.Slot(tt.a).Sub(tt.b)
			}
			if tt.res != res {
				t.Errorf("Slot.Sub() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).SubSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).SubSlot(primitives.Slot(tt.b))
				})
			} else {
				res = primitives.Slot(tt.a).SubSlot(primitives.Slot(tt.b))
			}
			if tt.res != res {
				t.Errorf("Slot.SubSlot() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).SafeSubSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			res, err := primitives.Slot(tt.a).SafeSubSlot(primitives.Slot(tt.b))
			if tt.panicMsg != "" && (err == nil || err.Error() != tt.panicMsg) {
				t.Errorf("Expected error not thrown, wanted: %v, got: %v", tt.panicMsg, err)
				return
			}
			if tt.res != res {
				t.Errorf("Slot.SafeSubSlot() = %v, want %v", res, tt.res)
			}
		})
	}
}

func TestSlot_Mod(t *testing.T) {
	tests := []struct {
		a, b     uint64
		res      primitives.Slot
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
		t.Run(fmt.Sprintf("Slot(%v).Mod(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).Mod(tt.b)
				})
			} else {
				res = primitives.Slot(tt.a).Mod(tt.b)
			}
			if tt.res != res {
				t.Errorf("Slot.Mod() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).ModSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			var res primitives.Slot
			if tt.panicMsg != "" {
				assertPanic(t, tt.panicMsg, func() {
					res = primitives.Slot(tt.a).ModSlot(primitives.Slot(tt.b))
				})
			} else {
				res = primitives.Slot(tt.a).ModSlot(primitives.Slot(tt.b))
			}
			if tt.res != res {
				t.Errorf("Slot.Mod() = %v, want %v", res, tt.res)
			}
		})
		t.Run(fmt.Sprintf("Slot(%v).SafeModSlot(%v) = %v", tt.a, tt.b, tt.res), func(t *testing.T) {
			res, err := primitives.Slot(tt.a).SafeModSlot(primitives.Slot(tt.b))
			if tt.panicMsg != "" && (err == nil || err.Error() != tt.panicMsg) {
				t.Errorf("Expected error not thrown, wanted: %v, got: %v", tt.panicMsg, err)
				return
			}
			if tt.res != res {
				t.Errorf("Slot.SafeModSlot() = %v, want %v", res, tt.res)
			}
		})
	}
}
