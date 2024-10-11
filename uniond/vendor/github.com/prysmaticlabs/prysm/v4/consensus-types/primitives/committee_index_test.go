package primitives

import (
	"testing"
)

func TestCommitteeIndex_Casting(t *testing.T) {
	committeeIdx := CommitteeIndex(42)

	t.Run("floats", func(t *testing.T) {
		var x1 float32 = 42.2
		if CommitteeIndex(x1) != committeeIdx {
			t.Errorf("Unequal: %v = %v", CommitteeIndex(x1), committeeIdx)
		}

		var x2 = 42.2
		if CommitteeIndex(x2) != committeeIdx {
			t.Errorf("Unequal: %v = %v", CommitteeIndex(x2), committeeIdx)
		}
	})

	t.Run("int", func(t *testing.T) {
		var x = 42
		if CommitteeIndex(x) != committeeIdx {
			t.Errorf("Unequal: %v = %v", CommitteeIndex(x), committeeIdx)
		}
	})
}
