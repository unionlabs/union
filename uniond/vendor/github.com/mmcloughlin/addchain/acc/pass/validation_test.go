package pass

import (
	"testing"

	"github.com/mmcloughlin/addchain/acc/ir"
)

func TestCheckDanglingInputsError(t *testing.T) {
	// Shift instruction followed by a reference to an intermediate result.
	p := &ir.Program{
		Instructions: []*ir.Instruction{
			{
				Output: ir.Index(3),
				Op: ir.Shift{
					X: ir.Index(0),
					S: 3,
				},
			},
			{
				Output: ir.Index(4),
				Op: ir.Add{
					X: ir.Index(1),
					Y: ir.Index(3),
				},
			},
		},
	}

	err := CheckDanglingInputs(p)

	if err == nil {
		t.Fatal("expected error")
	}
}
