package acc

import (
	"fmt"

	"github.com/mmcloughlin/addchain/acc/ast"
	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/internal/errutil"
)

// Translate converts an abstract syntax tree to an intermediate representation.
func Translate(c *ast.Chain) (*ir.Program, error) {
	s := newstate()
	for _, stmt := range c.Statements {
		if err := s.statement(stmt); err != nil {
			return nil, err
		}
	}
	return s.prog, nil
}

type state struct {
	prog     *ir.Program
	n        int
	variable map[ast.Identifier]*ir.Operand
}

func newstate() *state {
	return &state{
		prog:     &ir.Program{},
		n:        1,
		variable: map[ast.Identifier]*ir.Operand{},
	}
}

func (s *state) statement(stmt ast.Statement) error {
	out, err := s.expr(stmt.Expr)
	if err != nil {
		return err
	}
	if err := s.define(stmt.Name, out); err != nil {
		return err
	}
	return nil
}

func (s *state) expr(expr ast.Expr) (*ir.Operand, error) {
	switch e := expr.(type) {
	case ast.Operand:
		return &ir.Operand{Index: int(e)}, nil
	case ast.Identifier:
		return s.lookup(e)
	case ast.Add:
		return s.add(e)
	case ast.Double:
		return s.double(e)
	case ast.Shift:
		return s.shift(e)
	default:
		return nil, errutil.UnexpectedType(e)
	}
}

func (s *state) add(a ast.Add) (*ir.Operand, error) {
	x, err := s.expr(a.X)
	if err != nil {
		return nil, err
	}

	y, err := s.expr(a.Y)
	if err != nil {
		return nil, err
	}

	if x.Index > y.Index {
		x, y = y, x
	}

	out := ir.Index(s.n)
	inst := &ir.Instruction{
		Output: out,
		Op:     ir.Add{X: x, Y: y},
	}
	s.prog.AddInstruction(inst)
	s.n++

	return out, nil
}

func (s *state) double(d ast.Double) (*ir.Operand, error) {
	x, err := s.expr(d.X)
	if err != nil {
		return nil, err
	}

	out := ir.Index(s.n)
	inst := &ir.Instruction{
		Output: out,
		Op:     ir.Double{X: x},
	}
	s.prog.AddInstruction(inst)
	s.n++

	return out, nil
}

func (s *state) shift(sh ast.Shift) (*ir.Operand, error) {
	x, err := s.expr(sh.X)
	if err != nil {
		return nil, err
	}

	s.n += int(sh.S)
	out := ir.Index(s.n - 1)
	inst := &ir.Instruction{
		Output: out,
		Op:     ir.Shift{X: x, S: sh.S},
	}
	s.prog.AddInstruction(inst)

	return out, nil
}

func (s *state) define(name ast.Identifier, op *ir.Operand) error {
	if _, found := s.variable[name]; found {
		return fmt.Errorf("cannot redefine %q", name)
	}
	op.Identifier = string(name)
	s.variable[name] = op
	return nil
}

func (s state) lookup(name ast.Identifier) (*ir.Operand, error) {
	operand, ok := s.variable[name]
	if !ok {
		return nil, fmt.Errorf("variable %q undefined", name)
	}
	return operand, nil
}
