package pass

import (
	"errors"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/internal/errutil"
)

// Compile generates the fully unrolled sequence of additions. The result is
// stored in the Program field.
func Compile(p *ir.Program) error {
	if p.Program != nil {
		return nil
	}

	p.Program = addchain.Program{}
	for _, i := range p.Instructions {
		var out int
		var err error

		switch op := i.Op.(type) {
		case ir.Add:
			out, err = p.Program.Add(op.X.Index, op.Y.Index)
		case ir.Double:
			out, err = p.Program.Double(op.X.Index)
		case ir.Shift:
			out, err = p.Program.Shift(op.X.Index, op.S)
		default:
			return errutil.UnexpectedType(op)
		}

		if err != nil {
			return err
		}
		if out != i.Output.Index {
			return errors.New("incorrect output index")
		}
	}

	return nil
}

// Eval evaluates the program and places the result in the Chain field.
func Eval(p *ir.Program) error {
	if p.Chain != nil {
		return nil
	}

	if err := Compile(p); err != nil {
		return err
	}

	p.Chain = p.Program.Evaluate()
	return nil
}
