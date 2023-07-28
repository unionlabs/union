package pass

import (
	"fmt"

	"github.com/mmcloughlin/addchain/acc/ir"
)

// Validate is a pass to sanity check an intermediate representation program.
var Validate = Func(CheckDanglingInputs)

// CheckDanglingInputs looks for program inputs that have no instruction
// outputting them. Note this can happen and still be technically correct. For
// example a shift instruction produces many intermediate results, and one of
// these can later be referenced. The resulting program is still correct, but
// undesirable.
func CheckDanglingInputs(p *ir.Program) error {
	outputset := map[int]bool{0: true}
	for _, i := range p.Instructions {
		for _, input := range i.Op.Inputs() {
			if !outputset[input.Index] {
				return fmt.Errorf("no output instruction for input index %d", input.Index)
			}
		}
		outputset[i.Output.Index] = true
	}
	return nil
}
