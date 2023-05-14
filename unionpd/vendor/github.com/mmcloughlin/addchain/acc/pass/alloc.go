package pass

import (
	"fmt"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/internal/container/heap"
	"github.com/mmcloughlin/addchain/internal/errutil"
)

// Allocator pass assigns a minimal number of temporary variables to execute a program.
type Allocator struct {
	// Input is the name of the input variable. Note this is index 0, or the
	// identity element of the addition chain.
	Input string

	// Output is the name to give to the final output of the addition chain. This
	// variable may itself be used as a temporary during execution.
	Output string

	// Format defines how to format any temporary variables. This format string
	// must accept one integer value. For example "t%d" would be a reasonable
	// choice.
	Format string
}

// Execute performs temporary variable allocation.
func (a Allocator) Execute(p *ir.Program) error {
	// Canonicalize operands and delete all names.
	if err := Exec(p, Func(CanonicalizeOperands), Func(ClearNames)); err != nil {
		return err
	}

	// Initialize allocation. This maps operand index to variable index. The
	// inidicies 0 and 1 are special, reserved for the input and output
	// respectively. Any indicies above that are temporaries.
	out := p.Output()
	allocation := map[int]int{
		0:         0,
		out.Index: 1,
	}
	n := 2

	// Keep a heap of available indicies. Initially none.
	available := heap.NewMinInts()

	// Process instructions in reverse.
	for i := len(p.Instructions) - 1; i >= 0; i-- {
		inst := p.Instructions[i]

		// The output operand variable now becomes available.
		v, ok := allocation[inst.Output.Index]
		if !ok {
			return errutil.AssertionFailure("output operand %d missing allocation", inst.Output.Index)
		}
		available.Push(v)

		// Inputs may need variables, if they are not already live.
		for _, input := range inst.Op.Inputs() {
			_, ok := allocation[input.Index]
			if ok {
				continue
			}

			// If there's nothing available, we'll need one more temporary.
			if available.Empty() {
				available.Push(n)
				n++
			}

			allocation[input.Index] = available.Pop()
		}
	}

	// Record allocation.
	for _, op := range p.Operands {
		op.Identifier = a.name(allocation[op.Index])
	}

	temps := []string{}
	for i := 2; i < n; i++ {
		temps = append(temps, a.name(i))
	}
	p.Temporaries = temps

	return nil
}

func (a Allocator) name(v int) string {
	switch v {
	case 0:
		return a.Input
	case 1:
		return a.Output
	default:
		return fmt.Sprintf(a.Format, v-2)
	}
}
