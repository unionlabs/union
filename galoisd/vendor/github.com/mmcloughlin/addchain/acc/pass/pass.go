// Package pass implements analysis and processing passes on acc programs.
package pass

import (
	"fmt"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/internal/errutil"
)

// Interface for a processing pass.
type Interface interface {
	Execute(*ir.Program) error
}

// Func adapts a function to the pass Interface.
type Func func(*ir.Program) error

// Execute calls p.
func (f Func) Execute(p *ir.Program) error {
	return f(p)
}

// Concat returns a pass that executes the given passes in order, stopping on
// the first error.
func Concat(passes ...Interface) Interface {
	return Func(func(p *ir.Program) error {
		for _, pass := range passes {
			if err := pass.Execute(p); err != nil {
				return err
			}
		}
		return nil
	})
}

// Exec is a convenience for executing a list of passes on p.
func Exec(p *ir.Program, passes ...Interface) error {
	return Concat(passes...).Execute(p)
}

// CanonicalizeOperands ensures there is only one Operand object for each
// operand index in the program. In particular, this ensures there are not
// conflicting names for the same index. Populates the Operands field of the
// program.
func CanonicalizeOperands(p *ir.Program) error {
	if p.Operands != nil {
		return nil
	}

	p.Operands = map[int]*ir.Operand{}

	// First pass through determines canonical operand for each index.
	for _, i := range p.Instructions {
		for _, operand := range i.Operands() {
			// Look for an existing operand object for this index.
			existing, found := p.Operands[operand.Index]
			if !found {
				p.Operands[operand.Index] = operand
				continue
			}

			if existing == operand {
				continue
			}

			// They're different objects. Check for a name conflict.
			if existing.Identifier != "" && operand.Identifier != "" && existing.Identifier != operand.Identifier {
				return fmt.Errorf("identifier conflict: index %d named %q and %q", operand.Index, operand.Identifier, existing.Identifier)
			}

			if operand.Identifier != "" {
				existing.Identifier = operand.Identifier
			}
		}
	}

	// Second pass through replaces all operands with the canonical version.
	for _, i := range p.Instructions {
		switch op := i.Op.(type) {
		case ir.Add:
			i.Op = ir.Add{
				X: p.Operands[op.X.Index],
				Y: p.Operands[op.Y.Index],
			}
		case ir.Double:
			i.Op = ir.Double{
				X: p.Operands[op.X.Index],
			}
		case ir.Shift:
			i.Op = ir.Shift{
				X: p.Operands[op.X.Index],
				S: op.S,
			}
		default:
			return errutil.UnexpectedType(op)
		}
	}

	return nil
}

// ReadCounts computes how many times each index is read in the program. This
// populates the ReadCount field of the program.
func ReadCounts(p *ir.Program) error {
	if p.ReadCount != nil {
		return nil
	}

	p.ReadCount = map[int]int{}
	for _, i := range p.Instructions {
		for _, input := range i.Op.Inputs() {
			p.ReadCount[input.Index]++
		}
	}
	return nil
}
