package primitives

import (
	"testing"
	"time"
)

func TestValidatorIndex_Casting(t *testing.T) {
	valIdx := ValidatorIndex(42)

	t.Run("time.Duration", func(t *testing.T) {
		if uint64(time.Duration(valIdx)) != uint64(valIdx) {
			t.Error("ValidatorIndex should produce the same result with time.Duration")
		}
	})

	t.Run("floats", func(t *testing.T) {
		var x1 float32 = 42.2
		if ValidatorIndex(x1) != valIdx {
			t.Errorf("Unequal: %v = %v", ValidatorIndex(x1), valIdx)
		}

		var x2 = 42.2
		if ValidatorIndex(x2) != valIdx {
			t.Errorf("Unequal: %v = %v", ValidatorIndex(x2), valIdx)
		}
	})

	t.Run("int", func(t *testing.T) {
		var x = 42
		if ValidatorIndex(x) != valIdx {
			t.Errorf("Unequal: %v = %v", ValidatorIndex(x), valIdx)
		}
	})
}
