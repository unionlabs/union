package acc

import (
	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/acc/ir"
)

// Decompile an unrolled program into concise intermediate representation.
func Decompile(p addchain.Program) (*ir.Program, error) {
	numreads := p.ReadCounts()
	r := &ir.Program{}
	for i := 0; i < len(p); i++ {
		op := p[i]

		// Regular addition.
		if !op.IsDouble() {
			r.AddInstruction(&ir.Instruction{
				Output: ir.Index(i + 1),
				Op: ir.Add{
					X: ir.Index(op.I),
					Y: ir.Index(op.J),
				},
			})
			continue
		}

		// We have a double. Look ahead to see if this is a chain of doublings, which
		// can be encoded as a shift. Note we can only follow the the doublings as
		// long as the intermediate values are not required anywhere else later in the
		// program.
		j := i + 1
		for ; j < len(p) && numreads[j] == 1 && p[j].I == j && p[j].J == j; j++ {
		}

		s := j - i

		// Shift size 1 encoded as a double.
		if s == 1 {
			r.AddInstruction(&ir.Instruction{
				Output: ir.Index(i + 1),
				Op: ir.Double{
					X: ir.Index(op.I),
				},
			})
			continue
		}

		i = j - 1
		r.AddInstruction(&ir.Instruction{
			Output: ir.Index(i + 1),
			Op: ir.Shift{
				X: ir.Index(op.I),
				S: uint(s),
			},
		})
	}
	return r, nil
}
